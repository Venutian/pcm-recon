<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { toast } from "../stores";
  import { setTeamKit, isDesktop, type KitSummary } from "../api";
  import type { Team } from "../types";
  import Modal from "./Modal.svelte";
  import KitThumb from "./KitThumb.svelte";
  import Icon from "./Icon.svelte";

  export let team: Team;
  export let kits: KitSummary[];

  const dispatch = createEventDispatcher<{ close: void }>();
  let q = "";
  let pick = team.jersey.toLowerCase();
  let color1 = (team.color1 || "#888888").toLowerCase();
  let color2 = (team.color2 || "#444444").toLowerCase();
  let busy = false;
  let error = "";

  $: shown = kits.filter((k) => !q || k.abbr.includes(q.toLowerCase().trim())).slice(0, 400);
  $: changed = pick !== team.jersey.toLowerCase() || color1 !== team.color1.toLowerCase() || color2 !== team.color2.toLowerCase();
  const thumbPart = (k: KitSummary) => (k.parts.includes("minimaillot") ? "minimaillot" : "maillot");

  async function write() {
    busy = true;
    error = "";
    try {
      await setTeamKit(team.id, pick, color1, color2);
      toast(`${team.name} now wears ${pick}`);
      dispatch("close");
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<Modal title="Change {team.name}'s kit" width={920} on:close={() => !busy && dispatch("close")}>
  <div class="top">
    <input type="search" placeholder="Search {kits.length.toLocaleString()} kits…" bind:value={q} data-autofocus />
    <label class="color"><input type="color" bind:value={color1} /><span>Main colour</span></label>
    <label class="color"><input type="color" bind:value={color2} /><span>Second colour</span></label>
  </div>
  <p class="muted small">Team colours are used in menus and on the calendar. The kit is what riders wear.</p>

  <div class="grid">
    {#each shown as k (k.abbr)}
      <button class="kit" class:on={k.abbr === pick} on:click={() => (pick = k.abbr)} title={k.abbr}>
        <div class="img"><KitThumb kit={k.abbr} part={thumbPart(k)} alt={k.abbr} /></div>
        <span>{k.abbr}</span>
        {#if k.abbr === team.jersey.toLowerCase()}<small>current</small>{/if}
      </button>
    {/each}
  </div>
  {#if shown.length === 400}<p class="muted small">Showing the first 400. Search to narrow it down.</p>{/if}

  <ul class="notes">
    <li><Icon name="alert" size={15} /><span>In PCM, go back to the main menu first, then load the career again after saving here.</span></li>
    <li><Icon name="backup" size={15} /><span>The save is backed up first. Edits you made to the old kit stay saved and still apply.</span></li>
  </ul>
  {#if error}<p class="err">{error}</p>{/if}
  <div class="row">
    <button class="btn" on:click={() => dispatch("close")} disabled={busy}>Cancel</button>
    <button class="btn btn-primary" on:click={write} disabled={busy || !changed || !isDesktop}>{busy ? "Saving…" : "Write to save"}</button>
  </div>
</Modal>

<style>
  .top { display: flex; gap: 12px; align-items: center; }
  .top input[type="search"] { flex: 1; }
  .color { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--ink-3); }
  .color input { width: 34px; height: 28px; padding: 0; border: 1px solid var(--rule-2); border-radius: 4px; background: none; cursor: pointer; }
  .small { font-size: 12px; margin: 6px 0 10px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(118px, 1fr)); gap: 8px; max-height: 52vh; overflow-y: auto; padding: 2px; }
  .kit { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 8px 6px; border-radius: 8px; border: 1px solid var(--rule); background: var(--bg-2); color: var(--ink-2); font: inherit; font-size: 11.5px; cursor: pointer; }
  .kit:hover { border-color: var(--rule-2); color: var(--ink); }
  .kit.on { border-color: var(--jaune); box-shadow: 0 0 0 1px var(--jaune); color: var(--ink); }
  .img { width: 92px; height: 92px; }
  .kit span { max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .kit small { color: var(--jaune); font-size: 10px; }
  .notes { list-style: none; display: flex; flex-direction: column; gap: 8px; font-size: 13px; color: var(--ink-2); margin-top: 14px; }
  .notes li { display: flex; gap: 9px; }
  .notes :global(svg) { flex-shrink: 0; margin-top: 2px; color: var(--ink-3); }
  .err { color: var(--pois); margin-top: 8px; }
  .row { display: flex; justify-content: flex-end; gap: 8px; margin-top: 14px; }
</style>
