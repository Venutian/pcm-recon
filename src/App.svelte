<script lang="ts">
  import { onMount } from "svelte";
  import { activeSection, save, meta, finance, myTeam, isLoading, loadStatus, selectedRider, paletteOpen, shortlist, compareIds, lastSavePath, goTo } from "./lib/stores";
  import { loadWorkspace, loadSave, reloadSave, pickSave, saveModified, openExternal, isDesktop } from "./lib/api";
  import { NAV } from "./lib/nav";
  import { eurShort, fmtDate, teamColor } from "./lib/format";
  import { DONATE_URL } from "./lib/config";
  import Icon from "./lib/components/Icon.svelte";
  import DetailPanel from "./lib/components/DetailPanel.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import CommandPalette from "./lib/components/CommandPalette.svelte";
  import Welcome from "./lib/components/Welcome.svelte";
  import WriteConfirm from "./lib/components/WriteConfirm.svelte";
  import Overview from "./lib/sections/Overview.svelte";
  import Prospects from "./lib/sections/Prospects.svelte";
  import Scout from "./lib/sections/Scout.svelte";
  import Market from "./lib/sections/Market.svelte";
  import Shortlist from "./lib/sections/Shortlist.svelte";
  import Compare from "./lib/sections/Compare.svelte";
  import MyTeam from "./lib/sections/MyTeam.svelte";
  import Finances from "./lib/sections/Finances.svelte";
  import Teams from "./lib/sections/Teams.svelte";
  import Rankings from "./lib/sections/Rankings.svelte";
  import Database from "./lib/sections/Database.svelte";
  import NamePacks from "./lib/sections/NamePacks.svelte";

  const SECTIONS: Record<string, any> = {
    Overview, Prospects, Scout, Market, Shortlist, Compare,
    "My team": MyTeam, Finances, Teams, Rankings, Database, "Name packs": NamePacks,
  };
  const GROUPS = ["Scouting", "Your team", "League", "Tools"] as const;

  let changedOnDisk = false;

  onMount(async () => {
    const ws = await loadWorkspace();
    const last = ws.lastSave || $lastSavePath;
    if (last) await loadSave(last, true);
    else if (!isDesktop && import.meta.env.VITE_MOCK_SAVE) await loadSave("preview.cdb", true);
  });

  async function checkDisk() {
    if (!$save || !isDesktop) return;
    const m = await saveModified($save.meta.path).catch(() => 0);
    changedOnDisk = m > 0 && Math.abs(m - $save.modified_ms) > 1500;
  }
  $: if ($save) changedOnDisk = false;

  function onKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") { e.preventDefault(); paletteOpen.update((v) => !v); }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "o") { e.preventDefault(); pickSave(); }
    if (e.key === "F5" && $save) { e.preventDefault(); reloadSave(); }
  }

  $: badge = (name: string) => name === "Shortlist" ? $shortlist.size : name === "Compare" ? $compareIds.length : 0;
  $: balance = $finance?.balance ?? 0;
</script>

<svelte:window on:keydown={onKey} on:focus={checkDisk} />

