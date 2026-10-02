<script lang="ts">
  import { prospects, selectRider, selectedId } from "../stores";
  import { gemGap, stars } from "../format";
  import { RIDER_TYPES, SCOUT_CATS, TYPE_COLOR, type Col, type Cyclist } from "../types";
  import { exportCsv } from "../api";
  import RiderTable from "../components/RiderTable.svelte";
  import Bib from "../components/Bib.svelte";
  import Flag from "../components/Flag.svelte";
  import Stars from "../components/Stars.svelte";
  import Icon from "../components/Icon.svelte";

  let source: "mine" | "all" = "all";
  let q = "";
  let types: number[] = [];
  let ages: string[] = [];
  let continent = "";
  let nation = "";
  let minPot = 0;
  let gemsOnly = false;
  let signableOnly = false;
  let board: "best" | "gems" | "unseen" = "best";

  $: myCount = $prospects.filter((c) => c.my_report).length;
  $: if (myCount === 0 && source === "mine") source = "all";
  $: continents = [...new Set($prospects.map((c) => c.continent))].filter((n) => n && n !== "Unknown").sort();
  // Nations follow the chosen continent; clear a nation that no longer fits.
  $: nations = [...new Set($prospects.filter((c) => !continent || c.continent === continent).map((c) => c.nationality))].filter((n) => n !== "Unknown").sort();
  $: if (nation && !nations.includes(nation)) nation = "";

  $: rows = $prospects.filter((c) => {
    if (source === "mine" && !c.my_report) return false;
    if (q && !c.name.toLowerCase().includes(q.toLowerCase())) return false;
    if (types.length && !types.includes(c.rider_type_id)) return false;
    if (ages.length && !ages.includes(c.age >= 18 ? "18+" : String(c.age))) return false;
    if (continent && c.continent !== continent) return false;
    if (nation && c.nationality !== nation) return false;
    if (c.potential < minPot) return false;
    if (gemsOnly && gemGap(c) < 0.5) return false;
    if (signableOnly && !c.signable) return false;
    return true;
  });

  // Draft board: one bold strip of six cards.
  $: boardRows = (() => {
    const r = [...rows];
    // Underrated by at least half a star, best real talent first.
    if (board === "gems") return r.filter((c) => gemGap(c) >= 0.5).sort((a, b) => b.potential - a.potential || gemGap(b) - gemGap(a)).slice(0, 6);
    if (board === "unseen") return r.filter((c) => c.scout_reports <= 2).sort((a, b) => b.potential - a.potential || b.skill_ceiling - a.skill_ceiling).slice(0, 6);
    return r.sort((a, b) => b.potential - a.potential || b.skill_ceiling - a.skill_ceiling).slice(0, 6);
  })();

  $: gemCount = rows.filter((c) => gemGap(c) >= 0.5).length;

  function toggle<T>(arr: T[], v: T): T[] { return arr.includes(v) ? arr.filter((x) => x !== v) : [...arr, v]; }

  const cols: Col[] = [
    { key: "name", label: "Rider", width: 210, kind: "rider" },
    { key: "age", label: "Age", width: 50, align: "center" },
    { key: "signable", label: "Can sign", width: 92, kind: "sign", title: "Riders can be signed from the season they turn 18", value: (c) => (c.signable ? 1 : 0) },
    { key: "rider_type", label: "Type", width: 122, kind: "type" },
    { key: "current_ability", label: "Now", width: 64, kind: "ca", align: "center" },
    { key: "potential", label: "True potential", width: 112, kind: "stars", title: "Real potential from the save" },
    { key: "scout_estimate", label: "Scout says", width: 104, kind: "scout", title: "Best category in the scout report" },
    { key: "gem", label: "Gap", width: 58, kind: "gem", align: "center", title: "True potential minus scout estimate. Positive = underrated" },
    ...SCOUT_CATS.map((s) => ({ key: s.key as string, label: s.short, width: 76, kind: "scout" as const, title: `Scout: ${s.label}` })),
    { key: "skill_ceiling", label: "Ceiling", width: 66, align: "center", title: "Average of all attribute ceilings" },
    { key: "scout_reports", label: "Reports", width: 70, align: "center", title: "Scout reports filed by all teams. Fewer = less competition" },
    { key: "nationality", label: "Nation", width: 130, kind: "nation" },
  ];

  function exportRows() {
    exportCsv(rows.map((c) => ({ ...c, gap: gemGap(c) })), ["name", "nationality", "age", "rider_type", "current_ability", "potential", "scout_estimate", "gap", ...SCOUT_CATS.map((s) => s.key as string), "skill_ceiling", "scout_reports"], "pcm_prospects.csv");
  }
  function bestCat(c: Cyclist) {
    let best = SCOUT_CATS[0], v = 0;
    for (const s of SCOUT_CATS) { const x = Number(c[s.key]) || 0; if (x > v) { v = x; best = s; } }
    return v ? best.label : "";
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Prospects</h1>
      <p>
        Young riders with scout reports, matched to the real rider record in the save. Blue stars are what the scout wrote; gold stars are the rider's true potential.
        {#if gemCount}<strong class="gem-n">{gemCount} underrated</strong> in this view.{/if}
      </p>
    </div>
    <div class="head-actions">
      <div class="seg" role="tablist" aria-label="Report source">
        <button class:on={source === "all"} on:click={() => (source = "all")}>All league reports <span>{$prospects.length}</span></button>
        <button class:on={source === "mine"} on:click={() => (source = "mine")} disabled={!myCount}>Your scouts <span>{myCount}</span></button>
      </div>
      <button class="btn btn-sm btn-quiet" on:click={exportRows}><Icon name="export" size={14} />CSV</button>
    </div>
  </div>

  {#if $prospects.length === 0}
    <div class="page-body"><p class="muted">No scout reports in this save yet. Send scouts out in PCM and reload.</p></div>
  {:else}
    <div class="board">
      <div class="board-tabs">
        <button class:on={board === "best"} on:click={() => (board = "best")}>Highest potential</button>
        <button class:on={board === "gems"} on:click={() => (board = "gems")}><Icon name="gem" size={13} />Hidden gems</button>
        <button class:on={board === "unseen"} on:click={() => (board = "unseen")}><Icon name="eye" size={13} />Barely scouted</button>
      </div>
      <div class="cards">
        {#each boardRows as c, i (c.id)}
          {@const g = gemGap(c)}
          <button class="card" class:sel={$selectedId === c.id} on:click={() => selectRider(c.id)}>
            <span class="rank">{i + 1}</span>
            <div class="card-top">
              <Bib value={c.current_ability} size="m" />
              <div class="card-who">
                <strong>{c.name}</strong>
                <span><Flag code={c.flag} size={12} /> {c.age} · <span style="color:{TYPE_COLOR[c.rider_type]}">{c.rider_type}</span></span>
              </div>
            </div>
            <div class="card-row"><span>True</span><Stars value={c.potential} size={13} /><b>{stars(c.potential)}</b></div>
            <div class="card-row"><span>Scout</span><Stars value={c.scout_estimate} size={13} color="var(--azur)" /><b>{stars(c.scout_estimate)}</b></div>
            <div class="card-foot">
              <span class:pos={g > 0} class:neg={g < 0}>{g > 0 ? `+${g.toFixed(1)} underrated` : g < 0 ? `${g.toFixed(1)} overrated` : "Accurate report"}</span>
              <span class="muted">{bestCat(c)}</span>
            </div>
            <div class="card-foot"><span class="muted">{c.scout_reports} report{c.scout_reports === 1 ? "" : "s"}{#if c.my_report} · <span class="mine">yours</span>{/if}</span>
              {#if c.signable}<span class="pos">Signable</span>{:else}<span class="muted">From {c.signable_from}</span>{/if}</div>
          </button>
        {:else}
          <p class="muted">Nothing on this board with the current filters.</p>
        {/each}
      </div>
    </div>

    <div class="toolbar">
      <input bind:value={q} placeholder="Search name…" style="width:180px" />
      <div class="chips">
        {#each ["16", "17", "18+"] as a}
          <button class="chip" class:on={ages.includes(a)} on:click={() => (ages = toggle(ages, a))}>Age {a}</button>
        {/each}
      </div>
      <div class="chips">
        {#each RIDER_TYPES as t}
          <button class="chip" class:on={types.includes(t.id)} on:click={() => (types = toggle(types, t.id))}><span class="dot" style="background:{t.color}"></span>{t.label}</button>
        {/each}
      </div>
      <select bind:value={continent} aria-label="Continent"><option value="">All continents</option>{#each continents as n}<option>{n}</option>{/each}</select>
      <select bind:value={nation} aria-label="Nation"><option value="">All nations</option>{#each nations as n}<option>{n}</option>{/each}</select>
      <label class="field inline"><span>True potential ≥ {minPot.toFixed(1)}★</span><input type="range" min="0" max="6" step="0.5" bind:value={minPot} /></label>
      <label class="chk"><input type="checkbox" bind:checked={gemsOnly} /> Underrated only</label>
      <label class="chk"><input type="checkbox" bind:checked={signableOnly} /> Signable now</label>
      <span class="count muted">{rows.length.toLocaleString()} prospects</span>
    </div>

    <div class="table"><RiderTable data={rows} {cols} sortKey="potential" /></div>
  {/if}
</div>

<style>
  .head-actions { display: flex; gap: 10px; align-items: center; }
  .seg { display: inline-flex; border: 1px solid var(--rule); border-radius: 6px; overflow: hidden; }
  .seg button { background: none; border: none; color: var(--ink-2); font: inherit; font-size: 12.5px; padding: 6px 12px; cursor: pointer; }
  .seg button span { color: var(--ink-4); margin-left: 4px; }
  .seg button.on { background: var(--panel-2); color: var(--ink); }
  .seg button:disabled { opacity: 0.4; cursor: default; }
  .gem-n { color: var(--vert); font-weight: 600; }

  .board { padding: 0 22px 14px; flex-shrink: 0; }
  .board-tabs { display: flex; gap: 2px; margin-bottom: 8px; }
  .board-tabs button {
    display: inline-flex; align-items: center; gap: 5px; background: none; border: none; border-bottom: 2px solid transparent;
    color: var(--ink-3); font: inherit; font-size: 13px; padding: 4px 10px 6px; cursor: pointer;
  }
  .board-tabs button.on { color: var(--ink); border-bottom-color: var(--jaune); }
  .cards { display: grid; grid-auto-flow: column; grid-auto-columns: minmax(206px, 1fr); gap: 10px; overflow-x: auto; padding-bottom: 4px; }
  .card {
    position: relative; text-align: left; background: var(--panel); border: 1px solid var(--rule); border-radius: 8px;
    padding: 12px 12px 10px; color: var(--ink); font: inherit; cursor: pointer; display: flex; flex-direction: column; gap: 6px; min-width: 0;
  }
  .card:hover { border-color: var(--rule-2); background: var(--panel-2); }
  .card.sel { border-color: var(--jaune); }
  .rank { position: absolute; top: 6px; right: 10px; font-family: var(--font-cond); font-size: 28px; font-weight: 700; color: var(--rule-2); line-height: 1; }
  .card-top { display: flex; gap: 9px; align-items: center; margin-bottom: 4px; }
  .card-who { min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .card-who strong { font-family: var(--font-cond); font-size: 17px; font-weight: 600; line-height: 1.05; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; padding-right: 18px; }
  .card-who span { font-size: 11.5px; color: var(--ink-2); display: flex; align-items: center; gap: 4px; }
  .card-row { display: grid; grid-template-columns: 38px auto 1fr; align-items: center; gap: 6px; font-size: 11.5px; color: var(--ink-3); }
  .card-row b { text-align: right; color: var(--ink); font-weight: 600; }
  .card-foot { display: flex; justify-content: space-between; gap: 6px; font-size: 11.5px; }
  .pos { color: var(--vert); font-weight: 600; }
  .neg { color: var(--pois); }
  .mine { color: var(--azur); }

  .chips { display: flex; gap: 4px; flex-wrap: wrap; }
  .dot { width: 7px; height: 7px; border-radius: 50%; }
  .field.inline { flex-direction: row; align-items: center; gap: 8px; }
  .field.inline input { width: 110px; }
  .chk { display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: var(--ink-2); cursor: pointer; }
  .count { margin-left: auto; font-size: 12px; }
  .table { position: relative; flex: 1; overflow: hidden; border-top: 1px solid var(--rule); min-height: 200px; }
</style>
