<script lang="ts">
  import { toasts } from "../stores";
  import Icon from "./Icon.svelte";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each $toasts as t (t.id)}
    <div class="toast {t.kind}">
      <Icon name={t.kind === "error" ? "alert" : t.kind === "info" ? "info" : "check"} size={16} />
      <span>{t.text}</span>
    </div>
  {/each}
</div>

<style>
  .toasts { position: fixed; bottom: 18px; left: 50%; transform: translateX(-50%); display: flex; flex-direction: column; gap: 8px; z-index: 10000; pointer-events: none; }
  .toast {
    display: flex; align-items: center; gap: 10px; padding: 10px 14px; min-width: 280px; max-width: 560px;
    background: var(--panel-2); border: 1px solid var(--rule-2); border-left: 3px solid var(--vert);
    border-radius: var(--radius-m); box-shadow: var(--shadow-pop); font-size: 13px;
    animation: rise 0.18s ease-out;
  }
  .toast.error { border-left-color: var(--pois); }
  .toast.info { border-left-color: var(--azur); }
  .toast.ok :global(svg) { color: var(--vert); }
  .toast.error :global(svg) { color: var(--pois); }
  .toast.info :global(svg) { color: var(--azur); }
  @keyframes rise { from { transform: translateY(8px); opacity: 0; } }
</style>
