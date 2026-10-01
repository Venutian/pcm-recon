<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  export let title = "";
  export let width = 480;
  const dispatch = createEventDispatcher();
  let box: HTMLDivElement;
  onMount(() => box?.querySelector<HTMLElement>("[data-autofocus], input, button.btn-primary")?.focus());
  function key(e: KeyboardEvent) {
    if (e.key === "Escape") dispatch("close");
  }
</script>

<svelte:window on:keydown={key} />
<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="scrim" on:click|self={() => dispatch("close")}>
  <div class="modal" role="dialog" aria-modal="true" aria-label={title} style="width:{width}px" bind:this={box}>
    {#if title}<h2>{title}</h2>{/if}
    <slot />
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(5, 8, 12, 0.66); z-index: 9000; display: grid; place-items: center; }
  .modal {
    background: var(--panel); border: 1px solid var(--rule-2); border-radius: 10px; padding: 20px 22px;
    box-shadow: var(--shadow-pop); max-height: 86vh; overflow: auto; max-width: calc(100vw - 32px);
  }
  h2 { margin-bottom: 12px; font-size: 22px; }
</style>
