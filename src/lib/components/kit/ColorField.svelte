<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { useEditor, recentColors, rememberColor, screenEyedropper } from "../../kits/editor";
  import Icon from "../Icon.svelte";

  export let label: string;
  export let value: string | null;
  /** Offer "none" (e.g. a pattern without background). */
  export let allowNone = false;

  const dispatch = createEventDispatcher<{ change: string | null }>();
  const ed = useEditor();
  $: swatches = [...new Set([...(ed?.teamColors ?? []), "#ffffff", "#000000", ...$recentColors].map((c) => c.toLowerCase()))].slice(0, 12);

  function set(v: string | null) {
    if (v) rememberColor(v.toLowerCase());
    dispatch("change", v ? v.toLowerCase() : null);
  }
  async function eyedrop() {
    if (ed) ed.pickColor((hex) => set(hex));
    else {
      const hex = await screenEyedropper();
      if (hex) set(hex);
    }
  }
</script>

<div class="color">
  <div class="top">
    <span>{label}</span>
    {#if allowNone}<label class="none"><input type="checkbox" checked={value === null} on:change={(e) => set(e.currentTarget.checked ? null : "#ffffff")} /> none</label>{/if}
  </div>
  {#if value !== null}
    <div class="row">
      <input type="color" value={value} on:input={(e) => set(e.currentTarget.value)} aria-label={label} />
      <button class="pick" title="Pick a colour from the kit" on:click={eyedrop}><Icon name="eye" size={14} /></button>
      <div class="swatches">
        {#each swatches as s}
          <button class="sw" class:on={s === value?.toLowerCase()} style="background:{s}" title={s} on:click={() => set(s)}></button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .color { display: flex; flex-direction: column; gap: 4px; }
  .top { display: flex; justify-content: space-between; font-size: 11.5px; color: var(--ink-3); }
  .none { display: inline-flex; gap: 4px; align-items: center; }
  .row { display: flex; align-items: center; gap: 6px; }
  input[type="color"] { width: 34px; height: 26px; padding: 0; border: 1px solid var(--rule-2); border-radius: 4px; background: none; cursor: pointer; flex-shrink: 0; }
  .pick { width: 26px; height: 26px; display: grid; place-items: center; border: 1px solid var(--rule-2); border-radius: 4px; background: var(--panel); color: var(--ink-2); cursor: pointer; flex-shrink: 0; }
  .pick:hover { color: var(--jaune); border-color: var(--jaune); }
  .swatches { display: flex; flex-wrap: wrap; gap: 3px; }
  .sw { width: 16px; height: 16px; border-radius: 3px; border: 1px solid rgba(255, 255, 255, 0.18); cursor: pointer; padding: 0; }
  .sw.on { outline: 2px solid var(--jaune); outline-offset: 1px; }
</style>
