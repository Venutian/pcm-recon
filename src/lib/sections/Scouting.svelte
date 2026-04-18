<script lang="ts">
  import { filters, filtered, dropdownTeams, dropdownCountries, dropdownConts, defaultFilters } from "../stores";
  import RiderTable from "../components/RiderTable.svelte";
  import type { Col } from "../types";
  import { TYPE_COLORS, GRADES } from "../types";
  import { fmtNat } from "../format";

  const ROSTER_COLS: Col[] = [
    { key:"name",            label:"Name",    width:200, align:"left" },
    { key:"nat_flag",        label:"Country", width:140, align:"left", fmt:(_,r)=>fmtNat(r) },
    { key:"team_short",      label:"Team",    width:150, align:"left" },
    { key:"age",             label:"Age",     width:44,  align:"center" },
    { key:"current_ability", label:"CA",      width:48,  align:"center" },
    { key:"potential",       label:"Stars",   width:52,  align:"center" },
    { key:"growth",          label:"Upside",  width:58,  align:"center" },
    { key:"scout_grade",     label:"Grade",   width:110, align:"left" },
    { key:"rider_type",      label:"Type",    width:110, align:"left" },
    { key:"flat",            label:"Flat",    width:46,  align:"center" },
    { key:"mountain",        label:"Mtn",     width:46,  align:"center" },
    { key:"timetrial",       label:"TT",      width:46,  align:"center" },
    { key:"sprint",          label:"Spr",     width:46,  align:"center" },
    { key:"cobble",          label:"Cob",     width:46,  align:"center" },
    { key:"endurance",       label:"End",     width:46,  align:"center" },
  ];
  const YOUTH_COLS: Col[] = [
    { key:"name",            label:"Name",    width:200, align:"left" },
    { key:"nat_flag",        label:"Country", width:130, align:"left", fmt:(_,r)=>fmtNat(r) },
    { key:"team_short",      label:"Team",    width:150, align:"left" },
    { key:"age",             label:"Age",     width:44,  align:"center" },
    { key:"current_ability", label:"CA",      width:48,  align:"center" },
    { key:"potential",       label:"Stars",   width:52,  align:"center" },
    { key:"growth",          label:"Upside",  width:58,  align:"center" },
    { key:"scout_grade",     label:"Grade",   width:110, align:"left" },
    { key:"top_skill_text",  label:"Top Skills", width:260, align:"left" },
  ];
  const FREE_COLS: Col[] = [
    { key:"name",            label:"Name",    width:200, align:"left" },
    { key:"nat_flag",        label:"Country", width:130, align:"left", fmt:(_,r)=>fmtNat(r) },
    { key:"age",             label:"Age",     width:44,  align:"center" },
    { key:"current_ability", label:"CA",      width:48,  align:"center" },
    { key:"potential",       label:"Stars",   width:52,  align:"center" },
    { key:"growth",          label:"Upside",  width:58,  align:"center" },
    { key:"scout_grade",     label:"Grade",   width:110, align:"left" },
    { key:"rider_type",      label:"Type",    width:110, align:"left" },
    { key:"flat",            label:"Flat",    width:46,  align:"center" },
    { key:"mountain",        label:"Mtn",     width:46,  align:"center" },
    { key:"timetrial",       label:"TT",      width:46,  align:"center" },
    { key:"sprint",          label:"Spr",     width:46,  align:"center" },
  ];

  const TYPES = [null,2,4,3,1,6,5,7];
  const TYPE_LABELS = ["All","Climber","Sprinter","Time Trialist","All-Rounder","Classics","Ardennes","Rouleur"];

  $: youth = $filtered.filter(c => c.age > 0 && c.age <= 23)
    .sort((a,b) => b.potential-a.potential || b.growth-a.growth);
  $: agents = $filtered.filter(c => c.free_agent);

  let tab = 0;
</script>

