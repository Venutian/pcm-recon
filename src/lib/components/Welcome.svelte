<script lang="ts">
  import { onMount } from "svelte";
  import { findSaves, loadSave, pickSave } from "../api";
  import { timeAgo } from "../format";
  import type { SaveFile } from "../types";
  import Icon from "./Icon.svelte";

  let saves: SaveFile[] = [];
  let searched = false;
  onMount(async () => {
    saves = await findSaves().catch(() => []);
    searched = true;
  });
</script>

<div class="welcome">
  <div class="hero">
    <h1><img src="/logo.svg" alt="" />PCM Recon</h1>
    <p>Scouting and finances for your Pro Cycling Manager career. Open a career save to begin.</p>
  </div>

  <div class="list">
    <div class="list-head">
      <h3>Saves found on this PC</h3>
      <button class="btn btn-primary" on:click={pickSave}><Icon name="open" size={15} />Browse for a save</button>
    </div>
    {#if !searched}
      <p class="muted pad">Looking for Pro Cycling Manager saves…</p>
    {:else if saves.length === 0}
      <p class="muted pad">No career saves found in the usual PCM folders. Use “Browse for a save” and pick a <code>Career_*.cdb</code> file.</p>
    {:else}
      <ul>
        {#each saves as s}
          <li>
            <button on:click={() => loadSave(s.path)}>
              <span class="game">{s.game}</span>
              <span class="nm">{s.name}</span>
              <span class="when">{timeAgo(s.modified_ms)}</span>
              <span class="sz">{(s.size / 1e6).toFixed(1)} MB</span>
              <Icon name="chevron" size={16} />
            </button>
          </li>
        {/each}
      </ul>
    {/if}
    <p class="note">PCM Recon reads saves directly. Edits such as your balance are only written when you confirm them, and a backup is taken first.</p>
  </div>
</div>

<style>
  .welcome { height: 100%; overflow: auto; display: flex; flex-direction: column; align-items: center; padding: 8vh 24px 40px; gap: 36px; }
  .hero { position: relative; width: min(640px, 100%); text-align: left; }
  h1 { font-size: 64px; font-weight: 700; margin-top: 14px; letter-spacing: -0.01em; display: flex; align-items: center; gap: 16px; }
  h1 img { width: 76px; height: 76px; }
  .hero p { color: var(--ink-2); font-size: 16px; margin-top: 6px; }
  .list { width: min(640px, 100%); }
  .list-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
  ul { list-style: none; border: 1px solid var(--rule); border-radius: 8px; overflow: hidden; }
  li + li { border-top: 1px solid var(--rule); }
  li button {
    width: 100%; display: grid; grid-template-columns: 76px 1fr auto 64px 18px; gap: 14px; align-items: center;
    padding: 12px 14px; background: var(--panel); border: none; color: var(--ink); font: inherit; cursor: pointer; text-align: left;
  }
  li button:hover { background: var(--panel-2); }
  .game { font-size: 12px; color: var(--jaune); font-weight: 600; }
  .nm { font-weight: 600; }
  .when, .sz { font-size: 12px; color: var(--ink-3); text-align: right; }
  .pad { padding: 16px 0; }
  .note { margin-top: 14px; font-size: 12px; color: var(--ink-3); }
  code { font-size: 12px; color: var(--ink-2); }
</style>
