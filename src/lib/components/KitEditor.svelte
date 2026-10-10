<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { toast, save } from "../stores";
  import { kitSaveEdit, kitDeleteEdit, writePng, pickImage, pickSavePath } from "../api";
  import { render as renderKit, imageData, fitToSize, bytesToDataUrl, canvasPng, isSplit, kitMask } from "../kits/render";
  import { migrate, emptyRecipe, newLayer, imageLayer, cloneLayer, isPositioned, type Recipe, type LayerType, type LayerContext } from "../kits/layers";
  import { provideEditor, History, snapshot, rememberColor } from "../kits/editor";
  import { baseImage, findEdit, invalidate, refreshEdits, teamLogo } from "../kits/store";
  import KitCanvas from "./kit/KitCanvas.svelte";
  import LayerList from "./kit/LayerList.svelte";
  import LayerProps from "./kit/LayerProps.svelte";
  import CopyLayers from "./kit/CopyLayers.svelte";
  import Slider from "./kit/Slider.svelte";
  import ColorField from "./kit/ColorField.svelte";
  import Icon from "./Icon.svelte";

  export let kit: string;
  export let part: string;
  export let label: string;
  /** Recipe to start from when the part has no saved edit yet. */
  export let initial: Recipe | null = null;
  /** Every part of this kit, for "copy layers to". */
  export let parts: string[] = [];
  export let countryName: (code: string) => string | undefined = () => undefined;

  const dispatch = createEventDispatcher<{ close: void; saved: void }>();
  const existing = findEdit(kit, part);
  const split = isSplit(part);

  let recipe: Recipe = structuredClone(migrate(existing?.recipe ?? initial ?? emptyRecipe()));
  let base: ImageData | null = null;
  let mask: HTMLCanvasElement | null = null;
  let rendered: HTMLCanvasElement | null = null;
  let selected: string | null = null;
  let tool: "select" | "brush" | "pick" = "select";
  let brush = { color: "#ffffff", size: 0.012, erase: false };
  let zoom = 1;
  let compare = false;
  let guides = false;
  let busy = false;
  let dirty = !existing && !!initial;
  let error = "";
  let copying = false;
  const history = new History();
  let histTick = 0; // re-evaluates undo/redo buttons

  // ─── Team context ───────────────────────────────────────────────────────────
  $: myTeam = ($save?.teams ?? []).find((t) => t.is_mine);
  $: teamColors = [myTeam?.color1, myTeam?.color2].filter(Boolean).map((c) => String(c).toLowerCase());
  $: countries = [...new Map(($save?.cyclists ?? []).filter((c) => c.flag).map((c) => [c.flag, c.nationality])).entries()]
    .map(([flag, name]) => ({ flag, name }))
    .sort((a, b) => a.name.localeCompare(b.name));
  $: squadTop = (() => {
    const n = new Map<string, number>();
    for (const c of $save?.cyclists ?? []) if (c.team_id === myTeam?.id && c.flag) n.set(c.flag, (n.get(c.flag) ?? 0) + 1);
    const top = [...n.entries()].sort((a, b) => b[1] - a[1])[0]?.[0];
    const c = countries.find((x) => x.flag === top);
    return c ? { flag: c.flag, country: c.name } : null;
  })();
  const layerCtx = (): LayerContext => ({ split, color1: teamColors[0] ?? "#c6a04f", color2: teamColors[1] ?? "#ffffff", flag: squadTop, teamName: myTeam?.short ?? "" });

  // ─── Editing ────────────────────────────────────────────────────────────────
  let pickCb: ((hex: string) => void) | null = null;
  let toolBeforePick: "select" | "brush" | "pick" = "select";
  function change(fn: () => void, gesture?: string) {
    history.record(recipe, gesture);
    fn();
    recipe = recipe;
    dirty = true;
    histTick++;
  }
  provideEditor({
    change,
    pickColor: beginPick,
    get teamColors() { return teamColors; },
  });

  // Colours for a layer are picked from the kit as it looks below that layer, so e.g. a
  // Replace colour layer can be pointed at the original colour it is about to change.
  let pickFrom: HTMLCanvasElement | null = null;
  $: if (tool !== "pick") pickFrom = null;
  async function beginPick(cb: (hex: string) => void) {
    pickCb = cb;
    if (tool !== "pick") toolBeforePick = tool;
    tool = "pick";
    const idx = recipe.layers.findIndex((l) => l.id === selected);
    if (base && idx >= 0) {
      const c = await renderKit(base, recipe, part, { upTo: idx });
      if (pickCb === cb) pickFrom = c;
    }
  }

  function onPick(e: CustomEvent<string>) {
    rememberColor(e.detail);
    if (pickCb) { pickCb(e.detail); pickCb = null; tool = toolBeforePick; }
    else { brush = { ...brush, color: e.detail, erase: false }; toast(`Brush colour ${e.detail}`, "info", 1600); }
  }
  function setTool(t: typeof tool) {
    if (t !== "pick") pickCb = null;
    if (t === "brush" && sel?.type !== "brush") {
      const b = recipe.layers.findLast((l) => l.type === "brush");
      if (b) selected = b.id;
      else addLayer("brush");
    }
    tool = t;
  }

  $: sel = recipe.layers.find((l) => l.id === selected) ?? null;

  async function addLayer(t: LayerType) {
    if (t === "image") {
      const picked = await pickImage();
      if (!picked) return;
      const l = imageLayer(bytesToDataUrl(picked.bytes, picked.name), picked.name.split(/[\\/]/).pop() ?? "Logo", split);
      change(() => recipe.layers.push(l));
      selected = l.id;
      tool = "select";
      return;
    }
    const l = newLayer(t, layerCtx());
    change(() => recipe.layers.push(l));
    selected = l.id;
    if (t === "brush") tool = "brush";
    else if (tool === "brush") tool = "select";
    if (l.type === "replace") {
      toast("Click the colour on the kit you want to replace", "info", 4000);
      toolBeforePick = "select";
      beginPick((hex) => change(() => { l.from = hex; }));
    }
  }
  function addTeamLogo() {
    if (!$teamLogo) return;
    const l = { ...imageLayer($teamLogo, "Team logo", split), link: "team" as const, removeBg: true };
    change(() => recipe.layers.push(l));
    selected = l.id;
    tool = "select";
  }
  async function replaceImage() {
    if (sel?.type !== "image") return;
    const picked = await pickImage();
    if (!picked) return;
    const l = sel;
    change(() => { l.src = bytesToDataUrl(picked.bytes, picked.name); l.name = picked.name.split(/[\\/]/).pop() ?? l.name; });
  }

  function undo() {
    const prev = history.back<Recipe>(recipe);
    if (prev) { recipe = prev; dirty = true; histTick++; }
  }
  function redo() {
    const next = history.forward<Recipe>(recipe);
    if (next) { recipe = next; dirty = true; histTick++; }
  }
  $: canUndo = histTick >= 0 && history.canUndo;
  $: canRedo = histTick >= 0 && history.canRedo;

  // ─── Base picture ───────────────────────────────────────────────────────────
  let baseKey = "";
  $: if (snapshot(recipe.base) !== baseKey) loadBase(snapshot(recipe.base));
  async function loadBase(key: string) {
    baseKey = key;
    try {
      const b = await baseImage(kit, part, recipe);
      if (key !== baseKey) return;
      base = b;
      mask = kitMask(b);
      error = "";
    } catch (e) {
      error = String(e);
    }
  }
  async function importPicture() {
    const picked = await pickImage();
    if (!picked || !base) return;
    try {
      const fit = await fitToSize(bytesToDataUrl(picked.bytes, picked.name), base.width, base.height);
      if (fit.aspectOff) toast(`The picture was stretched to ${base.width}×${base.height}, the size of this part`, "info", 6000);
      change(() => (recipe.base = { kind: "image", src: fit.src }));
    } catch (e) {
      toast(String(e), "error");
    }
  }
  function useOriginal() {
    change(() => (recipe.base = initial?.base.kind === "part" ? initial.base : { kind: "original" }));
  }

  // ─── Rendering ──────────────────────────────────────────────────────────────
  let token = 0;
  $: if (base) schedule(recipe, base);
  function schedule(r: Recipe, b: ImageData) {
    const t = ++token;
    requestAnimationFrame(async () => {
      if (t !== token) return;
      try {
        const c = await renderKit(b, r, part);
        if (t === token) rendered = c;
      } catch (e) {
        error = String(e);
      }
    });
  }

  // ─── Save / export ──────────────────────────────────────────────────────────
  async function finalCanvas() {
    if (!base) throw new Error("Nothing to save yet");
    return renderKit(base, recipe, part);
  }
  async function saveEdit() {
    busy = true;
    try {
      await kitSaveEdit(kit, part, recipe, imageData(await finalCanvas()));
      invalidate(kit, part);
      await refreshEdits();
      dirty = false;
      toast(`${label} saved. Apply to game to see it in PCM`);
      dispatch("saved");
      dispatch("close");
    } catch (e) {
      toast(String(e), "error", 8000);
    } finally {
      busy = false;
    }
  }
  async function exportPng() {
    const path = await pickSavePath(`${kit}_${part}.png`);
    if (!path) return;
    try {
      await writePng(path, await canvasPng(await finalCanvas()));
      toast(`Exported to ${path}`);
    } catch (e) {
      toast(String(e), "error");
    }
  }
  async function reset() {
    if (!existing) return dispatch("close");
    if (!confirm(`Reset ${label} to the original? Your layers for this part are deleted.`)) return;
    busy = true;
    try {
      await kitDeleteEdit(kit, part);
      invalidate(kit, part);
      await refreshEdits();
      toast(`${label} reset to the original. Apply to game to update PCM`);
      dispatch("saved");
      dispatch("close");
    } finally {
      busy = false;
    }
  }
  function close() {
    if (dirty && !confirm("Discard your changes to this part?")) return;
    dispatch("close");
  }

  // ─── Keyboard ───────────────────────────────────────────────────────────────
  function key(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const typing = target.closest("input, textarea, select, [contenteditable]") && !(target as HTMLInputElement).type?.match(/range|checkbox|color/);
    const mod = e.ctrlKey || e.metaKey;
    if (e.key === "Escape") {
      if (tool === "pick") { pickCb = null; tool = toolBeforePick; }
      else if (!typing && selected) selected = null;
      else if (!busy && !copying) close();
      return;
    }
    if (mod && e.key.toLowerCase() === "s") { e.preventDefault(); if (!busy) saveEdit(); return; }
    if (typing) return;
    if (mod && e.key.toLowerCase() === "z") { e.preventDefault(); e.shiftKey ? redo() : undo(); return; }
    if (mod && e.key.toLowerCase() === "y") { e.preventDefault(); redo(); return; }
    if (mod && e.key.toLowerCase() === "d" && sel) {
      e.preventDefault();
      const copy = cloneLayer(sel);
      copy.name = `${sel.name} copy`;
      if (isPositioned(copy)) { copy.x += 0.02; copy.y += 0.02; }
      const at = recipe.layers.indexOf(sel) + 1;
      change(() => recipe.layers.splice(at, 0, copy));
      selected = copy.id;
      return;
    }
    if ((e.key === "Delete" || e.key === "Backspace") && sel) {
      const l = sel;
      change(() => recipe.layers.splice(recipe.layers.indexOf(l), 1));
      selected = null;
      return;
    }
    if (e.key.startsWith("Arrow") && sel && isPositioned(sel) && base) {
      e.preventDefault();
      const step = (e.shiftKey ? 10 : 1);
      const l = sel;
      change(() => {
        if (e.key === "ArrowLeft") l.x -= step / base!.width;
        if (e.key === "ArrowRight") l.x += step / base!.width;
        if (e.key === "ArrowUp") l.y -= step / base!.height;
        if (e.key === "ArrowDown") l.y += step / base!.height;
      }, `nudge:${l.id}`);
      return;
    }
    if (mod) return;
    if (e.key === "v") setTool("select");
    else if (e.key === "b") setTool("brush");
    else if (e.key === "i") { pickCb = null; tool = "pick"; }
    else if (e.key === "e" && tool === "brush") brush = { ...brush, erase: !brush.erase };
    else if (e.key === "g") guides = !guides;
    else if (e.key === "+" || e.key === "=") zoom = Math.min(8, zoom * 1.25);
    else if (e.key === "-") zoom = Math.max(0.25, zoom / 1.25);
    else if (e.key === "0") zoom = 1;
  }

  onMount(() => {
    if (!dirty) history.cut();
  });
