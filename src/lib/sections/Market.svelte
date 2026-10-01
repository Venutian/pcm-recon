<script lang="ts">
  import { allCyclists, meta, myRiders } from "../stores";
  import { RIDER_TYPES, type Col, type Cyclist } from "../types";
  import { eurShort } from "../format";
  import { exportCsv } from "../api";
  import RiderTable from "../components/RiderTable.svelte";
  import Icon from "../components/Icon.svelte";

  let tab: "free" | "expiring" | "all" = "expiring";
  let types: number[] = [];
  let minCA = 62;
  let maxAge = 40;

  $: season = $meta?.season ?? 0;
  $: market = $allCyclists.filter((c) => c.on_market && !c.is_mine && c.signable);
  $: free = market.filter((c) => c.free_agent);
  $: expiring = market.filter((c) => !c.free_agent);
  $: base = tab === "free" ? free : tab === "expiring" ? expiring : market;
  $: rows = base.filter((c) => c.current_ability >= minCA && c.age <= maxAge && (!types.length || types.includes(c.rider_type_id)));

  // Upgrade = how much better than your best rider of the same type.
  $: myBestByType = (() => {
    const m = new Map<number, number>();
    for (const r of $myRiders) m.set(r.rider_type_id, Math.max(m.get(r.rider_type_id) ?? 0, r.current_ability));
    return m;
  })();
  // NaN when you have nobody of that type: the table shows "new role" and sorts it last.
  const upgrade = (c: Cyclist) => myBestByType.has(c.rider_type_id) ? Math.round((c.current_ability - myBestByType.get(c.rider_type_id)!) * 10) / 10 : NaN;

  $: cols = [
    { key: "name", label: "Rider", width: 210, kind: "rider" },
    { key: "team", label: "Current team", width: 170, kind: "team" },
    { key: "age", label: "Age", width: 50, align: "center" },
    { key: "current_ability", label: "CA", width: 62, kind: "ca", align: "center" },
    { key: "upgrade", label: "vs your best", width: 92, align: "center", title: "CA difference to your best rider of the same type", kind: "delta", value: upgrade },
    { key: "potential", label: "Potential", width: 100, kind: "stars" },
    { key: "rider_type", label: "Type", width: 122, kind: "type" },
    { key: "wage", label: "Wage now", width: 86, kind: "money", align: "right", title: "Current monthly wage; expect to pay at least this" },
    { key: "popularity", label: "Pop.", width: 54, align: "center", title: "Popularity" },
    { key: "wins", label: "Wins", width: 54, align: "center" },
    { key: "mountain", label: "MO", width: 54, kind: "stat", align: "center" },
    { key: "hill", label: "HIL", width: 54, kind: "stat", align: "center" },
    { key: "timetrial", label: "TT", width: 54, kind: "stat", align: "center" },
    { key: "sprint", label: "SP", width: 54, kind: "stat", align: "center" },
    { key: "cobble", label: "COB", width: 54, kind: "stat", align: "center" },
    { key: "nationality", label: "Nation", width: 130, kind: "nation" },
  ] as Col[];

  $: avgWage = (() => {
    const w = expiring.filter((c) => c.current_ability >= 72).map((c) => c.wage).filter(Boolean);
    return w.length ? w.reduce((a, b) => a + b, 0) / w.length : 0;
  })();
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Transfer market</h1>
      <p>Riders you can sign for {season + 1}: free agents and everyone whose contract ends in {season}. “vs your best” shows how much each one would improve on your current best rider of that type.</p>
    </div>
    <button class="btn btn-sm btn-quiet" on:click={() => exportCsv(rows, ["name", "team", "age", "rider_type", "current_ability", "potential", "wage", "nationality"], "pcm_market.csv")}><Icon name="export" size={14} />CSV</button>
  </div>

  <div class="tabs">
    <button class:on={tab === "expiring"} on:click={() => (tab = "expiring")}>Contract ending <span>{expiring.length}</span></button>
    <button class:on={tab === "free"} on:click={() => (tab = "free")}>Free agents <span>{free.length}</span></button>
    <button class:on={tab === "all"} on:click={() => (tab = "all")}>Everyone available <span>{market.length}</span></button>
    {#if avgWage}<span class="muted hint">Riders with CA 72+ earn about {eurShort(avgWage)} a month now.</span>{/if}
  </div>

  <div class="toolbar">
    {#each RIDER_TYPES as t}
      <button class="chip" class:on={types.includes(t.id)} on:click={() => (types = types.includes(t.id) ? types.filter((x) => x !== t.id) : [...types, t.id])}>
        <span class="dot" style="background:{t.color}"></span>{t.label}
      </button>
    {/each}
    <label class="field inline"><span>CA ≥ {minCA}</span><input type="range" min="50" max="85" bind:value={minCA} /></label>
    <label class="field inline"><span>Age ≤ {maxAge}</span><input type="range" min="18" max="40" bind:value={maxAge} /></label>
    <span class="count">{rows.length.toLocaleString()} riders</span>
  </div>

  <div class="table"><RiderTable data={rows} {cols} sortKey="current_ability" /></div>
</div>

<style>
  .tabs { display: flex; gap: 2px; padding: 0 22px 10px; align-items: center; }
  .tabs button { background: none; border: none; border-bottom: 2px solid transparent; color: var(--ink-3); font: inherit; font-size: 13.5px; padding: 4px 12px 7px; cursor: pointer; }
  .tabs button span { color: var(--ink-4); margin-left: 3px; }
  .tabs button.on { color: var(--ink); border-bottom-color: var(--jaune); }
  .hint { margin-left: auto; font-size: 12px; }
  .dot { width: 7px; height: 7px; border-radius: 50%; }
  .field.inline { flex-direction: row; align-items: center; gap: 8px; }
  .field.inline input { width: 110px; }
  .count { margin-left: auto; font-size: 12px; color: var(--ink-3); }
  .table { position: relative; flex: 1; overflow: hidden; border-top: 1px solid var(--rule); }
</style>
