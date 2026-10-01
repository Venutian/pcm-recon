<script lang="ts">
  import { compareIds, cyclistById, toggleCompare, selectRider, paletteOpen } from "../stores";
  import { terrainProfile, eurShort, stars, statColor, stat } from "../format";
  import { STAT_KEYS, STAT_LABELS, TERRAINS } from "../types";
  import Radar from "../components/Radar.svelte";
  import Bib from "../components/Bib.svelte";
  import Flag from "../components/Flag.svelte";
  import Icon from "../components/Icon.svelte";

  // Fixed slot colours: a rider keeps their colour while others are added or removed.
  const SLOT = ["#f5c518", "#5aa8e6", "#e8524a", "#3dbe6e"];
  $: riders = $compareIds.map((id) => $cyclistById.get(id)).filter((c): c is NonNullable<typeof c> => !!c);
  let showCeiling = false;

  const rowsDef = [
    { label: "Age", get: (c: any) => c.age, best: "low" },
    { label: "Current ability", get: (c: any) => c.current_ability, best: "high" },
    { label: "Potential", get: (c: any) => c.potential, best: "high", fmt: (v: number) => `${stars(v)}★` },
    { label: "Growth left", get: (c: any) => c.growth, best: "high" },
    { label: "Wage / month", get: (c: any) => c.wage, best: "low", fmt: (v: number) => (v ? eurShort(v) : "–") },
    { label: "Contract until", get: (c: any) => c.contract_end, fmt: (v: number) => (v ? String(v) : "Free") },
    { label: "Wins", get: (c: any) => c.wins, best: "high" },
    { label: "Popularity", get: (c: any) => c.popularity, best: "high", fmt: (v: number) => v.toFixed(0) },
  ] as { label: string; get: (c: any) => number; best?: "high" | "low"; fmt?: (v: number) => string }[];

  function bestOf(values: number[], dir?: "high" | "low") {
    if (!dir || values.length < 2) return -Infinity;
    const usable = values.filter((v) => v > 0);
    return dir === "high" ? Math.max(...usable) : Math.min(...usable);
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Compare</h1>
      <p>Up to four riders side by side. Add riders with the Compare button in any profile.</p>
    </div>
    <div class="actions">
      <label class="chk"><input type="checkbox" bind:checked={showCeiling} /> Show ceilings instead of current</label>
      <button class="btn btn-sm" on:click={() => paletteOpen.set(true)}><Icon name="plus" size={14} />Add rider</button>
    </div>
  </div>

  {#if riders.length === 0}
    <div class="page-body"><p class="muted">Nobody to compare yet. Open a rider and press Compare, or use “Add rider”.</p></div>
  {:else}
    <div class="page-body">
      <div class="top">
        <div class="radar panel">
          <Radar size={360} min={45} max={85} labels={TERRAINS.map((t) => t.label)}
                 series={riders.map((c, i) => ({ label: c.name, values: terrainProfile(c, showCeiling), color: SLOT[i] }))} />
          <div class="legend">
            {#each riders as c, i}<span><i style="background:{SLOT[i]}"></i>{c.name}</span>{/each}
          </div>
        </div>
        <div class="cards">
          {#each riders as c, i (c.id)}
            <div class="card" style="--slot:{SLOT[i]}">
              <button class="rm" on:click={() => toggleCompare(c.id)} aria-label="Remove {c.name}"><Icon name="close" size={14} /></button>
              <Bib value={c.current_ability} size="m" />
              <button class="nm" on:click={() => selectRider(c.id)}>{c.name}</button>
              <span class="muted"><Flag code={c.flag} size={12} /> {c.team_short} · {c.rider_type}</span>
            </div>
          {/each}
        </div>
      </div>

      <section class="panel">
        <table class="cmp">
          <thead>
            <tr><th></th>{#each riders as c, i}<th style="color:{SLOT[i]}">{c.lastname || c.name}</th>{/each}</tr>
          </thead>
          <tbody>
            {#each rowsDef as r}
              {@const vals = riders.map(r.get)}
              {@const b = bestOf(vals, r.best)}
              <tr><td class="lbl">{r.label}</td>{#each vals as v}<td class:best={v === b}>{r.fmt ? r.fmt(v) : v}</td>{/each}</tr>
            {/each}
            <tr class="sep"><td colspan={riders.length + 1}>Attributes {showCeiling ? "(ceiling)" : "(current)"}</td></tr>
            {#each STAT_KEYS as k}
              {@const vals = riders.map((c) => stat(c, showCeiling ? `${k}_p` : k))}
              {@const b = bestOf(vals, "high")}
              <tr><td class="lbl">{STAT_LABELS[k]}</td>{#each vals as v}<td class:best={v === b} style="color:{statColor(v)}">{v}</td>{/each}</tr>
            {/each}
          </tbody>
        </table>
      </section>
    </div>
  {/if}
</div>

<style>
  .actions { display: flex; gap: 12px; align-items: center; }
  .chk { display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: var(--ink-2); cursor: pointer; }
  .page-body { display: flex; flex-direction: column; gap: 16px; }
  .top { display: grid; grid-template-columns: auto 1fr; gap: 16px; align-items: start; }
  .radar { padding: 14px 18px; display: flex; flex-direction: column; align-items: center; }
  .legend { display: flex; flex-wrap: wrap; gap: 6px 14px; justify-content: center; font-size: 12px; color: var(--ink-2); margin-top: 6px; }
  .legend i { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 5px; vertical-align: -1px; }
  .cards { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
  .card { position: relative; background: var(--panel); border: 1px solid var(--rule); border-top: 3px solid var(--slot); border-radius: 8px; padding: 14px; display: flex; flex-direction: column; gap: 6px; align-items: flex-start; }
  .card .muted { font-size: 12px; display: flex; align-items: center; gap: 5px; }
  .nm { background: none; border: none; color: var(--ink); font-family: var(--font-cond); font-size: 22px; font-weight: 600; cursor: pointer; padding: 0; text-align: left; }
  .nm:hover { text-decoration: underline; }
  .rm { position: absolute; top: 8px; right: 8px; background: none; border: none; color: var(--ink-3); cursor: pointer; padding: 3px; border-radius: 4px; }
  .rm:hover { color: var(--ink); background: var(--panel-2); }
  .cmp { width: 100%; border-collapse: collapse; font-size: 13px; table-layout: fixed; }
  .cmp th { text-align: center; padding: 10px; font-family: var(--font-cond); font-size: 17px; font-weight: 600; border-bottom: 1px solid var(--rule); }
  .cmp td { text-align: center; padding: 6px 10px; border-bottom: 1px solid #222a36; font-variant-numeric: tabular-nums; }
  .cmp td.lbl, .cmp th:first-child { text-align: left; color: var(--ink-2); width: 170px; }
  .cmp td.best { font-weight: 700; }
  .cmp td.best::after { content: ""; display: inline-block; width: 6px; height: 6px; border-radius: 50%; background: var(--jaune); margin-left: 6px; vertical-align: 1px; }
  .sep td { text-align: left; color: var(--ink-3); font-size: 12px; padding-top: 14px; }
</style>