</script>

<svelte:window on:keydown={key} />

<div class="editor" role="dialog" aria-modal="true" aria-label="Edit {label}">
  <header>
    <div class="title">
      <h2>{label}</h2>
      <small class="muted">{kit} · {part}{base ? ` · ${base.width}×${base.height}` : ""}</small>
    </div>
    <div class="group">
      <button class="btn btn-quiet btn-sm" on:click={undo} disabled={!canUndo} title="Undo (Ctrl+Z)">↶ Undo</button>
      <button class="btn btn-quiet btn-sm" on:click={redo} disabled={!canRedo} title="Redo (Ctrl+Y)">↷ Redo</button>
    </div>
    <div class="spacer"></div>
    <button class="btn btn-quiet btn-sm" on:click={() => (copying = true)} disabled={!recipe.layers.length}><Icon name="compare" size={14} />Copy layers to…</button>
    <button class="btn btn-quiet btn-sm" on:click={exportPng} disabled={!base}><Icon name="export" size={14} />Export PNG</button>
    {#if existing}<button class="btn btn-quiet btn-sm" on:click={reset} disabled={busy}><Icon name="reload" size={14} />Reset</button>{/if}
    <button class="btn" on:click={close} disabled={busy}>Cancel</button>
    <button class="btn btn-primary" on:click={saveEdit} disabled={busy || !base} title="Save (Ctrl+S)">{busy ? "Saving…" : "Save"}</button>
  </header>

  <div class="body">
    <div class="stage">
      <div class="toolbar">
        <div class="tools" role="toolbar" aria-label="Tools">
          <button class:on={tool === "select"} on:click={() => setTool("select")} title="Select and move (V)">⬚ Select</button>
          <button class:on={tool === "brush"} on:click={() => setTool("brush")} title="Paint (B)">✎ Brush</button>
          <button class:on={tool === "pick"} on:click={() => { pickCb = null; tool = "pick"; }} title="Pick a colour from the kit (I)"><Icon name="eye" size={13} /> Pick colour</button>
        </div>
        {#if tool === "brush"}
          <div class="brush">
            <ColorField label="" value={brush.color} on:change={(e) => e.detail && (brush = { ...brush, color: e.detail, erase: false })} />
            <div class="bsize"><Slider label="Size" value={brush.size} min={0.002} max={0.2} step={0.001} scale={100} unit="%" on:input={(e) => (brush = { ...brush, size: e.detail })} /></div>
            <button class="btn btn-sm" class:btn-primary={brush.erase} on:click={() => (brush = { ...brush, erase: !brush.erase })} title="Eraser (E)">Eraser</button>
          </div>
        {/if}
        {#if tool === "pick"}<span class="picking">{pickCb ? `Click a colour on the kit${pickFrom ? " (showing it below this layer)" : ""}… Esc cancels` : "Click the kit to pick a brush colour"}</span>{/if}
        <div class="spacer"></div>
        <div class="view">
          <button on:click={() => (zoom = Math.max(0.25, zoom / 1.25))} title="Zoom out (-)">−</button>
          <button class="pct" on:click={() => (zoom = 1)} title="Fit (0)">{Math.round(zoom * 100)}%</button>
          <button on:click={() => (zoom = Math.min(8, zoom * 1.25))} title="Zoom in (+)">+</button>
          <button class:on={guides} on:click={() => (guides = !guides)} title="Show the template's areas (G)">Guides</button>
          <button class:on={compare} on:pointerdown={() => (compare = true)} on:pointerup={() => (compare = false)} on:pointerleave={() => (compare = false)} title="Hold to see the original">Before</button>
        </div>
      </div>
      {#if error}<p class="err">{error}</p>{/if}
      {#if !base && !error}<p class="muted loading">Loading…</p>{/if}
      <KitCanvas {rendered} {base} {mask} {recipe} {selected} {tool} {split} {compare} {guides} {pickFrom} bind:brush bind:zoom on:select={(e) => (selected = e.detail)} on:pick={onPick} />
      {#if split && base}<div class="panels"><span>Front</span><span>Back</span></div>{/if}
      <p class="hint muted">Drag to move · corner handle resizes · top handle rotates (Shift snaps) · scroll over a selection to resize · Ctrl+scroll zooms · arrows nudge · Ctrl+D duplicates · Del deletes</p>
    </div>

    <aside>
      <section>
        <h4>Picture</h4>
        <p class="muted small">
          {#if recipe.base.kind === "image"}Your imported picture.{:else if recipe.base.kind === "part"}Starts from {recipe.base.part === "maillot" ? "the jersey" : recipe.base.part}, as you edited it.{:else}The kit's original artwork.{/if}
        </p>
        <div class="row">
          <button class="btn btn-sm" on:click={importPicture}><Icon name="open" size={14} />Import picture…</button>
          {#if $teamLogo}<button class="btn btn-sm" on:click={addTeamLogo}><Icon name="plus" size={14} />Add team logo</button>{/if}
          {#if recipe.base.kind === "image"}<button class="btn btn-sm btn-quiet" on:click={useOriginal}>Use original</button>{/if}
        </div>
      </section>

      <section>
        <LayerList {recipe} {selected} on:select={(e) => { selected = e.detail; if (tool === "brush" && sel?.type !== "brush") tool = "select"; }} on:add={(e) => addLayer(e.detail)} />
      </section>

      {#if sel}
        <section class="propsec">
          {#key sel.id}<LayerProps layer={sel} {split} {countries} on:replaceImage={replaceImage} />{/key}
        </section>
      {/if}
    </aside>
  </div>
</div>

{#if copying}
  <CopyLayers {kit} {part} layers={recipe.layers} {selected} {parts} {countryName} on:close={() => (copying = false)} />
{/if}

<style>
  .editor { position: fixed; inset: 0; z-index: 8000; background: var(--bg); display: flex; flex-direction: column; }
  header { display: flex; align-items: center; gap: 8px; padding: 10px 16px; border-bottom: 1px solid var(--rule); background: var(--bg-2); }
  .title h2 { font-size: 20px; }
  .group { display: flex; gap: 2px; margin-left: 12px; }
  .spacer { flex: 1; }
  .body { flex: 1; display: grid; grid-template-columns: minmax(0, 1fr) 340px; overflow: hidden; }
  .stage { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  .toolbar { display: flex; align-items: center; gap: 12px; padding: 8px 14px; border-bottom: 1px solid var(--rule); flex-wrap: wrap; }
  .tools, .view { display: flex; border: 1px solid var(--rule-2); border-radius: 6px; overflow: hidden; }
  .tools button, .view button { display: inline-flex; align-items: center; gap: 4px; padding: 5px 10px; font: inherit; font-size: 12.5px; background: var(--panel); color: var(--ink-2); border: none; border-right: 1px solid var(--rule); cursor: pointer; }
  .tools button:last-child, .view button:last-child { border-right: none; }
  .tools button.on, .view button.on { background: var(--jaune); color: var(--jaune-ink); }
  .view .pct { min-width: 52px; justify-content: center; }
  .brush { display: flex; align-items: center; gap: 10px; }
  .bsize { width: 160px; }
  .picking { font-size: 12.5px; color: var(--jaune); }
  .panels { display: flex; justify-content: space-around; font-size: 12px; color: var(--ink-3); padding: 0 14px; }
  .hint { font-size: 11.5px; text-align: center; padding: 4px 14px 8px; }
  .loading, .err { padding: 14px; }
  .err { color: var(--pois); }
  aside { border-left: 1px solid var(--rule); overflow-y: auto; padding: 12px 14px; display: flex; flex-direction: column; gap: 16px; background: var(--bg-2); }
  section { display: flex; flex-direction: column; gap: 8px; }
  .propsec { padding-top: 12px; border-top: 1px solid var(--rule); }
  h4 { font-family: var(--font-cond); font-size: 16px; font-weight: 600; }
  .small { font-size: 12px; }
  .row { display: flex; gap: 6px; flex-wrap: wrap; }
</style>
