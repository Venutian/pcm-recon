<script lang="ts">
  import { invoke }       from "@tauri-apps/api/core";
  import { open }         from "@tauri-apps/plugin-dialog";
  import { activeSection, allCyclists, teams, teamRosters, gameDate,
           currentPath, isLoading, loadStatus, notes, shortlistIds, filters } from "./lib/stores";
  import DetailPanel from "./lib/components/DetailPanel.svelte";
  import Dashboard   from "./lib/sections/Dashboard.svelte";
  import Scouting    from "./lib/sections/Scouting.svelte";
  import Teams       from "./lib/sections/Teams.svelte";
  import AllRiders   from "./lib/sections/AllRiders.svelte";
  import FreeMarket  from "./lib/sections/FreeMarket.svelte";
  import Rankings    from "./lib/sections/Rankings.svelte";
  import Analytics   from "./lib/sections/Analytics.svelte";
  import Shortlist   from "./lib/sections/Shortlist.svelte";
  import type { SaveData } from "./lib/types";

  const NAV = [
    { icon:"◈",  name:"Dashboard"  },
    { icon:"🔭", name:"Scouting"   },
    { icon:"👥", name:"Teams"      },
    { icon:"🚴", name:"All Riders" },
    { icon:"💰", name:"Free Market"},
    { icon:"🏆", name:"Rankings"   },
    { icon:"📊", name:"Analytics"  },
    { icon:"★",  name:"Shortlist"  },
  ];

  async function openFile() {
    const path = await open({
      filters: [{ name:"PCM Career Save", extensions:["cdb"] }],
    }).catch(()=>null) as string|null;
    if (!path) return;
    await loadSave(path);
  }

  async function loadSave(path: string) {
    isLoading.set(true);
    loadStatus.set(`Loading ${path.split(/[\\/]/).pop()}…`);
    try {
      const data: SaveData = await invoke("load_save", { path });
      allCyclists.set(data.cyclists);
      teams.set(data.teams);
      gameDate.set(data.game_date);
      currentPath.set(path);

      // Build team rosters
      const rosters: Record<number, typeof data.cyclists> = {};
      data.cyclists.forEach(c => {
        (rosters[c.team_id] ??= []).push(c);
      });
      teamRosters.set(rosters);

      // Load notes
      const dir  = path.replace(/[^/\\]+$/, "");
      const npath = dir + "pcm_recon_notes.json";
      const savedNotes: Record<string,string> = await invoke("load_notes", { path: npath });
      notes.set(savedNotes);

      loadStatus.set(`${data.cyclists.length.toLocaleString()} riders · ${data.teams.length} teams · ${data.game_date}`);
      activeSection.set("Dashboard");
    } catch (e) {
      loadStatus.set(`Error: ${e}`);
      alert("Failed to load save:\n" + e);
    } finally {
      isLoading.set(false);
    }
  }

  // Auto-load Career_1.cdb from same dir as exe
  async function autoLoad() {
    try {
      // Try current working directory
      const paths = ["Career_1.cdb"];
      for (const p of paths) {
        const ok = await invoke("load_save", { path: p }).then(()=>true).catch(()=>false);
        if (ok) { await loadSave(p); return; }
      }
    } catch {}
  }

  autoLoad();
</script>

