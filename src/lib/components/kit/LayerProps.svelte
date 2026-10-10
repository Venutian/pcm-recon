<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { BLENDS, FONTS, LAYER_INFO, type Layer } from "../../kits/layers";
  import { useEditor } from "../../kits/editor";
  import Slider from "./Slider.svelte";
  import Segmented from "./Segmented.svelte";
  import ColorField from "./ColorField.svelte";
  import FlagFields from "./FlagFields.svelte";
  import { teamLogo } from "../../kits/store";

  export let layer: Layer;
  export let split: boolean;
  export let countries: { flag: string; name: string }[];

  const dispatch = createEventDispatcher<{ replaceImage: void }>();
  const ed = useEditor();

  /** Sets a field on the layer through the editor (undo, re-render). */
  function set(k: string, v: unknown) {
    ed.change(() => { (layer as unknown as Record<string, unknown>)[k] = v; }, `${layer.id}:${k}`);
  }
  const num = (k: string) => (e: CustomEvent<number>) => set(k, e.detail);
  const col = (k: string) => (e: CustomEvent<string | null>) => set(k, e.detail);
  function followTeam(on: boolean) {
    const l = layer;
    if (l.type !== "image") return;
    ed.change(() => {
      l.link = on ? "team" : null;
      if (on && $teamLogo) { l.src = $teamLogo; l.removeBg = true; }
    });
  }
  function setArea(k: "x" | "y" | "w" | "h" | "feather" | "ellipse", v: number | boolean) {
    const a = layer.area;
    if (!a) return;
    ed.change(() => { (a as unknown as Record<string, unknown>)[k] = v; }, `${layer.id}:area:${k}`);
  }
  /** Crop sides stay inside the picture. */
  function setCrop(k: "x" | "y" | "w" | "h", v: number) {
    const l = layer;
    if (l.type !== "image" || !l.crop) return;
    const c = l.crop;
    ed.change(() => {
      c[k] = v;
      c.w = Math.min(c.w, 1 - c.x);
      c.h = Math.min(c.h, 1 - c.y);
    }, `${l.id}:crop:${k}`);
  }
  const panelOpts = [{ id: "both", label: "Front + back" }, { id: "front", label: "Front" }, { id: "back", label: "Back" }];
</script>

