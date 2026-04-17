<script lang="ts">
  import { allCyclists, shortlistIds, currentPath, filtered } from "../stores";
  import RiderTable from "../components/RiderTable.svelte";
  import type { Col } from "../types";
  import { fmtNat } from "../format";
  import { invoke } from "@tauri-apps/api/core";

  const COLS: Col[] = [
    { key:"name",            label:"Name",    width:200, align:"left" },
    { key:"nat_flag",        label:"Country", width:130, align:"left", fmt:(_,r)=>fmtNat(r) },
    { key:"team_short",      label:"Team",    width:150, align:"left" },
    { key:"age",             label:"Age",     width:44,  align:"center" },
    { key:"current_ability", label:"CA",      width:50,  align:"center" },
    { key:"potential",       label:"Stars",   width:52,  align:"center" },
    { key:"growth",          label:"Upside",  width:58,  align:"center" },
    { key:"scout_grade",     label:"Grade",   width:110, align:"left" },
    { key:"rider_type",      label:"Type",    width:110, align:"left" },
    { key:"flat",            label:"Flat",    width:46,  align:"center" },
    { key:"mountain",        label:"Mtn",     width:46,  align:"center" },
    { key:"timetrial",       label:"TT",      width:46,  align:"center" },
    { key:"sprint",          label:"Spr",     width:46,  align:"center" },
  ];

  $: data = [...$allCyclists.filter(c=>$shortlistIds.has(c.id))]
    .sort((a,b)=>b.current_ability-a.current_ability);

  function clearAll() {
    if (confirm("Remove all riders from your shortlist?")) shortlistIds.set(new Set());
  }

  async function exportCSV() {
    if (!data.length) return;
    const fields = ["name","nationality","iso","team","age","rider_type","current_ability",
                    "potential","growth","scout_grade","flat","mountain","timetrial","sprint","cobble"];
    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({ filters:[{name:"CSV",extensions:["csv"]}], defaultPath:"pcm_shortlist.csv" }).catch(()=>null);
    if (!path) return;
    await invoke("export_csv", { path, data, fields }).catch(e=>alert("Export failed: "+e));
  }
</script>

<div class="sl section-enter">
  <div class="toolbar">
    <span class="hdr-text">SHORTLISTED RIDERS</span>
    <span class="count">{$shortlistIds.size} riders</span>
    <button class="btn btn-ghost" on:click={exportCSV}>Export CSV</button>
    {#if $shortlistIds.size > 0}
      <button class="btn btn-danger" on:click={clearAll}>Clear All</button>
    {/if}
  </div>
  {#if data.length === 0}
    <div class="empty">
      <span style="font-size:32px;opacity:0.3">★</span>
      <p>Star riders from any table to add them here</p>
    </div>
  {:else}
    <div class="table-wrap">
      <RiderTable {data} cols={COLS} prefix="sl" />
    </div>
  {/if}
</div>

<style>
  .sl { display:flex; flex-direction:column; height:100%; overflow:hidden; }
  .toolbar { display:flex; align-items:center; gap:10px; padding:10px 16px;
             background:#0d1525; border-bottom:1px solid #111c30; flex-shrink:0; }
  .hdr-text { font-size:11px; font-weight:700; color:#dce8ff; letter-spacing:0.06em; }
  .count { font-size:11px; color:#3a4e72; }
  .table-wrap { flex:1; overflow:hidden; }
  .empty { display:flex; flex-direction:column; align-items:center; justify-content:center;
           height:100%; color:#3a4e72; gap:12px; }
  .empty p { font-size:13px; }
</style>
