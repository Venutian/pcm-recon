<script lang="ts">
  export let title = "";
  export let value = "—";
  export let sub   = "";
  export let accent = "#4d88f5";
  export let tip   = "";
</script>

<div class="card" title={tip}>
  <div class="bar" style="background:{accent}"></div>
  <div class="body">
    <div class="title">{title.toUpperCase()}</div>
    <div class="value" style="color:{accent}">{value}</div>
    {#if sub}<div class="sub">{sub}</div>{/if}
  </div>
</div>

<style>
  .card {
    display: flex; border: 1px solid #1e2d4a; border-radius: 8px;
    background: #0f1829; overflow: hidden; min-width: 0;
  }
  .bar { width: 4px; flex-shrink: 0; }
  .body { padding: 10px 12px; min-width: 0; }
  .title { font-size: 10px; font-weight: 700; color: #3a4e72; letter-spacing: 0.08em; text-transform: uppercase; }
  .value { font-size: 22px; font-weight: 700; line-height: 1.1; margin: 2px 0 1px; }
  .sub   { font-size: 11px; color: #6478a0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
