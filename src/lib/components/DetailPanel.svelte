<script lang="ts">
  import { selectedRider, shortlist, notes, compareIds, toggleShortlist, toggleCompare, selectRider, teamById, meta, goTo } from "../stores";
  import { eurShort, stars, gemGap, terrainProfile, teamColor, fmtDate } from "../format";
  import { GRADE_COLOR, GRADE_DESC, STAT_KEYS, STAT_LABELS, TYPE_COLOR, TERRAINS, SCOUT_CATS } from "../types";
  import Bib from "./Bib.svelte";
  import Flag from "./Flag.svelte";
  import Stars from "./Stars.svelte";
  import StatBar from "./StatBar.svelte";
  import Radar from "./Radar.svelte";
  import Icon from "./Icon.svelte";

  $: c = $selectedRider;
  $: team = c ? $teamById.get(c.team_id) : undefined;
  $: season = $meta?.season ?? 0;
  $: listed = c ? $shortlist.has(c.id) : false;
  $: comparing = c ? $compareIds.includes(c.id) : false;
  $: topKeys = new Set(c?.top_skills.map((s) => s.key) ?? []);
  $: gap = c ? gemGap(c) : 0;
  $: profileNow = c ? terrainProfile(c) : [];
  $: profileCeil = c ? terrainProfile(c, true) : [];
  // Scale the radar to the rider so juniors (stats in the 50s) don't collapse to a dot.
  $: radarMin = Math.max(20, Math.floor((Math.min(...profileNow, 80) - 6) / 5) * 5);
  $: radarMax = Math.max(radarMin + 20, Math.ceil((Math.max(...profileCeil, 60) + 2) / 5) * 5);

  let noteText = "";
  let noteFor = -1;
  $: if (c && c.id !== noteFor) { noteFor = c.id; noteText = $notes[String(c.id)] ?? ""; }
  function saveNote() {
    if (!c) return;
    const id = String(c.id);
    if (($notes[id] ?? "") === noteText) return;
    notes.update((n) => {
      const next = { ...n };
      if (noteText.trim()) next[id] = noteText; else delete next[id];
      return next;
    });
  }

  function stat(k: string) { return c ? (c as unknown as Record<string, number>)[k] ?? 0 : 0; }
</script>

