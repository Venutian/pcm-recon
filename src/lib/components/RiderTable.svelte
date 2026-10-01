<script lang="ts">
  import type { Cyclist, Col } from "../types";
  import { GRADE_COLOR, TYPE_COLOR } from "../types";
  import { eurShort, statColor, gemGap, teamColor, stat } from "../format";
  import { selectedId, selectRider, shortlist, teamById, meta } from "../stores";
  import Bib from "./Bib.svelte";
  import Stars from "./Stars.svelte";
  import Flag from "./Flag.svelte";

  export let data: Cyclist[] = [];
  export let cols: Col[] = [];
  export let rowHeight = 36;
  /** Initial sort; clicking a header overrides it. Empty = keep incoming order. */
  export let sortKey = "";
  export let sortDir: 1 | -1 = -1;
  export let empty = "No riders match these filters.";

  let container: HTMLDivElement;
  let scrollTop = 0;
  let viewHeight = 600;

  $: season = $meta?.season ?? 0;

  function valueOf(c: Cyclist, col: Col): number | string {
    if (col.value) return col.value(c);
    if (col.kind === "gem") return gemGap(c);
    return (c as unknown as Record<string, number | string>)[col.key];
  }

  $: rows = sortKey ? sortRows(data, sortKey, sortDir) : data;
  function sortRows(src: Cyclist[], key: string, dir: number): Cyclist[] {
    const col = cols.find((c) => c.key === key);
    const get = (c: Cyclist) => (col ? valueOf(c, col) : (c as unknown as Record<string, unknown>)[key]);
    return [...src].sort((a, b) => {
      const av = get(a), bv = get(b);
      if (typeof av === "number" && typeof bv === "number") {
        if (Number.isNaN(av) || Number.isNaN(bv)) return Number.isNaN(av) ? (Number.isNaN(bv) ? 0 : 1) : -1;
        return (av - bv) * dir || b.current_ability - a.current_ability;
      }
      return String(av ?? "").localeCompare(String(bv ?? "")) * dir;
    });
  }
  function headerClick(col: Col) {
    if (sortKey === col.key) sortDir = sortDir === -1 ? 1 : -1;
    else {
      sortKey = col.key;
      sortDir = col.kind === "rider" || col.kind === "team" || col.kind === "nation" || col.kind === "text" ? 1 : -1;
    }
  }

  $: start = Math.max(0, Math.floor(scrollTop / rowHeight) - 10);
  $: end = Math.min(rows.length, Math.ceil((scrollTop + viewHeight) / rowHeight) + 10);
  $: visible = rows.slice(start, end);
  $: width = cols.reduce((s, c) => s + c.width, 0);

  function resize(node: HTMLElement) {
    const ro = new ResizeObserver((e) => (viewHeight = e[0]?.contentRect.height ?? 600));
    ro.observe(node);
    return { destroy: () => ro.disconnect() };
  }

  function onKey(e: KeyboardEvent) {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const i = rows.findIndex((r) => r.id === $selectedId);
    const next = Math.max(0, Math.min(rows.length - 1, i + (e.key === "ArrowDown" ? 1 : -1)));
    if (!rows[next]) return;
    selectRider(rows[next].id);
    const top = next * rowHeight, bottom = top + rowHeight + rowHeight;
    if (top < container.scrollTop + rowHeight) container.scrollTop = top - rowHeight;
    else if (bottom > container.scrollTop + viewHeight) container.scrollTop = bottom - viewHeight;
  }
</script>

