<script lang="ts">
  import { eur, eurShort, fmtDate } from "../format";

  /** Balance over time, drawn like a stage elevation profile. */
  export let points: { date: string; value: number; note?: string }[] = [];
  export let height = 220;

  let width = 600;
  let hover: number | null = null;

  const PAD = { l: 58, r: 14, t: 14, b: 26 };
  $: ts = points.map((p) => new Date(p.date + "T00:00:00").getTime());
  $: t0 = Math.min(...ts);
  $: t1 = Math.max(...ts, t0 + 1);
  $: vals = points.map((p) => p.value);
  $: rawLo = Math.min(0, ...vals);
  $: rawHi = Math.max(0, ...vals);
  $: step = niceStep((rawHi - rawLo) / 4 || 1);
  $: lo = Math.floor(rawLo / step) * step;
  $: hi = Math.ceil(rawHi / step) * step || step;
  $: ticks = Array.from({ length: Math.round((hi - lo) / step) + 1 }, (_, i) => lo + i * step);

  function niceStep(raw: number) {
    const p = Math.pow(10, Math.floor(Math.log10(raw)));
    const f = raw / p;
    return (f <= 1 ? 1 : f <= 2 ? 2 : f <= 2.5 ? 2.5 : f <= 5 ? 5 : 10) * p;
  }
  $: x = (t: number) => PAD.l + ((t - t0) / (t1 - t0)) * (width - PAD.l - PAD.r);
  $: y = (v: number) => PAD.t + (1 - (v - lo) / (hi - lo)) * (height - PAD.t - PAD.b);
  $: line = points.map((p, i) => `${i ? "L" : "M"}${x(ts[i]).toFixed(1)},${y(p.value).toFixed(1)}`).join("");
  $: zeroY = y(0);
  $: area = points.length ? `${line}L${x(ts[ts.length - 1])},${zeroY}L${x(ts[0])},${zeroY}Z` : "";
  $: months = monthTicks(t0, t1);

  function monthTicks(a: number, b: number) {
    const out: { t: number; label: string }[] = [];
    if (!isFinite(a) || !isFinite(b)) return out;
    const d = new Date(a); d.setDate(1); d.setMonth(d.getMonth() + 1);
    const span = (b - a) / (30 * 864e5);
    const every = span > 14 ? 3 : span > 7 ? 2 : 1;
    while (d.getTime() <= b) {
      if (d.getMonth() % every === 0) out.push({ t: d.getTime(), label: d.toLocaleDateString("en-GB", { month: "short", year: d.getMonth() === 0 ? "2-digit" : undefined }) });
      d.setMonth(d.getMonth() + 1);
    }
    return out;
  }

  function move(e: MouseEvent) {
    const rect = (e.currentTarget as SVGElement).getBoundingClientRect();
    const mx = e.clientX - rect.left;
    let best = 0, bd = Infinity;
    ts.forEach((t, i) => { const d = Math.abs(x(t) - mx); if (d < bd) { bd = d; best = i; } });
    hover = best;
  }
</script>

<div class="wrap" bind:clientWidth={width}>
  {#if points.length < 2}
    <div class="empty">Not enough ledger history yet to draw the balance profile.</div>
  {:else}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <svg {width} {height} on:mousemove={move} on:mouseleave={() => (hover = null)} role="img" aria-label="Balance over time">
      <defs>
        <linearGradient id="above" x1="0" x2="0" y1="0" y2="1">
          <stop offset="0" stop-color="#3dbe6e" stop-opacity="0.35" />
          <stop offset="1" stop-color="#3dbe6e" stop-opacity="0.04" />
        </linearGradient>
        <linearGradient id="below" x1="0" x2="0" y1="0" y2="1">
          <stop offset="0" stop-color="#e8524a" stop-opacity="0.05" />
          <stop offset="1" stop-color="#e8524a" stop-opacity="0.35" />
        </linearGradient>
        <clipPath id="clip-above"><rect x="0" y="0" width={width} height={Math.max(0, zeroY)} /></clipPath>
        <clipPath id="clip-below"><rect x="0" y={zeroY} width={width} height={Math.max(0, height - zeroY)} /></clipPath>
      </defs>

      {#each ticks as t}
        <line x1={PAD.l} x2={width - PAD.r} y1={y(t)} y2={y(t)} class:zero={t === 0} class="grid" />
        <text x={PAD.l - 8} y={y(t) + 4} text-anchor="end" class="axis">{eurShort(t)}</text>
      {/each}
      {#each months as m}
        <text x={x(m.t)} y={height - 8} text-anchor="middle" class="axis">{m.label}</text>
      {/each}

      <path d={area} fill="url(#above)" clip-path="url(#clip-above)" />
      <path d={area} fill="url(#below)" clip-path="url(#clip-below)" />
      <path d={line} fill="none" stroke="var(--ink)" stroke-width="2" stroke-linejoin="round" />

      {#if hover !== null}
        {@const hx = x(ts[hover])}
        {@const hy = y(points[hover].value)}
        <line x1={hx} x2={hx} y1={PAD.t} y2={height - PAD.b} class="cross" />
        <circle cx={hx} cy={hy} r="5" fill={points[hover].value < 0 ? "var(--pois)" : "var(--vert)"} stroke="var(--panel)" stroke-width="2" />
      {/if}
      <!-- final point: where you are now -->
      <circle cx={x(ts[ts.length - 1])} cy={y(vals[vals.length - 1])} r="4" fill="var(--jaune)" stroke="var(--panel)" stroke-width="2" />
    </svg>
    {#if hover !== null}
      {@const p = points[hover]}
      <div class="tip" style="left:{Math.min(width - 190, Math.max(0, x(ts[hover]) - 90))}px">
        <div class="tip-date">{fmtDate(p.date)}</div>
        <div class="tip-val" class:neg={p.value < 0}>{eur(p.value)}</div>
        {#if p.note}<div class="tip-note">{p.note}</div>{/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .wrap { position: relative; width: 100%; }
  svg { display: block; }
  .grid { stroke: var(--rule); stroke-width: 1; }
  .grid.zero { stroke: var(--ink-3); stroke-dasharray: 3 3; }
  .axis { fill: var(--ink-3); font-size: 11px; font-family: var(--font); }
  .cross { stroke: var(--ink-3); stroke-width: 1; }
  .tip {
    position: absolute; top: 4px; width: 180px; pointer-events: none;
    background: var(--panel-2); border: 1px solid var(--rule-2); border-radius: 6px; padding: 7px 10px;
    box-shadow: var(--shadow-pop);
  }
  .tip-date { font-size: 11px; color: var(--ink-3); }
  .tip-val { font-family: var(--font-cond); font-weight: 700; font-size: 20px; color: var(--vert); }
  .tip-val.neg { color: var(--pois); }
  .tip-note { font-size: 11px; color: var(--ink-2); }
  .empty { padding: 30px; text-align: center; color: var(--ink-3); }
</style>
