<script lang="ts">
  import { allCyclists, myRiders, myTeam, meta, scoutPreset, goTo, selectRider } from "../stores";
  import { TERRAINS, RIDER_TYPES, type Col, type Cyclist } from "../types";
  import { eurShort, terrainScore } from "../format";
  import RiderTable from "../components/RiderTable.svelte";
  import Bib from "../components/Bib.svelte";

  $: season = $meta?.season ?? 0;
  $: team = $myTeam;

  // Strength on a terrain = average of the team's best three riders there.
  function top3(riders: Cyclist[], stats: typeof TERRAINS[number]["stats"]) {
    const v = riders.map((r) => terrainScore(r, stats)).sort((a, b) => b - a).slice(0, 3);
    return v.length ? v.reduce((a, b) => a + b, 0) / v.length : 0;
  }
  $: divisionTeams = (() => {
    if (!team) return new Map<number, Cyclist[]>();
    const m = new Map<number, Cyclist[]>();
    for (const c of $allCyclists) if (!c.free_agent && c.division === team.division) (m.get(c.team_id) ?? m.set(c.team_id, []).get(c.team_id)!).push(c);
    return m;
  })();
  $: strength = TERRAINS.map((t) => {
    const mine = top3($myRiders, t.stats);
    const others = [...divisionTeams.entries()].filter(([id]) => id !== team?.id).map(([, rs]) => top3(rs, t.stats));
    const avg = others.length ? others.reduce((a, b) => a + b, 0) / others.length : 0;
    const best = Math.max(0, ...others);
    const rank = others.filter((o) => o > mine).length + 1;
    return { ...t, mine, avg, best, rank, of: others.length + 1, gap: Math.round((mine - avg) * 10) / 10 || 0 };
  });
  $: weakest = [...strength].sort((a, b) => a.gap - b.gap)[0];
  const LO = 55, HI = 85;
  const pct = (v: number) => Math.max(0, Math.min(100, ((v - LO) / (HI - LO)) * 100));

  function findFor(t: typeof strength[number]) {
    const key = t.stats[0];
    const mineBest = Math.max(0, ...$myRiders.map((r) => (r as unknown as Record<string, number>)[key]));
    scoutPreset.set({ statKey: key, statMin: Math.round(Math.min(85, Math.max(70, mineBest))), status: "market", hideMine: true, sort: key, minAge: 18 });
    goTo("Scout");
  }

  $: expiring = $myRiders.filter((c) => c.contract_end > 0 && c.contract_end <= season);
  $: avgCA = $myRiders.length ? $myRiders.reduce((s, c) => s + c.current_ability, 0) / $myRiders.length : 0;
  function advice(c: Cyclist) {
    if (c.will_retire) return { t: "Retiring", cls: "dim" };
    if (c.age <= 24 && c.growth >= 3) return { t: "Keep, still growing", cls: "pos" };
    if (c.current_ability >= avgCA + 2) return { t: "Keep, key rider", cls: "pos" };
    if (c.age >= 32 && c.current_ability < avgCA) return { t: "Let go", cls: "neg" };
    return { t: "Renew if cheap", cls: "" };
  }
  $: typeCounts = RIDER_TYPES.map((t) => ({ ...t, n: $myRiders.filter((r) => r.rider_type_id === t.id).length }));

  const cols: Col[] = [
    { key: "name", label: "Rider", width: 210, kind: "rider" },
    { key: "age", label: "Age", width: 50, align: "center" },
    { key: "current_ability", label: "CA", width: 62, kind: "ca", align: "center" },
    { key: "potential", label: "Potential", width: 100, kind: "stars" },
    { key: "growth", label: "Growth", width: 66, kind: "upside", align: "center" },
    { key: "rider_type", label: "Type", width: 122, kind: "type" },
    { key: "wage", label: "Wage / mo", width: 86, kind: "money", align: "right" },
    { key: "contract_end", label: "Contract", width: 74, kind: "contract", align: "center" },
    { key: "mountain", label: "MO", width: 54, kind: "stat", align: "center" },
    { key: "hill", label: "HIL", width: 54, kind: "stat", align: "center" },
    { key: "timetrial", label: "TT", width: 54, kind: "stat", align: "center" },
    { key: "sprint", label: "SP", width: 54, kind: "stat", align: "center" },
    { key: "cobble", label: "COB", width: 54, kind: "stat", align: "center" },
    { key: "endurance", label: "STA", width: 54, kind: "stat", align: "center" },
    { key: "popularity", label: "Pop.", width: 54, align: "center" },
    { key: "nationality", label: "Nation", width: 130, kind: "nation" },
  ];
</script>

