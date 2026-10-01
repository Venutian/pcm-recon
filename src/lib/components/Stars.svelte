<script lang="ts">
  /** Half-star rating from 0 to `max`. */
  export let value = 0;
  export let max = 6;
  export let size = 11;
  export let color = "var(--jaune)";
  $: v = Math.max(0, Math.min(max, Math.round(value * 2) / 2));
</script>

<span class="stars" title="{value ? value.toFixed(1) : 0} of {max} stars" style="--s:{size}px; --c:{color}">
  {#each Array(max) as _, i}
    {@const fill = v >= i + 1 ? 1 : v >= i + 0.5 ? 0.5 : 0}
    <span class="st">
      <span class="bg">★</span>
      {#if fill > 0}<span class="fg" style="width:{fill * 100}%">★</span>{/if}
    </span>
  {/each}
</span>

<style>
  .stars { display: inline-flex; line-height: 1; font-size: var(--s); vertical-align: -1px; }
  .st { position: relative; display: inline-block; width: 0.95em; }
  .bg { color: var(--rule-2); }
  .fg { position: absolute; left: 0; top: 0; overflow: hidden; color: var(--c); white-space: nowrap; }
</style>
