<script lang="ts">
  import type { Cyclist, Col } from "../types";
  import { rowClass, valFor } from "../format";
  import { selectedRider, shortlistIds } from "../stores";
  import { GRADE_COLOR } from "../types";

  export let data: Cyclist[] = [];
  export let cols: Col[] = [];
  export let prefix = "r";
  export let rowHeight = 36;

  let container: HTMLDivElement;
  let scrollTop = 0;
  let viewHeight = 500;

  $: startIdx = Math.max(0, Math.floor(scrollTop / rowHeight) - 8);
  $: endIdx   = Math.min(data.length, Math.ceil((scrollTop + viewHeight) / rowHeight) + 8);
  $: visible  = data.slice(startIdx, endIdx);
  $: total    = data.length * rowHeight;
  $: offsetY  = startIdx * rowHeight;

  function onScroll() {
    scrollTop = container?.scrollTop ?? 0;
  }
  function onResize(e: ResizeObserverEntry[]) {
    viewHeight = e[0]?.contentRect.height ?? 500;
  }
  function select(c: Cyclist) { selectedRider.set(c); }

  // Sort state
  let sortKey = "";
  let sortDir = -1;
  function sortBy(key: string) {
    if (sortKey === key) sortDir = -sortDir;
    else { sortKey = key; sortDir = -1; }
    const dir = sortDir;
    data = [...data].sort((a, b) => {
      const av = (a as Record<string,unknown>)[key];
      const bv = (b as Record<string,unknown>)[key];
      if (typeof av === "number" && typeof bv === "number") return (av - bv) * dir;
      return String(av).localeCompare(String(bv)) * dir;
    });
  }

  function gradeStyle(g: string): string {
    const c = GRADE_COLOR[g] ?? "#6478a0";
    return `color:${c}; font-weight:700; font-size:11px;`;
  }
</script>

<div class="table-wrap" bind:this={container} on:scroll={onScroll}
     use:resize={onResize}>
  <table>
    <thead>
      <tr>
        {#each cols as col}
          <th style="width:{col.width}px; text-align:{col.align??'left'}"
              on:click={() => sortBy(col.key as string)}>
            {col.label}{sortKey===col.key ? (sortDir===-1?" ↓":" ↑") : ""}
          </th>
        {/each}
      </tr>
    </thead>
    <tbody>
      <!-- spacer top -->
      {#if offsetY > 0}
        <tr style="height:{offsetY}px"><td colspan={cols.length}></td></tr>
      {/if}
      {#each visible as c (c.id)}
        {@const rc = $shortlistIds.has(c.id) ? "row-sl" : rowClass(c.current_ability, c.free_agent)}
        <tr class={rc} style="height:{rowHeight}px"
            on:click={() => select(c)}
            class:selected={$selectedRider?.id === c.id}>
          {#each cols as col}
            <td style="text-align:{col.align??'left'}; width:{col.width}px">
              {#if col.key === "scout_grade"}
                <span style={gradeStyle(c.scout_grade)}>{c.scout_grade}</span>
              {:else if col.key === "rider_type"}
                {c.rider_type}
              {:else}
                {col.fmt ? col.fmt((c as Record<string,unknown>)[col.key as string], c) : valFor(c, col.key as string)}
              {/if}
            </td>
          {/each}
        </tr>
      {/each}
      <!-- spacer bottom -->
      {@const bottomPad = total - offsetY - visible.length * rowHeight}
      {#if bottomPad > 0}
        <tr style="height:{bottomPad}px"><td colspan={cols.length}></td></tr>
      {/if}
    </tbody>
  </table>
</div>

<style>
  .table-wrap {
    overflow-y: auto; overflow-x: auto;
    height: 100%; width: 100%;
    position: relative;
  }
  tr.selected td { background: #1a2d50 !important; }
</style>

<script context="module">
  // svelte action for ResizeObserver
  export function resize(node: Element, callback: (e: ResizeObserverEntry[]) => void) {
    const ro = new ResizeObserver(callback);
    ro.observe(node);
    return { destroy() { ro.disconnect(); } };
  }
</script>
