<script lang="ts">
  export let label = "";
  export let cur   = 0;
  export let pot   = 0;

  function barColor(v: number) {
    if (v >= 90) return "#ffd700";
    if (v >= 80) return "#c8e857";
    if (v >= 70) return "#2ecc82";
    if (v >= 55) return "#e8b800";
    return "#4a5e80";
  }
  $: col = barColor(cur);
</script>

<div class="row">
  <span class="lbl">{label}</span>
  <div class="bar-wrap">
    {#if pot > 0}
      <div class="bar-pot" style="width:{pot}%"></div>
    {/if}
    <div class="bar-cur" style="width:{cur}%; background:{col}"></div>
  </div>
  <span class="val" style="color:{col}">{cur}<span class="pot-lbl">/{pot}</span></span>
</div>

<style>
  .row { display: flex; align-items: center; gap: 8px; margin: 3px 0; }
  .lbl { font-size: 11px; color: #6478a0; width: 110px; flex-shrink: 0; text-align: right; }
  .bar-wrap { flex: 1; height: 10px; background: #111c30; border-radius: 5px; position: relative; overflow: hidden; }
  .bar-pot { position: absolute; height: 100%; background: #1e2d4a; border-radius: 5px; }
  .bar-cur { position: absolute; height: 100%; border-radius: 5px; transition: width 0.3s ease; }
  .val { font-size: 11px; font-weight: 700; width: 46px; text-align: right; flex-shrink: 0; }
  .pot-lbl { color: #3a4e72; font-weight: 400; }
</style>
