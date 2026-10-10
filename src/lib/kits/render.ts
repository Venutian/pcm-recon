// Renders a kit part from its recipe: base picture, then each layer bottom to top.
// The same code draws the editor preview and the image that gets saved, so what you see
// is what ends up in the game.
import { migrate, type Recipe, type FlagLayer, type ImageLayer, type TextLayer, type ShapeLayer, type GradientLayer, type PatternLayer, type BrushLayer, type ReplaceLayer, type AdjustLayer, type PatchLayer, type Panel, type Positioned } from "./layers";

export type { Recipe, Layer } from "./layers";
export { emptyRecipe, migrate } from "./layers";

const FLAG_URLS = import.meta.glob("/node_modules/flag-icons/flags/4x3/*.svg", { eager: true, query: "?url", import: "default" }) as Record<string, string>;
export const flagUrl = (code: string) => FLAG_URLS[`/node_modules/flag-icons/flags/4x3/${code}.svg`];

const images = new Map<string, Promise<HTMLImageElement>>();
export function loadImage(src: string): Promise<HTMLImageElement> {
  let p = images.get(src);
  if (!p) {
    p = new Promise((ok, fail) => {
      const img = new Image();
      img.onload = () => ok(img);
      img.onerror = () => fail(new Error("Couldn't load image"));
      img.src = src;
    });
    images.set(src, p);
    p.catch(() => images.delete(src));
  }
  return p;
}

/** Jersey templates hold a front and a back panel side by side. */
export const isSplit = (part: string) => part.startsWith("maillot");

export function canvas(w: number, h: number) {
  const c = document.createElement("canvas");
  c.width = Math.max(1, Math.round(w));
  c.height = Math.max(1, Math.round(h));
  return c;
}
const ctx2d = (c: HTMLCanvasElement) => c.getContext("2d", { willReadFrequently: true })!;

/** Opaque where the template has kit, transparent on the neutral dark grey filler around it. */
export function kitMask(base: ImageData): HTMLCanvasElement {
  const m = new ImageData(base.width, base.height);
  const s = base.data, d = m.data;
  for (let i = 0; i < s.length; i += 4) {
    const filler = s[i] === s[i + 1] && s[i + 1] === s[i + 2] && s[i] < 0x40;
    d[i + 3] = filler ? 0 : 255;
  }
  const c = canvas(base.width, base.height);
  ctx2d(c).putImageData(m, 0, 0);
  return c;
}

/** [x, width] of the panels a layer covers. */
function panels(W: number, split: boolean, panel: Panel): [number, number][] {
  if (!split) return [[0, W]];
  const all: [number, number][] = [[0, W / 2], [W / 2, W / 2]];
  return panel === "front" ? [all[0]] : panel === "back" ? [all[1]] : all;
}

