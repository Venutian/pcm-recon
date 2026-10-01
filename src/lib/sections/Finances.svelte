<script lang="ts">
  import { onMount } from "svelte";
  import { finance, meta, myTeam, requestWrite, save, toast } from "../stores";
  import { listBackups, restoreBackup, openBackupFolder, isDesktop } from "../api";
  import { eur, eurShort, signedEur, fmtDate, timeAgo } from "../format";
  import type { BackupInfo, CellEdit, LedgerEntry } from "../types";
  import MoneyEditor from "../components/MoneyEditor.svelte";
  import ProfileChart from "../components/ProfileChart.svelte";
  import Modal from "../components/Modal.svelte";
  import Icon from "../components/Icon.svelte";

  $: f = $finance;
  $: burn = f ? f.monthly_rider_wages + f.monthly_staff_wages : 0;

  // Sponsor money lands on the same day each season (1 Nov in stock PCM); read it from the ledger.
  $: payDay = (() => {
    const last = f?.ledger.filter((e) => e.category === "sponsor").at(-1);
    return last ? last.date.slice(5) : "11-01";
  })();
  $: nextPay = (() => {
    if (!$meta?.game_date) return "";
    const y = Number($meta.game_date.slice(0, 4));
    const candidate = `${y}-${payDay}`;
    return candidate > $meta.game_date ? candidate : `${y + 1}-${payDay}`;
  })();
  $: monthsToPay = (() => {
    if (!nextPay || !$meta?.game_date) return 0;
    const a = new Date($meta.game_date + "T00:00:00"), b = new Date(nextPay + "T00:00:00");
    return Math.max(0, (b.getFullYear() - a.getFullYear()) * 12 + b.getMonth() - a.getMonth() - (b.getDate() < a.getDate() ? 1 : 0));
  })();
  $: projected = f ? f.balance - monthsToPay * burn : 0;
  $: neededForZero = Math.max(0, -projected);
  $: runway = f && burn > 0 && f.balance > 0 ? Math.floor(f.balance / burn) : 0;

  // ── Balance timeline: replay the ledger, re-syncing at each monthly snapshot
  $: points = (() => {
    if (!f) return [];
    const pts: { date: string; value: number; note?: string }[] = [];
    let running: number | null = null;
    for (const e of f.ledger) {
      if (e.category === "balance") running = e.amount;
      else if (running !== null) running += e.amount;
      if (running !== null) pts.push({ date: e.date, value: running, note: e.category === "balance" ? "Month start" : `${e.label}${e.detail ? ` · ${e.detail}` : ""}: ${signedEur(e.amount)}` });
    }
    if ($meta?.game_date) pts.push({ date: $meta.game_date, value: f.balance, note: "Today" });
    // keep the last point per day so the profile stays readable
    const byDay = new Map<string, typeof pts[number]>();
    for (const p of pts) byDay.set(p.date, p);
    return [...byDay.values()];
  })();

  // ── Income & expenses by category (everything in the ledger except snapshots)
  const CAT_LABEL: Record<string, string> = {
    sponsor: "Sponsor", prize: "Prize money", wages: "Rider wages", staff: "Staff",
    equipment: "Equipment", training: "Training camps", other: "Other",
  };
  $: flows = (() => {
    const m = new Map<string, number>();
    for (const e of f?.ledger ?? []) if (e.category !== "balance") m.set(e.category, (m.get(e.category) ?? 0) + e.amount);
    return [...m.entries()].map(([k, v]) => ({ key: k, label: CAT_LABEL[k] ?? k, value: v })).sort((a, b) => b.value - a.value);
  })();
  $: flowMax = Math.max(1, ...flows.map((x) => Math.abs(x.value)));
  $: income = flows.filter((x) => x.value > 0).reduce((s, x) => s + x.value, 0);
  $: spend = flows.filter((x) => x.value < 0).reduce((s, x) => s + x.value, 0);

  let ledgerCat = "all";
  $: ledgerRows = (f?.ledger ?? []).filter((e) => e.category !== "balance" && (ledgerCat === "all" || e.category === ledgerCat)).slice().reverse();

  // ── Edits
  function editBalance(to: number) {
    if (!f) return;
    requestWrite({
      title: "Change your balance",
      lines: [{ label: "Cash balance", from: f.balance, to }],
      edits: [{ table: "GAM_career_data", column: "value", key_column: "CONSTANT", key: "SOLDE", value: to }],
      done: `Balance set to ${eur(to)}`,
    });
  }

  let sponsorTab: "now" | "next" = "now";
  function editSponsor(to: number) {
    if (!f) return;
    const edits: CellEdit[] = [];
    const lines = [];
    if (sponsorTab === "now") {
      edits.push({ table: "DYN_team", column: "value_i_budget", key_column: "IDteam", key: f.team_id, value: to });
      lines.push({ label: "Season budget", from: f.season_budget, to });
      if (f.sponsor_deal_id) {
        edits.push({ table: "DYN_team_sponsor", column: "value_i_budget", key_column: "IDteam_sponsor", key: f.sponsor_deal_id, value: to });
        lines.push({ label: `${f.sponsor || "Sponsor"} deal, this season`, from: f.sponsor_budget, to });
      }
    } else if (f.sponsor_deal_id) {
      edits.push({ table: "DYN_team_sponsor", column: "value_i_budget_next", key_column: "IDteam_sponsor", key: f.sponsor_deal_id, value: to });
      lines.push({ label: `${f.sponsor || "Sponsor"} deal, next season`, from: f.sponsor_budget_next, to });
    }
    requestWrite({ title: "Change sponsor budget", lines, edits, done: "Sponsor budget updated" });
  }

  // ── Backups
  let backups: BackupInfo[] = [];
  let restoreTarget: BackupInfo | null = null;
  async function refreshBackups() { backups = isDesktop ? await listBackups().catch(() => []) : []; }
  onMount(refreshBackups);
  $: if ($save) refreshBackups();
  async function doRestore() {
    if (!restoreTarget) return;
    try {
      await restoreBackup(restoreTarget.path);
      toast(`Restored the backup from ${timeAgo(restoreTarget.created_ms)}`);
    } catch (e) {
      toast(`Restore failed: ${e}`, "error", 7000);
    }
    restoreTarget = null;
  }

  function catClass(e: LedgerEntry) { return e.amount >= 0 ? "pos" : "neg"; }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>Finances</h1>
      <p>{$myTeam?.name ?? "Your team"}{f?.sponsor ? `, sponsored by ${f.sponsor}` : ""}. Change your cash balance or sponsor money and write it straight into the save.</p>
    </div>
  </div>

  {#if !f}
    <div class="page-body"><p class="muted">This save has no manager career data, so there are no finances to show.</p></div>
  {:else}
    <div class="page-body">
      <div class="top">
        <section class="panel balance">
          <div class="bal-head">
            <div>
              <small>Cash balance</small>
              <div class="figure" class:neg={f.balance < 0}>{eur(f.balance)}</div>
              <small class="sub">{f.balance < 0 ? "Your team is in debt." : "Available to spend."} Monthly costs {eurShort(burn)}.</small>
            </div>
          </div>
          {#if f.balance_editable}
            <MoneyEditor current={Math.round(f.balance)} on:apply={(e) => editBalance(e.detail)}
              presets={[
                ...(f.balance < 0 ? [{ label: "Clear debt", value: 0 }] : []),
                ...(neededForZero > 0 ? [{ label: "Cover costs until sponsor day", value: Math.round(f.balance + neededForZero) }] : []),
                { label: "+ a year of wages", value: Math.round(f.balance + burn * 12) },
              ]} />
          {:else}
            <p class="muted">The balance row (SOLDE) wasn't found in this save, so it can't be edited.</p>
          {/if}
        </section>

        <section class="panel outlook">
          <h3>Outlook</h3>
          <dl>
            <div><dt>Rider wages</dt><dd>{eurShort(f.monthly_rider_wages)}<small>/ month</small></dd></div>
            <div><dt>Staff wages</dt><dd>{eurShort(f.monthly_staff_wages)}<small>/ month</small></dd></div>
            <div><dt>Next sponsor payment</dt><dd>{fmtDate(nextPay)}<small>in {monthsToPay} mo</small></dd></div>
            <div><dt>Expected then</dt><dd>{eurShort(f.sponsor_budget_next || f.season_budget)}</dd></div>
            <div class="wide">
              <dt>Balance just before that payment</dt>
              <dd class:neg={projected < 0} class:pos={projected >= 0}>{eur(projected)}</dd>
              <small class="muted">Wages only. Prize money and equipment are not included.</small>
            </div>
            {#if runway > 0}
              <div class="wide"><dt>Runway</dt><dd>{runway} months<small>before the cash runs out with no income</small></dd></div>
            {/if}
          </dl>
        </section>
      </div>

      <section class="panel">
        <div class="panel-head">
          <h3>Sponsor budget</h3>
          <span class="muted">{f.sponsor || "No sponsor"}{f.sponsor_contract_end ? ` · contract ${f.sponsor_contract_start}–${f.sponsor_contract_end}` : ""}</span>
        </div>
        <div class="sp-body">
          <div class="tabs" role="tablist">
            <button role="tab" class:on={sponsorTab === "now"} on:click={() => (sponsorTab = "now")}>
              This season <strong>{eurShort(f.season_budget)}</strong>
            </button>
            <button role="tab" class:on={sponsorTab === "next"} disabled={!f.sponsor_deal_id} on:click={() => (sponsorTab = "next")}>
              Next season <strong>{eurShort(f.sponsor_budget_next)}</strong>
            </button>
          </div>
          <p class="muted explain">
            {#if sponsorTab === "now"}
              The yearly budget your sponsor grants. It sets your spending limits for wages and transfers this season. It doesn't add cash today; change the balance above for that.
            {:else}
              What the sponsor has agreed to pay next season. The game uses it when the new season starts.
            {/if}
          </p>
          {#key sponsorTab}
            <MoneyEditor current={sponsorTab === "now" ? f.season_budget : f.sponsor_budget_next} min={0} max={200_000_000}
              steps={[-500_000, 250_000, 500_000, 1_000_000, 2_500_000, 5_000_000]} on:apply={(e) => editSponsor(e.detail)} />
          {/key}
        </div>
      </section>

      <section class="panel">
        <div class="panel-head"><h3>Balance profile</h3><span class="muted">Replayed from the in-game ledger. Hover to see each payment.</span></div>
        <div class="chart"><ProfileChart {points} /></div>
      </section>

      <div class="two">
        <section class="panel">
          <div class="panel-head"><h3>Where the money went</h3><span class="muted">In {eurShort(income)} · out {eurShort(-spend)}</span></div>
          <div class="flows">
            {#each flows as fl}
              <span class="fl-l">{fl.label}</span>
              <div class="fl-track">
                <div class="mid"></div>
                {#if fl.value >= 0}
                  <div class="bar in" style="left:50%; width:{(fl.value / flowMax) * 50}%"></div>
                {:else}
                  <div class="bar out" style="right:50%; width:{(-fl.value / flowMax) * 50}%"></div>
                {/if}
              </div>
              <span class="fl-v" class:pos={fl.value > 0} class:neg={fl.value < 0}>{signedEur(fl.value)}</span>
            {/each}
          </div>
        </section>

        <section class="panel">
          <div class="panel-head"><h3>Staff</h3><span class="muted">{f.staff.length} people · {eurShort(f.monthly_staff_wages)} / month</span></div>
          <table class="grid-table">
            <thead><tr><th>Name</th><th>Role</th><th class="r">Wage / month</th><th class="r">Contract</th></tr></thead>
            <tbody>
              {#each f.staff as s}
                <tr><td>{s.name}</td><td class="muted">{s.role}</td><td class="r">{eurShort(s.wage)}</td><td class="r" class:neg={s.contract_end <= ($meta?.season ?? 0)}>{s.contract_end || "–"}</td></tr>
              {/each}
            </tbody>
          </table>
        </section>
      </div>

      <div class="two">
        <section class="panel">
          <div class="panel-head"><h3>Equipment deals</h3><span class="muted">Brand money per season</span></div>
          {#if f.brands.length}
            <table class="grid-table">
              <thead><tr><th>Brand</th><th>Kind</th><th class="r">Budget</th><th class="r">Years left</th></tr></thead>
              <tbody>
                {#each f.brands as b}
                  <tr><td>{b.brand}</td><td class="muted">{b.category}</td><td class="r">{eurShort(b.budget)}</td><td class="r">{b.years_left}</td></tr>
                {/each}
              </tbody>
            </table>
          {:else}<p class="muted pad">No active equipment deals.</p>{/if}
        </section>

        <section class="panel">
          <div class="panel-head"><h3>Sponsor offers</h3><span class="muted">Open approaches from sponsors</span></div>
          {#if f.offers.length}
            <table class="grid-table">
              <thead><tr><th>Sponsor</th><th class="r">Budget</th><th class="r">Years</th><th class="r">Deadline</th></tr></thead>
              <tbody>
                {#each f.offers.slice().sort((a, b) => b.budget - a.budget) as o}
                  <tr><td>{o.sponsor}{#if o.state === 1}<span class="tag">accepted</span>{/if}</td><td class="r">{o.budget ? eurShort(o.budget) : "Not set yet"}</td><td class="r">{o.duration}</td><td class="r">{fmtDate(o.deadline)}</td></tr>
                {/each}
              </tbody>
            </table>
          {:else}<p class="muted pad">No sponsor offers right now.</p>{/if}
        </section>
      </div>

      <section class="panel">
        <div class="panel-head">
          <h3>Ledger</h3>
          <div class="cats">
            {#each [["all", "All"], ...Object.entries(CAT_LABEL)] as [k, l]}
              <button class="chip" class:on={ledgerCat === k} on:click={() => (ledgerCat = k)}>{l}</button>
            {/each}
          </div>
        </div>
        <div class="ledger">
          <table class="grid-table">
            <thead><tr><th>Date</th><th>Type</th><th>Details</th><th class="r">Amount</th></tr></thead>
            <tbody>
              {#each ledgerRows as e}
                <tr><td class="muted">{fmtDate(e.date)}</td><td>{e.label}</td><td class="muted">{e.detail}</td><td class="r {catClass(e)}">{signedEur(e.amount)}</td></tr>
              {:else}
                <tr><td colspan="4" class="muted">Nothing in this category yet.</td></tr>
              {/each}
            </tbody>
          </table>
        </div>
      </section>

      <section class="panel">
        <div class="panel-head">
          <h3>Backups</h3>
          <div class="bk-actions">
            <span class="muted">Taken automatically before every change</span>
            {#if isDesktop}<button class="btn btn-sm btn-quiet" on:click={() => openBackupFolder()}><Icon name="open" size={14} />Open folder</button>{/if}
          </div>
        </div>
        {#if backups.length}
          <table class="grid-table">
            <thead><tr><th>Taken</th><th>File</th><th class="r">Size</th><th></th></tr></thead>
            <tbody>
              {#each backups as b}
                <tr>
                  <td>{timeAgo(b.created_ms)}</td>
                  <td class="muted">{b.file_name}</td>
                  <td class="r muted">{(b.size / 1e6).toFixed(1)} MB</td>
                  <td class="r"><button class="btn btn-sm" on:click={() => (restoreTarget = b)}><Icon name="backup" size={14} />Restore</button></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {:else}
          <p class="muted pad">No backups yet. One is created the first time you change something.</p>
        {/if}
      </section>
    </div>
  {/if}
</div>

{#if restoreTarget}
  <Modal title="Restore this backup?" on:close={() => (restoreTarget = null)}>
    <p class="modal-p">The save goes back to how it was {timeAgo(restoreTarget.created_ms)}. Your current save is backed up first, so you can undo this too.</p>
    <div class="modal-row">
      <button class="btn" on:click={() => (restoreTarget = null)}>Cancel</button>
      <button class="btn btn-primary" on:click={doRestore}>Restore backup</button>
    </div>
  </Modal>
{/if}

<style>
  .page-body { display: flex; flex-direction: column; gap: 16px; }
  .top { display: grid; grid-template-columns: minmax(0, 1.6fr) minmax(280px, 1fr); gap: 16px; }
  .balance { padding: 18px; display: flex; flex-direction: column; gap: 16px; }
  .bal-head small { font-size: 12px; color: var(--ink-3); }
  .figure { font-family: var(--font-cond); font-size: 64px; font-weight: 700; line-height: 1; color: var(--vert); letter-spacing: -0.01em; margin: 2px 0 4px; }
  .figure.neg { color: var(--pois); }
  .sub { display: block; }

  .outlook { padding: 16px 18px; }
  .outlook h3 { margin-bottom: 10px; }
  dl { display: grid; grid-template-columns: 1fr 1fr; gap: 14px 16px; }
  dl .wide { grid-column: span 2; padding-top: 10px; border-top: 1px solid var(--rule); }
  dt { font-size: 11px; color: var(--ink-3); }
  dd { font-family: var(--font-cond); font-size: 22px; font-weight: 700; display: flex; align-items: baseline; gap: 6px; }
  dd small { font-family: var(--font); font-size: 11px; font-weight: 400; color: var(--ink-3); }
  .wide small.muted { font-size: 11px; }

  .sp-body { padding: 0 14px 16px; display: flex; flex-direction: column; gap: 10px; }
  .tabs { display: flex; gap: 6px; }
  .tabs button {
    background: var(--bg-2); border: 1px solid var(--rule); color: var(--ink-2); border-radius: 6px;
    padding: 8px 14px; font: inherit; font-size: 13px; cursor: pointer; display: flex; gap: 10px; align-items: baseline;
  }
  .tabs button strong { font-family: var(--font-cond); font-size: 18px; color: var(--ink); }
  .tabs button.on { border-color: var(--jaune); color: var(--ink); }
  .tabs button:disabled { opacity: 0.4; cursor: default; }
  .explain { font-size: 12.5px; max-width: 80ch; }

  .chart { padding: 4px 14px 12px; }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  .two .panel { overflow: hidden; }
  .flows { display: grid; grid-template-columns: 110px 1fr 72px; gap: 8px 10px; align-items: center; padding: 4px 14px 16px; }
  .fl-l { font-size: 12.5px; color: var(--ink-2); }
  .fl-track { position: relative; height: 14px; }
  .mid { position: absolute; left: 50%; top: -3px; bottom: -3px; width: 1px; background: var(--rule-2); }
  .bar { position: absolute; top: 0; bottom: 0; }
  .bar.in { background: var(--vert); border-radius: 0 4px 4px 0; }
  .bar.out { background: var(--pois); border-radius: 4px 0 0 4px; }
  .fl-v { font-size: 12.5px; text-align: right; font-weight: 600; }
  .pos { color: var(--vert); }
  .neg { color: var(--pois); }
  .pad { padding: 4px 14px 16px; }
  .tag { margin-left: 6px; font-size: 10px; color: var(--vert); border: 1px solid #3dbe6e66; border-radius: 999px; padding: 0 6px; }
  .cats { display: flex; gap: 5px; flex-wrap: wrap; }
  .ledger { max-height: 380px; overflow: auto; }
  .bk-actions { display: flex; align-items: center; gap: 10px; }
  .modal-p { color: var(--ink-2); font-size: 13.5px; }
  .modal-row { display: flex; justify-content: flex-end; gap: 8px; margin-top: 18px; }

  @media (max-width: 1100px) {
    .top, .two { grid-template-columns: 1fr; }
  }
</style>
