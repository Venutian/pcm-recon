<script lang="ts">
  import type { FlagLayer } from "../../kits/layers";
  import { useEditor } from "../../kits/editor";
  import Slider from "./Slider.svelte";
  import Segmented from "./Segmented.svelte";
  import ColorField from "./ColorField.svelte";
  import Flag from "../Flag.svelte";

  export let layer: FlagLayer;
  /** Countries to choose from; leave empty to hide the picker (champion jerseys set it per nation). */
  export let countries: { flag: string; name: string }[] = [];
  export let split = true;

  const ed = useEditor();
  function set<K extends keyof FlagLayer>(k: K, v: FlagLayer[K]) {
    ed.change(() => { layer[k] = v; }, `${layer.id}:${String(k)}`);
  }
  function setCountry(flag: string) {
    const name = countries.find((c) => c.flag === flag)?.name ?? flag;
    ed.change(() => {
      if (layer.name === `${layer.country} flag`) layer.name = `${name} flag`;
      layer.flag = flag;
      layer.country = name;
    });
  }
</script>

{#if countries.length}
  <label class="field"><span>Country</span>
    <div class="country">
      <Flag code={layer.flag} size={16} />
      <select value={layer.flag} on:change={(e) => setCountry(e.currentTarget.value)}>
        {#each countries as c}<option value={c.flag}>{c.name}</option>{/each}
      </select>
    </div>
  </label>
{/if}

<Segmented label="Style" value={layer.style} on:change={(e) => set("style", e.detail)}
  options={[{ id: "fade", label: "Fade" }, { id: "band", label: "Band / sash" }, { id: "full", label: "Full" }, { id: "sleeves", label: "Sleeves" }]} />

<Segmented label="Orientation" value={layer.rotate} on:change={(e) => set("rotate", e.detail)}
  options={[{ id: false, label: "Horizontal", title: "The flag as it's drawn" }, { id: true, label: "Vertical", title: "Turned 90°: vertical stripes become horizontal and the other way round" }]} />

<div class="fitrow">
  <Segmented label="Fit" value={layer.fit} on:change={(e) => set("fit", e.detail)} options={[{ id: "stretch", label: "Stretch" }, { id: "cover", label: "Keep shape" }, { id: "repeat", label: "Repeat", title: "Tile the flag at its real proportions" }]} />
  <Segmented label="Mirror" value={layer.flipX} on:change={(e) => set("flipX", e.detail)} options={[{ id: false, label: "Off" }, { id: true, label: "On" }]} />
</div>

{#if layer.style === "fade"}
  <Segmented label="Fades towards" value={layer.direction} on:change={(e) => set("direction", e.detail)}
    options={[{ id: "down", label: "↓ Down" }, { id: "up", label: "↑ Up" }, { id: "right", label: "→ Right" }, { id: "left", label: "← Left" }]} />
  <Slider label="Solid until" value={layer.hold} min={0} max={0.9} scale={100} unit="%" on:input={(e) => ed.change(() => { layer.hold = e.detail; if (layer.length < layer.hold + 0.02) layer.length = Math.min(1, layer.hold + 0.02); }, `${layer.id}:hold`)} />
  <Slider label="Gone by" value={layer.length} min={0.02} max={1} scale={100} unit="%" on:input={(e) => ed.change(() => { layer.length = e.detail; if (layer.hold > layer.length - 0.02) layer.hold = Math.max(0, layer.length - 0.02); }, `${layer.id}:length`)} />
{:else if layer.style === "band"}
  <Slider label="Height on the kit" value={layer.pos} min={0} max={1} scale={100} unit="%" on:input={(e) => set("pos", e.detail)} />
  <Slider label="Thickness" value={layer.size} min={0.01} max={0.6} scale={100} unit="%" on:input={(e) => set("size", e.detail)} />
  <Slider label="Angle (0 = band, tilt for a sash)" value={layer.angle} min={-75} max={75} step={1} unit="°" on:input={(e) => set("angle", e.detail)} />
  <Slider label="Edge lines" value={layer.edgeWidth} min={0} max={0.03} step={0.001} scale={100} unit="%" on:input={(e) => set("edgeWidth", e.detail)} />
  {#if layer.edgeWidth > 0}<ColorField label="Edge colour" value={layer.edgeColor} on:change={(e) => e.detail && set("edgeColor", e.detail)} />{/if}
{/if}

{#if split}
  <Segmented label="On" value={layer.panel} on:change={(e) => set("panel", e.detail)} options={[{ id: "both", label: "Front + back" }, { id: "front", label: "Front" }, { id: "back", label: "Back" }]} />
{/if}

<style>
  .country { display: flex; align-items: center; gap: 8px; }
  .country select { flex: 1; }
  .fitrow { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 8px; }
</style>