<div class="scout section-enter">
  <!-- Filter bar -->
  <div class="filter-bar">
    <div class="fg">
      <label class="flbl" for="f-search">SEARCH</label>
      <input id="f-search" bind:value={$filters.search} placeholder="Name, team, nation…" style="width:200px" />
    </div>
    <div class="fg">
      <label class="flbl" for="f-team">TEAM</label>
      <select id="f-team" bind:value={$filters.team}>
        {#each $dropdownTeams as t}<option>{t}</option>{/each}
      </select>
    </div>
    <div class="fg">
      <label class="flbl" for="f-country">COUNTRY</label>
      <select id="f-country" bind:value={$filters.country}>
        {#each $dropdownCountries as c}<option>{c}</option>{/each}
      </select>
    </div>
    <div class="fg">
      <label class="flbl" for="f-cont">CONTINENT</label>
      <select id="f-cont" bind:value={$filters.continent}>
        {#each $dropdownConts as c}<option>{c}</option>{/each}
      </select>
    </div>
    <div class="fg">
      <label class="flbl" for="f-grade">GRADE</label>
      <select id="f-grade" bind:value={$filters.grade}>
        <option>All Grades</option>
        {#each GRADES as g}<option>{g}</option>{/each}
      </select>
    </div>
    <div class="fg">
      <label class="flbl" for="f-minca">MIN CA</label>
      <input id="f-minca" type="number" bind:value={$filters.minCA} min="0" max="100" style="width:60px" />
    </div>
    <div class="fg">
      <label class="flbl" for="f-minage">AGE</label>
      <input id="f-minage" type="number" bind:value={$filters.minAge} min="0" max="50" style="width:50px" />
      <span style="color:#3a4e72;margin:0 4px">–</span>
      <input type="number" bind:value={$filters.maxAge} min="0" max="50" style="width:50px" />
    </div>
    <div class="checks">
      <label><input type="checkbox" bind:checked={$filters.onlyU23}   /> U23</label>
      <label><input type="checkbox" bind:checked={$filters.onlyFree}  /> Free Only</label>
      <label><input type="checkbox" bind:checked={$filters.onlyUpside}/> High Upside</label>
      <label><input type="checkbox" bind:checked={$filters.onlyHighPot}/> High Pot</label>
    </div>
    <button class="btn btn-ghost" on:click={() => filters.set(defaultFilters())}>Reset</button>
  </div>

  <!-- Type bar -->
  <div class="type-bar">
    <span class="flbl">TYPE</span>
    {#each TYPES as tid, i}
      {@const lbl = TYPE_LABELS[i]}
      {@const col = TYPE_COLORS[lbl] ?? "#263756"}
      <button class="btn btn-type" class:active={$filters.typeId===tid}
              style:background={$filters.typeId===tid ? col : ""}
              on:click={() => filters.update(f=>({...f,typeId:f.typeId===tid?null:tid}))}>
        {lbl}
      </button>
    {/each}
    <span class="count">{$filtered.length.toLocaleString()} riders</span>
  </div>

  <!-- Tabs -->
  <div class="tabs">
    {#each ["Roster","Youth Scout","Free Agents"] as t, i}
      <button class="tab" class:active={tab===i} on:click={()=>tab=i}>
        {t}
        {#if i===1}<span class="badge-num">{youth.length}</span>{/if}
        {#if i===2}<span class="badge-num">{agents.length}</span>{/if}
      </button>
    {/each}
  </div>

  <!-- Table -->
  <div class="table-area">
    {#if tab===0}
      <RiderTable data={$filtered} cols={ROSTER_COLS} />
    {:else if tab===1}
      <RiderTable data={youth} cols={YOUTH_COLS} />
    {:else}
      <RiderTable data={agents} cols={FREE_COLS} />
    {/if}
  </div>
</div>

<style>
  .scout { display:flex; flex-direction:column; height:100%; overflow:hidden; }
  .filter-bar { display:flex; flex-wrap:wrap; align-items:flex-end; gap:12px;
                padding:12px 16px; background:#111c30; border-bottom:1px solid #253550; flex-shrink:0; }
  .fg { display:flex; flex-direction:column; gap:4px; }
  .flbl { font-size:9px; font-weight:700; color:#4a5e80; letter-spacing:0.1em; text-transform:uppercase; }
  .checks { display:flex; flex-wrap:wrap; gap:10px; align-items:center; align-self:flex-end; padding-bottom:2px; }
  .checks label { display:flex; gap:5px; align-items:center; font-size:12px; color:#7888b0; cursor:pointer; }
  .type-bar { display:flex; align-items:center; gap:6px; padding:8px 16px;
              background:#0d1525; border-bottom:1px solid #1c2d48; flex-shrink:0; flex-wrap:wrap; }
  .count { margin-left:auto; font-size:11px; color:#4a5e80; }
  .tabs { display:flex; gap:0; background:#111c30; border-bottom:1px solid #253550; flex-shrink:0; }
  .tab { padding:10px 18px; background:none; border:none; color:#7284a8; font-size:13px;
          cursor:pointer; border-bottom:2px solid transparent; font-family:inherit; }
  .tab:hover { color:#dce8ff; }
  .tab.active { color:#e8b800; border-bottom-color:#e8b800; }
  .badge-num { background:#253550; color:#e8b800; border-radius:10px; padding:1px 6px;
               font-size:10px; font-weight:700; margin-left:5px; }
  .table-area { flex:1; overflow:hidden; }
</style>
