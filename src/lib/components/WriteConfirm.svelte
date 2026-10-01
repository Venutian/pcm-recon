<script lang="ts">
  import { pendingWrite, toast } from "../stores";
  import { applyEdits, isDesktop } from "../api";
  import { eur, signedEur } from "../format";
  import Modal from "./Modal.svelte";
  import Icon from "./Icon.svelte";

  let busy = false;
  let error = "";

  async function confirm() {
    const w = $pendingWrite;
    if (!w) return;
    busy = true;
    error = "";
    try {
      const res = await applyEdits(w.edits);
      toast(w.done ?? `Saved ${res.outcomes.length} change${res.outcomes.length === 1 ? "" : "s"} to ${res.meta.file_name}`);
      pendingWrite.set(null);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
  function close() {
    if (!busy) { pendingWrite.set(null); error = ""; }
  }
</script>

{#if $pendingWrite}
  <Modal title={$pendingWrite.title} width={500} on:close={close}>
    <table class="changes">
      {#each $pendingWrite.lines as l}
        <tr>
          <td class="lbl">{l.label}</td>
          <td class="from">{eur(l.from)}</td>
          <td class="arrow">→</td>
          <td class="to" class:neg={l.to < 0}>{eur(l.to)}</td>
          <td class="delta" class:pos={l.to > l.from} class:neg={l.to < l.from}>{signedEur(l.to - l.from)}</td>
        </tr>
      {/each}
    </table>

    <ul class="notes">
      <li><Icon name="alert" size={15} /><span>In PCM, go back to the main menu (don't keep this career loaded), then load it again after saving here. If the game is still on this career, its next save overwrites these changes.</span></li>
      <li><Icon name="backup" size={15} /><span>A copy of the current save is backed up first. You can restore it from Finances → Backups.</span></li>
      {#if !isDesktop}<li><Icon name="info" size={15} /><span>Preview mode: nothing is written to disk.</span></li>{/if}
    </ul>

    {#if error}<p class="err">{error}</p>{/if}

    <div class="row">
      <button class="btn" on:click={close} disabled={busy}>Cancel</button>
      <button class="btn btn-primary" on:click={confirm} disabled={busy}>{busy ? "Saving…" : "Write to save"}</button>
    </div>
  </Modal>
{/if}

<style>
  .changes { width: 100%; border-collapse: collapse; margin: 4px 0 14px; }
  .changes td { padding: 8px 4px; border-bottom: 1px solid var(--rule); font-variant-numeric: tabular-nums; }
  .lbl { color: var(--ink-2); font-size: 13px; }
  .from { color: var(--ink-3); text-align: right; }
  .arrow { color: var(--ink-4); text-align: center; width: 20px; }
  .to { font-weight: 700; text-align: right; }
  .delta { font-size: 12px; text-align: right; width: 76px; }
  .pos { color: var(--vert); }
  .neg { color: var(--pois); }
  .notes { list-style: none; display: flex; flex-direction: column; gap: 9px; font-size: 12.5px; color: var(--ink-2); }
  .notes li { display: flex; gap: 9px; align-items: flex-start; }
  .notes :global(svg) { flex-shrink: 0; margin-top: 1px; color: var(--ink-3); }
  .err { margin-top: 12px; color: var(--pois); font-size: 13px; }
  .row { display: flex; justify-content: flex-end; gap: 8px; margin-top: 18px; }
</style>