export function hexRgb(hex: string): [number, number, number] {
  const h = hex.replace("#", "");
  const n = parseInt(h.length === 3 ? h.split("").map((c) => c + c).join("") : h, 16) || 0;
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}
export const rgba = (hex: string, a: number) => `rgba(${hexRgb(hex).join(",")},${a})`;
export const rgbHex = (r: number, g: number, b: number) => "#" + [r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("");

/** Sizes of positioned layers from the last render, in pixels, for selection handles. */
export const measured = new Map<string, { w: number; h: number }>();

// ─── Layer painters ───────────────────────────────────────────────────────────

function drawFit(c: CanvasRenderingContext2D, src: CanvasImageSource & { width: number; height: number }, x: number, y: number, w: number, h: number, fit: "stretch" | "cover" | "repeat") {
  if (fit === "stretch") return c.drawImage(src, x, y, w, h);
  if (fit === "repeat") {
    // Tiles along the longer side at the flag's own proportions, centred.
    c.save();
    c.beginPath();
    c.rect(x, y, w, h);
    c.clip();
    if (w >= h) {
      const tw = (h * src.width) / src.height, n = Math.ceil(w / tw) + 1, x0 = x + w / 2 - (n * tw) / 2;
      for (let i = 0; i < n; i++) c.drawImage(src, x0 + i * tw, y, tw, h);
    } else {
      const th = (w * src.height) / src.width, n = Math.ceil(h / th) + 1, y0 = y + h / 2 - (n * th) / 2;
      for (let i = 0; i < n; i++) c.drawImage(src, x, y0 + i * th, w, th);
    }
    c.restore();
    return;
  }
  const s = Math.max(w / src.width, h / src.height);
  const sw = w / s, sh = h / s;
  c.drawImage(src, (src.width - sw) / 2, (src.height - sh) / 2, sw, sh, x, y, w, h);
}

async function flagSource(l: FlagLayer): Promise<HTMLCanvasElement | null> {
  const url = flagUrl(l.flag);
  if (!url) return null;
  const img = await loadImage(url);
  const w = 640, h = 480;
  const c = l.rotate ? canvas(h, w) : canvas(w, h);
  const x = ctx2d(c);
  x.translate(c.width / 2, c.height / 2);
  if (l.rotate) x.rotate(Math.PI / 2);
  if (l.flipX) x.scale(-1, 1);
  x.drawImage(img, -w / 2, -h / 2, w, h);
  return c;
}

/** Template sleeve areas on a 1200×826 jersey panel, as fractions of the panel. */
const SLEEVES: [number, number, number, number][] = [[0, 0, 0.235, 0.235], [0.75, 0, 0.235, 0.235]];

async function paintFlag(c: CanvasRenderingContext2D, l: FlagLayer, W: number, H: number, split: boolean) {
  const src = await flagSource(l);
  if (!src) return;
  for (const [px, pw] of panels(W, split, l.panel)) {
    c.save();
    c.beginPath();
    c.rect(px, 0, pw, H);
    c.clip();
    if (l.style === "full") {
      drawFit(c, src, px, 0, pw, H, l.fit);
    } else if (l.style === "sleeves") {
      for (const [fx, fy, fw, fh] of split ? SLEEVES : [[0, 0, 1, 0.25] as [number, number, number, number]]) drawFit(c, src, px + fx * pw, fy * H, fw * pw, fh * H, l.fit);
    } else if (l.style === "band") {
      c.translate(px + pw / 2, l.pos * H);
      c.rotate((l.angle * Math.PI) / 180);
      const len = Math.hypot(pw, H) * 1.5, th = Math.max(1, l.size * H);
      drawFit(c, src, -len / 2, -th / 2, len, th, l.fit);
      if (l.edgeWidth > 0) {
        const e = l.edgeWidth * H;
        c.fillStyle = l.edgeColor;
        c.fillRect(-len / 2, -th / 2 - e, len, e);
        c.fillRect(-len / 2, th / 2, len, e);
      }
    } else {
      // Fade from one edge of the panel.
      const vertical = l.direction === "down" || l.direction === "up";
      const span = vertical ? H : pw;
      const len = Math.max(1, l.length * span);
      const f = canvas(W, H);
      const fc = ctx2d(f);
      const rect: [number, number, number, number] =
        l.direction === "down" ? [px, 0, pw, len] : l.direction === "up" ? [px, H - len, pw, len] : l.direction === "right" ? [px, 0, len, H] : [px + pw - len, 0, len, H];
      drawFit(fc, src, ...rect, l.fit);
      const [x0, y0, x1, y1] =
        l.direction === "down" ? [0, 0, 0, len] : l.direction === "up" ? [0, H, 0, H - len] : l.direction === "right" ? [px, 0, px + len, 0] : [px + pw, 0, px + pw - len, 0];
      const g = fc.createLinearGradient(x0, y0, x1, y1);
      g.addColorStop(0, "#000");
      g.addColorStop(Math.min(0.999, Math.max(0, l.hold / Math.max(l.length, 0.001))), "#000");
      g.addColorStop(1, "rgba(0,0,0,0)");
      fc.globalCompositeOperation = "destination-in";
      fc.fillStyle = g;
      fc.fillRect(...rect);
      c.drawImage(f, 0, 0);
    }
    c.restore();
  }
}

const tinted = new Map<string, HTMLCanvasElement>();
function tint(img: HTMLCanvasElement, color: string, id: string): HTMLCanvasElement {
  const key = `${id}|${img.width}x${img.height}|${color}`;
  let t = tinted.get(key);
  if (!t || t.width !== img.width) {
    t = canvas(img.width, img.height);
    const x = ctx2d(t);
    x.drawImage(img, 0, 0);
    x.globalCompositeOperation = "source-in";
    x.fillStyle = color;
    x.fillRect(0, 0, t.width, t.height);
    if (tinted.size > 40) tinted.clear();
    tinted.set(key, t);
  }
  return t;
}

function shadowOn(c: CanvasRenderingContext2D, amount: number, W: number) {
  if (amount <= 0) return;
  c.shadowColor = `rgba(0,0,0,${Math.min(0.85, 0.35 + amount * 0.5)})`;
  c.shadowBlur = amount * W * 0.012;
  c.shadowOffsetX = c.shadowOffsetY = amount * W * 0.004;
}

/** The picture as drawn: cropped, background removed and trimmed to what's left. Cached. */
const prepared = new Map<string, HTMLCanvasElement>();
export async function prepareImage(l: Pick<ImageLayer, "src" | "removeBg" | "bgTolerance" | "bgSoftness" | "crop">): Promise<HTMLCanvasElement | null> {
  const key = `${l.src.length}:${l.src.slice(-48)}|${l.removeBg}|${l.bgTolerance}|${l.bgSoftness}|${JSON.stringify(l.crop)}`;
  const hit = prepared.get(key);
  if (hit) return hit;
  const img = await loadImage(l.src).catch(() => null);
  if (!img) return null;
  const iw = img.naturalWidth, ih = img.naturalHeight;
  const cr = l.crop ?? { x: 0, y: 0, w: 1, h: 1 };
  const sx = Math.round(cr.x * iw), sy = Math.round(cr.y * ih);
  const sw = Math.max(1, Math.round(cr.w * iw)), sh = Math.max(1, Math.round(cr.h * ih));
  let c = canvas(sw, sh);
  const x = ctx2d(c);
  x.drawImage(img, sx, sy, sw, sh, 0, 0, sw, sh);
  if (l.removeBg) {
    // Key colour from the whole picture's corners (before cropping).
    const full = canvas(iw, ih);
    const fx = ctx2d(full);
    fx.drawImage(img, 0, 0);
    const corners = [[0, 0], [iw - 1, 0], [0, ih - 1], [iw - 1, ih - 1]].map(([px, py]) => fx.getImageData(px, py, 1, 1).data);
    const key3 = [0, 1, 2].map((i) => corners.reduce((a, d) => a + d[i], 0) / 4);
    const id = x.getImageData(0, 0, sw, sh);
    const d = id.data;
    const tol = l.bgTolerance * 2.55, soft = Math.max(1, l.bgSoftness * 2.55);
    let minX = sw, minY = sh, maxX = -1, maxY = -1;
    for (let i = 0, p = 0; i < d.length; i += 4, p++) {
      const dist = Math.hypot(d[i] - key3[0], d[i + 1] - key3[1], d[i + 2] - key3[2]) / Math.sqrt(3);
      const a = dist <= tol ? 0 : dist >= tol + soft ? 1 : (dist - tol) / soft;
      if (a > 0 && a < 1) {
        // Undo the blend with the background so edges don't keep a dark (or light) fringe.
        for (let k = 0; k < 3; k++) d[i + k] = Math.min(255, Math.max(0, (d[i + k] - key3[k] * (1 - a)) / a));
      }
      d[i + 3] = Math.round(d[i + 3] * a);
      if (d[i + 3] > 8) {
        const px = p % sw, py = (p / sw) | 0;
        if (px < minX) minX = px; if (px > maxX) maxX = px;
        if (py < minY) minY = py; if (py > maxY) maxY = py;
      }
    }
    x.putImageData(id, 0, 0);
    if (maxX >= minX && maxY >= minY) {
      const t = canvas(maxX - minX + 1, maxY - minY + 1);
      ctx2d(t).drawImage(c, minX, minY, t.width, t.height, 0, 0, t.width, t.height);
      c = t;
    }
  }
  if (prepared.size > 30) prepared.clear();
  prepared.set(key, c);
  return c;
}

async function paintImage(c: CanvasRenderingContext2D, l: ImageLayer, W: number) {
  const img = await prepareImage(l);
  if (!img) return;
  const w = l.scale * W;
  const h = (w * img.height) / Math.max(1, img.width);
  measured.set(l.id, { w, h });
  c.save();
  c.translate(l.x * W, l.y * c.canvas.height);
  c.rotate((l.rotation * Math.PI) / 180);
  c.scale(l.flipX ? -1 : 1, l.flipY ? -1 : 1);
  shadowOn(c, l.shadow, W);
  c.drawImage(l.tint ? tint(img, l.tint, l.id) : img, -w / 2, -h / 2, w, h);
  c.restore();
}

export const fontString = (l: TextLayer, H: number) => `${l.italic ? "italic " : ""}${l.weight} ${Math.max(1, l.size * H)}px "${l.font}"`;

async function paintText(c: CanvasRenderingContext2D, l: TextLayer, W: number, H: number) {
  const font = fontString(l, H);
  try { await document.fonts.load(font, l.text); } catch { /* system font */ }
  const px = l.size * H;
  const lines = l.text.split("\n");
  c.save();
  c.font = font;
  (c as CanvasRenderingContext2D & { letterSpacing: string }).letterSpacing = `${l.spacing * px}px`;
  c.textAlign = l.align;
  c.textBaseline = "middle";
  const widths = lines.map((t) => c.measureText(t).width);
  const lh = px * 1.12;
  const bw = Math.max(1, ...widths), bh = lh * lines.length;
  measured.set(l.id, { w: bw, h: bh });
  c.translate(l.x * W, l.y * H);
  c.rotate((l.rotation * Math.PI) / 180);
  // Lines are laid out around the anchor; the box is centred on it.
  const ax = l.align === "left" ? -bw / 2 : l.align === "right" ? bw / 2 : 0;
  lines.forEach((t, i) => {
    const y = -bh / 2 + lh * (i + 0.5);
    if (l.strokeWidth > 0) {
      c.lineJoin = "round";
      c.lineWidth = l.strokeWidth * px;
      c.strokeStyle = l.strokeColor;
      c.save();
      shadowOn(c, l.shadow, W);
      c.strokeText(t, ax, y);
      c.restore();
    } else {
      shadowOn(c, l.shadow, W);
    }
    c.fillStyle = l.color;
    c.fillText(t, ax, y);
  });
  c.restore();
}

function shapePath(c: CanvasRenderingContext2D, l: ShapeLayer, w: number, h: number) {
  c.beginPath();
  const hw = w / 2, hh = h / 2;
  switch (l.shape) {
    case "rect":
      c.roundRect(-hw, -hh, w, h, Math.min(hw, hh) * l.radius);
      break;
    case "ellipse":
      c.ellipse(0, 0, hw, hh, 0, 0, Math.PI * 2);
      break;
    case "triangle":
      c.moveTo(0, -hh); c.lineTo(hw, hh); c.lineTo(-hw, hh); c.closePath();
      break;
    case "diamond":
      c.moveTo(0, -hh); c.lineTo(hw, 0); c.lineTo(0, hh); c.lineTo(-hw, 0); c.closePath();
      break;
    case "chevron": {
      const t = Math.min(h, w) * 0.35;
      c.moveTo(-hw, -hh); c.lineTo(0, hh - t); c.lineTo(hw, -hh); c.lineTo(hw, -hh + t); c.lineTo(0, hh); c.lineTo(-hw, -hh + t); c.closePath();
      break;
    }
    case "star":
      for (let i = 0; i < 10; i++) {
        const a = -Math.PI / 2 + (i * Math.PI) / 5, r = i % 2 ? 0.45 : 1;
        c[i ? "lineTo" : "moveTo"](Math.cos(a) * hw * r, Math.sin(a) * hh * r);
      }
      c.closePath();
      break;
  }
}

function paintShape(c: CanvasRenderingContext2D, l: ShapeLayer, W: number, H: number) {
  const w = l.w * W, h = l.h * W;
  measured.set(l.id, { w, h });
  c.save();
  c.translate(l.x * W, l.y * H);
  c.rotate((l.rotation * Math.PI) / 180);
  shapePath(c, l, w, h);
  if (l.fill2) {
    const a = (l.gradientAngle * Math.PI) / 180, r = Math.hypot(w, h) / 2;
    const g = c.createLinearGradient(-Math.sin(a) * r, Math.cos(a) * r, Math.sin(a) * r, -Math.cos(a) * r);
    g.addColorStop(0, l.fill);
    g.addColorStop(1, l.fill2);
    c.fillStyle = g;
  } else {
    c.fillStyle = l.fill;
  }
  c.fill();
  if (l.strokeWidth > 0) {
    c.lineWidth = l.strokeWidth * W;
    c.strokeStyle = l.strokeColor;
    c.lineJoin = "round";
    c.stroke();
  }
  c.restore();
}

function paintGradient(c: CanvasRenderingContext2D, l: GradientLayer, W: number, H: number, split: boolean) {
  for (const [px, pw] of panels(W, split, l.panel)) {
    const cx = px + pw / 2, cy = H / 2;
    let g: CanvasGradient;
    if (l.kind === "radial") {
      g = c.createRadialGradient(cx, cy, 0, cx, cy, Math.hypot(pw, H) / 2);
    } else {
      // Angle 0 runs left→right, 90 top→bottom… matching CSS-style "towards" angles minus 90.
      const a = ((l.angle - 90) * Math.PI) / 180;
      const half = (Math.abs(pw * Math.cos(a)) + Math.abs(H * Math.sin(a))) / 2;
      g = c.createLinearGradient(cx - Math.cos(a) * half, cy - Math.sin(a) * half, cx + Math.cos(a) * half, cy + Math.sin(a) * half);
    }
    const s = Math.min(l.start, l.end - 0.001), e = Math.max(l.end, s + 0.001);
    g.addColorStop(Math.max(0, s), rgba(l.from, l.fromAlpha));
    g.addColorStop(Math.min(1, e), rgba(l.to, l.toAlpha));
    c.fillStyle = g;
    c.fillRect(px, 0, pw, H);
  }
}

function paintPattern(c: CanvasRenderingContext2D, l: PatternLayer, W: number, H: number, split: boolean) {
  const step = Math.max(2, l.spacing * W), th = Math.max(0.5, l.thickness * W);
  for (const [px, pw] of panels(W, split, l.panel)) {
    c.save();
    c.beginPath();
    c.rect(px, 0, pw, H);
    c.clip();
    if (l.background) { c.fillStyle = l.background; c.fillRect(px, 0, pw, H); }
    c.translate(px + pw / 2, H / 2);
    c.rotate((l.angle * Math.PI) / 180);
    const D = Math.hypot(pw, H);
    c.fillStyle = l.color;
    c.strokeStyle = l.color;
    if (l.kind === "stripes" || l.kind === "pinstripes") {
      const t = l.kind === "pinstripes" ? Math.max(0.5, th / 4) : th;
      for (let x = -D; x < D; x += step) c.fillRect(x, -D, t, 2 * D);
    } else if (l.kind === "dots") {
      for (let y = -D; y < D; y += step) for (let x = -D; x < D; x += step) {
        c.beginPath();
        c.arc(x + ((Math.round(y / step) % 2) * step) / 2, y, th / 2, 0, Math.PI * 2);
        c.fill();
      }
    } else if (l.kind === "checks") {
      for (let y = -D, j = 0; y < D; y += step, j++) for (let x = -D, i = 0; x < D; x += step, i++) if ((i + j) % 2 === 0) c.fillRect(x, y, step, step);
    } else {
      c.lineWidth = th;
      c.lineJoin = "miter";
      for (let y = -D; y < D; y += step) {
        c.beginPath();
        for (let x = -D, i = 0; x < D; x += step / 2, i++) c.lineTo(x, y + (i % 2 ? step / 3 : -step / 3));
        c.stroke();
      }
    }
    c.restore();
  }
}

/** A full-size mask that is opaque inside `a`, with soft edges. */
function areaMask(a: { x: number; y: number; w: number; h: number; feather: number; ellipse: boolean }, W: number, H: number): HTMLCanvasElement {
  const m = canvas(W, H);
  const mc = ctx2d(m);
  const w = a.w * W, h = a.h * W, cx = a.x * W, cy = a.y * H;
  const f = Math.max(0, a.feather) * Math.min(w, h) * 0.5;
  if (f > 0.5) mc.filter = `blur(${f / 2}px)`;
  mc.fillStyle = "#000";
  mc.beginPath();
  if (a.ellipse) mc.ellipse(cx, cy, Math.max(1, w / 2 - f), Math.max(1, h / 2 - f), 0, 0, Math.PI * 2);
  else mc.rect(cx - w / 2 + f, cy - h / 2 + f, Math.max(1, w - 2 * f), Math.max(1, h - 2 * f));
  mc.fill();
  return m;
}

/** Copies kit from (x+dx, y+dy) over the patch area, with soft edges. */
function paintPatch(c: CanvasRenderingContext2D, l: PatchLayer, src: HTMLCanvasElement, W: number, H: number) {
  const w = Math.max(2, l.w * W), h = Math.max(2, l.h * W);
  measured.set(l.id, { w, h });
  const tx = l.x * W - w / 2, ty = l.y * H - h / 2;
  const p = canvas(w, h);
  const pc = ctx2d(p);
  pc.drawImage(src, tx + l.dx * W, ty + l.dy * H, w, h, 0, 0, w, h);
  const f = Math.max(0, l.feather) * Math.min(w, h) * 0.5;
  const m = canvas(w, h);
  const mc = ctx2d(m);
  if (f > 0.5) mc.filter = `blur(${f / 2}px)`;
  mc.fillStyle = "#000";
  mc.beginPath();
  if (l.ellipse) mc.ellipse(w / 2, h / 2, Math.max(1, w / 2 - f), Math.max(1, h / 2 - f), 0, 0, Math.PI * 2);
  else mc.rect(f, f, Math.max(1, w - 2 * f), Math.max(1, h - 2 * f));
  mc.fill();
  pc.globalCompositeOperation = "destination-in";
  pc.drawImage(m, 0, 0);
  c.drawImage(p, tx, ty);
}

function paintBrush(c: CanvasRenderingContext2D, l: BrushLayer, W: number, H: number) {
  c.save();
  c.lineCap = "round";
  c.lineJoin = "round";
  for (const s of l.strokes) {
    c.globalCompositeOperation = s.erase ? "destination-out" : "source-over";
    c.strokeStyle = s.color;
    c.fillStyle = s.color;
    c.lineWidth = s.size * W;
    if (s.points.length <= 2) {
      c.beginPath();
      c.arc(s.points[0] * W, s.points[1] * H, (s.size * W) / 2, 0, Math.PI * 2);
      c.fill();
      continue;
    }
    c.beginPath();
    for (let i = 0; i < s.points.length; i += 2) c[i ? "lineTo" : "moveTo"](s.points[i] * W, s.points[i + 1] * H);
    c.stroke();
  }
  c.restore();
}

/** Swaps pixels near `from` for `to`; with `keepShading` each pixel keeps its offset from `from`. */
export function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b), l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min, s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  const h = max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [h * 60, s, l];
}
export function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const k = (n: number) => (n + h / 30) % 12, a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1));
  return [f(0) * 255, f(8) * 255, f(4) * 255];
}
const clamp01 = (v: number) => Math.min(1, Math.max(0, v));

