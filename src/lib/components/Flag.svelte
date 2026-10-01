<script lang="ts">
  export let code = "";
  export let title = "";
  export let size = 14;
</script>

{#if code}
  <span class="fi fi-{code}" {title} style="width:{Math.round(size * 1.34)}px;height:{size}px"></span>
{:else}
  <span class="none" style="width:{Math.round(size * 1.34)}px;height:{size}px" {title}></span>
{/if}

<style>
  span { display: inline-block; vertical-align: -2px; background-size: cover; background-position: center; flex-shrink: 0; }
  .none { background: var(--rule); border-radius: 2px; }
</style>
