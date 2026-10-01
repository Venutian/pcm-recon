<script lang="ts">
  import { allCyclists, shortlist, notes, compareIds, goTo } from "../stores";
  import { exportCsv } from "../api";
  import type { Col } from "../types";
  import RiderTable from "../components/RiderTable.svelte";
  import Modal from "../components/Modal.svelte";
  import Icon from "../components/Icon.svelte";

  $: rows = $allCyclists.filter((c) => $shortlist.has(c.id));
  let confirmClear = false;

  const cols: Col[] = [
    { key: "name", label: "Rider", width: 210, kind: "rider" },
    { key: "team", label: "Team", width: 170, kind: "team" },
    { key: "age", label: "Age", width: 50, align: "center" },
    { key: "current_ability", label: "CA", width: 62, kind: "ca", align: "center" },
    { key: "potential", label: "Potential", width: 100, kind: "stars" },
    { key: "rider_type", label: "Type", width: 122, kind: "type" },
    { key: "wage", label: "Wage / mo", width: 86, kind: "money", align: "right" },
    { key: "contract_end", label: "Contract", width: 74, kind: "contract", align: "center" },
    { key: "note", label: "Your note", width: 320, value: (c) => $notes[String(c.id)] ?? "" },
  ];

  function compareAll() {
    compareIds.set(rows.slice().sort((a, b) => b.current_ability - a.current_ability).slice(0, 4).map((c) => c.id));
    goTo("Compare");
  }
  function exportRows() {
    exportCsv(rows.map((c) => ({ ...c, note: $notes[String(c.id)] ?? "" })),
      ["name", "team", "nationality", "age", "rider_type", "current_ability", "potential", "wage", "contract_end", "note"], "pcm_shortlist.csv");
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Shortlist</h1>
      <p>Riders you've starred. The list and your notes are kept between sessions, separately from the save file.</p>
    </div>
    <div class="actions">
      {#if rows.length > 1}<button class="btn btn-sm" on:click={compareAll}><Icon name="compare" size={14} />Compare top {Math.min(4, rows.length)}</button>{/if}
      <button class="btn btn-sm btn-quiet" on:click={exportRows} disabled={!rows.length}><Icon name="export" size={14} />CSV</button>
      {#if rows.length}<button class="btn btn-sm btn-danger" on:click={() => (confirmClear = true)}><Icon name="trash" size={14} />Clear</button>{/if}
    </div>
  </div>
  <div class="table">
    <RiderTable data={rows} {cols} sortKey="current_ability" empty="Your shortlist is empty. Open any rider and press Shortlist to add them here." />
  </div>
</div>

{#if confirmClear}
  <Modal title="Clear the shortlist?" on:close={() => (confirmClear = false)}>
    <p class="muted">All {rows.length} riders are removed from the shortlist. Your notes are kept.</p>
    <div class="row">
      <button class="btn" on:click={() => (confirmClear = false)}>Cancel</button>
      <button class="btn btn-primary" on:click={() => { shortlist.set(new Set()); confirmClear = false; }}>Clear shortlist</button>
    </div>
  </Modal>
{/if}

<style>
  .actions { display: flex; gap: 6px; }
  .table { position: relative; flex: 1; overflow: hidden; border-top: 1px solid var(--rule); }
  .row { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
</style>
