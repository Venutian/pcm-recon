<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { partImage, thumbVersion } from "../kits/store";

  export let kit: string;
  export let part: string;
  export let edited = false;
  /** Pre-rendered image to show instead of loading one. */
  export let image: ImageData | null = null;
  export let alt = "";

  let el: HTMLCanvasElement;
  let visible = false;
  let failed = false;
  let observer: IntersectionObserver | null = null;

  onMount(() => {
    observer = new IntersectionObserver((entries) => {
      if (entries.some((e) => e.isIntersecting)) {
        visible = true;
        observer?.disconnect();
      }
    }, { rootMargin: "200px" });
    observer.observe(el);
  });
  onDestroy(() => observer?.disconnect());

  function draw(img: ImageData) {
    if (!el) return;
    el.width = img.width;
    el.height = img.height;
    el.getContext("2d")!.putImageData(img, 0, 0);
    failed = false;
  }

  $: if (image && el) draw(image);
  $: if (visible && !image) load(kit, part, edited, $thumbVersion);
  async function load(k: string, p: string, e: boolean, _v: number) {
    try { draw(await partImage(k, p, e)); } catch { failed = true; }
  }
</script>

<canvas bind:this={el} class:failed aria-label={alt}></canvas>

<style>
  canvas { width: 100%; height: 100%; object-fit: contain; display: block; }
  .failed { background: repeating-linear-gradient(45deg, var(--bg-2), var(--bg-2) 6px, var(--panel) 6px, var(--panel) 12px); }
</style>