<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
<div class="host">
<div class="wrap" bind:this={container} on:scroll={() => (scrollTop = container.scrollTop)} use:resize tabindex="0" role="grid" aria-rowcount={rows.length} on:keydown={onKey}>
  <div class="head" style="min-width:{width}px">
    {#each cols as col}
      <button class="th {col.align ?? 'left'}" style="width:{col.width}px" title={col.title ?? ""} on:click={() => headerClick(col)} class:sorted={sortKey === col.key}>
        {col.label}{#if sortKey === col.key}<span class="arrow">{sortDir === -1 ? "↓" : "↑"}</span>{/if}
      </button>
    {/each}
  </div>

  {#if rows.length === 0}
    <div class="empty">{empty}</div>
  {:else}
    <div class="body" style="height:{rows.length * rowHeight}px; min-width:{width}px">
      {#each visible as c, i (c.id)}
        <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
        <div class="tr" class:sel={$selectedId === c.id} class:mine={c.is_mine}
             style="top:{(start + i) * rowHeight}px; height:{rowHeight}px" on:click={() => selectRider(c.id)}>
          {#each cols as col}
            {@const v = valueOf(c, col)}
            <div class="td {col.align ?? 'left'}" style="width:{col.width}px">
              {#if col.kind === "rider"}
                <Flag code={c.flag} title={c.nationality} />
                <span class="name">{c.name}</span>
                {#if $shortlist.has(c.id)}<span class="mark sl" title="On your shortlist">★</span>{/if}
                {#if c.injured}<span class="mark inj" title="Injured">+</span>{/if}
                {#if c.my_report}<span class="mark rep" title="Your scout has a report">●</span>{/if}
                {#if !c.signable}<span class="u18" title="Turns 18 in {c.signable_from}; can't be signed before that season">U18</span>{/if}
              {:else if col.kind === "team"}
                {@const t = $teamById.get(c.team_id)}
                {#if c.free_agent}
                  <span class="free">Free agent</span>
                {:else}
                  <span class="swatch" style="background:{teamColor(t?.color1, c.team_id)}"></span>
                  <span class="ellip">{c.team_short || c.team}</span>
                {/if}
              {:else if col.kind === "ca"}
                <Bib value={Number(v)} />
              {:else if col.kind === "stars" || col.kind === "scout"}
                {#if Number(v) > 0}<Stars value={Number(v)} size={col.kind === "scout" ? 10 : 11} color={col.kind === "scout" ? "var(--azur)" : "var(--jaune)"} />{:else}<span class="dim">–</span>{/if}
              {:else if col.kind === "upside"}
                {#if Number(v) > 0}<span class="up">+{Number(v).toFixed(1)}</span>{:else}<span class="dim">–</span>{/if}
              {:else if col.kind === "grade"}
                <span style="color:{GRADE_COLOR[c.scout_grade] ?? 'var(--ink-3)'}">{c.scout_grade}</span>
              {:else if col.kind === "type"}
                <span class="dot" style="background:{TYPE_COLOR[c.rider_type] ?? 'var(--ink-4)'}"></span>{c.rider_type}
              {:else if col.kind === "money"}
                {#if Number(v) > 0}{eurShort(Number(v))}{:else}<span class="dim">–</span>{/if}
              {:else if col.kind === "contract"}
                {#if c.free_agent || !c.contract_end}<span class="dim">–</span>
                {:else}<span class:expiring={c.contract_end <= season}>{c.contract_end}</span>{/if}
              {:else if col.kind === "stat"}
                <span class="statv" style="color:{statColor(Number(v))}">{v}</span>
                {#if col.key + "_p" in c && stat(c, col.key + "_p") > Number(v)}<span class="ceil">{stat(c, col.key + "_p")}</span>{/if}
              {:else if col.kind === "gem"}
                {@const g = Number(v)}
                {#if !c.scout_estimate}<span class="dim">–</span>
                {:else}<span class:pos={g > 0} class:neg={g < 0}>{g > 0 ? "+" : ""}{g.toFixed(1)}</span>{/if}
              {:else if col.kind === "delta"}
                {@const d = Number(v)}
                {#if Number.isNaN(d)}<span class="newrole">new role</span>{:else}<span class:pos={d > 0} class:neg={d < 0}>{d > 0 ? "+" : ""}{d.toFixed(1)}</span>{/if}
              {:else if col.kind === "sign"}
                {#if c.signable}<span class="sign ok">Signable</span>{:else}<span class="sign wait" title="Turns 18 in {c.signable_from}">From {c.signable_from}</span>{/if}
              {:else if col.kind === "nation"}
                <Flag code={c.flag} /> <span class="ellip">{c.nationality}</span>
              {:else}
                <span class="ellip">{v ?? ""}</span>
              {/if}
            </div>
          {/each}
        </div>
      {/each}
    </div>
  {/if}
</div>
</div>

<style>
  /* The host takes its size from the layout; the absolutely positioned scroller can never
     grow it, so only the visible rows are ever rendered. */
  .host { position: absolute; inset: 0; }
  .wrap { position: absolute; inset: 0; overflow: auto; outline: none; }
  .wrap:focus-visible { box-shadow: inset 0 0 0 1px var(--jaune); }
  .head {
    position: sticky; top: 0; z-index: 2; display: flex; background: var(--bg);
    border-bottom: 1px solid var(--rule);
  }
  .th {
    flex-shrink: 0; padding: 9px 10px; background: none; border: none; color: var(--ink-3);
    font: inherit; font-size: 12px; font-weight: 500; cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .th:hover { color: var(--ink); }
  .th.sorted { color: var(--jaune); }
  .arrow { margin-left: 3px; }
  .body { position: relative; }
  .tr {
    position: absolute; left: 0; right: 0; display: flex; align-items: center;
    border-bottom: 1px solid #1c232e; cursor: pointer;
  }
  .tr:hover { background: var(--panel); }
  .tr.sel { background: var(--panel-2); box-shadow: inset 3px 0 0 var(--jaune); }
  .tr.mine .name { color: var(--jaune); }
  .td {
    flex-shrink: 0; padding: 0 10px; display: flex; align-items: center; gap: 7px;
    font-size: 13px; white-space: nowrap; overflow: hidden; height: 100%;
  }
  .td.center { justify-content: center; }
  .td.right { justify-content: flex-end; }
  .left, .th.left { text-align: left; }
  .th.center { text-align: center; }
  .th.right { text-align: right; }
  .name { font-weight: 500; overflow: hidden; text-overflow: ellipsis; }
  .ellip { overflow: hidden; text-overflow: ellipsis; }
  .mark { font-size: 11px; line-height: 1; }
  .mark.sl { color: var(--jaune); }
  .mark.inj { color: var(--pois); font-weight: 800; font-size: 14px; }
  .mark.rep { color: var(--azur); font-size: 9px; }
  .newrole { color: var(--azur); font-size: 12px; }
  .u18 { font-size: 9.5px; font-weight: 700; color: var(--ink-3); border: 1px solid var(--rule-2); border-radius: 3px; padding: 0 3px; line-height: 14px; }
  .sign { font-size: 12px; font-weight: 600; }
  .sign.ok { color: var(--vert); }
  .sign.wait { color: var(--ink-3); }
  .swatch { width: 9px; height: 9px; border-radius: 2px; flex-shrink: 0; }
  .free { color: var(--vert); font-size: 12px; }
  .dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
  .up { color: var(--vert); font-weight: 600; }
  .dim { color: var(--ink-4); }
  .statv { font-family: var(--font-cond); font-weight: 700; font-size: 15px; }
  .ceil { font-size: 10px; color: var(--ink-4); }
  .expiring { color: var(--pois); font-weight: 600; }
  .pos { color: var(--vert); font-weight: 600; }
  .neg { color: var(--pois); }
  .empty { padding: 48px 20px; text-align: center; color: var(--ink-3); }
</style>
