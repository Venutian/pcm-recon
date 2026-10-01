<script lang="ts">
  import { teams, allCyclists, requestWrite } from "../stores";
  import { eurShort, teamColor } from "../format";
  import type { Col, Team, CellEdit } from "../types";
  import RiderTable from "../components/RiderTable.svelte";
  import MoneyEditor from "../components/MoneyEditor.svelte";
  import Flag from "../components/Flag.svelte";

  let q = "";
  let division = "";
  let sort: "avg_ca" | "budget" | "payroll" | "name" = "avg_ca";
  let selectedTeam: number | null = null;
  let editing = false;

  $: divisions = [...new Set($teams.map((t) => t.division))].filter(Boolean).sort();
  $: list = $teams
    .filter((t) => t.riders > 0 && (!q || t.name.toLowerCase().includes(q.toLowerCase())) && (!division || t.division === division))
    .sort((a, b) => (sort === "name" ? a.name.localeCompare(b.name) : (b[sort] as number) - (a[sort] as number)));
  $: if (selectedTeam === null && $teams.length) selectedTeam = ($teams.find((t) => t.is_mine) ?? list[0])?.id ?? null;
  $: team = $teams.find((t) => t.id === selectedTeam) ?? null;
  $: roster = team ? $allCyclists.filter((c) => c.team_id === team!.id) : [];
  $: maxBudget = Math.max(1, ...list.map((t) => t.budget));

  function pick(t: Team) { selectedTeam = t.id; editing = false; }

  function saveBudget(to: number) {
    if (!team) return;
    const edits: CellEdit[] = [{ table: "DYN_team", column: "value_i_budget", key_column: "IDteam", key: team.id, value: to }];
    const lines = [{ label: `${team.name} season budget`, from: team.budget, to }];
    if (team.sponsor_deal_id) {
      edits.push({ table: "DYN_team_sponsor", column: "value_i_budget", key_column: "IDteam_sponsor", key: team.sponsor_deal_id, value: to });
      lines.push({ label: `${team.sponsor || "Sponsor"} deal, this season`, from: team.sponsor_budget, to });
    }
    requestWrite({ title: "Change team budget", lines, edits, done: `${team.name} budget set to ${eurShort(to)}` });
    editing = false;
  }

  const cols: Col[] = [
    { key: "name", label: "Rider", width: 210, kind: "rider" },
    { key: "age", label: "Age", width: 50, align: "center" },
    { key: "current_ability", label: "CA", width: 62, kind: "ca", align: "center" },
    { key: "potential", label: "Potential", width: 100, kind: "stars" },
    { key: "rider_type", label: "Type", width: 122, kind: "type" },
    { key: "wage", label: "Wage / mo", width: 86, kind: "money", align: "right" },
    { key: "contract_end", label: "Contract", width: 74, kind: "contract", align: "center" },
    { key: "mountain", label: "MO", width: 54, kind: "stat", align: "center" },
    { key: "timetrial", label: "TT", width: 54, kind: "stat", align: "center" },
    { key: "sprint", label: "SP", width: 54, kind: "stat", align: "center" },
    { key: "cobble", label: "COB", width: 54, kind: "stat", align: "center" },
    { key: "nationality", label: "Nation", width: 130, kind: "nation" },
  ];
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Teams</h1>
      <p>Every team in the league with its sponsor money and wage bill. You can change any team's season budget.</p>
    </div>
  </div>
  <div class="split">
    <aside class="list">
      <div class="list-tools">
        <input bind:value={q} placeholder="Search teams…" />
        <div class="row">
          <select bind:value={division}><option value="">All divisions</option>{#each divisions as d}<option>{d}</option>{/each}</select>
          <select bind:value={sort} aria-label="Sort teams">
            <option value="avg_ca">Avg CA</option><option value="budget">Budget</option><option value="payroll">Wages</option><option value="name">Name</option>
          </select>
        </div>
      </div>
      <ul>
        {#each list as t (t.id)}
          <li>
            <button class:on={t.id === selectedTeam} on:click={() => pick(t)}>
              <span class="sw" style="background:{teamColor(t.color1, t.id)}"></span>
              <span class="tn">{t.name}{#if t.is_mine}<em>you</em>{/if}<small>{t.division} · {t.riders} riders</small></span>
              <span class="tv">
                {#if sort === "budget"}{eurShort(t.budget)}{:else if sort === "payroll"}{eurShort(t.payroll)}{:else}{t.avg_ca}{/if}
                <span class="mini"><span style="width:{(t.budget / maxBudget) * 100}%"></span></span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </aside>

    {#if team}
      <section class="detail">
        <div class="t-head">
          <span class="jersey" style="background:{teamColor(team.color1, team.id)}; --c2:{team.color2 || 'transparent'}"></span>
          <div>
            <h2>{team.name}</h2>
            <p class="muted"><Flag code={team.flag} size={12} /> {team.country_name} · {team.division}{team.sponsor ? ` · sponsor ${team.sponsor}` : ""}</p>
          </div>
        </div>
        <dl class="facts">
          <div><dt>Season budget</dt><dd>{eurShort(team.budget)}</dd></div>
          <div><dt>Next season</dt><dd>{team.sponsor_budget_next ? eurShort(team.sponsor_budget_next) : "–"}</dd></div>
          <div><dt>Rider wages</dt><dd>{eurShort(team.payroll)}<small>/mo</small></dd></div>
          <div><dt>Staff wages</dt><dd>{eurShort(team.staff_payroll)}<small>/mo</small></dd></div>
          <div><dt>Wages / budget</dt><dd class:neg={team.budget > 0 && (team.payroll + team.staff_payroll) * 12 > team.budget}>{team.budget ? Math.round(((team.payroll + team.staff_payroll) * 12 / team.budget) * 100) + "%" : "–"}</dd></div>
          <div><dt>Avg CA</dt><dd>{team.avg_ca}</dd></div>
          <div><dt>Best rider</dt><dd>{team.top_ca}</dd></div>
          <div><dt>Sponsor deal ends</dt><dd>{team.sponsor_contract_end || "–"}</dd></div>
        </dl>
        <div class="edit">
          {#if editing}
            <MoneyEditor current={team.budget} min={0} max={200_000_000} steps={[-1_000_000, 500_000, 1_000_000, 2_500_000, 5_000_000]} on:apply={(e) => saveBudget(e.detail)} />
            <button class="btn btn-sm btn-quiet" on:click={() => (editing = false)}>Cancel</button>
          {:else}
            <button class="btn btn-sm" on:click={() => (editing = true)}>Change season budget</button>
          {/if}
        </div>
        <div class="roster"><RiderTable data={roster} {cols} sortKey="current_ability" /></div>
      </section>
    {/if}
  </div>
</div>

<style>
  .split { flex: 1; display: grid; grid-template-columns: 320px minmax(0, 1fr); overflow: hidden; border-top: 1px solid var(--rule); }
  .list { border-right: 1px solid var(--rule); display: flex; flex-direction: column; overflow: hidden; }
  .list-tools { padding: 10px; display: flex; flex-direction: column; gap: 6px; }
  .list-tools .row { display: flex; gap: 6px; }
  .list-tools select { flex: 1; }
  ul { list-style: none; overflow-y: auto; flex: 1; padding: 0 6px 10px; }
  li button {
    width: 100%; display: flex; align-items: center; gap: 9px; padding: 7px 8px; border-radius: 6px;
    background: none; border: none; color: var(--ink); font: inherit; cursor: pointer; text-align: left;
  }
  li button:hover { background: var(--panel); }
  li button.on { background: var(--panel-2); box-shadow: inset 3px 0 0 var(--jaune); }
  .sw { width: 10px; height: 26px; border-radius: 2px; flex-shrink: 0; }
  .tn { flex: 1; min-width: 0; font-size: 13px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: flex; flex-direction: column; }
  .tn em { font-style: normal; font-size: 10px; color: var(--jaune); margin-left: 6px; }
  .tn small { font-size: 11px; color: var(--ink-3); font-weight: 400; }
  .tv { display: flex; flex-direction: column; align-items: flex-end; gap: 3px; font-family: var(--font-cond); font-weight: 700; font-size: 15px; }
  .mini { width: 46px; height: 3px; background: var(--rule); border-radius: 2px; overflow: hidden; display: block; }
  .mini span { display: block; height: 100%; background: var(--ink-3); }
  .detail { display: flex; flex-direction: column; overflow: hidden; padding: 16px 0 0; }
  .t-head { display: flex; gap: 14px; align-items: center; padding: 0 20px; }
  .jersey { width: 34px; height: 40px; border-radius: 7px 7px 4px 4px; box-shadow: inset 0 -9px 0 var(--c2); flex-shrink: 0; }
  .t-head h2 { font-size: 26px; }
  .t-head p { font-size: 13px; display: flex; align-items: center; gap: 6px; }
  .facts { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px 18px; padding: 16px 20px 6px; }
  dt { font-size: 11px; color: var(--ink-3); }
  dd { font-family: var(--font-cond); font-size: 21px; font-weight: 700; }
  dd small { font-family: var(--font); font-size: 11px; color: var(--ink-3); font-weight: 400; margin-left: 2px; }
  .neg { color: var(--pois); }
  .edit { padding: 8px 20px 14px; display: flex; flex-direction: column; gap: 6px; align-items: flex-start; }
  .edit :global(.ed) { width: 100%; }
  .roster { position: relative; flex: 1; border-top: 1px solid var(--rule); overflow: hidden; min-height: 200px; }
</style>
