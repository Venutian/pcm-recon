<script lang="ts">
  export let labels: string[] = [];
  export let series: { label: string; values: number[]; color: string; dashed?: boolean }[] = [];
  export let min = 45;
  export let max = 85;
  export let size = 260;

  $: n = labels.length;
  $: r = size / 2 - 34;
  $: cx = size / 2;
  $: cy = size / 2;
  const angle = (i: number, count: number) => -Math.PI / 2 + (i * 2 * Math.PI) / count;
  function pt(i: number, v: number) {
    const t = Math.max(0, Math.min(1, (v - min) / (max - min)));
    const a = angle(i, n);
    return [cx + Math.cos(a) * r * t, cy + Math.sin(a) * r * t];
  }
  $: rings = [0.25, 0.5, 0.75, 1].map((f) => labels.map((_, i) => {
    const a = angle(i, n);
    return `${cx + Math.cos(a) * r * f},${cy + Math.sin(a) * r * f}`;
  }).join(" "));
  let hover = -1;
</script>

<svg width={size} height={size} viewBox="0 0 {size} {size}" role="img" aria-label="Rider profile: {labels.join(', ')}">
  {#each rings as ring}
    <polygon points={ring} fill="none" stroke="var(--rule)" stroke-width="1" />
  {/each}
  {#each labels as l, i}
    {@const a = angle(i, n)}
    <line x1={cx} y1={cy} x2={cx + Math.cos(a) * r} y2={cy + Math.sin(a) * r} stroke="var(--rule)" />
    <text
      x={cx + Math.cos(a) * (r + 16)} y={cy + Math.sin(a) * (r + 16) + 4}
      text-anchor={Math.abs(Math.cos(a)) < 0.3 ? "middle" : Math.cos(a) > 0 ? "start" : "end"}
      class="lbl" class:hl={hover === i}>{l}</text>
  {/each}
  {#each series as s}
    <polygon points={labels.map((_, i) => pt(i, s.values[i]).join(",")).join(" ")}
             fill={s.dashed ? "none" : s.color} fill-opacity={s.dashed ? 0 : 0.16}
             stroke={s.color} stroke-width="2" stroke-dasharray={s.dashed ? "4 3" : ""} stroke-linejoin="round" />
  {/each}
  {#each series as s}
    {#if !s.dashed}
      {#each labels as _, i}
        {@const p = pt(i, s.values[i])}
        <circle cx={p[0]} cy={p[1]} r="3" fill={s.color} stroke="var(--panel)" stroke-width="2" />
      {/each}
    {/if}
  {/each}
  <!-- invisible hit wedges for per-axis tooltips -->
  {#each labels as l, i}
    {@const a = angle(i, n)}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <circle cx={cx + Math.cos(a) * r * 0.7} cy={cy + Math.sin(a) * r * 0.7} r={r * 0.32} fill="transparent"
            on:mouseenter={() => (hover = i)} on:mouseleave={() => (hover = -1)}>
      <title>{l}: {series.map((s) => `${s.label} ${Math.round(s.values[i])}`).join(" · ")}</title>
    </circle>
  {/each}
</svg>

<style>
  svg { display: block; overflow: visible; }
  .lbl { fill: var(--ink-3); font-size: 11px; font-family: var(--font); }
  .lbl.hl { fill: var(--ink); }
</style>
