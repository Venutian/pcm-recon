<script lang="ts">
  import { teams, teamRosters, teamColors } from "../stores";
  import RiderTable from "../components/RiderTable.svelte";
  import SummaryCard from "../components/SummaryCard.svelte";
  import type { Col, Team } from "../types";
  import { fmtNat, flagEmoji, resolveTeamColor } from "../format";

  const COLS: Col[] = [
    { key: "name", label: "Name", width: 200, align: "left" },
    { key: "nat_flag", label: "Country", width: 130, align: "left", fmt: (_, r) => fmtNat(r) },
    { key: "age", label: "Age", width: 44, align: "center" },
    { key: "current_ability", label: "CA", width: 50, align: "center" },
    { key: "potential", label: "Stars", width: 52, align: "center" },
    { key: "growth", label: "Upside", width: 58, align: "center" },
    { key: "rider_type", label: "Type", width: 110, align: "left" },
    { key: "scout_grade", label: "Grade", width: 110, align: "left" },
  ];

  let search = "";
  let selected: Team | null = null;

  $: filtered = $teams.filter((t) => !search || t.name.toLowerCase().includes(search.toLowerCase()));
  $: roster = selected ? ($teamRosters[selected.id] ?? []).sort((a, b) => b.current_ability - a.current_ability) : [];
  $: avgCA = roster.length ? (roster.reduce((s, r) => s + r.current_ability, 0) / roster.length).toFixed(1) : "-";
  $: best = roster[0];
  $: {
    const m: Record<string, number> = {};
    roster.forEach((r) => {
      m[r.rider_type] = (m[r.rider_type] ?? 0) + 1;
    });
    topType = Object.entries(m).sort((a, b) => b[1] - a[1])[0]?.[0] ?? "-";
  }
  let topType = "-";

  function sel(t: Team) {
    selected = t;
  }
</script>

<div class="teams section-enter">
  <div class="team-list">
    <div class="list-hdr">TEAMS</div>
    <input bind:value={search} placeholder="Search teams..." style="margin:8px 10px;width:calc(100% - 20px)" />
    <div class="list-scroll">
      {#each filtered as t}
        {@const tc = resolveTeamColor(t.color1, t.id)}
        <button class="team-btn" class:active={selected?.id === t.id} on:click={() => sel(t)}
                style={selected?.id === t.id ? `border-left-color:${tc}` : ""}>
          <span class="t-color-dot" style="background:{tc}"></span>
          {@html flagEmoji(t.country_iso)}
          <span class="t-name">{t.name}</span>
          <span class="t-count">{$teamRosters[t.id]?.length ?? 0}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="detail">
    {#if !selected}
      <div class="no-sel">Select a team from the list</div>
    {:else}
      <div class="team-hdr">
        {@html flagEmoji(selected.country_iso)}
        <span class="t-hname" style="color:{resolveTeamColor(selected.color1, selected.id)}">{selected.name}</span>
        <span class="t-short">({selected.short})</span>
      </div>
      <div class="stat-cards">
        <SummaryCard title="Riders" value={String(roster.length)} sub="Total squad" accent="#e8b800" />
        <SummaryCard title="Avg CA" value={String(avgCA)} sub="Mean current ability" accent="#2ecc82" />
        <SummaryCard title="Best CA" value={best ? String(best.current_ability) : "-"} sub={best?.name ?? "-"} accent="#ffd700" />
        <SummaryCard title="Top Type" value={topType} sub="Most common type" accent="#9470f0" />
        <SummaryCard title="Free" value={String(roster.filter((r) => r.free_agent).length)} sub="Free agents in squad" accent="#e87848" />
      </div>
      <div class="roster-table">
        <RiderTable data={roster} cols={COLS} />
      </div>
    {/if}
  </div>
</div>

<style>
  .teams {
    display: grid;
    grid-template-columns: 260px 1fr;
    height: 100%;
    overflow: hidden;
  }

  .team-list {
    background: #111c30;
    border-right: 1px solid #253550;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .list-hdr {
    padding: 12px 14px 4px;
    font-size: 10px;
    font-weight: 700;
    color: #4a5e80;
    letter-spacing: 0.1em;
    flex-shrink: 0;
  }

  .list-scroll {
    flex: 1;
    overflow-y: auto;
  }

  .t-color-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .team-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 9px 14px;
    background: none;
    border: none;
    border-left: 2px solid transparent;
    color: #7284a8;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s, border-left-color 0.1s;
  }

  .team-btn:hover {
    background: #172035;
    color: #dce8ff;
  }

  .team-btn.active {
    background: #172035;
    color: #dce8ff;
  }

  .t-flag {
    font-size: 16px;
    flex-shrink: 0;
  }

  .t-name {
    flex: 1;
    font-size: 12px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .t-count {
    font-size: 10px;
    color: #4a5e80;
  }

  .detail {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 16px;
    gap: 14px;
  }

  .no-sel {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #4a5e80;
  }

  .team-hdr {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .t-flag2 {
    font-size: 28px;
  }

  .t-hname {
    font-size: 20px;
    font-weight: 700;
    color: #dce8ff;
  }

  .t-short {
    font-size: 13px;
    color: #6478a0;
  }

  .stat-cards {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 10px;
    flex-shrink: 0;
  }

  .roster-table {
    flex: 1;
    overflow: hidden;
    background: #0f1829;
    border: 1px solid #1e2d4a;
    border-radius: 8px;
  }
</style>
