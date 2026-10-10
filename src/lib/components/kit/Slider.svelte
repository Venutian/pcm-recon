<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let label: string;
  export let value: number;
  export let min = 0;
  export let max = 1;
  export let step = 0.01;
  /** Shown value = value × scale (e.g. 100 for percentages). */
  export let scale = 1;
  export let unit = "";

  const dispatch = createEventDispatcher<{ input: number }>();
  $: shown = Math.round(value * scale * 10) / 10;
  const decimals = (n: number) => (String(n).split(".")[1] ?? "").length;

  function fromRange(e: Event) {
    dispatch("input", Number((e.currentTarget as HTMLInputElement).value));
  }
  function fromBox(e: Event) {
    const v = Number((e.currentTarget as HTMLInputElement).value) / scale;
    if (Number.isFinite(v)) dispatch("input", Math.min(max, Math.max(min, Number(v.toFixed(decimals(step) + 2)))));
  }
</script>

<div class="slider">
  <div class="top">
    <span>{label}</span>
    <label class="num"><input type="number" value={shown} step={step * scale} on:change={fromBox} aria-label={label} />{unit}</label>
  </div>
  <input type="range" {min} {max} {step} {value} on:input={fromRange} aria-label={label} />
</div>

<style>
  .slider { display: flex; flex-direction: column; gap: 2px; }
  .top { display: flex; justify-content: space-between; align-items: center; font-size: 11.5px; color: var(--ink-3); }
  .num { display: inline-flex; align-items: center; gap: 2px; }
  .num input { width: 54px; padding: 1px 4px; font-size: 11.5px; text-align: right; background: var(--bg); border: 1px solid var(--rule); border-radius: 4px; color: var(--ink-2); }
  input[type="range"] { width: 100%; }
</style>
