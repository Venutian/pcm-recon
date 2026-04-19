<script lang="ts">
  import { scoutReports } from "../stores";
  import RiderTable from "../components/RiderTable.svelte";
  import type { Col } from "../types";
  import { GRADES, TYPE_COLORS } from "../types";
  import { fmtNat } from "../format";

  const COLS: Col[] = [
    { key: "name", label: "Name", width: 190, align: "left" },
    { key: "nat_flag", sortKey: "nationality", label: "Country", width: 140, align: "left", fmt: (_, r) => fmtNat(r) },
    { key: "continent", label: "Continent", width: 95, align: "left" },
    { key: "age", label: "Age", width: 44, align: "center" },
    { key: "rider_type", label: "Type", width: 110, align: "left" },
    { key: "potential", label: "Stars", width: 56, align: "center" },
    { key: "scout_grade", label: "Grade", width: 110, align: "left" },
    { key: "flat_p", label: "Flat Pot", width: 64, align: "center" },
    { key: "mountain_p", label: "Mtn Pot", width: 68, align: "center" },
    { key: "med_mtn_p", label: "Med Pot", width: 68, align: "center" },
    { key: "hill_p", label: "Hill Pot", width: 64, align: "center" },
    { key: "timetrial_p", label: "TT Pot", width: 62, align: "center" },
    { key: "prologue_p", label: "Pro Pot", width: 66, align: "center" },
    { key: "sprint_p", label: "Spr Pot", width: 66, align: "center" },
    { key: "acceleration_p", label: "Acc Pot", width: 68, align: "center" },
    { key: "cobble_p", label: "Cob Pot", width: 66, align: "center" },
    { key: "downhill_p", label: "DH Pot", width: 62, align: "center" },
    { key: "endurance_p", label: "End Pot", width: 66, align: "center" },
    { key: "resistance_p", label: "Res Pot", width: 66, align: "center" },
    { key: "recuperation_p", label: "Rec Pot", width: 66, align: "center" },
    { key: "baroudeur_p", label: "Bar Pot", width: 66, align: "center" },
  ];

  const TYPE_OPTIONS = ["All", "Climber", "Sprinter", "Time Trialist", "All-Rounder", "Classics", "Ardennes", "Rouleur"];

  let search = "";
  let grade = "All Grades";
  let country = "All Countries";
  let continent = "All Continents";
  let riderType = "All";
  let minStars = 0;
  let minAge = 15;
  let maxAge = 23;

  $: countryOptions = [
    "All Countries",
    ...new Set(
      $scoutReports
        .map((r) => r.nationality)
        .filter((v) => v && v !== "Unknown"),
    ),
  ].sort();

  $: continentOptions = [
    "All Continents",
    ...new Set($scoutReports.map((r) => r.continent).filter(Boolean)),
  ].sort();

  $: sourceRows = $scoutReports.filter((c) => {
    if (!c.age || c.age < minAge || c.age > maxAge) return false;
    if (c.potential < minStars) return false;
    if (grade !== "All Grades" && c.scout_grade !== grade) return false;
    if (country !== "All Countries" && c.nationality !== country) return false;
    if (continent !== "All Continents" && c.continent !== continent) return false;
    if (riderType !== "All" && c.rider_type !== riderType) return false;

    const q = search.trim().toLowerCase();
    if (q && !["name", "nationality", "continent", "rider_type"].some((k) => String((c as Record<string, unknown>)[k] ?? "").toLowerCase().includes(q))) {
      return false;
    }

    return true;
  });
</script>

<div class="reports section-enter">
  <div class="toolbar toolbar-main">
    <div class="fg fg-search">
      <label class="flbl" for="sr-search">Search</label>
      <input id="sr-search" bind:value={search} placeholder="Name, country, continent, type..." />
    </div>
    <div class="fg">
      <label class="flbl" for="sr-country">Country</label>
      <select id="sr-country" bind:value={country}>
        {#each countryOptions as option}<option>{option}</option>{/each}
      </select>
    </div>
    <div class="fg">
      <label class="flbl" for="sr-continent">Continent</label>
      <select id="sr-continent" bind:value={continent}>
        {#each continentOptions as option}<option>{option}</option>{/each}
      </select>
    </div>
    <div class="fg">
      <label class="flbl" for="sr-grade">Grade</label>
      <select id="sr-grade" bind:value={grade}>
        <option>All Grades</option>
        {#each GRADES as g}<option>{g}</option>{/each}
      </select>
    </div>
    <div class="fg">
      <label class="flbl" for="sr-stars">Min Stars</label>
      <input id="sr-stars" type="number" bind:value={minStars} min="0" max="8" step="0.5" />
    </div>
    <div class="fg fg-age">
      <label class="flbl" for="sr-min-age">Age</label>
      <div class="age-range">
        <input id="sr-min-age" type="number" bind:value={minAge} min="0" max="40" />
        <span class="age-sep">-</span>
        <input type="number" bind:value={maxAge} min="0" max="40" />
      </div>
    </div>
    <span class="count">{sourceRows.length.toLocaleString()} / {$scoutReports.length.toLocaleString()} scout-report riders</span>
  </div>

  <div class="type-bar">
    <span class="flbl">Type</span>
    {#each TYPE_OPTIONS as option}
      {@const col = TYPE_COLORS[option] ?? "#263756"}
      <button
        class="btn btn-type"
        class:active={riderType === option}
        style:background={riderType === option ? col : ""}
        on:click={() => (riderType = option)}
      >
        {option}
      </button>
    {/each}
  </div>

  <div class="intro">
    <span>This view is driven only by the scout-report dataset and is trimmed to prospect-age riders by default. Team columns are intentionally removed so the focus stays on scout potential.</span>
  </div>

  <div class="table-wrap">
    <RiderTable data={sourceRows} cols={COLS} />
  </div>
</div>

<style>
  .reports {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    gap: 10px;
    padding: 10px 16px;
    align-items: flex-end;
    flex-wrap: wrap;
    background: #111c30;
    border-bottom: 1px solid #253550;
    flex-shrink: 0;
  }

  .fg {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .fg-search {
    min-width: 240px;
    flex: 1 1 260px;
  }

  .fg-age input {
    width: 58px;
  }

  .age-range {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .age-sep {
    color: #4a5e80;
    font-size: 11px;
  }

  .type-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    background: #0d1525;
    border-bottom: 1px solid #1c2d48;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .intro {
    padding: 8px 16px;
    font-size: 11px;
    color: #7284a8;
    background: #0d1525;
    border-bottom: 1px solid #1c2d48;
    flex-shrink: 0;
  }

  .flbl {
    font-size: 9px;
    font-weight: 700;
    color: #4a5e80;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .count {
    margin-left: auto;
    font-size: 11px;
    color: #4a5e80;
  }

  .table-wrap {
    flex: 1;
    overflow: hidden;
  }
</style>
