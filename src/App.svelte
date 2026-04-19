<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { tick } from "svelte";
  import {
    activeSection,
    allCyclists,
    scoutReports,
    teams,
    teamRosters,
    gameDate,
    currentPath,
    isLoading,
    loadStatus,
    notes,
    shortlistIds,
    filters,
  } from "./lib/stores";
  import DetailPanel from "./lib/components/DetailPanel.svelte";
  import Dashboard from "./lib/sections/Dashboard.svelte";
  import Scouting from "./lib/sections/Scouting.svelte";
  import ScoutReports from "./lib/sections/ScoutReports.svelte";
  import Teams from "./lib/sections/Teams.svelte";
  import AllRiders from "./lib/sections/AllRiders.svelte";
  import FreeMarket from "./lib/sections/FreeMarket.svelte";
  import Rankings from "./lib/sections/Rankings.svelte";
  import Analytics from "./lib/sections/Analytics.svelte";
  import Shortlist from "./lib/sections/Shortlist.svelte";
  import { DONATE_URL } from "./lib/config";
  import type { SaveData } from "./lib/types";

  const NAV = [
    { icon: "\u25C8", name: "Dashboard" },
    { icon: "\u{1F50D}", name: "Scouting" },
    { icon: "\u{1F9ED}", name: "Scout Reports" },
    { icon: "\u{1F465}", name: "Teams" },
    { icon: "\u{1F6B4}", name: "All Riders" },
    { icon: "\u{1F4B0}", name: "Free Market" },
    { icon: "\u{1F3C6}", name: "Rankings" },
    { icon: "\u{1F4CA}", name: "Analytics" },
    { icon: "\u2605", name: "Shortlist" },
  ];

  async function openFile() {
    const path = (await open({
      filters: [{ name: "PCM Career Save", extensions: ["cdb"] }],
    }).catch(() => null)) as string | null;
    if (!path) return;
    await loadSave(path);
  }

  async function loadSave(path: string) {
    isLoading.set(true);
    loadStatus.set(`Loading ${path.split(/[\\/]/).pop()}…`);
    await tick(); // ensure spinner renders before heavy IPC call
    try {
      const data: SaveData = await invoke("load_save", { path });
      loadStatus.set(`Processing ${data.cyclists.length.toLocaleString()} riders…`);
      await tick();
      allCyclists.set(data.cyclists);
      scoutReports.set(data.scout_reports ?? []);
      teams.set(data.teams);
      await tick();
      gameDate.set(data.game_date);
      currentPath.set(path);

      const rosters: Record<number, typeof data.cyclists> = {};
      data.cyclists.forEach((c) => {
        (rosters[c.team_id] ??= []).push(c);
      });
      teamRosters.set(rosters);

      const dir = path.replace(/[^/\\]+$/, "");
      const npath = dir + "pcm_recon_notes.json";
      const savedNotes: Record<string, string> = await invoke("load_notes", { path: npath });
      notes.set(savedNotes);

      loadStatus.set(`${data.cyclists.length.toLocaleString()} riders | ${(data.scout_reports ?? []).length.toLocaleString()} scout reports | ${data.teams.length} teams | ${data.game_date}`);
      activeSection.set("Dashboard");
    } catch (e) {
      loadStatus.set(`Error: ${e}`);
      alert("Failed to load save:\n" + e);
    } finally {
      isLoading.set(false);
    }
  }

  async function autoLoad() {
    try {
      const path = await invoke<string | null>("find_default_save", { filename: "Career_1.cdb" });
      if (path) await loadSave(path);
    } catch {}
  }

  async function openDonate() {
    if (!DONATE_URL) return;
    try {
      await invoke("open_external", { url: DONATE_URL });
    } catch (e) {
      alert("Failed to open donation link:\n" + e);
    }
  }

  autoLoad();
</script>

