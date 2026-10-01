<script lang="ts">
  import { onMount } from "svelte";
  import { meta, toast } from "../stores";
  import { namePacks, applyPackToGame, applyPackToSave, isDesktop, type NamePacksInfo, type NameDb, type Rename } from "../api";
  import Modal from "../components/Modal.svelte";
  import Icon from "../components/Icon.svelte";

  const PACK = "eri-tigrinya";
  let info: NamePacksInfo | null = null;
  let error = "";
  let busy = false;
  let confirm: { kind: "game"; db: NameDb } | { kind: "save" } | null = null;
  let renames: Rename[] | null = null;
  let gameDone = "";

  async function refresh() {
    if (!isDesktop) { error = "Name packs need the desktop app."; return; }
    try { info = await namePacks(PACK); error = ""; } catch (e) { error = String(e); }
  }
  onMount(refresh);

  $: pack = info?.packs.find((p) => p.key === PACK);
  $: packFirst = new Set(pack?.first ?? []);
  $: packLast = new Set(pack?.last ?? []);
  const isApplied = (db: NameDb) =>
    !!pack && db.current_first.length === pack.first.length && db.current_first.every((n) => packFirst.has(n)) && db.current_last.every((n) => packLast.has(n));

  async function run() {
    if (!confirm || !info) return;
    busy = true;
    try {
      if (confirm.kind === "game") {
        const r = await applyPackToGame(PACK, confirm.db.path, info.country_id);
        gameDone = `Removed ${r.report.removed_first.length + r.report.removed_last.length} old names, added ${r.report.added_first} first names and ${r.report.added_last} surnames. Copies kept in ${info.copies_folder}.`;
        toast(`${confirm.db.game} name list updated`);
      } else {
        renames = await applyPackToSave(PACK);
        toast(renames.length ? `Renamed ${renames.length} riders in ${$meta?.file_name}` : "No riders needed renaming");
      }
      confirm = null;
      await refresh();
    } catch (e) {
      toast(String(e), "error", 9000);
    } finally {
      busy = false;
    }
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Name packs</h1>
      <p>Replace the names PCM invents for new riders of a nation, and fix riders it already generated in your career. Real riders are never renamed.</p>
    </div>
  </div>

  <div class="page-body">
    {#if error}<p class="err">{error}</p>{/if}
    {#if pack && info}
      <section class="panel">
        <div class="panel-head"><h3>{pack.label}</h3><span class="muted">{pack.first.length} first names · {pack.last.length} surnames</span></div>
        <div class="pack">
          <div>
            <h4>First names</h4>
            <p class="muted">Names young Eritreans are given today.</p>
            <div class="chips">{#each pack.first as n}<span>{n}</span>{/each}</div>
          </div>
          <div>
            <h4>Surnames</h4>
            <p class="muted">Traditional names. In Eritrea the family name is the father's or grandfather's given name.</p>
            <div class="chips">{#each pack.last as n}<span>{n}</span>{/each}</div>
          </div>
        </div>
      </section>

      <div class="two">
        <section class="panel">
          <div class="panel-head"><h3>Game name list</h3><span class="muted">Used for every rider generated from now on</span></div>
          {#each info.databases as db}
            {@const applied = isApplied(db)}
            {@const wrong = [...db.current_first.filter((n) => !packFirst.has(n)), ...db.current_last.filter((n) => !packLast.has(n))]}
            <div class="db">
              <div class="db-head">
                <strong>{db.game}</strong>
                {#if applied}<span class="ok"><Icon name="check" size={14} />Pack applied</span>{:else}<span class="bad">{wrong.length} names not in the pack</span>{/if}
              </div>
              <code>{db.path}</code>
              {#if !applied && wrong.length}
                <div class="chips wrong">{#each wrong.slice(0, 40) as n}<span>{n}</span>{/each}{#if wrong.length > 40}<span>+{wrong.length - 40} more</span>{/if}</div>
              {/if}
              <button class="btn" class:btn-primary={!applied} on:click={() => (confirm = { kind: "game", db })}>
                {applied ? "Apply again" : "Apply to the game"}
              </button>
            </div>
          {:else}
            <p class="muted pad">No PCM installation with a name list was found.</p>
          {/each}
          {#if gameDone}<p class="done">{gameDone}</p>{/if}
          <p class="muted pad small">Copies of the original and the edited file are kept in <code>{info.copies_folder}</code>. After a game update, come back here and apply the pack again.</p>
        </section>

        <section class="panel">
          <div class="panel-head"><h3>Riders already in {$meta?.file_name}</h3></div>
          <p class="muted pad">Renames Eritrean riders the game generated whose names aren't genuine: glued-together names, typos, place names. Riders from your database and anyone born before the career started keep their names. Their results history is updated too.</p>
          <div class="pad"><button class="btn btn-primary" on:click={() => (confirm = { kind: "save" })}>Fix names in this save</button></div>
          {#if renames}
            {#if renames.length}
              <table class="grid-table">
                <thead><tr><th>Before</th><th>After</th></tr></thead>
                <tbody>{#each renames as r}<tr><td class="muted strike">{r.from}</td><td>{r.to}</td></tr>{/each}</tbody>
              </table>
            {:else}
              <p class="muted pad">Every generated Eritrean rider already has a genuine name.</p>
            {/if}
          {/if}
        </section>
      </div>
    {/if}
  </div>
</div>

{#if confirm}
  <Modal title={confirm.kind === "game" ? "Update the game's name list?" : "Fix rider names in this save?"} on:close={() => !busy && (confirm = null)}>
    <ul class="notes">
      <li><Icon name="alert" size={15} /><span>Quit PCM completely first{confirm.kind === "save" ? ", or at least go back to the main menu so this career isn't loaded" : ""}. Also close this file in PCM DBEdit if it's open there.</span></li>
      <li><Icon name="backup" size={15} /><span>A backup is taken before anything is written.</span></li>
    </ul>
    <div class="row">
      <button class="btn" on:click={() => (confirm = null)} disabled={busy}>Cancel</button>
      <button class="btn btn-primary" on:click={run} disabled={busy}>{busy ? "Working…" : confirm.kind === "game" ? "Update name list" : "Rename riders"}</button>
    </div>
  </Modal>
{/if}

<style>
  .page-body { display: flex; flex-direction: column; gap: 16px; }
  .pack { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; padding: 0 14px 16px; }
  h4 { font-family: var(--font-cond); font-size: 16px; font-weight: 600; }
  .pack p { font-size: 12px; margin: 2px 0 8px; }
  .chips { display: flex; flex-wrap: wrap; gap: 4px; }
  .chips span { font-size: 12px; padding: 2px 8px; border-radius: 999px; background: var(--bg-2); border: 1px solid var(--rule); }
  .chips.wrong span { border-color: #e8524a55; color: var(--pois); }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; align-items: start; }
  .db { margin: 0 14px 14px; padding: 12px; background: var(--bg-2); border-radius: 8px; display: flex; flex-direction: column; gap: 8px; align-items: flex-start; }
  .db-head { display: flex; gap: 10px; align-items: center; }
  .ok { color: var(--vert); font-size: 12px; display: inline-flex; gap: 4px; align-items: center; }
  .bad { color: var(--pois); font-size: 12px; }
  code { font-size: 11px; color: var(--ink-3); word-break: break-all; }
  .pad { padding: 0 14px 14px; font-size: 13px; }
  .small { font-size: 12px; }
  .done { margin: 0 14px 12px; color: var(--vert); font-size: 13px; }
  .strike { text-decoration: line-through; }
  .err { color: var(--pois); }
  .notes { list-style: none; display: flex; flex-direction: column; gap: 9px; font-size: 13px; color: var(--ink-2); }
  .notes li { display: flex; gap: 9px; }
  .notes :global(svg) { flex-shrink: 0; margin-top: 2px; color: var(--ink-3); }
  .row { display: flex; justify-content: flex-end; gap: 8px; margin-top: 18px; }
  @media (max-width: 1150px) { .two, .pack { grid-template-columns: 1fr; } }
</style>
