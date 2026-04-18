<script lang="ts">
  export let data: { label: string; value: number; color?: string }[] = [];
  export let title = "";
  export let defaultColor = "#e8b800";

  let width = 400;
  let height = 300;

  $: maxVal = Math.max(...data.map(d => d.value), 1);
  $: rowH   = data.length > 0 ? Math.max(18, Math.floor((height - 34) / data.length)) : 24;
  $: padL   = 140;
  $: padR   = 50;
</script>

<div class="wrap" bind:clientWidth={width} bind:clientHeight={height}>
  <svg {width} {height}>
    {#if title}
      <text x={padL} y="18" fill="#6478a0" font-size="10" font-weight="700" font-family="Segoe UI">{title}</text>
    {/if}
    {#each data as d, i}
      {@const y   = 30 + i * rowH}
      {@const bw  = Math.max(0, ((d.value / maxVal) * (width - padL - padR)))}
      {@const col = d.color ?? defaultColor}
      <!-- track -->
      <rect x={padL} y={y+2} width={width-padL-padR} height={rowH-4} fill="#111c30" rx="3"/>
      <!-- bar -->
      {#if bw > 0}
        <rect x={padL} y={y+2} width={bw} height={rowH-4} fill={col} rx="3"/>
      {/if}
      <!-- label -->
      <text x={padL-6} y={y + rowH/2 + 4} fill="#6478a0" font-size="11"
            font-family="Segoe UI" text-anchor="end">{d.label}</text>
      <!-- value -->
      <text x={padL+bw+5} y={y + rowH/2 + 4} fill="#dce8ff" font-size="11"
            font-family="Segoe UI" font-weight="700">{d.value}</text>
    {/each}
  </svg>
</div>

<style>
  .wrap { width: 100%; height: 100%; }
  svg { display: block; }
</style>
