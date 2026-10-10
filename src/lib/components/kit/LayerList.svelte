<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { LAYER_INFO, cloneLayer, type Layer, type LayerType, type Recipe } from "../../kits/layers";
  import { useEditor } from "../../kits/editor";
  import Icon from "../Icon.svelte";

  export let recipe: Recipe;
  export let selected: string | null;

  const dispatch = createEventDispatcher<{ select: string | null; add: LayerType }>();
  const ed = useEditor();
  let adding = false;
  const MENU = Object.entries(LAYER_INFO) as [LayerType, (typeof LAYER_INFO)[LayerType]][];

  $: shown = [...recipe.layers].reverse(); // top of the stack first

  function move(l: Layer, by: number) {
    ed.change(() => {
      const i = recipe.layers.indexOf(l);
      const j = Math.min(recipe.layers.length - 1, Math.max(0, i + by));
      recipe.layers.splice(i, 1);
      recipe.layers.splice(j, 0, l);
    });
  }
  function duplicate(l: Layer) {
    const copy = cloneLayer(l);
    copy.name = `${l.name} copy`;
    ed.change(() => recipe.layers.splice(recipe.layers.indexOf(l) + 1, 0, copy));
    dispatch("select", copy.id);
  }
  function remove(l: Layer) {
    ed.change(() => recipe.layers.splice(recipe.layers.indexOf(l), 1));
    if (selected === l.id) dispatch("select", null);
  }
  function add(t: LayerType) {
    adding = false;
    dispatch("add", t);
  }
</script>

<div class="layers">
  <div class="head">
    <h4>Layers</h4>
    <button class="btn btn-sm" class:btn-primary={!recipe.layers.length} on:click={() => (adding = !adding)}><Icon name="plus" size={14} />Add layer</button>
  </div>

  {#if adding}
    <div class="menu">
      {#each MENU as [t, info]}
        <button on:click={() => add(t)}><Icon name={info.icon} size={15} /><span><strong>{info.label}</strong><small>{info.hint}</small></span></button>
      {/each}
    </div>
  {/if}

  {#each shown as l (l.id)}
    <div class="row" class:on={l.id === selected} class:hidden={l.hidden}>
      <button class="vis" title={l.hidden ? "Show" : "Hide"} on:click={() => ed.change(() => (l.hidden = !l.hidden))}><Icon name={l.hidden ? "close" : "eye"} size={13} /></button>
      <button class="name" on:click={() => dispatch("select", l.id)}>
        <Icon name={LAYER_INFO[l.type].icon} size={13} /><span>{l.name}</span>
      </button>
      <button class="ic" title="Move up" on:click={() => move(l, 1)}>▲</button>
      <button class="ic" title="Move down" on:click={() => move(l, -1)}>▼</button>
      <button class="ic" title="Duplicate (Ctrl+D)" on:click={() => duplicate(l)}>⧉</button>
      <button class="ic del" title="Delete (Del)" on:click={() => remove(l)}><Icon name="trash" size={12} /></button>
    </div>
  {:else}
    {#if !adding}<p class="muted empty">No layers yet. Add a logo, text, flag, pattern…</p>{/if}
  {/each}
</div>

<style>
  .layers { display: flex; flex-direction: column; gap: 4px; }
  .head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
  h4 { font-family: var(--font-cond); font-size: 16px; font-weight: 600; }
  .menu { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; margin-bottom: 6px; }
  .menu button { display: flex; gap: 7px; align-items: flex-start; text-align: left; padding: 7px 8px; border-radius: 6px; border: 1px solid var(--rule); background: var(--panel); color: var(--ink-2); font: inherit; cursor: pointer; }
  .menu button:hover { border-color: var(--jaune); color: var(--ink); }
  .menu :global(svg) { margin-top: 2px; flex-shrink: 0; color: var(--jaune); }
  .menu span { display: flex; flex-direction: column; }
  .menu strong { font-size: 12.5px; font-weight: 600; }
  .menu small { font-size: 10.5px; color: var(--ink-3); }
  .row { display: flex; align-items: center; gap: 2px; border-radius: 6px; border: 1px solid transparent; padding: 1px 2px; }
  .row:hover { background: var(--panel); }
  .row.on { border-color: var(--jaune); background: var(--panel); }
  .row.hidden .name { opacity: 0.45; }
  button { background: none; border: none; color: var(--ink-3); font: inherit; cursor: pointer; }
  .vis, .ic { width: 22px; height: 24px; display: grid; place-items: center; border-radius: 4px; font-size: 10px; }
  .vis:hover, .ic:hover { color: var(--ink); background: var(--panel-2); }
  .del:hover { color: var(--pois); }
  .name { flex: 1; min-width: 0; display: flex; align-items: center; gap: 6px; padding: 4px; color: var(--ink-2); font-size: 12.5px; text-align: left; }
  .name span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .row.on .name { color: var(--ink); }
  .empty { font-size: 12px; padding: 4px 2px; }
</style>
