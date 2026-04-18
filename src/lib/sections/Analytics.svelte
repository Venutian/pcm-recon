<script lang="ts">
  import { allCyclists, filtered, teams, teamRosters } from "../stores";
  import BarChart from "../components/BarChart.svelte";
  import { TYPE_COLORS, GRADE_COLOR, GRADES } from "../types";

  $: natData = (() => {
    const m: Record<string,number>={};
    $filtered.forEach(c=>{if(c.nationality&&c.nationality!=="Unknown")m[c.nationality]=(m[c.nationality]??0)+1;});
    return Object.entries(m).sort((a,b)=>b[1]-a[1]).slice(0,20).map(([l,v])=>({label:l,value:v,color:"#e8b800"}));
  })();

  $: typeData = (() => {
    const m: Record<string,number>={};
    $filtered.forEach(c=>{m[c.rider_type]=(m[c.rider_type]??0)+1;});
    return Object.entries(m).sort((a,b)=>b[1]-a[1]).map(([l,v])=>({label:l,value:v,color:TYPE_COLORS[l]??"#e8b800"}));
  })();

  $: gradeData = GRADES.map(g => {
    const count = $filtered.filter(c=>c.scout_grade===g).length;
    return { label:g, value:count, color:GRADE_COLOR[g]??"#6478a0" };
  });

  $: teamAvgData = (() => {
    const rows = $teams.map(t=>{
      const r = $teamRosters[t.id]??[];
      if(r.length<3)return null;
      return {label:t.short||t.name, value:parseFloat((r.reduce((s,c)=>s+c.current_ability,0)/r.length).toFixed(1))};
    }).filter(Boolean) as {label:string;value:number}[];
    return rows.sort((a,b)=>b.value-a.value).slice(0,20).map(r=>({...r,color:"#2ecc82"}));
  })();
</script>

<div class="analytics section-enter">
  <div class="grid">
    <div class="chart-card">
      <div class="hdr">TOP NATIONALITIES — {$filtered.length.toLocaleString()} RIDERS FILTERED</div>
      <div class="chart-body"><BarChart data={natData} /></div>
    </div>
    <div class="chart-card">
      <div class="hdr">RIDER TYPE DISTRIBUTION</div>
      <div class="chart-body"><BarChart data={typeData} /></div>
    </div>
    <div class="chart-card">
      <div class="hdr">SCOUT GRADE BREAKDOWN</div>
      <div class="chart-body"><BarChart data={gradeData} /></div>
    </div>
    <div class="chart-card">
      <div class="hdr">TOP TEAMS BY AVERAGE CA</div>
      <div class="chart-body"><BarChart data={teamAvgData} /></div>
    </div>
  </div>
</div>

<style>
  .analytics { height:100%; overflow:auto; padding:16px; }
  .grid { display:grid; grid-template-columns:1fr 1fr; grid-template-rows:1fr 1fr; gap:14px; height:100%; min-height:600px; }
  .chart-card { background:#0f1829; border:1px solid #1e2d4a; border-radius:8px;
                display:flex; flex-direction:column; overflow:hidden; }
  .hdr { padding:10px 14px; font-size:10px; font-weight:700; color:#3a4e72;
         letter-spacing:0.08em; border-bottom:1px solid #111c30; flex-shrink:0; }
  .chart-body { flex:1; padding:10px; min-height:0; }
</style>
