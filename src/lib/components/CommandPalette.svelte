<script lang="ts">
  import { allCyclists, paletteOpen, selectRider, goTo, teams } from "../stores";
  import { NAV } from "../nav";
  import Flag from "./Flag.svelte";
  import Bib from "./Bib.svelte";
  import Icon from "./Icon.svelte";

  let q = "";
  let idx = 0;
  let input: HTMLInputElement;

  type Item = { kind: "rider"; id: number; label: string; sub: string; flag: string; ca: number }
    | { kind: "section"; name: string; icon: string; label: string; sub: string };

  function fold(s: string) {
    return s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  }

  $: items = build(q);
  function build(query: string): Item[] {
    const s = fold(query.trim());
    const sections: Item[] = NAV.filter((n) => !s || fold(n.name).includes(s))
      .map((n) => ({ kind: "section", name: n.name, icon: n.icon, label: n.name, sub: "Go to section" }));
    if (!s) return sections;
    const riders: Item[] = [];
    for (const c of $allCyclists) {
      if (fold(c.name).includes(s)) {
        riders.push({ kind: "rider", id: c.id, label: c.name, sub: `${c.team} · ${c.rider_type} · ${c.age || "?"} yrs`, flag: c.flag, ca: c.current_ability });
        if (riders.length >= 40) break;
      }
    }
    const teamHits = $teams.filter((t) => fold(t.name).includes(s)).slice(0, 4)
      .map((t) => ({ kind: "section", name: "Teams", icon: "teams", label: t.name, sub: `Team · ${t.division}` } as Item));
    return [...riders.slice(0, 12), ...teamHits, ...sections.slice(0, 3)];
  }
  $: if (q !== undefined) idx = 0;

  function choose(it: Item) {
    if (it.kind === "rider") selectRider(it.id);
    else goTo(it.name);
    close();
  }
  function close() { paletteOpen.set(false); q = ""; }
  function key(e: KeyboardEvent) {
    if (e.key === "Escape") close();
    else if (e.key === "ArrowDown") { e.preventDefault(); idx = Math.min(items.length - 1, idx + 1); }
    else if (e.key === "ArrowUp") { e.preventDefault(); idx = Math.max(0, idx - 1); }
    else if (e.key === "Enter" && items[idx]) choose(items[idx]);
  }
  $: if ($paletteOpen) setTimeout(() => input?.focus(), 0);
</script>

{#if $paletteOpen}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="scrim" on:click|self={close}>
    <div class="pal" role="dialog" aria-label="Search riders and sections">
      <div class="in">
        <Icon name="scout" size={18} />
        <input bind:this={input} bind:value={q} on:keydown={key} placeholder="Search a rider, team or section…" spellcheck="false" />
        <kbd>Esc</kbd>
      </div>
      <ul>
        {#each items as it, i}
          <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
          <li class:on={i === idx} on:mouseenter={() => (idx = i)} on:click={() => choose(it)}>
            {#if it.kind === "rider"}
              <Bib value={it.ca} />
              <Flag code={it.flag} />
            {:else}
              <span class="ico"><Icon name={it.icon} size={16} /></span>
            {/if}
            <span class="lbl">{it.label}</span>
            <span class="sub">{it.sub}</span>
          </li>
        {:else}
          <li class="none">No rider or section matches “{q}”.</li>
        {/each}
      </ul>
    </div>
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; background: rgba(5, 8, 12, 0.6); z-index: 9500; display: flex; justify-content: center; padding-top: 12vh; }
  .pal { width: 620px; max-width: calc(100vw - 32px); align-self: flex-start; background: var(--panel); border: 1px solid var(--rule-2); border-radius: 10px; box-shadow: var(--shadow-pop); overflow: hidden; }
  .in { display: flex; align-items: center; gap: 10px; padding: 12px 14px; border-bottom: 1px solid var(--rule); color: var(--ink-3); }
  .in input { flex: 1; border: none; background: none; font-size: 16px; padding: 4px 0; }
  kbd { font-size: 11px; border: 1px solid var(--rule-2); border-radius: 4px; padding: 1px 6px; color: var(--ink-3); }
  ul { list-style: none; max-height: 420px; overflow-y: auto; padding: 6px; }
  li { display: flex; align-items: center; gap: 10px; padding: 7px 9px; border-radius: 6px; cursor: pointer; }
  li.on { background: var(--panel-2); }
  .ico { width: 42px; display: flex; justify-content: center; color: var(--ink-2); }
  .lbl { font-weight: 600; }
  .sub { margin-left: auto; font-size: 12px; color: var(--ink-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 55%; }
  .none { color: var(--ink-3); cursor: default; justify-content: center; padding: 20px; }
</style>
