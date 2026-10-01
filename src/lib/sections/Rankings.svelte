<script lang="ts">
  import { allCyclists, teams } from "../stores";
  import { STAT_KEYS, STAT_LABELS, type Col } from "../types";
  import { eurShort, teamColor } from "../format";
  import RiderTable from "../components/RiderTable.svelte";
  import Flag from "../components/Flag.svelte";

  let tab: "riders" | "nations" | "teams" = "riders";
  let metric = "current_ability";
  let u23 = false;

  const METRICS = [
    { key: "current_ability", label: "Current ability" },
    { key: "potential", label: "Potential" },
    { key: "growth", label: "Growth left" },
    ...STAT_KEYS.map((k) => ({ key: k, label: STAT_LABELS[k] })),
  ];

  $: ranked = $allCyclists
    .filter((c) => !u23 || (c.age > 0 && c.age <= 23))
    .slice()
    .sort((a, b) => ((b as any)[metric] - (a as any)[metric]) || b.current_ability - a.current_ability)
    .slice(0, 200)
    .map((c, i) => ({ ...c, rank: i + 1 }));

  $: cols = [
    { key: "rank", label: "#", width: 50, align: "center" },
    { key: "name", label: "Rider", width: 220, kind: "rider" },
    { key: "team", label: "Team", width: 170, kind: "team" },
    { key: "age", label: "Age", width: 50, align: "center" },
    { key: "current_ability", label: "CA", width: 62, kind: "ca", align: "center" },
    ...(metric !== "current_ability" ? [{ key: metric, label: METRICS.find((m) => m.key === metric)?.label ?? metric, width: 110, kind: metric === "potential" ? "stars" : metric === "growth" ? "upside" : "stat", align: "center" }] : []),
    { key: "rider_type", label: "Type", width: 122, kind: "type" },
    { key: "nationality", label: "Nation", width: 130, kind: "nation" },
  ] as Col[];

  $: nations = (() => {
    const m = new Map<string, { flag: string; riders: number[]; best: string; bestCA: number }>();
    for (const c of $allCyclists) {
      if (c.nationality === "Unknown") continue;
      const e = m.get(c.nationality) ?? { flag: c.flag, riders: [], best: "", bestCA: 0 };
      e.riders.push(c.current_ability);
      if (c.current_ability > e.bestCA) { e.bestCA = c.current_ability; e.best = c.name; }
      m.set(c.nationality, e);
    }
    return [...m.entries()].map(([n, e]) => {
      const top = e.riders.sort((a, b) => b - a).slice(0, 8);
      return { n, flag: e.flag, count: e.riders.length, top8: top.reduce((a, b) => a + b, 0) / top.length, best: e.best, bestCA: e.bestCA };
    }).filter((x) => x.count >= 3).sort((a, b) => b.top8 - a.top8);
  })();

  $: teamRows = $teams.filter((t) => t.riders > 0).slice().sort((a, b) => b.avg_ca - a.avg_ca);
</script>

<div class="page">
  <div class="page-head">
    <div><h1>Rankings</h1><p>Who leads the league on any attribute, plus nations and teams.</p></div>
  </div>
  <div class="tabs">
    <button class:on={tab === "riders"} on:click={() => (tab = "riders")}>Riders</button>
    <button class:on={tab === "nations"} on:click={() => (tab = "nations")}>Nations</button>
    <button class:on={tab === "teams"} on:click={() => (tab = "teams")}>Teams</button>
  </div>

  {#if tab === "riders"}
    <div class="toolbar">
      <select bind:value={metric} aria-label="Rank by">{#each METRICS as m}<option value={m.key}>{m.label}</option>{/each}</select>
      <label class="chk"><input type="checkbox" bind:checked={u23} /> Under 23 only</label>
      <span class="muted count">Top 200</span>
    </div>
    <div class="table">{#key metric}<RiderTable data={ranked} {cols} />{/key}</div>
  {:else if tab === "nations"}
    <div class="page-body">
      <table class="grid-table">
        <thead><tr><th class="c">#</th><th>Nation</th><th class="r">Riders</th><th class="r">Top-8 average CA</th><th>Best rider</th><th class="r">CA</th></tr></thead>
        <tbody>
          {#each nations as r, i}
            <tr><td class="c muted">{i + 1}</td><td><Flag code={r.flag} /> {r.n}</td><td class="r">{r.count}</td><td class="r strong">{r.top8.toFixed(1)}</td><td>{r.best}</td><td class="r">{r.bestCA}</td></tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="page-body">
      <table class="grid-table">
        <thead><tr><th class="c">#</th><th>Team</th><th>Division</th><th class="r">Riders</th><th class="r">Avg CA</th><th class="r">Best</th><th class="r">Budget</th><th class="r">Wages / mo</th></tr></thead>
        <tbody>
          {#each teamRows as t, i}
            <tr class:mine={t.is_mine}><td class="c muted">{i + 1}</td><td><span class="sw" style="background:{teamColor(t.color1, t.id)}"></span>{t.name}</td><td class="muted">{t.division}</td><td class="r">{t.riders}</td><td class="r strong">{t.avg_ca}</td><td class="r">{t.top_ca}</td><td class="r">{eurShort(t.budget)}</td><td class="r">{eurShort(t.payroll)}</td></tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .tabs { display: flex; gap: 2px; padding: 0 22px 10px; }
  .tabs button { background: none; border: none; border-bottom: 2px solid transparent; color: var(--ink-3); font: inherit; font-size: 13.5px; padding: 4px 12px 7px; cursor: pointer; }
  .tabs button.on { color: var(--ink); border-bottom-color: var(--jaune); }
  .chk { display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: var(--ink-2); cursor: pointer; }
  .count { margin-left: auto; font-size: 12px; }
  .table { position: relative; flex: 1; overflow: hidden; border-top: 1px solid var(--rule); }
  .strong { font-weight: 700; color: var(--jaune); }
  .sw { display: inline-block; width: 9px; height: 9px; border-radius: 2px; margin-right: 8px; }
  tr.mine td { background: #f5c5180d; }
</style>