<div class="panel-root">
  {#if !c}
    <div class="empty">
      <Icon name="bike" size={40} stroke={1.25} />
      <p>Pick a rider from any list to see the full profile, scout report and contract.</p>
      <p class="hint">Tip: press <kbd>Ctrl</kbd> <kbd>K</kbd> to search any rider by name.</p>
    </div>
  {:else}
    <header>
      <Bib value={c.current_ability} size="l" />
      <div class="who">
        <h2>{c.name}</h2>
        <div class="line"><Flag code={c.flag} /> {c.nationality}{#if c.age} · {c.age} yrs{/if}</div>
        <div class="line"><span class="dot" style="background:{TYPE_COLOR[c.rider_type]}"></span>{c.rider_type}</div>
      </div>
      <button class="x" on:click={() => selectRider(null)} aria-label="Close profile"><Icon name="close" size={16} /></button>
    </header>

    <div class="team">
      {#if c.free_agent}
        <span class="free">Free agent</span>
      {:else}
        <span class="swatch" style="background:{teamColor(team?.color1, c.team_id)}"></span>
        <span>{c.team}</span>{#if c.division}<span class="muted">· {c.division}</span>{/if}
      {/if}
    </div>

    <div class="actions">
      <button class="btn btn-sm" class:on={listed} on:click={() => toggleShortlist(c.id)}>
        <Icon name="shortlist" size={14} />{listed ? "Shortlisted" : "Shortlist"}
      </button>
      <button class="btn btn-sm" class:on={comparing} on:click={() => toggleCompare(c.id)}>
        <Icon name="compare" size={14} />{comparing ? "In compare" : "Compare"}
      </button>
      {#if $compareIds.length > 1}
        <button class="btn btn-sm btn-quiet" on:click={() => goTo("Compare")}>Open compare ({$compareIds.length})</button>
      {/if}
    </div>

    <div class="scroll">
      <dl class="facts">
        <div><dt>Potential</dt><dd><Stars value={c.potential} size={13} /> <span class="sm">{stars(c.potential)}</span></dd></div>
        <div><dt>Room to grow</dt><dd class:pos={c.growth > 0}>{c.growth > 0 ? `+${c.growth.toFixed(1)}` : "Maxed"}</dd></div>
        <div><dt>Wage</dt><dd>{c.wage ? `${eurShort(c.wage)} / month` : "–"}</dd></div>
        <div><dt>Contract</dt><dd class:neg={!c.free_agent && c.contract_end > 0 && c.contract_end <= season}>
          {#if c.free_agent}Unsigned{:else if c.contract_end}Until {c.contract_end}{:else}–{/if}
        </dd></div>
        <div><dt>Popularity</dt><dd>{c.popularity.toFixed(0)}</dd></div>
        <div><dt>Career wins</dt><dd>{c.wins}</dd></div>
      </dl>

      <div class="flags">
        {#if c.signable && c.free_agent}<span class="tag market">Signable now</span>{/if}
        {#if !c.signable}<span class="tag" title="PCM allows signing from the season a rider turns 18">Can't sign until {c.signable_from}</span>{/if}
        {#if c.on_market}<span class="tag market">On the transfer market</span>{/if}
        {#if c.injured}<span class="tag inj">Injured</span>{/if}
        {#if c.will_retire}<span class="tag">Retiring after this season</span>{/if}
        {#if c.is_mine}<span class="tag mine">Your rider</span>{/if}
      </div>

      {#if c.scout_grade}
        <div class="grade" style="--g:{GRADE_COLOR[c.scout_grade]}">
          <strong>{c.scout_grade}</strong>
          <span>{GRADE_DESC[c.scout_grade] ?? ""}</span>
        </div>
      {/if}

      {#if c.scout_reports > 0}
        <section class="report">
          <div class="sec-head">
            <h3>{c.my_report ? "Your scout's report" : "Scout report"}</h3>
            <span class="muted">{fmtDate(c.scout_report_date)}</span>
          </div>
          <div class="cats">
            {#each SCOUT_CATS as cat}
              {@const v = Number(c[cat.key]) || 0}
              <span class="cat-l">{cat.label}</span>
              <Stars value={v} size={12} color="var(--azur)" />
              <span class="cat-v">{v ? v.toFixed(1) : "–"}</span>
            {/each}
          </div>
          <div class="truth">
            <div><span class="muted">Scout's best estimate</span><strong>{stars(c.scout_estimate)}★</strong></div>
            <div><span class="muted">True potential</span><strong class="gold">{stars(c.potential)}★</strong></div>
            <div><span class="muted">Difference</span><strong class:pos={gap > 0} class:neg={gap < 0}>{gap > 0 ? "+" : ""}{gap.toFixed(1)}</strong></div>
          </div>
          <p class="verdict">
            {#if gap >= 0.5}Underrated. The report undersells this rider, so rivals relying on similar reports are likely to pass.
            {:else if gap <= -0.5}Overrated. The report promises more than the real ceiling.
            {:else}The report is accurate.{/if}
            {c.scout_reports === 1 ? " Only one scout in the league has filed a report." : ` ${c.scout_reports} scout reports exist across the league.`}
          </p>
        </section>
      {/if}

      <section>
        <div class="sec-head"><h3>Terrain profile</h3><span class="muted"><span class="key now"></span>Now <span class="key ceil"></span>Ceiling</span></div>
        <div class="radar">
          <Radar size={270} min={radarMin} max={radarMax} labels={TERRAINS.map((t) => t.label)}
                 series={[
                   { label: "Ceiling", values: profileCeil, color: "#a9b3c1", dashed: true },
                   { label: "Now", values: profileNow, color: "#f5c518" },
                 ]} />
        </div>
      </section>

      <section>
        <div class="sec-head"><h3>Attributes</h3><span class="muted">current → ceiling</span></div>
        {#each STAT_KEYS as k}
          <StatBar label={STAT_LABELS[k]} cur={stat(k)} pot={stat(k + "_p")} best={topKeys.has(k)} />
        {/each}
      </section>

      <section class="phys">
        <div><span class="muted">Height</span>{c.size ? `${c.size} cm` : "–"}</div>
        <div><span class="muted">Weight</span>{c.weight ? `${c.weight} kg` : "–"}</div>
        <div><span class="muted">Stage races</span><Stars value={c.tour_rating} max={5} size={11} /></div>
        <div><span class="muted">One-day races</span><Stars value={c.classic_rating} max={5} size={11} /></div>
      </section>

      <section>
        <div class="sec-head"><h3>Your note</h3><span class="muted">Saved automatically</span></div>
        <textarea bind:value={noteText} on:blur={saveNote} rows="3" placeholder="Why he's interesting, wage you'd offer, races to watch…" spellcheck="false"></textarea>
      </section>
    </div>
  {/if}
</div>

<style>
  .panel-root { height: 100%; display: flex; flex-direction: column; background: var(--bg-2); overflow: hidden; }
  .empty { margin: auto; padding: 30px; text-align: center; color: var(--ink-3); display: flex; flex-direction: column; align-items: center; gap: 12px; max-width: 260px; }
  .empty .hint { font-size: 12px; color: var(--ink-4); }
  kbd { font-family: var(--font); font-size: 11px; border: 1px solid var(--rule-2); border-bottom-width: 2px; border-radius: 4px; padding: 0 5px; color: var(--ink-2); }

  header { display: flex; gap: 14px; align-items: flex-start; padding: 18px 16px 8px; position: relative; }
  .who { min-width: 0; flex: 1; }
  .who h2 { font-size: 24px; line-height: 1.05; margin-bottom: 5px; padding-right: 22px; }
  .line { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--ink-2); }
  .dot { width: 8px; height: 8px; border-radius: 50%; }
  .x { position: absolute; top: 12px; right: 10px; background: none; border: none; color: var(--ink-3); cursor: pointer; padding: 4px; border-radius: 4px; }
  .x:hover { color: var(--ink); background: var(--panel-2); }

  .team { display: flex; align-items: center; gap: 7px; padding: 0 16px 10px; font-size: 13px; font-weight: 500; }
  .swatch { width: 10px; height: 10px; border-radius: 2px; }
  .free { color: var(--vert); }

  .actions { display: flex; gap: 6px; padding: 0 16px 12px; flex-wrap: wrap; border-bottom: 1px solid var(--rule); }
  .actions .on { border-color: var(--jaune); color: var(--jaune); }

  .scroll { flex: 1; overflow-y: auto; padding: 4px 16px 24px; }
  section { margin-top: 18px; }
  .sec-head { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 8px; }
  .sec-head .muted { font-size: 11px; display: flex; align-items: center; gap: 5px; }

  .facts { display: grid; grid-template-columns: 1fr 1fr; gap: 10px 14px; margin-top: 12px; }
  .facts div { display: flex; flex-direction: column; gap: 1px; }
  dt { font-size: 11px; color: var(--ink-3); }
  dd { font-size: 14px; font-weight: 600; display: flex; align-items: center; gap: 6px; }
  dd .sm { font-size: 12px; color: var(--ink-2); font-weight: 500; }
  .pos { color: var(--vert); }
  .neg { color: var(--pois); }
  .gold { color: var(--jaune); }

  .flags { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 12px; }
  .tag { font-size: 11px; padding: 2px 8px; border-radius: 999px; border: 1px solid var(--rule-2); color: var(--ink-2); }
  .tag.market { border-color: #3dbe6e66; color: var(--vert); }
  .tag.inj { border-color: #e8524a66; color: var(--pois); }
  .tag.mine { border-color: #f5c51866; color: var(--jaune); }

  .grade { margin-top: 14px; padding: 9px 12px; border-left: 3px solid var(--g); background: var(--panel); border-radius: 0 6px 6px 0; display: flex; flex-direction: column; gap: 2px; }
  .grade strong { color: var(--g); font-family: var(--font-cond); font-size: 16px; font-weight: 600; }
  .grade span { font-size: 12px; color: var(--ink-2); }

  .report { background: var(--panel); border: 1px solid var(--rule); border-radius: 8px; padding: 12px; }
  .cats { display: grid; grid-template-columns: auto auto 1fr; align-items: center; gap: 4px 10px; }
  .cat-l { font-size: 12px; color: var(--ink-2); }
  .cat-v { font-size: 11px; color: var(--ink-3); }
  .truth { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; margin-top: 12px; padding-top: 10px; border-top: 1px solid var(--rule); }
  .truth div { display: flex; flex-direction: column; font-size: 11px; }
  .truth strong { font-family: var(--font-cond); font-size: 22px; font-weight: 700; }
  .verdict { font-size: 12px; color: var(--ink-2); margin-top: 8px; }

  .radar { display: flex; justify-content: center; }
  .key { display: inline-block; width: 12px; height: 2px; margin-left: 6px; }
  .key.now { background: var(--jaune); }
  .key.ceil { border-top: 2px dashed var(--ink-2); }

  .phys { display: grid; grid-template-columns: 1fr 1fr; gap: 8px 14px; font-size: 13px; }
  .phys div { display: flex; flex-direction: column; gap: 2px; }
  .phys .muted { font-size: 11px; }
  textarea { width: 100%; resize: vertical; }
</style>
