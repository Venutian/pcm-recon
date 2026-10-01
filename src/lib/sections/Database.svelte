<script lang="ts">
  import { onMount } from "svelte";
  import { dbTables, dbPage, isDesktop } from "../api";
  import type { TableSummary } from "../types";

  const PAGE = 200;
  let tables: TableSummary[] = [];
  let filter = "";
  let current: TableSummary | null = null;
  let rows: (number | string | null)[][] = [];
  let offset = 0;
  let error = "";
  let loading = false;

  onMount(async () => {
    if (!isDesktop) return;
    try {
      tables = (await dbTables()).sort((a, b) => a.name.localeCompare(b.name));
      open(tables.find((t) => t.name === "DYN_team") ?? tables[0]);
    } catch (e) { error = String(e); }
  });

  async function open(t: TableSummary | undefined, at = 0) {
    if (!t) return;
    current = t; offset = at; loading = true;
    try { rows = await dbPage(t.name, at, PAGE); error = ""; }
    catch (e) { error = String(e); rows = []; }
    finally { loading = false; }
  }
  $: shown = tables.filter((t) => !filter || t.name.toLowerCase().includes(filter.toLowerCase()) || t.columns.some((c) => c.name.toLowerCase().includes(filter.toLowerCase())));
  function fmt(v: number | string | null) {
    if (v === null || v === undefined) return "";
    if (typeof v === "number" && !Number.isInteger(v)) return v.toFixed(3).replace(/0+$/, "").replace(/\.$/, "");
    return String(v);
  }
</script>

<div class="page">
  <div class="page-head">
    <div><h1>Database</h1><p>Every table in the save, read-only. Handy for checking a value before or after you change it in PCM or PCM DBEdit.</p></div>
  </div>
  {#if !isDesktop}
    <div class="page-body"><p class="muted">The database browser needs the desktop app.</p></div>
  {:else}
    <div class="split">
      <aside>
        <input bind:value={filter} placeholder="Find a table or column…" />
        <ul>
          {#each shown as t}
            <li><button class:on={current?.name === t.name} on:click={() => open(t)}><span>{t.name}</span><small>{t.rows.toLocaleString()}</small></button></li>
          {/each}
        </ul>
      </aside>
      <section>
        {#if error}<p class="err">{error}</p>{/if}
        {#if current}
          <div class="bar">
            <strong>{current.name}</strong>
            <span class="muted">{current.rows.toLocaleString()} rows · {current.columns.length} columns</span>
            <div class="pager">
              <button class="btn btn-sm" disabled={offset === 0 || loading} on:click={() => open(current ?? undefined, Math.max(0, offset - PAGE))}>Previous</button>
              <span class="muted">{current.rows ? offset + 1 : 0}–{Math.min(offset + PAGE, current.rows)}</span>
              <button class="btn btn-sm" disabled={offset + PAGE >= current.rows || loading} on:click={() => open(current ?? undefined, offset + PAGE)}>Next</button>
            </div>
          </div>
          <div class="grid">
            <table class="grid-table">
              <thead><tr>{#each current.columns as c}<th title="{c.ty} · column {c.index}">{c.name}</th>{/each}</tr></thead>
              <tbody>
                {#each rows as r}
                  <tr>{#each r as v}<td class:n={typeof v === "number"}>{fmt(v)}</td>{/each}</tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .split { flex: 1; display: grid; grid-template-columns: 280px minmax(0, 1fr); overflow: hidden; border-top: 1px solid var(--rule); }
  aside { border-right: 1px solid var(--rule); display: flex; flex-direction: column; overflow: hidden; padding: 10px 6px 0; gap: 8px; }
  aside input { margin: 0 4px; }
  ul { list-style: none; overflow-y: auto; flex: 1; padding-bottom: 10px; }
  li button { width: 100%; display: flex; justify-content: space-between; gap: 8px; padding: 5px 9px; border-radius: 5px; background: none; border: none; color: var(--ink-2); font: inherit; font-size: 12.5px; cursor: pointer; text-align: left; }
  li button:hover { background: var(--panel); color: var(--ink); }
  li button.on { background: var(--panel-2); color: var(--ink); }
  li small { color: var(--ink-4); }
  section { display: flex; flex-direction: column; overflow: hidden; }
  .bar { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-bottom: 1px solid var(--rule); }
  .bar strong { font-family: var(--font-cond); font-size: 18px; }
  .pager { margin-left: auto; display: flex; align-items: center; gap: 8px; font-size: 12px; }
  .grid { flex: 1; overflow: auto; }
  .grid :global(th) { background: var(--bg); }
  td.n { text-align: right; font-variant-numeric: tabular-nums; }
  .err { color: var(--pois); padding: 12px 14px; }
</style>
