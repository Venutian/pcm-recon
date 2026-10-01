<script lang="ts">
  import { allCyclists, myRiders, myTeam, finance, meta, prospects, shortlist, goTo, selectRider, scoutPreset } from "../stores";
  import { eur, eurShort, gemGap, stars } from "../format";
  import { TYPE_COLOR } from "../types";
  import Bib from "../components/Bib.svelte";
  import Flag from "../components/Flag.svelte";
  import Stars from "../components/Stars.svelte";
  import Icon from "../components/Icon.svelte";

  $: season = $meta?.season ?? 0;
  $: f = $finance;
  $: burn = f ? f.monthly_rider_wages + f.monthly_staff_wages : 0;
  $: expiring = $myRiders.filter((c) => c.contract_end > 0 && c.contract_end <= season);
  $: injured = $myRiders.filter((c) => c.injured);
  $: gems = $prospects.filter((c) => gemGap(c) >= 0.5 && c.potential >= 3.5);
  $: myBest = new Map<number, number>();
  $: {
    const m = new Map<number, number>();
    for (const r of $myRiders) m.set(r.rider_type_id, Math.max(m.get(r.rider_type_id) ?? 0, r.current_ability));
    myBest = m;
  }
  $: targets = $allCyclists
    .filter((c) => c.on_market && !c.is_mine && c.signable && c.current_ability > (myBest.get(c.rider_type_id) ?? 0))
    .sort((a, b) => b.current_ability - a.current_ability)
    .slice(0, 8);
  $: topProspects = [...$prospects].sort((a, b) => b.potential - a.potential || b.skill_ceiling - a.skill_ceiling).slice(0, 8);
  $: wonderkids = $allCyclists.filter((c) => c.age > 0 && c.age <= 21 && c.potential >= 5 && !c.is_mine).length;

  interface Alert { tone: "bad" | "warn" | "good" | "info"; icon: string; title: string; body: string; action?: string; go?: () => void; }
  $: alerts = ((): Alert[] => {
    const out: Alert[] = [];
    if (f && f.balance < 0) out.push({ tone: "bad", icon: "finance", title: `You're ${eurShort(-f.balance)} in debt`, body: `Monthly costs are ${eurShort(burn)}. You can set the balance directly in Finances.`, action: "Fix the balance", go: () => goTo("Finances") });
    else if (f && burn > 0 && f.balance < burn * 3) out.push({ tone: "warn", icon: "finance", title: "Cash is running low", body: `${eur(f.balance)} covers about ${Math.max(0, Math.floor(f.balance / burn))} months of wages.`, action: "Open finances", go: () => goTo("Finances") });
    if (expiring.length) out.push({ tone: "warn", icon: "contract", title: `${expiring.length} contract${expiring.length === 1 ? "" : "s"} end this season`, body: expiring.slice(0, 3).map((c) => c.name).join(", ") + (expiring.length > 3 ? "…" : ""), action: "Review contracts", go: () => goTo("My team") });
    if (injured.length) out.push({ tone: "bad", icon: "alert", title: `${injured.length} rider${injured.length === 1 ? "" : "s"} injured`, body: injured.map((c) => c.name).join(", ") });
    if (gems.length) out.push({ tone: "good", icon: "gem", title: `${gems.length} underrated prospect${gems.length === 1 ? "" : "s"}`, body: "Scout reports undersell their real potential by half a star or more.", action: "See prospects", go: () => goTo("Prospects") });
    if (wonderkids) out.push({ tone: "info", icon: "prospects", title: `${wonderkids} wonderkids in the game`, body: "Riders 21 or younger with 5★ potential or more.", action: "List them", go: () => { scoutPreset.set({ maxAge: 21, minPot: 5, sort: "potential", hideMine: true }); goTo("Scout"); } });
    if ($shortlist.size) out.push({ tone: "info", icon: "shortlist", title: `${$shortlist.size} on your shortlist`, body: "Compare them side by side or export to CSV.", action: "Open shortlist", go: () => goTo("Shortlist") });
    return out;
  })();
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{$meta?.manager ? `Welcome back, ${$meta.manager}` : "Overview"}</h1>
      <p>{$myTeam?.name ?? "Your team"} · season {season}. Start with whatever needs your attention.</p>
    </div>
  </div>

  <div class="page-body">
    <div class="alerts">
      {#each alerts as a}
        <div class="alert {a.tone}">
          <span class="ai"><Icon name={a.icon} size={18} /></span>
          <div class="at"><strong>{a.title}</strong><span>{a.body}</span></div>
          {#if a.action}<button class="btn btn-sm" on:click={a.go}>{a.action}</button>{/if}
        </div>
      {:else}
        <p class="muted">Nothing urgent. Your squad, contracts and cash all look fine.</p>
      {/each}
    </div>

    <div class="cols">
      <section class="panel">
        <div class="panel-head"><h3>Upgrades on the market</h3><button class="link" on:click={() => goTo("Market")}>All {season} transfers</button></div>
        <p class="sub muted">Available riders better than your best rider of the same type.</p>
        <ul class="list">
          {#each targets as c}
            <li><button on:click={() => selectRider(c.id)}>
              <Bib value={c.current_ability} />
              <span class="nm"><Flag code={c.flag} size={12} />{c.name}</span>
              <span class="tp" style="color:{TYPE_COLOR[c.rider_type]}">{c.rider_type}</span>
              <span class="meta">{c.age} · {c.free_agent ? "free" : eurShort(c.wage) + "/mo"}</span>
              {#if myBest.has(c.rider_type_id)}<span class="pos">+{(c.current_ability - (myBest.get(c.rider_type_id) ?? 0)).toFixed(1)}</span>{:else}<span class="new" title="You have no {c.rider_type} yet">new</span>{/if}
            </button></li>
          {:else}<li class="muted none">No upgrades available right now.</li>{/each}
        </ul>
      </section>

      <section class="panel">
        <div class="panel-head"><h3>Best prospects</h3><button class="link" on:click={() => goTo("Prospects")}>Draft board</button></div>
        <p class="sub muted">Scouted juniors by true potential, with what the scout reported.</p>
        <ul class="list">
          {#each topProspects as c}
            {@const g = gemGap(c)}
            <li><button on:click={() => selectRider(c.id)}>
              <Bib value={c.current_ability} />
              <span class="nm"><Flag code={c.flag} size={12} />{c.name}</span>
              <Stars value={c.potential} size={11} />
              <span class="meta">scout {stars(c.scout_estimate)}★</span>
              <span class:pos={g > 0} class:neg={g < 0}>{g > 0 ? "+" : ""}{g.toFixed(1)}</span>
            </button></li>
          {:else}<li class="muted none">No scout reports yet.</li>{/each}
        </ul>
      </section>
    </div>
  </div>
</div>

<style>
  .page-body { display: flex; flex-direction: column; gap: 18px; }
  .alerts { display: grid; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); gap: 10px; }
  .alert { display: flex; align-items: center; gap: 12px; padding: 12px 14px; background: var(--panel); border: 1px solid var(--rule); border-left: 3px solid var(--azur); border-radius: 0 8px 8px 0; }
  .alert.bad { border-left-color: var(--pois); }
  .alert.warn { border-left-color: var(--jaune); }
  .alert.good { border-left-color: var(--vert); }
  .ai { color: var(--ink-2); display: flex; }
  .bad .ai { color: var(--pois); } .warn .ai { color: var(--jaune); } .good .ai { color: var(--vert); } .info .ai { color: var(--azur); }
  .at { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; }
  .at strong { font-size: 14px; }
  .at span { font-size: 12.5px; color: var(--ink-2); overflow: hidden; text-overflow: ellipsis; }
  .cols { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  .sub { padding: 0 14px 6px; font-size: 12px; margin-top: -4px; }
  .link { background: none; border: none; color: var(--jaune); font: inherit; font-size: 12.5px; cursor: pointer; }
  .link:hover { text-decoration: underline; }
  .list { list-style: none; padding: 0 6px 10px; }
  .list button {
    width: 100%; display: grid; grid-template-columns: auto minmax(0, 1fr) auto auto 44px; align-items: center; gap: 10px;
    padding: 6px 8px; background: none; border: none; border-radius: 6px; color: var(--ink); font: inherit; font-size: 13px; cursor: pointer; text-align: left;
  }
  .list button:hover { background: var(--panel-2); }
  .nm { display: flex; align-items: center; gap: 7px; font-weight: 500; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  .tp { font-size: 12px; }
  .meta { font-size: 12px; color: var(--ink-3); white-space: nowrap; }
  .pos { color: var(--vert); font-weight: 600; text-align: right; }
  .neg { color: var(--pois); text-align: right; }
  .new { color: var(--azur); font-size: 12px; text-align: right; }
  .none { padding: 10px; font-size: 13px; }
  @media (max-width: 1150px) { .cols { grid-template-columns: 1fr; } }
</style>
