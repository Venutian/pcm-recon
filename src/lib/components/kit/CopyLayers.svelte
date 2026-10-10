<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { toast } from "../../stores";
  import { kitSaveEdit } from "../../api";
  import { cloneLayer, emptyRecipe, migrate, LAYER_INFO, type Layer } from "../../kits/layers";
  import { render as renderKit, imageData, isSplit } from "../../kits/render";
  import { baseImage, findEdit, invalidate, refreshEdits } from "../../kits/store";
  import { partLabel } from "../../kits/parts";
  import Modal from "../Modal.svelte";
  import Icon from "../Icon.svelte";

  export let kit: string;
  export let part: string;
  export let layers: Layer[];
  export let selected: string | null;
  /** Every part of the kit, including champion jerseys made here. */
  export let parts: string[];
  export let countryName: (code: string) => string | undefined = () => undefined;

  const dispatch = createEventDispatcher<{ close: void }>();
  let picked = new Set(layers.filter((l) => (selected ? l.id === selected : true)).map((l) => l.id));
  // Positions are fractions of the template, so only parts with the same layout make sense.
  const targets = parts.filter((p) => p !== part && isSplit(p) === isSplit(part));
  let chosen = new Set<string>();
  let busy = "";

  function toggle(set: Set<string>, id: string) {
    set.has(id) ? set.delete(id) : set.add(id);
    return set;
  }

  async function copy() {
    const list = layers.filter((l) => picked.has(l.id));
    try {
      for (const [i, t] of [...chosen].entries()) {
        busy = `Copying to ${partLabel(t, countryName)} (${i + 1}/${chosen.size})…`;
        const saved = findEdit(kit, t);
        const recipe = saved ? migrate(saved.recipe) : emptyRecipe();
        recipe.layers.push(...list.map((l) => cloneLayer(l)));
        const img = imageData(await renderKit(await baseImage(kit, t, recipe), recipe, t));
        await kitSaveEdit(kit, t, recipe, img);
        invalidate(kit, t);
      }
      await refreshEdits();
      toast(`Copied ${list.length} layer${list.length === 1 ? "" : "s"} to ${chosen.size} part${chosen.size === 1 ? "" : "s"}`);
      dispatch("close");
    } catch (e) {
      toast(String(e), "error", 8000);
    } finally {
      busy = "";
    }
  }
</script>

<Modal title="Copy layers to other parts" width={560} on:close={() => !busy && dispatch("close")}>
  <div class="cols">
    <div>
      <h4>Layers</h4>
      {#each [...layers].reverse() as l (l.id)}
        <label class="chk"><input type="checkbox" checked={picked.has(l.id)} on:change={() => (picked = toggle(picked, l.id))} /><Icon name={LAYER_INFO[l.type].icon} size={13} />{l.name}</label>
      {/each}
    </div>
    <div>
      <h4>Copy to</h4>
      {#each targets as t}
        <label class="chk"><input type="checkbox" checked={chosen.has(t)} on:change={() => (chosen = toggle(chosen, t))} />{partLabel(t, countryName)}</label>
      {:else}
        <p class="muted">No other part uses this template.</p>
      {/each}
    </div>
  </div>
  <p class="muted small">The copies are added on top of what each part already has, and saved right away. Apply to game afterwards.</p>
  <div class="row">
    <button class="btn" on:click={() => dispatch("close")} disabled={!!busy}>Cancel</button>
    <button class="btn btn-primary" on:click={copy} disabled={!!busy || !picked.size || !chosen.size}>{busy || "Copy"}</button>
  </div>
</Modal>

<style>
  .cols { display: grid; grid-template-columns: 1fr 1fr; gap: 18px; max-height: 50vh; overflow-y: auto; }
  h4 { font-family: var(--font-cond); font-size: 15px; margin-bottom: 6px; }
  .chk { display: flex; align-items: center; gap: 7px; font-size: 13px; padding: 3px 0; color: var(--ink-2); }
  .chk :global(svg) { color: var(--ink-3); }
  .small { font-size: 12px; margin-top: 12px; }
  .row { display: flex; justify-content: flex-end; gap: 8px; margin-top: 14px; }
</style>
