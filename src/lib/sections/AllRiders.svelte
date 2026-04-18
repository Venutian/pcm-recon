<script lang="ts">
  import { allCyclists } from "../stores";
  import RiderTable from "../components/RiderTable.svelte";
  import type { Col } from "../types";
  import { GRADES } from "../types";
  import { fmtNat } from "../format";

  const COLS: Col[] = [
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

  let search = "";
  let grade  = "All Grades";

  $: data = $allCyclists.filter(c => {
    if (search && !["name","team","nationality","rider_type"].some(k => String((c as Record<string,unknown>)[k]??"").toLowerCase().includes(search.toLowerCase()))) return false;
    if (grade !== "All Grades" && c.scout_grade !== grade) return false;
    return true;
  });
</script>

<div class="allr section-enter">
  <div class="toolbar">
    <span class="flbl">SEARCH</span>
    <input bind:value={search} placeholder="Name, team, nation, type…" style="width:220px" />
    <span class="flbl" style="margin-left:16px">GRADE</span>
    <select bind:value={grade}>
      <option>All Grades</option>
      {#each GRADES as g}<option>{g}</option>{/each}
    </select>
    <span class="count">{data.length.toLocaleString()} / {$allCyclists.length.toLocaleString()} riders</span>
  </div>
  <div class="table-wrap">
    <RiderTable {data} cols={COLS} />
  </div>
</div>

<style>
  .allr { display:flex; flex-direction:column; height:100%; overflow:hidden; }
  .toolbar { display:flex; align-items:center; gap:10px; padding:10px 16px;
             background:#111c30; border-bottom:1px solid #253550; flex-shrink:0; }
  .flbl { font-size:9px; font-weight:700; color:#4a5e80; letter-spacing:0.1em; text-transform:uppercase; }
  .count { margin-left:auto; font-size:11px; color:#4a5e80; }
  .table-wrap { flex:1; overflow:hidden; }
</style>
