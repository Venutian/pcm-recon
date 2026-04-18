<script lang="ts">
  import { allCyclists, teams, gameDate, shortlistIds, filtered } from "../stores";
  import SummaryCard from "../components/SummaryCard.svelte";
  import BarChart from "../components/BarChart.svelte";
  import RiderTable from "../components/RiderTable.svelte";
  import type { Col } from "../types";
  import { TYPE_COLORS } from "../types";
  import { fmtNat } from "../format";

  const dashCols: Col[] = [
    { key:"name",            label:"Name",    width:190, align:"left" },
    { key:"nat_flag",        label:"Country", width:130, align:"left", fmt:(_,r)=>fmtNat(r) },
    { key:"team_short",      label:"Team",    width:140, align:"left" },
    { key:"age",             label:"Age",     width:44,  align:"center" },
    { key:"current_ability", label:"CA",      width:46,  align:"center" },
    { key:"rider_type",      label:"Type",    width:110, align:"left" },
  ];

  $: n   = $allCyclists.length;
  $: fa  = $allCyclists.filter(c => c.free_agent).length;
  $: wk  = $allCyclists.filter(c => ["Wonderkid","Elite Prospect"].includes(c.scout_grade)).length;
  $: u23 = $allCyclists.filter(c => c.age > 0 && c.age <= 23).length;
  $: avgCA = n ? ($allCyclists.reduce((s,c) => s+c.current_ability, 0)/n).toFixed(1) : "—";
  $: nats = new Set($allCyclists.map(c => c.nationality).filter(x => x && x !== "Unknown")).size;
  $: top15 = $allCyclists.slice(0,15);

  $: typeData = (() => {
    const m: Record<string,number> = {};
    $allCyclists.forEach(c => { m[c.rider_type] = (m[c.rider_type]??0)+1; });
    return Object.entries(m).sort((a,b)=>b[1]-a[1])
      .map(([l,v]) => ({ label:l, value:v, color: TYPE_COLORS[l]??"#e8b800" }));
  })();

  $: natData = (() => {
    const m: Record<string,number> = {};
    $allCyclists.forEach(c => { if(c.nationality && c.nationality!=="Unknown") m[c.nationality]=(m[c.nationality]??0)+1; });
    return Object.entries(m).sort((a,b)=>b[1]-a[1]).slice(0,18)
      .map(([l,v]) => ({ label:l, value:v, color:"#e8b800" }));
  })();
</script>

<div class="dash section-enter">
  <!-- Cards -->
  <div class="cards">
    <SummaryCard title="Total Riders"  value={n?String(n):"—"}      sub="{nats} nationalities"   accent="#e8b800" tip="All riders in this save" />
    <SummaryCard title="Teams"         value={String($teams.length)} sub="Active rosters"          accent="#e87848" />
    <SummaryCard title="Free Agents"   value={String(fa)}            sub="Available now"           accent="#2ecc82" />
    <SummaryCard title="Top Prospects" value={String(wk)}            sub="Wonderkids + Elite"      accent="#ffd700" />
    <SummaryCard title="U-23 Riders"   value={String(u23)}           sub="{n?((u23/n*100)|0):0}% of pool" accent="#24b0a8" />
    <SummaryCard title="Avg. CA"       value={String(avgCA)}         sub="Mean current ability"    accent="#6478a0" />
    <SummaryCard title="Shortlisted"   value={String($shortlistIds.size)} sub="Your watchlist"    accent="#e85598" />
    <SummaryCard title="Game Date"     value={$gameDate||"—"}        sub="In-game date"            accent="#9470f0" />
  </div>

  <div class="lower">
    <!-- Top 15 table -->
    <div class="card-block">
      <div class="block-hdr">TOP 15 RIDERS BY CURRENT ABILITY</div>
      <div class="table-wrap">
        <RiderTable data={top15} cols={dashCols} />
      </div>
    </div>

    <!-- Charts column -->
    <div class="charts-col">
      <div class="card-block">
        <div class="block-hdr">RIDER TYPE DISTRIBUTION</div>
        <div class="chart-wrap">
          <BarChart data={typeData} />
        </div>
      </div>
      <div class="card-block" style="flex:1">
        <div class="block-hdr">TOP NATIONS BY RIDER COUNT</div>
        <div class="chart-wrap">
          <BarChart data={natData} />
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .dash { display: flex; flex-direction: column; height: 100%; padding: 16px; gap: 14px; overflow: hidden; }
  .cards { display: grid; grid-template-columns: repeat(8,1fr); gap: 10px; flex-shrink: 0; }
  .lower { display: grid; grid-template-columns: 1fr 380px; gap: 14px; flex: 1; overflow: hidden; min-height: 0; }
  .card-block { background: #0f1829; border: 1px solid #1e2d4a; border-radius: 8px;
                display: flex; flex-direction: column; overflow: hidden; }
  .block-hdr { padding: 10px 14px; font-size: 10px; font-weight: 700; color: #3a4e72;
               letter-spacing: 0.08em; border-bottom: 1px solid #111c30; flex-shrink: 0; }
  .table-wrap { flex: 1; overflow: hidden; }
  .charts-col { display: flex; flex-direction: column; gap: 10px; overflow: hidden; min-height: 0; }
  .chart-wrap { flex: 1; min-height: 0; padding: 8px; }
</style>
