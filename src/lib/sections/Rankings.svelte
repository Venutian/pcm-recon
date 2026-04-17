<script lang="ts">
  import { allCyclists, teams, teamRosters } from "../stores";
  import RiderTable from "../components/RiderTable.svelte";
  import type { Col } from "../types";
  import { fmtNat, flagEmoji } from "../format";

  const RANK_COLS: Col[] = [
    { key:"rank",            label:"#",       width:40,  align:"center" },
    { key:"name",            label:"Name",    width:190, align:"left" },
    { key:"nat_flag",        label:"Country", width:130, align:"left", fmt:(_,r)=>fmtNat(r) },
    { key:"team_short",      label:"Team",    width:150, align:"left" },
    { key:"age",             label:"Age",     width:44,  align:"center" },
    { key:"current_ability", label:"CA",      width:48,  align:"center" },
    { key:"potential",       label:"Stars",   width:52,  align:"center" },
    { key:"rider_type",      label:"Type",    width:110, align:"left" },
    { key:"scout_grade",     label:"Grade",   width:110, align:"left" },
  ];

  const METRICS = [
    { key:"current_ability", label:"Current Ability", color:"#4d88f5" },
    { key:"potential",       label:"Potential",        color:"#f0c030" },
    { key:"growth",          label:"Growth Upside",    color:"#2ecc82" },
    { key:"flat",            label:"Flat",             color:"#c8e857" },
    { key:"mountain",        label:"Mountain",         color:"#2ecc82" },
    { key:"timetrial",       label:"Time Trial",       color:"#7aaaff" },
    { key:"sprint",          label:"Sprint",           color:"#f0c030" },
    { key:"cobble",          label:"Cobbles",          color:"#e87848" },
    { key:"endurance",       label:"Endurance",        color:"#24b0a8" },
    { key:"acceleration",    label:"Acceleration",     color:"#e85598" },
  ];

  let tab = 0;
  let metric = "current_ability";
  let metricColor = "#4d88f5";

  function setMetric(key: string, color: string) { metric = key; metricColor = color; }

  $: ranked = [...$allCyclists]
    .sort((a,b) => (b as Record<string,number>)[metric] - (a as Record<string,number>)[metric])
    .slice(0,100)
    .map((c,i) => ({ ...c, rank: i+1 }));

  // Country rankings
  $: countryRows = (() => {
    const m: Record<string,{riders:typeof $allCyclists, iso:string}> = {};
    $allCyclists.forEach(c => {
      if (!c.nationality || c.nationality==="Unknown") return;
      if (!m[c.nationality]) m[c.nationality]={riders:[],iso:c.iso};
      m[c.nationality].riders.push(c);
    });
    return Object.entries(m)
      .map(([nat,{riders,iso}]) => {
        const avg = riders.reduce((s,r)=>s+r.current_ability,0)/riders.length;
        const best = riders.sort((a,b)=>b.current_ability-a.current_ability)[0];
        return { nat, iso, count:riders.length, avgCA:avg, best };
      })
      .sort((a,b)=>b.avgCA-a.avgCA)
      .slice(0,80)
      .map((r,i)=>({...r,rank:i+1}));
  })();

  // Team rankings
  $: teamRows = (() => {
    return $teams.map(t => {
      const roster = $teamRosters[t.id]??[];
      if (!roster.length) return null;
      const avg = roster.reduce((s,r)=>s+r.current_ability,0)/roster.length;
      const best = [...roster].sort((a,b)=>b.current_ability-a.current_ability)[0];
      return { id:t.id, name:t.name, iso:t.country_iso, count:roster.length, avgCA:avg, best };
    }).filter(Boolean)
      .sort((a,b)=>b!.avgCA-a!.avgCA)
      .slice(0,80)
      .map((r,i)=>({...r,rank:i+1}));
  })();
</script>

<div class="rank section-enter">
  <div class="tabs">
    {#each ["Individual","Country Rankings","Team Rankings"] as t,i}
      <button class="tab" class:active={tab===i} on:click={()=>tab=i}>{t}</button>
    {/each}
  </div>

  {#if tab===0}
    <div class="metric-bar">
      {#each METRICS as m}
        <button class="btn btn-type" class:active={metric===m.key}
                style:background={metric===m.key ? m.color : ""}
                on:click={()=>setMetric(m.key,m.color)}>
          {m.label}
        </button>
      {/each}
    </div>
    <div class="table-wrap">
      <RiderTable data={ranked} cols={RANK_COLS} prefix="rk" />
    </div>

  {:else if tab===1}
    <div class="simple-table">
      <table>
        <thead>
          <tr><th>#</th><th>Country</th><th>Riders</th><th>Avg CA</th><th>Best Rider</th><th>Best CA</th></tr>
        </thead>
        <tbody>
          {#each countryRows as r}
            <tr>
              <td style="text-align:center;color:#3a4e72">{r.rank}</td>
              <td>{flagEmoji(r.iso)} {r.nat}</td>
              <td style="text-align:center">{r.count}</td>
              <td style="text-align:center;color:#4d88f5;font-weight:700">{r.avgCA.toFixed(1)}</td>
              <td>{r.best?.name}</td>
              <td style="text-align:center;color:#ffd700;font-weight:700">{r.best?.current_ability}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

  {:else}
    <div class="simple-table">
      <table>
        <thead>
          <tr><th>#</th><th>Team</th><th>Country</th><th>Riders</th><th>Avg CA</th><th>Best Rider</th><th>Best CA</th></tr>
        </thead>
        <tbody>
          {#each teamRows as r}
            <tr>
              <td style="text-align:center;color:#3a4e72">{r!.rank}</td>
              <td style="font-weight:600;color:#dce8ff">{r!.name}</td>
              <td>{flagEmoji(r!.iso)} {r!.iso.toUpperCase()}</td>
              <td style="text-align:center">{r!.count}</td>
              <td style="text-align:center;color:#4d88f5;font-weight:700">{r!.avgCA.toFixed(1)}</td>
              <td>{r!.best?.name}</td>
              <td style="text-align:center;color:#ffd700;font-weight:700">{r!.best?.current_ability}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .rank { display:flex; flex-direction:column; height:100%; overflow:hidden; }
  .tabs { display:flex; background:#0d1525; border-bottom:1px solid #1e2d4a; flex-shrink:0; }
  .tab { padding:10px 18px; background:none; border:none; color:#6478a0; font-size:13px;
          cursor:pointer; border-bottom:2px solid transparent; font-family:inherit; }
  .tab:hover { color:#dce8ff; }
  .tab.active { color:#4d88f5; border-bottom-color:#4d88f5; }
  .metric-bar { display:flex; flex-wrap:wrap; gap:6px; padding:10px 16px;
                background:#080d1a; border-bottom:1px solid #111c30; flex-shrink:0; }
  .table-wrap { flex:1; overflow:hidden; }
  .simple-table { flex:1; overflow:auto; padding:0 16px 16px; }
</style>
