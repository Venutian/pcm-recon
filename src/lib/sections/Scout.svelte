<script lang="ts">
  import { onDestroy } from "svelte";
  import { allCyclists, scoutFilters, scoutResults, defaultScoutFilters, scoutPreset, meta, type ScoutFilters } from "../stores";
  import { RIDER_TYPES, STAT_KEYS, STAT_LABELS, type Col } from "../types";
  import { exportCsv } from "../api";
  import RiderTable from "../components/RiderTable.svelte";
  import Icon from "../components/Icon.svelte";

  // Apply a hand-off from another section (e.g. "find a sprinter" in My team).
  const unsub = scoutPreset.subscribe((p) => {
    if (p) { scoutFilters.set({ ...defaultScoutFilters(), ...p }); scoutPreset.set(null); }
  });
  onDestroy(unsub);

  $: f = $scoutFilters;
  $: season = $meta?.season ?? 0;
  $: nations = [...new Set($allCyclists.map((c) => c.nationality))].filter((n) => n !== "Unknown").sort();
  $: divisions = [...new Set($allCyclists.map((c) => c.division))].filter(Boolean).sort();

  const PRESETS: { label: string; hint: string; f: Partial<ScoutFilters> }[] = [
    { label: "Wonderkids", hint: "21 or younger, 5★ potential or more", f: { maxAge: 21, minPot: 5, sort: "potential" } },
    { label: "Expiring stars", hint: "CA 72+, contract ends this season", f: { minCA: 72, status: "expiring", hideMine: true } },
    { label: "Free agents worth signing", hint: "Unsigned, CA 65+", f: { minCA: 65, status: "free", minAge: 18 } },
    { label: "Climbers", hint: "Mountain 75+, on the market", f: { statKey: "mountain", statMin: 75, status: "market", sort: "mountain" } },
    { label: "Sprinters", hint: "Sprint 75+, on the market", f: { statKey: "sprint", statMin: 75, status: "market", sort: "sprint" } },
    { label: "Time trial", hint: "TT 75+, on the market", f: { statKey: "timetrial", statMin: 75, status: "market", sort: "timetrial" } },
    { label: "Cobbles", hint: "Cobbles 75+, on the market", f: { statKey: "cobble", statMin: 75, status: "market", sort: "cobble" } },
    { label: "Bargain veterans", hint: "29+, CA 70+, wage under €8k", f: { minAge: 29, minCA: 70, maxWage: 8000, status: "signed", hideMine: true } },
  ];
  function preset(p: Partial<ScoutFilters>) { scoutFilters.set({ ...defaultScoutFilters(), ...p }); }
  function toggleType(id: number) {
    scoutFilters.update((x) => ({ ...x, types: x.types.includes(id) ? x.types.filter((t) => t !== id) : [...x.types, id] }));
  }

  $: statCol = f.statKey ? [{ key: f.statKey, label: STAT_LABELS[f.statKey], width: 96, kind: "stat" as const, align: "center" as const }] : [];
  $: cols = [
    { key: "name", label: "Rider", width: 210, kind: "rider" },
    { key: "team", label: "Team", width: 170, kind: "team" },
    { key: "age", label: "Age", width: 50, align: "center" },
    { key: "current_ability", label: "CA", width: 62, kind: "ca", align: "center" },
    { key: "potential", label: "Potential", width: 100, kind: "stars" },
    { key: "growth", label: "Growth", width: 66, kind: "upside", align: "center" },
    { key: "rider_type", label: "Type", width: 122, kind: "type" },
    ...statCol,
    { key: "wage", label: "Wage / mo", width: 86, kind: "money", align: "right" },
    { key: "contract_end", label: "Contract", width: 74, kind: "contract", align: "center" },
    { key: "mountain", label: "MO", width: 54, kind: "stat", align: "center" },
    { key: "hill", label: "HIL", width: 54, kind: "stat", align: "center" },
    { key: "timetrial", label: "TT", width: 54, kind: "stat", align: "center" },
    { key: "sprint", label: "SP", width: 54, kind: "stat", align: "center" },
    { key: "cobble", label: "COB", width: 54, kind: "stat", align: "center" },
    { key: "endurance", label: "STA", width: 54, kind: "stat", align: "center" },
    { key: "scout_grade", label: "Grade", width: 120, kind: "grade" },
    { key: "nationality", label: "Nation", width: 130, kind: "nation" },
  ] as Col[];

  function exportRows() {
    exportCsv($scoutResults, ["name", "team", "nationality", "age", "rider_type", "current_ability", "potential", "growth", "wage", "contract_end", ...STAT_KEYS], "pcm_scout.csv");
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Scout</h1>
      <p>Search all {$allCyclists.length.toLocaleString()} riders with their real attributes, ceilings, wages and contracts.</p>
    </div>
    <div class="head-actions">
      <button class="btn btn-sm btn-quiet" on:click={() => scoutFilters.set(defaultScoutFilters())}>Clear filters</button>
      <button class="btn btn-sm btn-quiet" on:click={exportRows}><Icon name="export" size={14} />CSV</button>
    </div>
  </div>

  <div class="presets">
    {#each PRESETS as p}
      <button class="preset" title={p.hint} on:click={() => preset(p.f)}>{p.label}</button>
    {/each}
  </div>

  <div class="filters">
    <input class="q" bind:value={$scoutFilters.q} placeholder="Name, team or nation…" />
    <div class="chips">
      {#each RIDER_TYPES as t}
        <button class="chip" class:on={f.types.includes(t.id)} on:click={() => toggleType(t.id)}><span class="dot" style="background:{t.color}"></span>{t.label}</button>
      {/each}
    </div>
    <div class="row">
      <label class="field"><span>Status</span>
        <select bind:value={$scoutFilters.status}>
          <option value="all">Everyone</option>
          <option value="market">On the transfer market</option>
          <option value="free">Free agents</option>
          <option value="expiring">Contract ends {season}</option>
          <option value="signed">Under contract</option>
        </select>
      </label>
      <label class="field"><span>Age</span>
        <span class="pair"><input type="number" bind:value={$scoutFilters.minAge} min="14" max="45" /> – <input type="number" bind:value={$scoutFilters.maxAge} min="14" max="45" /></span>
      </label>
      <label class="field"><span>Min CA</span><input type="number" bind:value={$scoutFilters.minCA} min="0" max="100" /></label>
      <label class="field"><span>Min potential ★</span><input type="number" bind:value={$scoutFilters.minPot} min="0" max="6" step="0.5" /></label>
      <label class="field"><span>Max wage / mo</span><input type="number" bind:value={$scoutFilters.maxWage} min="0" step="1000" placeholder="any" /></label>
      <label class="field"><span>Attribute</span>
        <span class="pair">
          <select bind:value={$scoutFilters.statKey}>
            <option value="">Any</option>
            {#each STAT_KEYS as k}<option value={k}>{STAT_LABELS[k]}</option>{/each}
          </select>
          {#if f.statKey}≥ <input type="number" bind:value={$scoutFilters.statMin} min="0" max="100" />{/if}
        </span>
      </label>
      <label class="field"><span>Nation</span>
        <select bind:value={$scoutFilters.nation}><option value="">All</option>{#each nations as n}<option>{n}</option>{/each}</select>
      </label>
      <label class="field"><span>Division</span>
        <select bind:value={$scoutFilters.division}><option value="">All</option>{#each divisions as d}<option>{d}</option>{/each}</select>
      </label>
      <label class="chk"><input type="checkbox" bind:checked={$scoutFilters.hideMine} /> Hide my riders</label>
      <span class="count">{$scoutResults.length.toLocaleString()} riders</span>
    </div>
  </div>

  <div class="table">
    {#key f.statKey}
      <RiderTable data={$scoutResults} {cols} />
    {/key}
  </div>
</div>

<style>
  .head-actions { display: flex; gap: 6px; }
  .presets { display: flex; gap: 6px; padding: 0 22px 10px; flex-wrap: wrap; }
  .preset {
    background: var(--panel); border: 1px solid var(--rule); color: var(--ink); border-radius: 6px;
    padding: 6px 12px; font: inherit; font-size: 12.5px; cursor: pointer;
  }
  .preset:hover { border-color: var(--jaune); }
  .filters { padding: 0 22px 12px; display: flex; flex-direction: column; gap: 8px; flex-shrink: 0; }
  .q { width: 260px; }
  .chips { display: flex; gap: 4px; flex-wrap: wrap; }
  .dot { width: 7px; height: 7px; border-radius: 50%; }
  .row { display: flex; gap: 12px; align-items: flex-end; flex-wrap: wrap; }
  .row input[type="number"] { width: 72px; }
  .pair { display: flex; align-items: center; gap: 5px; color: var(--ink-3); }
  .pair input { width: 58px !important; }
  .chk { display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: var(--ink-2); padding-bottom: 7px; cursor: pointer; }
  .count { margin-left: auto; font-size: 12px; color: var(--ink-3); padding-bottom: 7px; }
  .table { position: relative; flex: 1; overflow: hidden; border-top: 1px solid var(--rule); }
</style>