/**
 * Swaps pixels matching `from` for `to`. Keeping texture moves each pixel's hue to `to` and
 * shifts its lightness by the same step as `from` → `to`, so folds and shading survive.
 */
function replaceColour(src: ImageData, l: ReplaceLayer): ImageData {
  const out = new ImageData(new Uint8ClampedArray(src.data), src.width, src.height);
  const d = out.data;
  const [fr, fg, fb] = hexRgb(l.from), [tr, tg, tb] = hexRgb(l.to);
  const [fh, fs, fl] = rgbToHsl(fr, fg, fb), [th, ts, tl] = rgbToHsl(tr, tg, tb);
  const byHue = l.match === "hue";
  const texture = l.texture ?? 1;
  // Colour distances run 0–255 and hue distances 0–180°; the 0–100 sliders cover the useful part.
  const tol = byHue ? l.tolerance * 0.6 : l.tolerance * 1.2;
  const soft = Math.max(1, byHue ? l.softness * 0.6 : l.softness * 1.2);
  for (let i = 0; i < d.length; i += 4) {
    const r = d[i], g = d[i + 1], b = d[i + 2];
    let w: number;
    let ph = 0, ps = 0, pl = 0;
    if (byHue) {
      [ph, ps, pl] = rgbToHsl(r, g, b);
      if (ps < 0.08 || pl < 0.03 || pl > 0.97) continue; // greys have no reliable hue
      const dh = Math.abs(ph - fh), dist = Math.min(dh, 360 - dh);
      if (dist >= tol + soft) continue;
      w = (dist <= tol ? 1 : 1 - (dist - tol) / soft) * Math.min(1, (ps - 0.08) / 0.12);
    } else {
      const dr = r - fr, dg = g - fg, db = b - fb;
      const dist = Math.sqrt(2 * dr * dr + 4 * dg * dg + 3 * db * db) / 3;
      if (dist >= tol + soft) continue;
      w = dist <= tol ? 1 : 1 - (dist - tol) / soft;
      if (l.keepShading) [ph, ps, pl] = rgbToHsl(r, g, b);
    }
    let nr = tr, ng = tg, nb = tb;
    if (l.keepShading) {
      const ns = clamp01(fs > 0.02 ? ts * (ps / fs) : ts);
      const nl = clamp01(tl + (pl - fl) * texture);
      [nr, ng, nb] = hslToRgb(th, ns, nl);
    }
    d[i] += (nr - r) * w;
    d[i + 1] += (ng - g) * w;
    d[i + 2] += (nb - b) * w;
  }
  return out;
}

