<script lang="ts">
  import { onMount } from "svelte";
  import { myTeam, toast, save } from "../stores";
  import { kitStatus, kitOodleDownload, kitOodlePick, kitApply, kitRemove, pickFile, pickImage, isDesktop, type ApplyReport } from "../api";
  import { kitStatusStore, kitEditsStore, refreshEdits, thumbVersion, teamLogo, loadTeamLogo, setTeamLogo, placeTeamLogo, findEdit } from "../kits/store";
  import { bytesToDataUrl, migrate } from "../kits/render";
  import { partLabel, partOrder, championCode } from "../kits/parts";
  import type { Recipe } from "../kits/render";
  import KitThumb from "../components/KitThumb.svelte";
  import KitEditor from "../components/KitEditor.svelte";
  import KitPicker from "../components/KitPicker.svelte";
  import ChampionMaker from "../components/ChampionMaker.svelte";
  import Icon from "../components/Icon.svelte";

  let loading = true;
  let error = "";
  let busy = "";
  let picker = false;
  let editing: { part: string; label: string; initial: Recipe | null } | null = null;
  let report: ApplyReport | null = null;

  async function refresh() {
    if (!isDesktop && !import.meta.env.VITE_MOCK_SAVE) { error = "Kits need the desktop app."; loading = false; return; }
    try {
      kitStatusStore.set(await kitStatus());
      await refreshEdits();
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  onMount(() => { refresh(); loadTeamLogo(); });

  $: status = $kitStatusStore;
  $: team = $myTeam;
  $: kitName = team?.jersey.toLowerCase() ?? "";
  $: kit = status?.kits.find((k) => k.abbr === kitName) ?? null;
  $: editsHere = $kitEditsStore.filter((e) => e.kit === kitName);
  $: editsElsewhere = $kitEditsStore.filter((e) => e.kit !== kitName);
  $: edited = (part: string) => editsHere.some((e) => e.part === part);
  $: parts = (kit?.parts ?? []).filter((p) => !championCode(p)).sort((a, b) => partOrder(a) - partOrder(b));
  $: miniPart = kit?.parts.includes("minimaillot") ? "minimaillot" : "maillot";
  // Kit parts plus champion jerseys made here, for "copy layers to".
  $: allParts = [...new Set([...(kit?.parts ?? []), ...editsHere.map((e) => e.part)])];
  $: countryName = (code: string) => ($save?.cyclists ?? []).find((c) => c.iso === code)?.nationality;
  $: latestEdit = Math.max(0, ...$kitEditsStore.map((e) => e.modified_ms));

  async function setupOodle() {
    busy = "Downloading Oodle…";
    try {
      await kitOodleDownload();
      thumbVersion.update((n) => n + 1);
      await refresh();
      toast("Oodle is ready");
    } catch (e) {
      toast(String(e), "error", 10000);
    } finally {
      busy = "";
    }
  }
  async function pickOodle() {
    const path = await pickFile("Oodle library", ["dll"]);
    if (!path) return;
    try { await kitOodlePick(path); thumbVersion.update((n) => n + 1); await refresh(); } catch (e) { toast(String(e), "error"); }
  }

  async function apply() {
    busy = "Building…";
    try {
      report = await kitApply();
      toast(report.parts ? `${report.parts} kit part${report.parts === 1 ? "" : "s"} applied. Start PCM to see them` : "No edits, the override was removed");
      await refresh();
    } catch (e) {
      toast(String(e), "error", 10000);
    } finally {
      busy = "";
    }
  }
  async function remove() {
    busy = "Removing…";
    try {
      await kitRemove();
      report = null;
      toast("Kit changes removed from the game. Your edits are still saved here");
      await refresh();
    } catch (e) {
      toast(String(e), "error", 10000);
    } finally {
      busy = "";
    }
  }

  // ─── Team logo ──────────────────────────────────────────────────────────────
  let logoBusy = "";
  /** Where the logo goes by default: where this template has its main logo. */
  const LOGO_SPOTS: Record<string, { x: number; y: number; scale: number }[]> = {
    maillot: [{ x: 0.244, y: 0.263, scale: 0.13 }, { x: 0.756, y: 0.276, scale: 0.16 }],
    minimaillot: [{ x: 0.512, y: 0.434, scale: 0.29 }],
  };
  const spotsFor = (p: string) => LOGO_SPOTS[p.startsWith("maillot") ? "maillot" : p.startsWith("minimaillot") ? "minimaillot" : ""] ?? [];
  const followsLogo = (p: string) => {
    const e = findEdit(kitName, p);
    return !!e && migrate(e.recipe).layers.some((l) => l.type === "image" && l.link === "team");
  };
  $: logoParts = ["maillot", "maillot_tour", "minimaillot", "minimaillot_tour"].filter((p) => kit?.parts.includes(p));
  $: logoCount = $kitEditsStore && logoParts.filter(followsLogo).length;

  async function chooseLogo() {
    const picked = await pickImage();
    if (!picked) return;
    logoBusy = "Updating…";
    try {
      const n = await setTeamLogo(bytesToDataUrl(picked.bytes, picked.name), (m) => (logoBusy = m));
      toast(n ? `Team logo changed on ${n} kit part${n === 1 ? "" : "s"}. Apply to game to see it` : "Team logo set. Put it on the kit next");
    } catch (e) {
      toast(String(e), "error", 8000);
    } finally {
      logoBusy = "";
    }
  }
  async function putLogoOnKit() {
    logoBusy = "Adding…";
    try {
      const todo = logoParts.filter((p) => !followsLogo(p));
      for (const p of todo) {
        logoBusy = `Adding to ${partLabel(p)}…`;
        await placeTeamLogo(kitName, p, spotsFor(p));
      }
      // Champion jerseys start from these parts, so redraw them too.
      await setTeamLogo($teamLogo, (m) => (logoBusy = m));
      toast(`Logo added to ${todo.length} part${todo.length === 1 ? "" : "s"}. Open a part to move it, or add a Patch layer to hide the old logo`, "ok", 7000);
    } catch (e) {
      toast(String(e), "error", 8000);
    } finally {
      logoBusy = "";
    }
  }

  function edit(part: string) {
    editing = { part, label: partLabel(part), initial: null };
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Kits</h1>
      <p>Edit your team's kit: add logos, recolour it, import your own artwork, and make national champion jerseys for your riders. Changes go into one extra file in PCM's mod folder, so the game and the mod stay untouched.</p>
    </div>
  </div>

  <div class="page-body">
    {#if loading}
      <p class="muted">Reading the kits in PCM{status?.mod_label ? ` and ${status.mod_label}` : ""}. The first time takes a few seconds…</p>
    {:else if error}
      <p class="err">{error}</p>
    {:else if status && team}
      {#if !status.oodle_ready}
        <section class="panel setup">
          <Icon name="info" size={20} />
          <div>
            <h3>One-time setup</h3>
            <p>PCM compresses its kit files with Oodle, Epic Games' compression library. PCM Recon downloads it once (2 MB, checksum verified) into its own data folder.</p>
            <div class="row">
              <button class="btn btn-primary" on:click={setupOodle} disabled={!!busy}>{busy || "Download Oodle"}</button>
              <button class="btn btn-quiet" on:click={pickOodle}>I have oo2core_9_win64.dll…</button>
            </div>
          </div>
        </section>
      {/if}

      <section class="panel hero">
        <div class="mini"><KitThumb kit={kitName} part={miniPart} edited={edited(miniPart)} alt="{team.name} kit" /></div>
        <div class="info">
          <h2>{team.name}</h2>
          <p class="muted">Kit <code>{team.jersey || "none"}</code>{#if kit}{kit.from_mod ? ` · from ${status.mod_label ?? "the mod"}` : ` · from ${status.game}`}{/if}</p>
          <div class="swatches">
            <span style="background:{team.color1}" title="Main colour {team.color1}"></span>
            <span style="background:{team.color2}" title="Second colour {team.color2}"></span>
          </div>
          <button class="btn btn-sm" on:click={() => (picker = true)}>Change kit…</button>
        </div>
        <div class="apply">
          <div class="state">
            {#if status.override_installed && latestEdit > status.override_modified_ms}
              <span class="stale"><Icon name="alert" size={14} />Newer edits not in the game yet</span>
            {:else if status.override_installed}
              <span class="ok"><Icon name="check" size={14} />In the game</span>
            {:else}
              <span class="muted">Not applied</span>
            {/if}
            <small class="muted">{$kitEditsStore.length} edited part{$kitEditsStore.length === 1 ? "" : "s"}</small>
          </div>
          <button class="btn btn-primary" on:click={apply} disabled={!!busy || !status.oodle_ready || !$kitEditsStore.length}>{busy === "Building…" ? busy : "Apply to game"}</button>
          {#if status.override_installed}<button class="btn btn-quiet btn-sm" on:click={remove} disabled={!!busy}>Remove from game</button>{/if}
          <small class="muted">Close PCM first. Changes show the next time it starts.</small>
        </div>
      </section>

      <section class="panel logo">
        <div class="logo-img">{#if $teamLogo}<img src={$teamLogo} alt="Team logo" />{:else}<span class="muted">No logo</span>{/if}</div>
        <div class="logo-info">
          <h3>Team logo</h3>
          <p class="muted">Choose your logo once. Parts that follow it switch together when you change it: jerseys, small menu jersey and champion jerseys.
            A black or white background is removed automatically.</p>
          <div class="row">
            <button class="btn btn-sm" class:btn-primary={!$teamLogo} on:click={chooseLogo} disabled={!!logoBusy}>{$teamLogo ? "Change logo…" : "Choose logo…"}</button>
            {#if $teamLogo && logoCount < logoParts.length}
              <button class="btn btn-sm btn-primary" on:click={putLogoOnKit} disabled={!!logoBusy}>Put it on the kit</button>
            {/if}
            {#if logoBusy}<span class="muted small">{logoBusy}</span>{:else if logoCount}<span class="ok small"><Icon name="check" size={13} />On {logoCount} part{logoCount === 1 ? "" : "s"}</span>{/if}
          </div>
        </div>
      </section>

      {#if report?.skipped.length}
        <section class="panel warn">
          <h3>Some parts weren't applied</h3>
          <ul>{#each report.skipped as s}<li>{s}</li>{/each}</ul>
        </section>
      {/if}

      {#if !kit}
        <p class="err">Kit "{team.jersey}" isn't in {status.game}{status.mod_label ? ` or ${status.mod_label}` : ""}. Pick another with Change kit.</p>
      {:else if status.oodle_ready}
        <section class="panel">
          <div class="panel-head"><h3>Kit parts</h3><span class="muted">Click a part to edit it</span></div>
          <div class="parts">
            {#each parts as p (p)}
              <button class="part" on:click={() => edit(p)}>
                <div class="img" class:wide={p.startsWith("maillot")}><KitThumb kit={kitName} part={p} edited={edited(p)} alt={partLabel(p)} /></div>
                <span>{partLabel(p)}</span>
                {#if edited(p)}<small class="tag">Edited</small>{/if}
              </button>
            {/each}
          </div>
        </section>

        <ChampionMaker kit={kitName} parts={kit.parts} teamId={team.id} on:edit={(e) => (editing = { ...e.detail })} />

        {#if editsElsewhere.length}
          <section class="panel">
            <div class="panel-head"><h3>Edits on other kits</h3><span class="muted">Still applied when you press Apply</span></div>
            <p class="muted pad">{[...new Set(editsElsewhere.map((e) => e.kit))].join(", ")}</p>
          </section>
        {/if}
      {/if}
    {:else}
      <p class="muted">Load a career to edit its kits.</p>
    {/if}
  </div>
</div>

{#if editing}
  <KitEditor kit={kitName} part={editing.part} label={editing.label} initial={editing.initial} parts={allParts} {countryName} on:close={() => (editing = null)} />
{/if}
{#if picker && team && status}
  <KitPicker {team} kits={status.kits} on:close={() => (picker = false)} />
{/if}

<style>
  .page-body { display: flex; flex-direction: column; gap: 16px; }
  .setup { display: flex; gap: 14px; padding: 16px; border-color: #f5c51855; }
  .setup :global(svg) { color: var(--jaune); flex-shrink: 0; margin-top: 2px; }
  .setup p { font-size: 13px; color: var(--ink-2); margin: 4px 0 10px; max-width: 70ch; }
  .row { display: flex; gap: 8px; }
  .hero { display: flex; gap: 18px; align-items: center; padding: 14px 18px; }
  .mini { width: 120px; height: 120px; flex-shrink: 0; }
  .info { display: flex; flex-direction: column; gap: 6px; align-items: flex-start; }
  .info h2 { font-size: 24px; }
  .info p { font-size: 13px; }
  code { font-size: 12px; color: var(--ink-2); }
  .swatches { display: flex; gap: 6px; }
  .swatches span { width: 22px; height: 22px; border-radius: 5px; border: 1px solid var(--rule-2); }
  .apply { margin-left: auto; display: flex; flex-direction: column; align-items: flex-end; gap: 6px; text-align: right; }
  .state { display: flex; flex-direction: column; align-items: flex-end; font-size: 13px; }
  .ok { color: var(--vert); display: inline-flex; gap: 4px; align-items: center; }
  .stale { color: var(--jaune); display: inline-flex; gap: 4px; align-items: center; }
  .apply small { font-size: 11.5px; }
  .warn { padding: 12px 16px; border-color: #e8524a66; }
  .logo { display: flex; gap: 16px; align-items: center; padding: 12px 18px; }
  .logo-img { width: 84px; height: 84px; flex-shrink: 0; border-radius: 8px; display: grid; place-items: center; overflow: hidden; background: repeating-conic-gradient(#2a3140 0 25%, #222834 0 50%) 0 0 / 14px 14px; }
  .logo-img img { width: 100%; height: 100%; object-fit: contain; }
  .logo-info { display: flex; flex-direction: column; gap: 6px; }
  .logo-info p { font-size: 12.5px; max-width: 70ch; }
  .small { font-size: 12px; }
  .row { align-items: center; }
  .warn ul { margin: 6px 0 0 18px; font-size: 12.5px; color: var(--ink-2); }
  .parts { display: grid; grid-template-columns: repeat(auto-fill, minmax(170px, 1fr)); gap: 10px; padding: 0 14px 14px; }
  .part { display: flex; flex-direction: column; gap: 6px; align-items: center; padding: 10px; border-radius: 8px; border: 1px solid var(--rule); background: var(--bg-2); color: var(--ink-2); font: inherit; font-size: 12.5px; cursor: pointer; position: relative; }
  .part:hover { border-color: var(--jaune); color: var(--ink); }
  .img { width: 100%; height: 110px; }
  .tag { position: absolute; top: 8px; right: 8px; font-size: 10.5px; color: var(--vert); background: var(--panel); border: 1px solid #3fb95055; border-radius: 999px; padding: 0 6px; }
  .pad { padding: 0 14px 14px; font-size: 13px; }
  .err { color: var(--pois); }
</style>
