<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { toast, save } from "../stores";
  import { kitSquadNations, kitSaveEdit, type Nation } from "../api";
  import { render as renderKit, imageData } from "../kits/render";
  import { defaultFlag, migrate, newId, type Recipe, type FlagLayer } from "../kits/layers";
  import { provideEditor, screenEyedropper } from "../kits/editor";
  import { baseImage, findEdit, invalidate, refreshEdits, kitEditsStore } from "../kits/store";
  import KitThumb from "./KitThumb.svelte";
  import Flag from "./Flag.svelte";
  import FlagFields from "./kit/FlagFields.svelte";
  import Slider from "./kit/Slider.svelte";
  import Icon from "./Icon.svelte";

  export let kit: string;
  export let parts: string[];
  export let teamId: number;

  const dispatch = createEventDispatcher<{ edit: { part: string; label: string; initial: Recipe } }>();

  /** One design per nation: which jersey it starts from and its flag. */
  interface Design { base: string; flag: FlagLayer }

  let nations: Nation[] = [];
  let chosen = new Set<string>();
  let current: Nation | null = null;
  let designs: Record<string, Design> = {};
  /** Nations changed here since their jersey was last saved. */
  let unsaved = new Set<string>();
  let previewImg: ImageData | null = null;
  let replaceOwn = false;
  let busy = "";
  let error = "";

  const part = (n: Nation) => `maillot_${n.code}`;
  $: hasTour = parts.includes("maillot_tour");
  $: own = (n: Nation) => parts.includes(part(n));
  $: made = (n: Nation) => $kitEditsStore.some((e) => e.kit === kit && e.part === part(n));

  function touched() {
    if (current) unsaved = new Set(unsaved).add(current.code);
    designs = designs;
  }

  provideEditor({
    change(fn) { fn(); touched(); },
    async pickColor(cb) { const hex = await screenEyedropper(); if (hex) cb(hex); },
    get teamColors() { const t = $save?.teams.find((x) => x.is_mine); return [t?.color1, t?.color2].filter(Boolean).map((c) => String(c).toLowerCase()); },
  });

  /** The nation's saved design, or a fresh one. */
  function designFor(n: Nation): Design {
    if (!designs[n.code]) {
      const saved = findEdit(kit, part(n));
      const r = saved ? migrate(saved.recipe) : null;
      const flag = r?.layers.find((l): l is FlagLayer => l.type === "flag");
      designs[n.code] = r && flag
        ? { base: r.base.kind === "part" ? r.base.part : "maillot", flag: structuredClone(flag) }
        : { base: "maillot", flag: defaultFlag({ flag: { flag: n.flag, country: n.name } }) };
    }
    return designs[n.code];
  }

  onMount(async () => {
    try {
      nations = (await kitSquadNations(teamId)).filter((n) => n.code.length === 3);
      chosen = new Set(nations.filter((n) => n.flag && !own(n)).map((n) => n.code));
      current = nations.find((n) => n.flag) ?? null;
    } catch (e) {
      error = String(e);
    }
  });

  $: design = current ? designFor(current) : null;

  function recipeFor(n: Nation): Recipe {
    const d = designFor(n);
    const flag: FlagLayer = { ...structuredClone(d.flag), id: newId(), flag: n.flag, country: n.name, name: `${n.name} flag` };
    // Keep anything else added to this champion jersey in the full editor (logos, text…).
    const saved = findEdit(kit, part(n));
    const extra = saved ? migrate(saved.recipe).layers.filter((l) => l.type !== "flag") : [];
    return { v: 2, base: { kind: "part", part: d.base }, layers: [flag, ...extra] };
  }

  let token = 0;
  $: if (current) drawPreview(current, designs, $kitEditsStore);
  async function drawPreview(n: Nation, ..._deps: unknown[]) {
    const t = ++token;
    try {
      const r = recipeFor(n);
      const img = imageData(await renderKit(await baseImage(kit, part(n), r), r, part(n)));
      if (t === token) previewImg = img;
    } catch (e) {
      error = String(e);
    }
  }

  async function saveNations(list: Nation[]) {
    if (!list.length) return;
    error = "";
    try {
      for (const [i, n] of list.entries()) {
        busy = list.length > 1 ? `Saving ${n.name} (${i + 1}/${list.length})…` : `Saving ${n.name}…`;
        const r = recipeFor(n);
        const img = imageData(await renderKit(await baseImage(kit, part(n), r), r, part(n)));
        await kitSaveEdit(kit, part(n), r, img);
        invalidate(kit, part(n));
        unsaved.delete(n.code);
      }
      unsaved = unsaved;
      await refreshEdits();
      toast(`${list.length === 1 ? `${list[0].name} champion jersey` : `${list.length} champion jerseys`} saved. Apply to game to use ${list.length === 1 ? "it" : "them"}`);
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  /** Copies the current nation's design (not its flag) to every other nation. */
  function useForAll() {
    if (!current || !design) return;
    const src = design;
    for (const n of nations) {
      if (n.code === current.code || !n.flag) continue;
      const d = designFor(n);
      d.base = src.base;
      d.flag = { ...structuredClone(src.flag), id: d.flag.id, flag: n.flag, country: n.name, name: `${n.name} flag` };
      unsaved.add(n.code);
    }
    unsaved = unsaved;
    designs = designs;
    toast(`${current.name}'s design copied to the other nations. Save them to keep it`, "info", 4000);
  }

  function toggle(code: string) {
    chosen.has(code) ? chosen.delete(code) : chosen.add(code);
    chosen = chosen;
  }
  $: batch = nations.filter((n) => chosen.has(n.code) && n.flag && (replaceOwn || !own(n)));
</script>

<section class="panel">
  <div class="panel-head">
    <h3>National champion jerseys</h3>
    <span class="muted">Worn by your riders who win their national championship. Each nation has its own design.</span>
  </div>

  {#if error}<p class="err pad">{error}</p>{/if}
  <div class="wrap">
    <div class="list">
      {#each nations as n (n.code)}
        {@const status = made(n) ? "made" : own(n) ? "own" : "none"}
        <div class="nation" class:active={current?.code === n.code}>
          <input type="checkbox" checked={chosen.has(n.code)} disabled={!n.flag || (status === "own" && !replaceOwn)} on:change={() => toggle(n.code)} aria-label="Include {n.name} when saving the ticked nations" />
          <button class="name" on:click={() => (current = n)} disabled={!n.flag}>
            <Flag code={n.flag} size={14} />
            <span class="nm">{n.name}{#if unsaved.has(n.code)}<i class="dot" title="Unsaved changes"></i>{/if}</span>
            <small class="muted">{n.riders}</small>
          </button>
          {#if status === "made"}<span class="tag made" title="Saved here">Made</span>
          {:else if status === "own"}<span class="tag own" title="The kit has its own jersey for this nation">Kit's own</span>
          {:else}<span class="tag none" title="Riders wear the generic national jersey">Generic</span>{/if}
        </div>
      {:else}
        <p class="muted">Loading your squad…</p>
      {/each}
      {#if nations.length}
        <div class="batch">
          <label class="chk"><input type="checkbox" bind:checked={replaceOwn} /> Include nations whose kit has its own jersey</label>
          <button class="btn btn-sm" on:click={() => saveNations(batch)} disabled={!!busy || !batch.length}>Save {batch.length} ticked</button>
        </div>
      {/if}
    </div>

    <div class="preview-col">
      <div class="preview">
        {#if previewImg && current}
          <KitThumb kit={kit} part={part(current)} image={previewImg} alt="{current.name} champion jersey preview" />
        {/if}
      </div>
      {#if current}
        <div class="preview-bar">
          <Flag code={current.flag} size={16} /><strong>{current.name}</strong>
          {#if unsaved.has(current.code)}<span class="muted">· unsaved changes</span>{/if}
          <div class="spacer"></div>
          <button class="btn btn-sm btn-quiet" on:click={() => current && dispatch("edit", { part: part(current), label: `${current.name} champion jersey`, initial: recipeFor(current) })} title="Add logos, text… in the full editor"><Icon name="contract" size={13} />Full editor</button>
          <button class="btn btn-sm btn-primary" on:click={() => current && saveNations([current])} disabled={!!busy}>{busy || `Save ${current.name}`}</button>
        </div>
      {/if}
    </div>

    <div class="controls">
      {#if current && design}
        {#key current.code}
          <h4><Flag code={current.flag} size={14} /> {current.name}</h4>
          {#if hasTour}
            <label class="field"><span>Start from</span>
              <select value={design.base} on:change={(e) => { if (design) { design.base = e.currentTarget.value; touched(); } }}>
                <option value="maillot">Team jersey</option>
                <option value="maillot_tour">White alternate jersey</option>
              </select>
            </label>
          {/if}
          <FlagFields layer={design.flag} split />
          <Slider label="Flag opacity (lower lets the kit's logos show through)" value={design.flag.opacity} min={0.05} max={1} scale={100} unit="%"
            on:input={(e) => { if (design) { design.flag.opacity = e.detail; touched(); } }} />
          <button class="btn btn-sm" on:click={useForAll} disabled={nations.length < 2}>Use this design for all nations</button>
        {/key}
      {/if}
    </div>
  </div>
</section>

<style>
  .wrap { display: grid; grid-template-columns: 220px minmax(0, 1fr) 300px; gap: 16px; padding: 0 14px 14px; align-items: start; }
  .list { display: flex; flex-direction: column; gap: 2px; }
  .nation { display: flex; align-items: center; gap: 6px; padding: 3px 4px; border-radius: 6px; }
  .nation.active { background: var(--panel-2); box-shadow: inset 3px 0 0 var(--jaune); }
  .name { flex: 1; min-width: 0; display: flex; align-items: center; gap: 7px; background: none; border: none; color: var(--ink); font: inherit; font-size: 13px; cursor: pointer; text-align: left; padding: 4px 0; }
  .name:disabled { opacity: 0.5; cursor: default; }
  .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; display: inline-flex; align-items: center; gap: 5px; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--jaune); display: inline-block; flex-shrink: 0; }
  .name small { font-size: 11px; }
  .tag { font-size: 10.5px; padding: 0 6px; border-radius: 999px; white-space: nowrap; }
  .tag.made { color: var(--vert); border: 1px solid #3fb95055; }
  .tag.own { color: var(--ink-3); border: 1px solid var(--rule-2); }
  .tag.none { color: var(--jaune); border: 1px solid #f5c51855; }
  .batch { display: flex; flex-direction: column; gap: 8px; margin-top: 12px; padding-top: 10px; border-top: 1px solid var(--rule); }
  .chk { display: flex; align-items: center; gap: 7px; font-size: 12px; color: var(--ink-2); }
  /* The preview stays in view while the controls beside it scroll. */
  .preview-col { position: sticky; top: 0; display: flex; flex-direction: column; gap: 8px; }
  .preview { aspect-ratio: 1200 / 826; background: #1e1e1e; border-radius: 6px; overflow: hidden; }
  .preview-bar { display: flex; align-items: center; gap: 8px; font-size: 13px; }
  .spacer { flex: 1; }
  .controls { display: flex; flex-direction: column; gap: 10px; max-height: calc(100vh - 200px); overflow-y: auto; padding-right: 4px; }
  .controls h4 { display: flex; align-items: center; gap: 7px; font-family: var(--font-cond); font-size: 16px; font-weight: 600; }
  .pad { padding: 0 14px 12px; }
  .err { color: var(--pois); }
  @media (max-width: 1350px) { .wrap { grid-template-columns: 190px minmax(0, 1fr) 270px; } }
  @media (max-width: 1100px) { .wrap { grid-template-columns: 1fr; } .preview-col { position: static; } .controls { max-height: none; } }
</style>