function adjustFilter(l: AdjustLayer) {
  return `hue-rotate(${l.hue}deg) saturate(${l.saturation}%) brightness(${l.brightness}%) contrast(${l.contrast}%) grayscale(${l.grayscale}%) invert(${l.invert}%)${l.blur > 0 ? ` blur(${l.blur}px)` : ""}`;
}

// ─── Compositing ──────────────────────────────────────────────────────────────

export interface RenderOptions {
  /** Draw only layers before this index (for "before" comparisons). */
  upTo?: number;
}

export async function render(base: ImageData, recipeIn: Recipe, part: string, opts: RenderOptions = {}): Promise<HTMLCanvasElement> {
  const recipe = migrate(recipeIn);
  const { width: W, height: H } = base;
  const out = canvas(W, H);
  const ctx = ctx2d(out);
  ctx.putImageData(base, 0, 0);
  const split = isSplit(part);
  let mask: HTMLCanvasElement | null = null;
  const getMask = () => (mask ??= kitMask(base));

  const layers = recipe.layers.slice(0, opts.upTo ?? recipe.layers.length);
  for (const l of layers) {
    if (l.hidden || l.opacity <= 0) continue;
    const t = canvas(W, H);
    const tc = ctx2d(t);
    switch (l.type) {
      case "adjust":
        tc.filter = adjustFilter(l);
        tc.drawImage(out, 0, 0);
        tc.filter = "none";
        break;
      case "replace":
        tc.putImageData(replaceColour(ctx.getImageData(0, 0, W, H), l), 0, 0);
        break;
      case "flag": await paintFlag(tc, l, W, H, split); break;
      case "image": await paintImage(tc, l, W); break;
      case "text": await paintText(tc, l, W, H); break;
      case "shape": paintShape(tc, l, W, H); break;
      case "gradient": paintGradient(tc, l, W, H, split); break;
      case "pattern": paintPattern(tc, l, W, H, split); break;
      case "brush": paintBrush(tc, l, W, H); break;
      case "patch": paintPatch(tc, l, out, W, H); break;
    }
    if (l.area) {
      tc.globalCompositeOperation = "destination-in";
      tc.drawImage(areaMask(l.area, W, H), 0, 0);
      tc.globalCompositeOperation = "source-over";
    }
    if (l.clip) {
      tc.globalCompositeOperation = "destination-in";
      tc.drawImage(getMask(), 0, 0);
    }
    ctx.save();
    ctx.globalAlpha = l.opacity;
    ctx.globalCompositeOperation = l.blend;
    ctx.drawImage(t, 0, 0);
    ctx.restore();
  }
  return out;
}

