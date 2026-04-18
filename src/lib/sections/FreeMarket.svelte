<script lang="ts">
  import { allCyclists } from "../stores";
  import RiderTable from "../components/RiderTable.svelte";
  import type { Col } from "../types";
  import { TYPE_COLORS } from "../types";
  import { fmtNat } from "../format";

  const COLS: Col[] = [
    { key:"name",            label:"Name",    width:200, align:"left" },
    { key:"nat_flag",        label:"Country", width:130, align:"left", fmt:(_,r)=>fmtNat(r) },
    { key:"age",             label:"Age",     width:44,  align:"center" },
    { key:"current_ability", label:"CA",      width:48,  align:"center" },
    { key:"potential",       label:"Stars",   width:52,  align:"center" },
    { key:"growth",          label:"Upside",  width:58,  align:"center" },
    { key:"scout_grade",     label:"Grade",   width:110, align:"left" },
    { key:"rider_type",      label:"Type",    width:110, align:"left" },
    { key:"specialty_rating",label:"Spec",    width:50,  align:"center" },
    { key:"flat",            label:"Flat",    width:46,  align:"center" },
    { key:"mountain",        label:"Mtn",     width:46,  align:"center" },
    { key:"timetrial",       label:"TT",      width:46,  align:"center" },
    { key:"sprint",          label:"Spr",     width:46,  align:"center" },
  ];

  const TYPES: [string, number|null][] = [
    ["All",null],["Climber",2],["Sprinter",4],["Time Trialist",3],
    ["All-Rounder",1],["Classics",6],["Ardennes",5],["Rouleur",7],
  ];

  let typeId: number|null = null;

  $: data = $allCyclists.filter(c =>
    c.free_agent && (typeId === null || c.rider_type_id === typeId)
  );
</script>

<div class="fm section-enter">
  <div class="toolbar">
    <span class="flbl">TYPE</span>
    {#each TYPES as [lbl, tid]}
      {@const col = TYPE_COLORS[lbl] ?? "#263756"}
      <button class="btn btn-type" class:active={typeId===tid}
              style:background={typeId===tid ? col : ""}
              on:click={() => typeId = typeId===tid ? null : tid}>
        {lbl}
      </button>
    {/each}
    <span class="count">{data.length.toLocaleString()} free agents</span>
  </div>
  <div class="table-wrap">
    <RiderTable {data} cols={COLS} />
  </div>
</div>

<style>
  .fm { display:flex; flex-direction:column; height:100%; overflow:hidden; }
  .toolbar { display:flex; align-items:center; gap:6px; padding:10px 16px;
             background:#111c30; border-bottom:1px solid #253550; flex-shrink:0; flex-wrap:wrap; }
  .flbl { font-size:9px; font-weight:700; color:#4a5e80; letter-spacing:0.1em; text-transform:uppercase; }
  .count { margin-left:auto; font-size:11px; color:#4a5e80; }
  .table-wrap { flex:1; overflow:hidden; }
</style>
