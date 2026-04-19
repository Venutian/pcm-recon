<script lang="ts">
  import { selectedRider, shortlistIds, notes, currentPath } from "../stores";
  import { flagEmoji, fmtStars, fmtUpside } from "../format";
  import { GRADE_COLOR, GRADE_BG, GRADE_DESC, STAT_LABELS } from "../types";
  import StatBar from "./StatBar.svelte";
  import { invoke } from "@tauri-apps/api/core";

  $: c = $selectedRider;
  $: isShortlisted = c ? $shortlistIds.has(c.id) : false;
  let noteText = "";
  $: if (c) noteText = $notes[String(c.id)] ?? "";

  function toggleShortlist() {
    if (!c) return;
    shortlistIds.update((s) => {
      const n = new Set(s);
      n.has(c.id) ? n.delete(c.id) : n.add(c.id);
      return n;
    });
  }

  function statVal(rider: typeof c, key: string): number {
    return (rider as unknown as Record<string, number>)?.[key] ?? 0;
  }

  async function saveNote() {
    if (!c) return;
    notes.update((n) => ({ ...n, [String(c.id)]: noteText }));
    if ($currentPath) {
      const dir = $currentPath.replace(/[^/\\]+$/, "");
      await invoke("save_notes", {
        path: dir + "pcm_recon_notes.json",
        notes: $notes,
      }).catch(() => {});
    }
  }

  const STAT_KEYS = [
    "flat",
    "mountain",
    "med_mtn",
    "cobble",
    "timetrial",
    "prologue",
    "hill",
    "baroudeur",
    "sprint",
    "acceleration",
    "endurance",
    "resistance",
    "recuperation",
    "downhill",
  ];
</script>

<div class="panel">
  {#if !c}
    <div class="empty">
      <div class="empty-icon">&#x1F50D;</div>
      <p>Select a rider to view their profile</p>
    </div>
  {:else}
    <div class="name-row">
      <div class="name-block">
        {@html flagEmoji(c.iso)}
        <span class="name">{c.name}</span>
      </div>
      <button
        class="sl-btn"
        class:active={isShortlisted}
        on:click={toggleShortlist}
        title={isShortlisted ? "Remove from shortlist" : "Add to shortlist"}
      >
        {isShortlisted ? "\u2605" : "\u2606"}
      </button>
    </div>

    <div class="meta">
      {c.nationality} | {c.continent} | {c.rider_type}
      {#if c.age} | Age {c.age}{/if}
      {#if c.size && c.weight} | {c.size}cm / {c.weight}kg{/if}
    </div>
    <div class="team-line">{c.team}</div>

    {#if c.scout_grade}
      {@const gc = GRADE_COLOR[c.scout_grade] ?? "#6478a0"}
      {@const gb = GRADE_BG[c.scout_grade] ?? "#0d1525"}
      <div class="grade-banner" style="color:{gc}; background:{gb}; border-color:{gc}20">
        <span class="grade-name">{c.scout_grade}</span>
        <span class="grade-desc">{GRADE_DESC[c.scout_grade] ?? ""}</span>
      </div>
    {/if}

    <div class="metrics">
      <div class="metric">
        <div class="metric-lbl">CA</div>
        <div class="metric-val" style="color:#e8b800">{c.current_ability}</div>
        <div class="metric-sub">avg {c.skill_average}</div>
      </div>
      <div class="metric">
        <div class="metric-lbl">POT</div>
        <div class="metric-val" style="color:#f0c030">{fmtStars(c.potential)}{"\u2605"}</div>
        <div class="metric-sub">ceil {c.skill_ceiling}</div>
      </div>
      <div class="metric">
        <div class="metric-lbl">UPSIDE</div>
        <div class="metric-val" style="color:#2ecc82">{fmtUpside(c.growth)}</div>
        <div class="metric-sub">gap +{c.peak_gap}</div>
      </div>
      <div class="metric">
        <div class="metric-lbl">SPEC</div>
        <div class="metric-val" style="color:#9470f0">{c.specialty_rating}</div>
        <div class="metric-sub">{c.top_skills[0]?.label ?? "-"}</div>
      </div>
    </div>

    <div class="note-section">
      <div class="sec-label">SCOUT NOTE</div>
      <textarea bind:value={noteText} placeholder="Add a scouting note..." rows="3" spellcheck="false"></textarea>
      <button class="btn btn-ghost" style="margin-top:4px" on:click={saveNote}>Save Note</button>
    </div>

    <div class="stats-section">
      <div class="sec-label">ATTRIBUTES <span class="sec-sub">current / potential</span></div>
      <div class="stats-scroll">
        {#each STAT_KEYS as key}
          {@const cur = statVal(c, key)}
          {@const pot = statVal(c, key + "_p")}
          <StatBar label={STAT_LABELS[key] ?? key} {cur} {pot} />
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: #0f1829;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #3a4e72;
    gap: 12px;
  }

  .empty-icon {
    font-size: 40px;
    opacity: 0.4;
  }

  .empty p {
    font-size: 13px;
  }

  .name-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 14px 16px 6px;
    gap: 8px;
  }

  .name-block {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .name {
    font-size: 15px;
    font-weight: 700;
    color: #dce8ff;
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sl-btn {
    background: #111c30;
    border: 1px solid #1e2d4a;
    color: #6478a0;
    border-radius: 6px;
    width: 34px;
    height: 34px;
    font-size: 18px;
    cursor: pointer;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .sl-btn:hover {
    border-color: #f0c030;
    color: #f0c030;
  }

  .sl-btn.active {
    background: #1a1400;
    border-color: #f0c030;
    color: #f0c030;
  }

  .meta {
    padding: 0 16px;
    font-size: 11px;
    color: #6478a0;
    line-height: 1.6;
  }

  .team-line {
    padding: 0 16px 10px;
    font-size: 12px;
    color: #e8b800;
    font-weight: 600;
  }

  .grade-banner {
    margin: 0 12px 10px;
    padding: 8px 12px;
    border-radius: 6px;
    border: 1px solid;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .grade-name {
    font-size: 12px;
    font-weight: 700;
  }

  .grade-desc {
    font-size: 11px;
    opacity: 0.8;
    line-height: 1.4;
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    padding: 0 12px 10px;
  }

  .metric {
    background: #0d1525;
    border: 1px solid #253550;
    border-radius: 6px;
    padding: 8px 6px;
    text-align: center;
  }

  .metric-lbl {
    font-size: 9px;
    font-weight: 700;
    color: #4a5e80;
    letter-spacing: 0.08em;
  }

  .metric-val {
    font-size: 16px;
    font-weight: 700;
    line-height: 1.1;
    margin: 2px 0;
  }

  .metric-sub {
    font-size: 10px;
    color: #7888b0;
  }

  .note-section {
    padding: 0 12px 10px;
  }

  .sec-label {
    font-size: 9px;
    font-weight: 700;
    color: #4a5e80;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-bottom: 6px;
  }

  .sec-sub {
    font-weight: 400;
    color: #344a72;
    text-transform: none;
    letter-spacing: 0;
  }

  textarea {
    width: 100%;
    background: #0d1525;
    border: 1px solid #253550;
    border-radius: 6px;
    color: #dce8ff;
    padding: 7px 10px;
    font-family: inherit;
    font-size: 12px;
    resize: none;
    outline: none;
  }

  textarea:focus {
    border-color: #e8b800;
  }

  .stats-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0 12px 12px;
  }

  .stats-scroll {
    flex: 1;
    overflow-y: auto;
  }
</style>
