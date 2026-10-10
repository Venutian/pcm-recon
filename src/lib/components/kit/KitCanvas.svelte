<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from "svelte";
  import { isPositioned, type Positioned, type Recipe, type BrushLayer, type PatchLayer, type Stroke } from "../../kits/layers";
  import { bounds, rgbHex, toCanvas } from "../../kits/render";
  import { useEditor } from "../../kits/editor";

  export let rendered: HTMLCanvasElement | null;
  export let base: ImageData | null;
  export let mask: HTMLCanvasElement | null;
  export let recipe: Recipe;
  export let selected: string | null;
  export let tool: "select" | "brush" | "pick";
  export let brush: { color: string; size: number; erase: boolean };
  export let compare = false;
  export let guides = false;
  export let zoom = 1;
  export let split: boolean;
  /** While picking a colour for a layer: the kit as it looks below that layer. */
  export let pickFrom: HTMLCanvasElement | null = null;

  const dispatch = createEventDispatcher<{ select: string | null; pick: string }>();
  const ed = useEditor();

  let wrap: HTMLDivElement;
  let view: HTMLCanvasElement;
  let fitW = 600;
  let ro: ResizeObserver | null = null;
  $: W = rendered?.width ?? base?.width ?? 1;
  $: H = rendered?.height ?? base?.height ?? 1;

  onMount(() => {
    ro = new ResizeObserver(() => {
      const r = wrap.getBoundingClientRect();
      fitW = Math.max(200, Math.min(r.width - 24, ((r.height - 24) * W) / H));
    });
    ro.observe(wrap);
  });
  onDestroy(() => ro?.disconnect());

  $: baseCanvas = base ? toCanvas(base) : null;
  $: sel = recipe.layers.find((l) => l.id === selected) ?? null;
  $: selPos = sel && isPositioned(sel) && !sel.hidden ? sel : null;
  $: selBrush = sel?.type === "brush" ? (sel as BrushLayer) : null;
  $: selPatch = selPos?.type === "patch" ? (selPos as PatchLayer) : null;
  /** A selected layer's area limit, editable on the canvas (when the layer itself isn't movable). */
  $: selArea = sel && !selPos && sel.area ? sel.area : null;
  const areaBox = (a: { x: number; y: number; w: number; h: number }) => { const w = a.w * W, h = a.h * W; return { x: a.x * W - w / 2, y: a.y * H - h / 2, w, h }; };
  /** A patch's source box in image pixels. */
  const sourceBox = (l: PatchLayer) => { const w = l.w * W, h = l.h * W; return { x: (l.x + l.dx) * W - w / 2, y: (l.y + l.dy) * H - h / 2, w, h }; };

  let hover: { x: number; y: number } | null = null;
  let snapLines: { x?: number; y?: number } = {};

  // Guide overlay: the template's grey filler, hatched, plus the front|back divider.
  let guideCanvas: HTMLCanvasElement | null = null;
  $: guideCanvas = mask ? makeGuides(mask) : null;
  function makeGuides(m: HTMLCanvasElement) {
    const c = document.createElement("canvas");
    c.width = m.width;
    c.height = m.height;
    const x = c.getContext("2d")!;
    const tile = document.createElement("canvas");
    tile.width = tile.height = 14;
    const t = tile.getContext("2d")!;
    t.strokeStyle = "rgba(245,197,24,0.35)";
    t.lineWidth = 2;
    t.beginPath(); t.moveTo(0, 14); t.lineTo(14, 0); t.stroke();
    x.fillStyle = x.createPattern(tile, "repeat")!;
    x.fillRect(0, 0, c.width, c.height);
    x.globalCompositeOperation = "destination-out";
    x.drawImage(m, 0, 0);
    return c;
  }

  // Redraw whenever anything visible changes.
  $: draw(rendered, baseCanvas, pickFrom, compare, guides, guideCanvas, selPos, recipe, hover, tool, brush, snapLines);
  function draw(..._deps: unknown[]) {
    if (!view) return;
    const src = compare ? baseCanvas : tool === "pick" && pickFrom ? pickFrom : rendered ?? baseCanvas;
    if (!src) return;
    if (view.width !== src.width) view.width = src.width;
    if (view.height !== src.height) view.height = src.height;
    const c = view.getContext("2d")!;
    c.clearRect(0, 0, view.width, view.height);
    c.drawImage(src, 0, 0);
    const px = W / Math.max(1, view.clientWidth); // image pixels per screen pixel
    if (guides && guideCanvas) {
      c.drawImage(guideCanvas, 0, 0);
      if (split) {
        c.strokeStyle = "rgba(245,197,24,0.8)";
        c.setLineDash([10 * px, 6 * px]);
        c.lineWidth = px;
        c.beginPath(); c.moveTo(W / 2, 0); c.lineTo(W / 2, H); c.stroke();
        c.setLineDash([]);
      }
    }
    c.strokeStyle = "rgba(0,200,255,0.9)";
    c.lineWidth = px;
    if (snapLines.x !== undefined) { c.beginPath(); c.moveTo(snapLines.x, 0); c.lineTo(snapLines.x, H); c.stroke(); }
    if (snapLines.y !== undefined) { c.beginPath(); c.moveTo(0, snapLines.y); c.lineTo(W, snapLines.y); c.stroke(); }
    if (selPos && tool === "select" && !compare) {
      const b = bounds(selPos, W, H);
      const hs = 7 * px;
      c.save();
      c.translate(b.cx, b.cy);
      c.rotate(b.rot);
      c.strokeStyle = "#f5c518";
      c.lineWidth = 1.5 * px;
      c.setLineDash([6 * px, 4 * px]);
      c.strokeRect(-b.w / 2, -b.h / 2, b.w, b.h);
      c.setLineDash([]);
      c.fillStyle = "#f5c518";
      if (selPos.type !== "patch") {
        c.beginPath(); c.moveTo(0, -b.h / 2); c.lineTo(0, -b.h / 2 - 22 * px); c.stroke();
        c.beginPath(); c.arc(0, -b.h / 2 - 22 * px, hs, 0, Math.PI * 2); c.fill();
      }
      c.fillRect(b.w / 2 - hs, b.h / 2 - hs, hs * 2, hs * 2);
      c.restore();
      if (selPatch) {
        const s = sourceBox(selPatch);
        c.strokeStyle = "#38bdf8";
        c.lineWidth = 1.5 * px;
        c.setLineDash([4 * px, 4 * px]);
        c.strokeRect(s.x, s.y, s.w, s.h);
        c.setLineDash([]);
        c.beginPath(); c.moveTo(s.x + s.w / 2, s.y + s.h / 2); c.lineTo(b.cx, b.cy); c.stroke();
        c.fillStyle = "#38bdf8";
        c.font = `${12 * px}px sans-serif`;
        c.fillText("copied from here", s.x + 4 * px, s.y - 5 * px);
      }
    }
    if (selArea && tool === "select" && !compare) {
      const a = areaBox(selArea);
      const hs = 7 * px;
      c.strokeStyle = "#f472b6";
      c.lineWidth = 1.5 * px;
      c.setLineDash([6 * px, 4 * px]);
      if (selArea.ellipse) { c.beginPath(); c.ellipse(a.x + a.w / 2, a.y + a.h / 2, a.w / 2, a.h / 2, 0, 0, Math.PI * 2); c.stroke(); }
      c.strokeRect(a.x, a.y, a.w, a.h);
      c.setLineDash([]);
      c.fillStyle = "#f472b6";
      c.fillRect(a.x + a.w - hs, a.y + a.h - hs, hs * 2, hs * 2);
    }
    if (tool === "brush" && hover) {
      c.strokeStyle = brush.erase ? "#ff6b6b" : "#ffffff";
      c.lineWidth = px;
      c.beginPath(); c.arc(hover.x, hover.y, (brush.size * W) / 2, 0, Math.PI * 2); c.stroke();
    }
  }

  // ─── Pointer interaction ────────────────────────────────────────────────────
  function point(e: PointerEvent | WheelEvent) {
    const r = view.getBoundingClientRect();
    return { x: ((e.clientX - r.left) / r.width) * W, y: ((e.clientY - r.top) / r.height) * H };
  }
  /** Point in a layer's own (unrotated, centred) frame. */
  function local(p: { x: number; y: number }, l: Positioned) {
    const b = bounds(l, W, H);
    const dx = p.x - b.cx, dy = p.y - b.cy, a = -b.rot;
    return { x: dx * Math.cos(a) - dy * Math.sin(a), y: dx * Math.sin(a) + dy * Math.cos(a), b };
  }
  function inside(p: { x: number; y: number }, l: Positioned) {
    const q = local(p, l);
    return Math.abs(q.x) <= q.b.w / 2 && Math.abs(q.y) <= q.b.h / 2;
  }

  type Drag =
    | { mode: "move"; l: Positioned; dx: number; dy: number }
    | { mode: "scale"; l: Positioned; d0: number; start: number[] }
    | { mode: "rotate"; l: Positioned }
    | { mode: "source"; l: PatchLayer; ox: number; oy: number }
    | { mode: "area-move"; a: { x: number; y: number }; ox: number; oy: number }
    | { mode: "area-size"; a: { x: number; y: number; w: number; h: number } }
    | { mode: "paint"; stroke: Stroke };
  let drag: Drag | null = null;
  let gesture = 0; // each pointer press is one undo step

  const sizeOf = (l: Positioned) => (l.type === "image" ? [l.scale] : l.type === "text" ? [l.size] : [l.w, l.h]);

  function down(e: PointerEvent) {
    if (e.button !== 0 || compare) return;
    gesture++;
    const p = point(e);
    if (tool === "pick") {
      const src = pickFrom ?? rendered ?? baseCanvas;
      if (!src) return;
      const d = src.getContext("2d")!.getImageData(Math.floor(p.x), Math.floor(p.y), 1, 1).data;
      dispatch("pick", rgbHex(d[0], d[1], d[2]));
      return;
    }
    if (tool === "brush") {
      if (!selBrush) return;
      const stroke: Stroke = { color: brush.color, size: brush.size, erase: brush.erase, points: [p.x / W, p.y / H] };
      const layer = selBrush;
      ed.change(() => layer.strokes.push(stroke), `g${gesture}`);
      drag = { mode: "paint", stroke };
      view.setPointerCapture(e.pointerId);
      return;
    }
    const px = W / Math.max(1, view.clientWidth);
    if (selPos) {
      const q = local(p, selPos);
      const hs = 10 * px;
      if (selPos.type !== "patch" && Math.hypot(q.x, q.y + q.b.h / 2 + 22 * px) < hs * 1.4) {
        drag = { mode: "rotate", l: selPos };
      } else if (Math.abs(q.x - q.b.w / 2) < hs && Math.abs(q.y - q.b.h / 2) < hs) {
        drag = { mode: "scale", l: selPos, d0: Math.max(1, Math.hypot(q.x, q.y)), start: sizeOf(selPos) };
      }
      if (drag) { view.setPointerCapture(e.pointerId); return; }
    }
    if (selArea) {
      const a = areaBox(selArea);
      const hs = 10 * px;
      if (Math.abs(p.x - (a.x + a.w)) < hs && Math.abs(p.y - (a.y + a.h)) < hs) {
        drag = { mode: "area-size", a: selArea };
        view.setPointerCapture(e.pointerId);
        return;
      }
      if (p.x >= a.x && p.x <= a.x + a.w && p.y >= a.y && p.y <= a.y + a.h) {
        drag = { mode: "area-move", a: selArea, ox: p.x / W - selArea.x, oy: p.y / H - selArea.y };
        view.setPointerCapture(e.pointerId);
        return;
      }
    }
    if (selPatch) {
      const s = sourceBox(selPatch);
      if (p.x >= s.x && p.x <= s.x + s.w && p.y >= s.y && p.y <= s.y + s.h && !inside(p, selPatch)) {
        drag = { mode: "source", l: selPatch, ox: p.x / W - selPatch.dx, oy: p.y / H - selPatch.dy };
        view.setPointerCapture(e.pointerId);
        return;
      }
    }
    const hit = [...recipe.layers].reverse().find((l): l is Positioned => isPositioned(l) && !l.hidden && inside(p, l));
    dispatch("select", hit?.id ?? null);
    if (hit) {
      drag = { mode: "move", l: hit, dx: p.x / W - hit.x, dy: p.y / H - hit.y };
      view.setPointerCapture(e.pointerId);
    }
  }

  function move(e: PointerEvent) {
    const p = point(e);
    hover = p;
    if (!drag) return;
    const d = drag;
    if (d.mode === "paint") {
      ed.change(() => d.stroke.points.push(p.x / W, p.y / H), `g${gesture}`);
    } else if (d.mode === "move") {
      let x = p.x / W - d.dx, y = p.y / H - d.dy;
      const tol = (8 * W) / Math.max(1, view.clientWidth) / W;
      const xs = split ? [0.25, 0.75] : [0.5];
      const sx = xs.find((c) => Math.abs(x - c) < tol);
      const sy = Math.abs(y - 0.5) < tol * (W / H) ? 0.5 : undefined;
      if (sx !== undefined && !e.altKey) x = sx;
      if (sy !== undefined && !e.altKey) y = sy;
      snapLines = { x: sx !== undefined && !e.altKey ? sx * W : undefined, y: sy !== undefined && !e.altKey ? sy * H : undefined };
      ed.change(() => { d.l.x = Math.min(1.2, Math.max(-0.2, x)); d.l.y = Math.min(1.2, Math.max(-0.2, y)); }, `g${gesture}`);
    } else if (d.mode === "scale") {
      const q = local(p, d.l);
      const f = Math.max(0.05, Math.hypot(q.x, q.y) / d.d0);
      ed.change(() => {
        if (d.l.type === "image") d.l.scale = d.start[0] * f;
        else if (d.l.type === "text") d.l.size = d.start[0] * f;
        else { d.l.w = d.start[0] * f; d.l.h = d.start[1] * f; }
      }, `g${gesture}`);
    } else if (d.mode === "area-move") {
      ed.change(() => { d.a.x = p.x / W - d.ox; d.a.y = p.y / H - d.oy; }, `g${gesture}`);
    } else if (d.mode === "area-size") {
      // Bottom-right corner follows the pointer; the box grows from its top-left.
      const left = d.a.x * W - (d.a.w * W) / 2, top = d.a.y * H - (d.a.h * W) / 2;
      const w = Math.max(4, p.x - left), h = Math.max(4, p.y - top);
      ed.change(() => { d.a.w = w / W; d.a.h = h / W; d.a.x = (left + w / 2) / W; d.a.y = (top + h / 2) / H; }, `g${gesture}`);
    } else if (d.mode === "source") {
      ed.change(() => { d.l.dx = p.x / W - d.ox; d.l.dy = p.y / H - d.oy; }, `g${gesture}`);
    } else if (d.mode === "rotate") {
      const b = bounds(d.l, W, H);
      let a = (Math.atan2(p.y - b.cy, p.x - b.cx) * 180) / Math.PI + 90;
      if (a > 180) a -= 360;
      if (e.shiftKey) a = Math.round(a / 15) * 15;
      ed.change(() => (d.l.rotation = Math.round(a)), `g${gesture}`);
    }
  }

  function up() {
    drag = null;
    snapLines = {};
  }

  function wheel(e: WheelEvent) {
    if (e.ctrlKey) {
      e.preventDefault();
      zoom = Math.min(8, Math.max(0.25, zoom * (e.deltaY < 0 ? 1.12 : 1 / 1.12)));
      return;
    }
    if (tool === "brush") {
      e.preventDefault();
      brush = { ...brush, size: Math.min(0.2, Math.max(0.002, brush.size * (e.deltaY < 0 ? 1.12 : 1 / 1.12))) };
      return;
    }
    if (selPos && inside(point(e), selPos)) {
      e.preventDefault();
      const l = selPos, f = e.deltaY < 0 ? 1.05 : 1 / 1.05;
      ed.change(() => {
        if (l.type === "image") l.scale *= f;
        else if (l.type === "text") l.size *= f;
        else { l.w *= f; l.h *= f; }
      }, `wheel:${l.id}`);
    }
  }

  $: cursor = tool === "pick" ? "copy" : tool === "brush" ? "none" : drag?.mode === "move" ? "grabbing" : "default";
</script>

<div class="wrap" bind:this={wrap}>
  <div class="inner" style="width:{fitW * zoom}px">
    <canvas bind:this={view} style="cursor:{cursor}" on:pointerdown={down} on:pointermove={move} on:pointerup={up} on:pointercancel={up}
      on:pointerleave={() => (hover = null)} on:wheel={wheel}></canvas>
  </div>
</div>

<style>
  .wrap { flex: 1; overflow: auto; display: grid; place-items: center; padding: 12px; min-height: 0; }
  .inner { margin: auto; }
  canvas { width: 100%; display: block; border-radius: 6px; background: #1e1e1e; touch-action: none; }
</style>