<div class="page">
  {#if !team}
    <div class="page-head"><h1>My team</h1></div>
    <div class="page-body"><p class="muted">No manager team found in this save.</p></div>
  {:else}
    <div class="page-head">
      <div>
        <h1>{team.name}</h1>
        <p>{team.division} · {team.riders} riders · average CA {team.avg_ca} · wages {eurShort(team.payroll)} a month</p>
      </div>
      <div class="types">
        {#each typeCounts as t}
          <span class="tc" title={t.label}><span class="dot" style="background:{t.color}"></span>{t.label} <b>{t.n}</b></span>
        {/each}
      </div>
    </div>

    <div class="page-body">
      <div class="grid">
        <section class="panel">
          <div class="panel-head">
            <h3>Squad strength by terrain</h3>
            <span class="muted">Your best three riders vs the {team.division} average</span>
          </div>
          <div class="terrain">
            {#each strength as s}
              <div class="t-row" class:weak={s === weakest}>
                <span class="t-l">{s.label}</span>
                <div class="t-track">
                  <div class="t-bar" style="width:{pct(s.mine)}%"></div>
                  <div class="t-avg" style="left:{pct(s.avg)}%" title="Division average {s.avg.toFixed(1)}"></div>
                  <div class="t-best" style="left:{pct(s.best)}%" title="Best in division {s.best.toFixed(1)}"></div>
                </div>
                <span class="t-v">{s.mine.toFixed(1)}</span>
                <span class="t-g" class:pos={s.gap >= 0} class:neg={s.gap < 0}>{s.gap >= 0 ? "+" : ""}{s.gap.toFixed(1)}</span>
                <span class="t-r muted">{s.rank}/{s.of}</span>
                <button class="btn btn-sm btn-quiet" on:click={() => findFor(s)}>Find riders</button>
              </div>
            {/each}
            <div class="legend muted"><span class="lg bar"></span>Your top 3 <span class="lg avg"></span>Division average <span class="lg best"></span>Best team in division</div>
          </div>
          {#if weakest}
            <p class="advice">Biggest gap: <strong>{weakest.label}</strong>, {Math.abs(weakest.gap).toFixed(1)} below the division average. Use “Find riders” to search the market for that terrain.</p>
          {/if}
        </section>

        <section class="panel">
          <div class="panel-head"><h3>Contracts ending in {season}</h3><span class="muted">{expiring.length} riders</span></div>
          {#if expiring.length}
            <ul class="exp">
              {#each expiring as c}
                {@const a = advice(c)}
                <li>
                  <button on:click={() => selectRider(c.id)}>
                    <Bib value={c.current_ability} />
                    <span class="nm">{c.name}<small>{c.age} yrs · {c.rider_type} · {eurShort(c.wage)}/mo</small></span>
                    <span class="adv {a.cls}">{a.t}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="muted pad">Every rider is signed beyond this season.</p>
          {/if}
        </section>
      </div>

      <section class="panel roster">
        <div class="panel-head"><h3>Roster</h3><span class="muted">Click a rider for the full profile</span></div>
        <div class="roster-table"><RiderTable data={$myRiders} {cols} sortKey="current_ability" /></div>
      </section>
    </div>
  {/if}
</div>

<style>
  .page-body { display: flex; flex-direction: column; gap: 16px; }
  .types { display: flex; gap: 10px; flex-wrap: wrap; justify-content: flex-end; max-width: 520px; }
  .tc { display: inline-flex; align-items: center; gap: 5px; font-size: 12px; color: var(--ink-2); }
  .tc b { color: var(--ink); }
  .dot { width: 7px; height: 7px; border-radius: 50%; }
  .grid { display: grid; grid-template-columns: minmax(0, 1.5fr) minmax(300px, 1fr); gap: 16px; }

  .terrain { padding: 4px 14px 6px; display: flex; flex-direction: column; gap: 6px; }
  .t-row { display: grid; grid-template-columns: 92px 1fr 40px 46px 40px auto; align-items: center; gap: 10px; padding: 3px 0; }
  .t-row.weak .t-l { color: var(--pois); font-weight: 600; }
  .t-l { font-size: 13px; color: var(--ink-2); }
  .t-track { position: relative; height: 12px; background: var(--bg-2); border-radius: 3px; }
  .t-bar { position: absolute; inset: 0 auto 0 0; background: var(--jaune); border-radius: 3px; }
  .t-avg, .t-best { position: absolute; top: -4px; bottom: -4px; width: 2px; }
  .t-avg { background: var(--ink); }
  .t-best { background: var(--ink-3); width: 2px; border-radius: 1px; opacity: 0.8; }
  .t-best::after { content: ""; position: absolute; top: -3px; left: -2px; width: 6px; height: 6px; border-radius: 50%; background: var(--ink-3); }
  .t-v { font-family: var(--font-cond); font-weight: 700; font-size: 16px; text-align: right; }
  .t-g { font-size: 12px; text-align: right; }
  .t-r { font-size: 12px; text-align: right; }
  .legend { display: flex; gap: 8px; align-items: center; font-size: 11px; margin-top: 6px; }
  .lg { display: inline-block; width: 12px; height: 8px; margin-left: 8px; }
  .lg.bar { background: var(--jaune); border-radius: 2px; margin-left: 0; }
  .lg.avg { width: 2px; height: 12px; background: var(--ink); }
  .lg.best { width: 6px; height: 6px; border-radius: 50%; background: var(--ink-3); }
  .advice { padding: 6px 14px 14px; font-size: 13px; color: var(--ink-2); }
  .pos { color: var(--vert); }
  .neg { color: var(--pois); }
  .dim { color: var(--ink-3); }

  .exp { list-style: none; padding: 0 8px 10px; max-height: 330px; overflow: auto; }
  .exp button { width: 100%; display: flex; align-items: center; gap: 10px; padding: 7px 6px; background: none; border: none; border-radius: 6px; color: var(--ink); font: inherit; cursor: pointer; text-align: left; }
  .exp button:hover { background: var(--panel-2); }
  .nm { display: flex; flex-direction: column; font-weight: 500; min-width: 0; flex: 1; }
  .nm small { font-size: 11.5px; color: var(--ink-3); font-weight: 400; }
  .adv { font-size: 12px; white-space: nowrap; }
  .pad { padding: 4px 14px 16px; }
  .roster { display: flex; flex-direction: column; }
  .roster-table { position: relative; height: 520px; border-top: 1px solid var(--rule); }
  @media (max-width: 1150px) { .grid { grid-template-columns: 1fr; } }
</style>
