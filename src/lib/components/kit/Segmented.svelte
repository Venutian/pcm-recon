<script lang="ts">
  import { createEventDispatcher } from "svelte";

  type T = $$Generic<string | number | boolean>;
  export let label = "";
  export let options: { id: T; label: string; title?: string }[];
  export let value: T;

  const dispatch = createEventDispatcher<{ change: T }>();
</script>

<div class="seg">
  {#if label}<span>{label}</span>{/if}
  <div class="opts" role="radiogroup" aria-label={label}>
    {#each options as o}
      <button role="radio" aria-checked={o.id === value} class:on={o.id === value} title={o.title ?? o.label} on:click={() => dispatch("change", o.id)}>{o.label}</button>
    {/each}
  </div>
</div>

<style>
  .seg { display: flex; flex-direction: column; gap: 3px; }
  span { font-size: 11.5px; color: var(--ink-3); }
  .opts { display: flex; flex-wrap: wrap; border: 1px solid var(--rule-2); border-radius: 6px; overflow: hidden; }
  button { flex: 1; padding: 4px 6px; font: inherit; font-size: 12px; background: var(--panel); color: var(--ink-2); border: none; border-right: 1px solid var(--rule); cursor: pointer; white-space: nowrap; }
  button:last-child { border-right: none; }
  button:hover { color: var(--ink); }
  button.on { background: var(--jaune); color: var(--jaune-ink); font-weight: 600; }
</style>
