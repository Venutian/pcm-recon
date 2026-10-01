<script lang="ts">
  import { statColor } from "../format";
  export let label = "";
  export let cur = 0;
  export let pot = 0;
  export let best = false;
  // Stats below 40 are rare, so the track starts there to make differences visible.
  const LO = 40;
  const pct = (v: number) => Math.max(0, Math.min(100, ((v - LO) / (100 - LO)) * 100));
  $: col = statColor(cur);
</script>

<div class="row" class:best>
  <span class="lbl">{label}</span>
  <div class="track">
    {#if pot > cur}<div class="pot" style="width:{pct(pot)}%"></div>{/if}
    <div class="cur" style="width:{pct(cur)}%; background:{col}"></div>
  </div>
  <span class="val">{cur}</span>
  <span class="ceil" class:up={pot > cur}>{pot > cur ? `→ ${pot}` : ""}</span>
</div>

<style>
  .row { display: grid; grid-template-columns: 112px 1fr 26px 40px; align-items: center; gap: 8px; height: 22px; }
  .lbl { font-size: 12px; color: var(--ink-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .best .lbl { color: var(--ink); font-weight: 600; }
  .track { position: relative; height: 6px; background: var(--bg-2); border-radius: 3px; overflow: hidden; }
  .pot { position: absolute; inset: 0 auto 0 0; background: repeating-linear-gradient(135deg, #3a4558 0 3px, #283140 3px 6px); border-radius: 3px; }
  .cur { position: absolute; inset: 0 auto 0 0; border-radius: 3px; }
  .val { font-family: var(--font-cond); font-weight: 700; font-size: 15px; text-align: right; }
  .ceil { font-size: 11px; color: var(--ink-4); white-space: nowrap; }
  .ceil.up { color: var(--vert); }
</style>