<div class="app">
  <!-- Sidebar -->
  <nav class="sidebar">
    <div class="logo">
      <span class="logo-icon">🚴</span>
      <div class="logo-text">
        <span class="logo-name">PCM Recon</span>
        <span class="logo-ver">v2.0</span>
      </div>
    </div>
    <div class="nav-items">
      {#each NAV as item}
        <button class="nav-btn" class:active={$activeSection===item.name}
                on:click={() => activeSection.set(item.name)}>
          <span class="nav-icon">{item.icon}</span>
          <span class="nav-label">{item.name}</span>
          {#if item.name==="Shortlist" && $shortlistIds.size>0}
            <span class="nav-badge">{$shortlistIds.size}</span>
          {/if}
        </button>
      {/each}
    </div>
    <div class="sidebar-footer">
      <div class="save-info">
        <span class="save-label">SAVE FILE</span>
        <span class="save-val">{$currentPath ? $currentPath.split(/[\\/]/).pop() : "No file loaded"}</span>
        {#if $gameDate}<span class="save-date">{$gameDate}</span>{/if}
      </div>
    </div>
  </nav>

  <!-- Main content -->
  <div class="content">
    <!-- Header -->
    <header class="header">
      <div class="status-text">
        {#if $isLoading}
          <span class="loading-dot"></span>
        {/if}
        {$loadStatus || "Open a PCM career save to begin"}
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost" on:click={() => filters.update(f=>({...f,search:"",team:"All Teams",country:"All Countries"}))}>
          Reset Filters
        </button>
        <button class="btn btn-accent" on:click={openFile} disabled={$isLoading}>
          {$isLoading ? "Loading…" : "Open Save"}
        </button>
      </div>
    </header>

    <!-- Section views -->
    <div class="section-area">
      {#if $activeSection==="Dashboard"}  <Dashboard />
      {:else if $activeSection==="Scouting"}   <Scouting />
      {:else if $activeSection==="Teams"}       <Teams />
      {:else if $activeSection==="All Riders"}  <AllRiders />
      {:else if $activeSection==="Free Market"} <FreeMarket />
      {:else if $activeSection==="Rankings"}    <Rankings />
      {:else if $activeSection==="Analytics"}   <Analytics />
      {:else if $activeSection==="Shortlist"}   <Shortlist />
      {/if}
    </div>
  </div>

  <!-- Detail panel -->
  <aside class="detail-aside">
    <DetailPanel />
  </aside>
</div>

<style>
  .app { display:grid; grid-template-columns:196px 1fr 340px; grid-template-rows:100vh;
         height:100vh; overflow:hidden; background:#0d1525; }

  /* Sidebar */
  .sidebar { background:#0b1422; border-right:1px solid #1c2d48; display:flex;
             flex-direction:column; overflow:hidden; }
  .logo { display:flex; align-items:center; gap:10px; padding:16px 14px;
          border-bottom:1px solid #1c2d48; }
  .logo-icon { font-size:22px; }
  .logo-text { display:flex; flex-direction:column; }
  .logo-name { font-size:14px; font-weight:700; color:#dce8ff; letter-spacing:0.01em; }
  .logo-ver  { font-size:10px; color:#4a5e80; }
  .nav-items { flex:1; overflow-y:auto; padding:8px 0; }
  .nav-btn { display:flex; align-items:center; gap:10px; width:100%; padding:10px 14px;
             background:none; border:none; color:#7284a8; cursor:pointer; text-align:left;
             transition:background 0.12s, color 0.12s; font-family:inherit; font-size:13px; }
  .nav-btn:hover  { background:#172035; color:#dce8ff; }
  .nav-btn.active { background:linear-gradient(90deg,#1e2a10,#161e0a); color:#e8b800;
                    border-left:2px solid #e8b800; }
  .nav-icon  { font-size:15px; width:20px; text-align:center; flex-shrink:0; }
  .nav-label { flex:1; font-weight:500; }
  .nav-badge { background:#e85598; color:#fff; border-radius:10px; padding:1px 6px;
               font-size:10px; font-weight:700; }
  .sidebar-footer { border-top:1px solid #1c2d48; padding:12px 14px; }
  .save-label { display:block; font-size:9px; font-weight:700; color:#4a5e80;
                letter-spacing:0.1em; text-transform:uppercase; margin-bottom:4px; }
  .save-val  { display:block; font-size:11px; color:#7284a8; word-break:break-all; }
  .save-date { display:block; font-size:10px; color:#4a5e80; margin-top:2px; }

  /* Content */
  .content { display:flex; flex-direction:column; overflow:hidden; }
  .header { display:flex; align-items:center; justify-content:space-between;
            padding:0 16px; height:52px; background:#111c30;
            border-bottom:1px solid #253550; flex-shrink:0; gap:12px; }
  .status-text { font-size:12px; color:#7888b0; flex:1; display:flex; align-items:center; gap:8px; }
  .loading-dot { width:8px; height:8px; border-radius:50%; background:#e8b800;
                 animation:pulse 1s infinite; flex-shrink:0; }
  @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.3} }
  .header-actions { display:flex; gap:8px; }
  .section-area { flex:1; overflow:hidden; }

  /* Detail aside */
  .detail-aside { border-left:1px solid #1c2d48; overflow:hidden; }
</style>
