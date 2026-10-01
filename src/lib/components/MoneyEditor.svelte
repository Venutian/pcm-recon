<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { eur, signedEur, parseMoney } from "../format";

  /** Adjust a euro amount by steps or set it outright; emits `apply` with the target. */
  export let current: number;
  export let steps: number[] = [-1_000_000, -100_000, 100_000, 500_000, 1_000_000, 5_000_000];
  export let presets: { label: string; value: number }[] = [];
  export let min = -100_000_000;
  export let max = 500_000_000;
  export let applyLabel = "Review change";

  const dispatch = createEventDispatcher<{ apply: number }>();
  let target = current;
  let text = "";
  let lastCurrent = current;
  $: if (current !== lastCurrent) { lastCurrent = current; target = current; text = ""; }

  $: delta = target - current;
  $: invalid = target < min || target > max;
  function add(v: number) { target = Math.round(target + v); text = ""; }
  /** "+1m" adds to the current amount; anything else (including "-250k") is an absolute target. */
  function typed() {
    const raw = text.trim();
    if (!raw) { target = current; return; }
    const relative = raw.startsWith("+");
    const v = parseMoney(relative ? raw.slice(1) : raw);
    if (v !== null) target = relative ? current + v : v;
  }
</script>

<div class="ed">
  <div class="steps">
    {#each steps as s}
      <button class="btn btn-sm" class:minus={s < 0} on:click={() => add(s)}>{signedEur(s)}</button>
    {/each}
  </div>
  <div class="set">
    <label class="field grow">
      <span>Set an exact amount (e.g. 2.5m, 750k, +1m)</span>
      <input bind:value={text} on:input={typed} placeholder={eur(current)} spellcheck="false" />
    </label>
    {#each presets as p}
      <button class="btn btn-sm btn-quiet" on:click={() => { target = p.value; text = ""; }}>{p.label}</button>
    {/each}
  </div>
  <div class="preview">
    <div><small>Now</small><span class:neg={current < 0}>{eur(current)}</span></div>
    <div class="arrow">→</div>
    <div><small>After</small><strong class:neg={target < 0} class:pos={target >= 0 && delta !== 0}>{eur(target)}</strong></div>
    <div class="d" class:pos={delta > 0} class:neg={delta < 0}>{delta === 0 ? "No change" : signedEur(delta)}</div>
    <div class="go">
      {#if delta !== 0}<button class="btn btn-quiet btn-sm" on:click={() => { target = current; text = ""; }}>Reset</button>{/if}
      <button class="btn btn-primary" disabled={delta === 0 || invalid} on:click={() => dispatch("apply", target)}>{applyLabel}</button>
    </div>
  </div>
  {#if invalid}<p class="warn">Keep it between {eur(min)} and {eur(max)} so the game handles it safely.</p>{/if}
</div>

<style>
  .ed { display: flex; flex-direction: column; gap: 12px; }
  .steps { display: flex; flex-wrap: wrap; gap: 6px; }
  .steps .btn { min-width: 64px; color: var(--vert); }
  .steps .btn.minus { color: var(--pois); }
  .set { display: flex; gap: 8px; align-items: flex-end; flex-wrap: wrap; }
  .grow { flex: 1 1 240px; }
  .preview { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; padding: 10px 12px; background: var(--bg-2); border-radius: 6px; }
  .preview small { display: block; font-size: 11px; color: var(--ink-3); }
  .preview span, .preview strong { font-family: var(--font-cond); font-size: 20px; font-weight: 700; }
  .arrow { color: var(--ink-4); }
  .d { font-size: 13px; color: var(--ink-3); }
  .go { margin-left: auto; display: flex; gap: 6px; }
  .pos { color: var(--vert); }
  .neg { color: var(--pois); }
  .warn { font-size: 12px; color: var(--pois); }
</style>