{#if $isLoading}
  <div class="loading" role="status">
    <div class="wheel"><Icon name="wheel" size={44} stroke={1.5} /></div>
    <div>{$loadStatus || "Loading…"}</div>
  </div>
{/if}

<div class="app" class:with-detail={!!$selectedRider && !!$save}>
  <nav class="side">
    <div class="brand">
      <img class="mark" src="/logo.svg" alt="" />
      <div><strong>PCM Recon</strong><small>v3.1</small></div>
    </div>
    {#if $save}
      {#each GROUPS as g}
        <div class="group">
          <span class="glabel">{g}</span>
          {#each NAV.filter((n) => n.group === g) as item}
            <button class="nav" class:active={$activeSection === item.name} on:click={() => goTo(item.name)}>
              <Icon name={item.icon} size={17} />
              <span>{item.name}</span>
              {#if badge(item.name)}<span class="badge">{badge(item.name)}</span>{/if}
            </button>
          {/each}
        </div>
      {/each}
    {/if}
    <div class="side-foot">
      {#if $save}
        <div class="file" title={$save.meta.path}>
          <span>{$save.meta.file_name}</span>
          <small>{$save.meta.mod_name || "PCM database"}</small>
        </div>
      {/if}
      {#if DONATE_URL}
        <button class="btn btn-quiet btn-sm" on:click={() => openExternal(DONATE_URL)}>Support on Ko-fi</button>
      {/if}
    </div>
  </nav>

  <main>
    {#if $save}
      <header class="strip">
        <div class="career">
          <span class="jersey" style="background:{teamColor($myTeam?.color1, $myTeam?.id ?? 0)}; --c2:{$myTeam?.color2 || 'transparent'}"></span>
          <div>
            <strong>{$meta?.user_team || "No team"}</strong>
            <small>{$myTeam?.division ? `${$myTeam.division} · ` : ""}{fmtDate($meta?.game_date ?? "")}</small>
          </div>
        </div>
        {#if $finance}
          <button class="bal" class:neg={balance < 0} on:click={() => goTo("Finances")} title="Open finances to adjust your balance">
            <small>Balance</small>
            <span>{eurShort(balance)}</span>
          </button>
          <div class="budget"><small>Season budget</small><span>{eurShort($finance.season_budget)}</span></div>
        {/if}
        <div class="spacer"></div>
        {#if changedOnDisk}
          <button class="btn btn-sm changed" on:click={reloadSave}><Icon name="alert" size={14} />Save changed in PCM, reload</button>
        {/if}
        <button class="btn btn-quiet btn-sm" on:click={() => paletteOpen.set(true)} title="Search (Ctrl+K)"><Icon name="scout" size={15} />Search<kbd>Ctrl K</kbd></button>
        <button class="btn btn-quiet btn-sm" on:click={reloadSave} title="Reload from disk (F5)"><Icon name="reload" size={15} />Reload</button>
        <button class="btn btn-sm" on:click={pickSave} title="Open another save (Ctrl+O)"><Icon name="open" size={15} />Open save</button>
      </header>
      <div class="section">
        {#key $activeSection}
          <svelte:component this={SECTIONS[$activeSection] ?? Overview} />
        {/key}
      </div>
    {:else}
      <Welcome />
    {/if}
  </main>

  {#if $save && $selectedRider}
    <aside class="detail"><DetailPanel /></aside>
  {/if}
</div>

<CommandPalette />
<WriteConfirm />
<Toasts />

<style>
  .app { display: grid; grid-template-columns: 210px minmax(0, 1fr); height: 100vh; overflow: hidden; }
  .app.with-detail { grid-template-columns: 210px minmax(0, 1fr) 380px; }

  .side { background: var(--bg-2); border-right: 1px solid var(--rule); display: flex; flex-direction: column; overflow-y: auto; }
  .brand { display: flex; align-items: center; gap: 10px; padding: 16px 16px 14px; }
  .mark { width: 38px; height: 38px; margin: -2px; }
  .brand strong { display: block; font-family: var(--font-cond); font-size: 19px; font-weight: 700; line-height: 1; }
  .brand small { color: var(--ink-4); font-size: 11px; }
  .group { padding: 8px 8px 4px; }
  .glabel { display: block; font-size: 11px; color: var(--ink-4); padding: 6px 10px 4px; }
  .nav {
    width: 100%; display: flex; align-items: center; gap: 10px; padding: 7px 10px; border-radius: 6px;
    background: none; border: none; color: var(--ink-2); font: inherit; font-size: 13.5px; cursor: pointer; text-align: left;
  }
  .nav:hover { background: var(--panel); color: var(--ink); }
  .nav.active { background: var(--panel-2); color: var(--ink); box-shadow: inset 3px 0 0 var(--jaune); }
  .nav.active :global(svg) { color: var(--jaune); }
  .badge { margin-left: auto; font-size: 11px; font-weight: 700; background: var(--rule); color: var(--ink); border-radius: 999px; padding: 0 7px; }
  .side-foot { margin-top: auto; padding: 12px 16px 16px; display: flex; flex-direction: column; gap: 10px; }
  .file { display: flex; flex-direction: column; font-size: 12px; color: var(--ink-2); overflow: hidden; }
  .file span { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .file small { color: var(--ink-4); }

  main { display: flex; flex-direction: column; overflow: hidden; min-width: 0; }
  .strip {
    display: flex; align-items: center; gap: 22px; padding: 10px 18px; min-height: 60px;
    border-bottom: 1px solid var(--rule); background: var(--bg-2); flex-shrink: 0;
  }
  .career { display: flex; align-items: center; gap: 11px; }
  .jersey { width: 26px; height: 30px; border-radius: 5px 5px 3px 3px; position: relative; box-shadow: inset 0 -6px 0 var(--c2); }
  .career strong { display: block; font-family: var(--font-cond); font-size: 19px; font-weight: 600; line-height: 1.05; }
  .career small, .bal small, .budget small { display: block; font-size: 11px; color: var(--ink-3); }
  .bal { background: none; border: none; color: var(--vert); text-align: left; cursor: pointer; padding: 2px 8px; border-radius: 6px; font: inherit; }
  .bal:hover { background: var(--panel); }
  .bal.neg { color: var(--pois); }
  .bal span, .budget span { font-family: var(--font-cond); font-size: 22px; font-weight: 700; line-height: 1; }
  .spacer { flex: 1; }
  kbd { font-size: 10px; border: 1px solid var(--rule-2); border-radius: 3px; padding: 0 4px; color: var(--ink-3); margin-left: 2px; }
  .changed { border-color: var(--jaune); color: var(--jaune); }
  .section { flex: 1; overflow: hidden; }

  .detail { border-left: 1px solid var(--rule); overflow: hidden; animation: slide 0.16s ease-out; }
  @keyframes slide { from { transform: translateX(16px); opacity: 0; } }

  .loading {
    position: fixed; inset: 0; z-index: 9999; background: rgba(11, 15, 20, 0.92);
    display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; color: var(--ink-2);
  }
  .wheel { color: var(--jaune); animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  @media (max-width: 1250px) {
    .app.with-detail { grid-template-columns: 64px minmax(0, 1fr) 360px; }
    .app.with-detail .nav span, .app.with-detail .glabel, .app.with-detail .brand div, .app.with-detail .side-foot { display: none; }
    .app.with-detail .badge { display: none; }
  }
</style>
