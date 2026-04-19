<script lang="ts">
  import { gameDate, scoutReports } from "../stores";
  import RiderTable from "../components/RiderTable.svelte";
  import type { Col } from "../types";
  import { TYPE_COLORS } from "../types";
  import { fmtNat, fmtStars } from "../format";

  function scoutStars(raw: number): string {
    const score = Math.max(0, Math.min(6, Math.round(raw * 2) / 2));
    return score > 0 ? fmtStars(score) : "-";
  }

  function scoutStatus(birthdate: string, saveDate: string): string {
    const birthYear = Number(String(birthdate).slice(0, 4));
    const saveYear = Number(saveDate.slice(0, 4));
    if (!birthYear || !saveYear) return "";
    return saveYear - birthYear <= 17 ? "Junior" : "Signable";
  }

  const COLS: Col[] = [
    { key: "name", label: "Name", width: 190, align: "left" },
    { key: "nat_flag", sortKey: "nationality", label: "Country", width: 140, align: "left", fmt: (_, r) => fmtNat(r) },
    { key: "continent", label: "Continent", width: 95, align: "left" },
    { key: "scout_status", label: "Status", width: 78, align: "left", fmt: (_, r) => scoutStatus(r.birthdate, $gameDate) },
    { key: "age", label: "Age", width: 44, align: "center" },
    { key: "rider_type", label: "Type", width: 110, align: "left" },
    { key: "scout_tour_potential", label: "Tour", width: 62, align: "center", fmt: (v) => scoutStars(Number(v)) },
    { key: "scout_mountain_potential", label: "Mtn", width: 62, align: "center", fmt: (v) => scoutStars(Number(v)) },
    { key: "scout_timetrial_potential", label: "TT", width: 62, align: "center", fmt: (v) => scoutStars(Number(v)) },
    { key: "scout_sprint_potential", label: "Spr", width: 62, align: "center", fmt: (v) => scoutStars(Number(v)) },
    { key: "scout_ardennes_potential", label: "Ard", width: 62, align: "center", fmt: (v) => scoutStars(Number(v)) },
    { key: "scout_cobble_potential", label: "Cob", width: 62, align: "center", fmt: (v) => scoutStars(Number(v)) },
    { key: "scout_flat_potential", label: "Flat", width: 62, align: "center", fmt: (v) => scoutStars(Number(v)) },
  ];

  const TYPE_OPTIONS = ["All", "Climber", "Sprinter", "Time Trialist", "All-Rounder", "Classics", "Ardennes", "Rouleur"];

  let search = "";
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