{#if $isLoading}
  <div class="loading-overlay">
    <div class="loading-spinner"></div>
    <div class="loading-label">{$loadStatus || "Loading…"}</div>
  </div>
{/if}

<div class="app">
  <nav class="sidebar">
    <div class="logo">
      <span class="logo-icon">&#x1F6B4;</span>
      <div class="logo-text">
        <span class="logo-name">PCM Recon</span>
        <span class="logo-ver">v2.0.1</span>
      </div>
    </div>
    <div class="nav-items">
      {#each NAV as item}
        <button class="nav-btn" class:active={$activeSection === item.name} on:click={() => activeSection.set(item.name)}>
          <span class="nav-icon">{item.icon}</span>
          <span class="nav-label">{item.name}</span>
          {#if item.name === "Shortlist" && $shortlistIds.size > 0}
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
      {#if DONATE_URL}
        <button class="donate-btn" on:click={openDonate}>Donate</button>
      {/if}
    </div>
  </nav>

  <div class="content">
    <header class="header">
      <div class="status-text">
        {#if $isLoading}
          <span class="loading-dot"></span>
        {/if}
        {$loadStatus || "Open a PCM career save to begin"}
      </div>
      <div class="header-actions">
        <button
          class="btn btn-ghost"
          on:click={() => filters.update((f) => ({ ...f, search: "", team: "All Teams", country: "All Countries" }))}
        >
          Reset Filters
        </button>
        <button class="btn btn-accent" on:click={openFile} disabled={$isLoading}>
          {$isLoading ? "Loading..." : "Open Save"}
        </button>
      </div>
    </header>

    <div class="section-area">
      {#if $activeSection === "Dashboard"}<Dashboard />
      {:else if $activeSection === "Scouting"}<Scouting />
      {:else if $activeSection === "Scout Reports"}<ScoutReports />
      {:else if $activeSection === "Teams"}<Teams />
      {:else if $activeSection === "All Riders"}<AllRiders />
      {:else if $activeSection === "Free Market"}<FreeMarket />
      {:else if $activeSection === "Rankings"}<Rankings />
      {:else if $activeSection === "Analytics"}<Analytics />
      {:else if $activeSection === "Shortlist"}<Shortlist />
      {/if}
    </div>
  </div>

  <aside class="detail-aside">
    <DetailPanel />
  </aside>
</div>

<style>
  .loading-overlay {
    position: fixed; inset: 0; z-index: 9999;
    background: #0b1422;
    display: flex; flex-direction: column;
    align-items: center; justify-content: center; gap: 20px;
  }
  .loading-spinner {
    width: 52px; height: 52px;
    border: 3px solid #1c2d48;
    border-top-color: #e8b800;
    border-radius: 50%;
    animation: spin 0.75s linear infinite;
  }
  .loading-label {
    font-size: 13px; color: #7888b0; letter-spacing: 0.04em;
    max-width: 400px; text-align: center;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .app {
    display: grid;
    grid-template-columns: 196px 1fr 340px;
    grid-template-rows: 100vh;
    height: 100vh;
    overflow: hidden;
    background: #0d1525;
  }

  .sidebar {
    background: #0b1422;
    border-right: 1px solid #1c2d48;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 16px 14px;
    border-bottom: 1px solid #1c2d48;
  }

  .logo-icon {
    font-size: 22px;
  }

  .logo-text {
    display: flex;
    flex-direction: column;
  }

  .logo-name {
    font-size: 14px;
    font-weight: 700;
    color: #dce8ff;
    letter-spacing: 0.01em;
  }

  .logo-ver {
    font-size: 10px;
    color: #4a5e80;
  }

  .nav-items {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .nav-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 10px 14px;
    background: none;
    border: none;
    color: #7284a8;
    cursor: pointer;
    text-align: left;
    transition: background 0.12s, color 0.12s;
    font-family: inherit;
    font-size: 13px;
  }

  .nav-btn:hover {
    background: #172035;
    color: #dce8ff;
  }

  .nav-btn.active {
    background: linear-gradient(90deg, #1e2a10, #161e0a);
    color: #e8b800;
    border-left: 2px solid #e8b800;
  }

  .nav-icon {
    font-size: 15px;
    width: 20px;
    text-align: center;
    flex-shrink: 0;
  }

  .nav-label {
    flex: 1;
    font-weight: 500;
  }

  .nav-badge {
    background: #e85598;
    color: #fff;
    border-radius: 10px;
    padding: 1px 6px;
    font-size: 10px;
    font-weight: 700;
  }

  .sidebar-footer {
    border-top: 1px solid #1c2d48;
    padding: 12px 14px;
  }

  .donate-btn {
    margin-top: 12px;
    width: 100%;
    border: 1px solid #e8b80055;
    background: linear-gradient(180deg, #2a2308, #1c1808);
    color: #f2cf48;
    border-radius: 8px;
    padding: 9px 10px;
    font: inherit;
    font-size: 12px;
    font-weight: 700;
    cursor: pointer;
    transition: background 0.12s, transform 0.12s, border-color 0.12s;
  }

  .donate-btn:hover {
    background: linear-gradient(180deg, #3a300a, #241d08);
    border-color: #e8b80088;
  }

  .save-label {
    display: block;
    font-size: 9px;
    font-weight: 700;
    color: #4a5e80;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-bottom: 4px;
  }

  .save-val {
    display: block;
    font-size: 11px;
    color: #7284a8;
    word-break: break-all;
  }

  .save-date {
    display: block;
    font-size: 10px;
    color: #4a5e80;
    margin-top: 2px;
  }

  .content {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    height: 52px;
    background: #111c30;
    border-bottom: 1px solid #253550;
    flex-shrink: 0;
    gap: 12px;
  }

  .status-text {
    font-size: 12px;
    color: #7888b0;
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .loading-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #e8b800;
    animation: pulse 1s infinite;
    flex-shrink: 0;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }

    50% {
      opacity: 0.3;
    }
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .section-area {
    flex: 1;
    overflow: hidden;
  }

  .detail-aside {
    border-left: 1px solid #1c2d48;
    overflow: hidden;
  }
</style>