<div class="props">
  <div class="head">
    <input class="name" value={layer.name} on:change={(e) => set("name", e.currentTarget.value)} aria-label="Layer name" />
    <small class="muted">{LAYER_INFO[layer.type].label}</small>
  </div>

  {#if layer.type === "image"}
    {@const img = layer}
    {#if $teamLogo}
      <label class="chk"><input type="checkbox" checked={img.link === "team"} on:change={(e) => followTeam(e.currentTarget.checked)} /> Follow the team logo (changes when you change the logo)</label>
    {/if}
    {#if img.link !== "team"}<button class="btn btn-sm" on:click={() => dispatch("replaceImage")}>Replace image…</button>{/if}
    <label class="chk"><input type="checkbox" checked={img.removeBg} on:change={(e) => set("removeBg", e.currentTarget.checked)} /> Remove background (e.g. a logo on black or white)</label>
    {#if img.removeBg}
      <div class="two">
        <Slider label="Background range" value={img.bgTolerance} min={0} max={80} step={1} on:input={num("bgTolerance")} />
        <Slider label="Edge softness" value={img.bgSoftness} min={1} max={60} step={1} on:input={num("bgSoftness")} />
      </div>
    {/if}
    <label class="chk"><input type="checkbox" checked={!!img.crop} on:change={(e) => set("crop", e.currentTarget.checked ? { x: 0, y: 0, w: 1, h: 1 } : null)} /> Crop (use part of the picture)</label>
    {#if img.crop}
      {@const cr = img.crop}
      <div class="two">
        <Slider label="From left" value={cr.x} max={0.95} scale={100} unit="%" on:input={(e) => setCrop("x", e.detail)} />
        <Slider label="From top" value={cr.y} max={0.95} scale={100} unit="%" on:input={(e) => setCrop("y", e.detail)} />
        <Slider label="Width" value={cr.w} min={0.05} max={1} scale={100} unit="%" on:input={(e) => setCrop("w", e.detail)} />
        <Slider label="Height" value={cr.h} min={0.05} max={1} scale={100} unit="%" on:input={(e) => setCrop("h", e.detail)} />
      </div>
    {/if}
    <div class="two">
      <Slider label="Left / right" value={layer.x} scale={100} unit="%" on:input={num("x")} />
      <Slider label="Up / down" value={layer.y} scale={100} unit="%" on:input={num("y")} />
    </div>
    <Slider label="Size" value={layer.scale} min={0.01} max={1.5} step={0.005} scale={100} unit="%" on:input={num("scale")} />
    <Slider label="Rotation" value={layer.rotation} min={-180} max={180} step={1} unit="°" on:input={num("rotation")} />
    <div class="two">
      <Segmented label="Flip" value={layer.flipX} on:change={(e) => set("flipX", e.detail)} options={[{ id: false, label: "↔ Off" }, { id: true, label: "↔ On" }]} />
      <Segmented label="Flip vertical" value={layer.flipY} on:change={(e) => set("flipY", e.detail)} options={[{ id: false, label: "↕ Off" }, { id: true, label: "↕ On" }]} />
    </div>
    <ColorField label="Tint (one colour)" value={layer.tint} allowNone on:change={col("tint")} />
    <Slider label="Shadow" value={layer.shadow} min={0} max={1} scale={100} unit="%" on:input={num("shadow")} />

  {:else if layer.type === "text"}
    <label class="field"><span>Text</span><textarea rows="2" value={layer.text} on:input={(e) => set("text", e.currentTarget.value)}></textarea></label>
    <div class="two">
      <label class="field"><span>Font</span>
        <select value={layer.font} on:change={(e) => set("font", e.currentTarget.value)}>
          {#each FONTS as f}<option value={f} style="font-family:'{f}'">{f}</option>{/each}
        </select>
      </label>
      <label class="field"><span>Weight</span>
        <select value={layer.weight} on:change={(e) => set("weight", Number(e.currentTarget.value))}>
          {#each [300, 400, 500, 600, 700, 800, 900] as w}<option value={w}>{w}</option>{/each}
        </select>
      </label>
    </div>
    <div class="two">
      <Segmented label="Style" value={layer.italic} on:change={(e) => set("italic", e.detail)} options={[{ id: false, label: "Upright" }, { id: true, label: "Italic" }]} />
      <Segmented label="Align" value={layer.align} on:change={(e) => set("align", e.detail)} options={[{ id: "left", label: "⇤" }, { id: "center", label: "≡" }, { id: "right", label: "⇥" }]} />
    </div>
    <Slider label="Size" value={layer.size} min={0.005} max={0.4} step={0.001} scale={100} unit="%" on:input={num("size")} />
    <ColorField label="Colour" value={layer.color} on:change={col("color")} />
    <Slider label="Outline" value={layer.strokeWidth} min={0} max={0.4} step={0.005} scale={100} unit="%" on:input={num("strokeWidth")} />
    {#if layer.strokeWidth > 0}<ColorField label="Outline colour" value={layer.strokeColor} on:change={col("strokeColor")} />{/if}
    <Slider label="Letter spacing" value={layer.spacing} min={-0.2} max={1} step={0.005} scale={100} unit="%" on:input={num("spacing")} />
    <div class="two">
      <Slider label="Left / right" value={layer.x} scale={100} unit="%" on:input={num("x")} />
      <Slider label="Up / down" value={layer.y} scale={100} unit="%" on:input={num("y")} />
    </div>
    <Slider label="Rotation" value={layer.rotation} min={-180} max={180} step={1} unit="°" on:input={num("rotation")} />
    <Slider label="Shadow" value={layer.shadow} min={0} max={1} scale={100} unit="%" on:input={num("shadow")} />

  {:else if layer.type === "shape"}
    <Segmented label="Shape" value={layer.shape} on:change={(e) => set("shape", e.detail)}
      options={[{ id: "rect", label: "▭" , title: "Rectangle" }, { id: "ellipse", label: "◯", title: "Ellipse" }, { id: "triangle", label: "△", title: "Triangle" }, { id: "diamond", label: "◇", title: "Diamond" }, { id: "chevron", label: "∨", title: "Chevron" }, { id: "star", label: "☆", title: "Star" }]} />
    <div class="two">
      <Slider label="Width" value={layer.w} min={0.005} max={1.2} step={0.001} scale={100} unit="%" on:input={num("w")} />
      <Slider label="Height" value={layer.h} min={0.005} max={1.2} step={0.001} scale={100} unit="%" on:input={num("h")} />
    </div>
    <div class="two">
      <Slider label="Left / right" value={layer.x} scale={100} unit="%" on:input={num("x")} />
      <Slider label="Up / down" value={layer.y} scale={100} unit="%" on:input={num("y")} />
    </div>
    <Slider label="Rotation" value={layer.rotation} min={-180} max={180} step={1} unit="°" on:input={num("rotation")} />
    {#if layer.shape === "rect"}<Slider label="Rounded corners" value={layer.radius} min={0} max={1} scale={100} unit="%" on:input={num("radius")} />{/if}
    <ColorField label="Fill" value={layer.fill} on:change={(e) => e.detail && set("fill", e.detail)} />
    <ColorField label="Fade fill into" value={layer.fill2} allowNone on:change={col("fill2")} />
    {#if layer.fill2}<Slider label="Fill direction" value={layer.gradientAngle} min={0} max={360} step={1} unit="°" on:input={num("gradientAngle")} />{/if}
    <Slider label="Outline" value={layer.strokeWidth} min={0} max={0.03} step={0.0005} scale={100} unit="%" on:input={num("strokeWidth")} />
    {#if layer.strokeWidth > 0}<ColorField label="Outline colour" value={layer.strokeColor} on:change={col("strokeColor")} />{/if}

  {:else if layer.type === "flag"}
    <FlagFields {layer} {countries} {split} />

  {:else if layer.type === "pattern"}
    <Segmented label="Pattern" value={layer.kind} on:change={(e) => set("kind", e.detail)}
      options={[{ id: "stripes", label: "Stripes" }, { id: "pinstripes", label: "Pinstripes" }, { id: "dots", label: "Dots" }, { id: "checks", label: "Checks" }, { id: "zigzag", label: "Zig-zag" }]} />
    <ColorField label="Colour" value={layer.color} on:change={(e) => e.detail && set("color", e.detail)} />
    <ColorField label="Background" value={layer.background} allowNone on:change={col("background")} />
    <Slider label="Spacing" value={layer.spacing} min={0.005} max={0.25} step={0.001} scale={100} unit="%" on:input={num("spacing")} />
    <Slider label="Thickness" value={layer.thickness} min={0.001} max={0.15} step={0.001} scale={100} unit="%" on:input={num("thickness")} />
    <Slider label="Angle (0 = vertical stripes, 90 = hoops)" value={layer.angle} min={-90} max={90} step={1} unit="°" on:input={num("angle")} />
    {#if split}<Segmented label="On" value={layer.panel} on:change={(e) => set("panel", e.detail)} options={panelOpts} />{/if}

  {:else if layer.type === "gradient"}
    <Segmented label="Kind" value={layer.kind} on:change={(e) => set("kind", e.detail)} options={[{ id: "linear", label: "Linear" }, { id: "radial", label: "Radial" }]} />
    <ColorField label="From" value={layer.from} on:change={(e) => e.detail && set("from", e.detail)} />
    <Slider label="From opacity" value={layer.fromAlpha} scale={100} unit="%" on:input={num("fromAlpha")} />
    <ColorField label="To" value={layer.to} on:change={(e) => e.detail && set("to", e.detail)} />
    <Slider label="To opacity" value={layer.toAlpha} scale={100} unit="%" on:input={num("toAlpha")} />
    {#if layer.kind === "linear"}<Slider label="Direction (180 = top to bottom)" value={layer.angle} min={0} max={360} step={1} unit="°" on:input={num("angle")} />{/if}
    <div class="two">
      <Slider label="Starts at" value={layer.start} scale={100} unit="%" on:input={num("start")} />
      <Slider label="Ends at" value={layer.end} scale={100} unit="%" on:input={num("end")} />
    </div>
    {#if split}<Segmented label="On" value={layer.panel} on:change={(e) => set("panel", e.detail)} options={panelOpts} />{/if}

  {:else if layer.type === "replace"}
    <p class="hint">Click the eyedropper next to "Find", then click the colour on the kit you want to change.</p>
    <ColorField label="Find" value={layer.from} on:change={(e) => e.detail && set("from", e.detail)} />
    <ColorField label="Replace with" value={layer.to} on:change={(e) => e.detail && set("to", e.detail)} />
    <Slider label="Tolerance" value={layer.tolerance} min={0} max={100} step={1} on:input={num("tolerance")} />
    <Slider label="Soft edge" value={layer.softness} min={0} max={100} step={1} on:input={num("softness")} />
    <Segmented label="Match" value={layer.match ?? "colour"} on:change={(e) => set("match", e.detail)}
      options={[{ id: "colour", label: "Similar colour", title: "Pixels close to the Find colour" }, { id: "hue", label: "All shades", title: "Every light and dark shade of the Find colour's hue, e.g. a whole gold logo" }]} />
    <Segmented label="Shading" value={layer.keepShading} on:change={(e) => set("keepShading", e.detail)} options={[{ id: true, label: "Keep texture" }, { id: false, label: "Flat colour" }]} />
    {#if layer.keepShading}<Slider label="Texture kept" value={layer.texture ?? 1} min={0} max={1.5} scale={100} unit="%" on:input={num("texture")} />{/if}

  {:else if layer.type === "adjust"}
    <Slider label="Hue" value={layer.hue} min={-180} max={180} step={1} unit="°" on:input={num("hue")} />
    <Slider label="Saturation" value={layer.saturation} min={0} max={300} step={1} unit="%" on:input={num("saturation")} />
    <Slider label="Brightness" value={layer.brightness} min={0} max={250} step={1} unit="%" on:input={num("brightness")} />
    <Slider label="Contrast" value={layer.contrast} min={0} max={250} step={1} unit="%" on:input={num("contrast")} />
    <Slider label="Greyscale" value={layer.grayscale} min={0} max={100} step={1} unit="%" on:input={num("grayscale")} />
    <Slider label="Invert" value={layer.invert} min={0} max={100} step={1} unit="%" on:input={num("invert")} />
    <Slider label="Blur" value={layer.blur} min={0} max={20} step={0.5} unit="px" on:input={num("blur")} />

  {:else if layer.type === "patch"}
    <p class="hint">Drag the yellow box over what you want to hide (e.g. the old logo). Drag the blue box to a plain part of the kit: that area is copied over it.</p>
    <div class="two">
      <Slider label="Left / right" value={layer.x} scale={100} unit="%" on:input={num("x")} />
      <Slider label="Up / down" value={layer.y} scale={100} unit="%" on:input={num("y")} />
      <Slider label="Width" value={layer.w} min={0.01} max={0.6} step={0.001} scale={100} unit="%" on:input={num("w")} />
      <Slider label="Height" value={layer.h} min={0.01} max={0.6} step={0.001} scale={100} unit="%" on:input={num("h")} />
      <Slider label="Copy from: sideways" value={layer.dx} min={-1} max={1} step={0.001} scale={100} unit="%" on:input={num("dx")} />
      <Slider label="Copy from: up / down" value={layer.dy} min={-1} max={1} step={0.001} scale={100} unit="%" on:input={num("dy")} />
    </div>
    <Slider label="Soft edges" value={layer.feather} min={0} max={1} scale={100} unit="%" on:input={num("feather")} />
    <Segmented label="Shape" value={layer.ellipse} on:change={(e) => set("ellipse", e.detail)} options={[{ id: false, label: "Box" }, { id: true, label: "Oval" }]} />

  {:else if layer.type === "brush"}
    <p class="hint">Pick the Brush tool above the kit and paint. Brush colour, size and eraser are in the toolbar.</p>
    <p class="muted small">{layer.strokes.length} stroke{layer.strokes.length === 1 ? "" : "s"}</p>
    {#if layer.strokes.length}<button class="btn btn-sm" on:click={() => ed.change(() => { layer.type === "brush" && (layer.strokes = []); })}>Clear painting</button>{/if}
  {/if}

  <div class="common">
    <Slider label="Layer opacity" value={layer.opacity} scale={100} unit="%" on:input={num("opacity")} />
    <label class="field"><span>Blend</span>
      <select value={layer.blend} on:change={(e) => set("blend", e.currentTarget.value)}>
        {#each BLENDS as b}<option value={b.id}>{b.label}</option>{/each}
      </select>
    </label>
    <label class="chk"><input type="checkbox" checked={layer.clip} on:change={(e) => set("clip", e.currentTarget.checked)} /> Keep on the kit (off the grey template area)</label>
    <label class="chk"><input type="checkbox" checked={!!layer.area} on:change={(e) => set("area", e.currentTarget.checked ? { x: split ? 0.25 : 0.5, y: 0.3, w: 0.15, h: 0.15, feather: 0.1, ellipse: false } : null)} /> Only in an area (drag the pink box on the kit)</label>
    {#if layer.area}
      <div class="two">
        <Slider label="Area left / right" value={layer.area.x} scale={100} unit="%" on:input={(e) => setArea("x", e.detail)} />
        <Slider label="Area up / down" value={layer.area.y} scale={100} unit="%" on:input={(e) => setArea("y", e.detail)} />
        <Slider label="Area width" value={layer.area.w} min={0.01} max={1} step={0.001} scale={100} unit="%" on:input={(e) => setArea("w", e.detail)} />
        <Slider label="Area height" value={layer.area.h} min={0.01} max={1} step={0.001} scale={100} unit="%" on:input={(e) => setArea("h", e.detail)} />
      </div>
      <div class="two">
        <Slider label="Area soft edge" value={layer.area.feather} min={0} max={1} scale={100} unit="%" on:input={(e) => setArea("feather", e.detail)} />
        <Segmented label="Area shape" value={layer.area.ellipse} on:change={(e) => setArea("ellipse", e.detail)} options={[{ id: false, label: "Box" }, { id: true, label: "Oval" }]} />
      </div>
    {/if}
  </div>
</div>

<style>
  .props { display: flex; flex-direction: column; gap: 10px; }
  .head { display: flex; align-items: baseline; gap: 8px; }
  .name { flex: 1; font-family: var(--font-cond); font-size: 16px; font-weight: 600; background: transparent; border: 1px solid transparent; border-radius: 4px; padding: 2px 4px; color: var(--ink); }
  .name:hover, .name:focus { border-color: var(--rule-2); background: var(--bg); }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  textarea { resize: vertical; font: inherit; font-size: 13px; }
  .common { display: flex; flex-direction: column; gap: 8px; padding-top: 10px; border-top: 1px solid var(--rule); }
  .chk { display: flex; gap: 6px; align-items: center; font-size: 12px; color: var(--ink-2); }
  .hint { font-size: 12px; color: var(--ink-3); }
  .small { font-size: 12px; }
</style>