/** Centre, size (px) and rotation of a positioned layer, from its last render. */
export function bounds(l: Positioned, W: number, H: number) {
  const m = measured.get(l.id) ?? { w: (l.type === "shape" || l.type === "patch" ? l.w : 0.1) * W, h: (l.type === "shape" || l.type === "patch" ? l.h : 0.05) * W };
  return { cx: l.x * W, cy: l.y * H, w: m.w, h: m.h, rot: (l.rotation * Math.PI) / 180 };
}

export function imageData(c: HTMLCanvasElement): ImageData {
  return ctx2d(c).getImageData(0, 0, c.width, c.height);
}

export function toCanvas(img: ImageData): HTMLCanvasElement {
  const c = canvas(img.width, img.height);
  ctx2d(c).putImageData(img, 0, 0);
  return c;
}

/** Draws `src` stretched to `w`×`h` and returns it as a PNG data URL. */
export async function fitToSize(src: string, w: number, h: number): Promise<{ src: string; aspectOff: boolean }> {
  const img = await loadImage(src);
  const c = canvas(w, h);
  ctx2d(c).drawImage(img, 0, 0, w, h);
  const aspectOff = Math.abs(img.naturalWidth / img.naturalHeight - w / h) > 0.02;
  return { src: c.toDataURL("image/png"), aspectOff };
}

export function bytesToDataUrl(bytes: ArrayBuffer, name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() ?? "png";
  const mime = ext === "jpg" || ext === "jpeg" ? "image/jpeg" : ext === "svg" ? "image/svg+xml" : ext === "webp" ? "image/webp" : "image/png";
  let bin = "";
  const u8 = new Uint8Array(bytes);
  for (let i = 0; i < u8.length; i += 0x8000) bin += String.fromCharCode(...u8.subarray(i, i + 0x8000));
  return `data:${mime};base64,${btoa(bin)}`;
}

export async function canvasPng(c: HTMLCanvasElement): Promise<Uint8Array> {
  const blob: Blob = await new Promise((ok, fail) => c.toBlob((b) => (b ? ok(b) : fail(new Error("PNG encoding failed"))), "image/png"));
  return new Uint8Array(await blob.arrayBuffer());
}
