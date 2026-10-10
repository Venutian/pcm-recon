// Kit design layers. A recipe is a base picture plus a stack of layers drawn bottom to top.
// Positions and sizes are fractions of the part's width/height, so a design moves between
// parts that share a template (every `maillot_*` is the same 1200×826 front|back layout).

export type Blend = GlobalCompositeOperation;
export type Panel = "both" | "front" | "back";

/** Part of the kit a layer is limited to: centre and size (fractions of width), soft edge. */
export interface Area { x: number; y: number; w: number; h: number; feather: number; ellipse: boolean }

interface Common {
  /** Limits the layer to this area of the kit. */
  area?: Area | null;
  id: string;
  name: string;
  hidden?: boolean;
  opacity: number;
  blend: Blend;
  /** Keep the layer on the kit, off the template's grey filler. */
  clip: boolean;
}

export interface AdjustLayer extends Common { type: "adjust"; hue: number; saturation: number; brightness: number; contrast: number; grayscale: number; invert: number; blur: number }
export interface ReplaceLayer extends Common {
  type: "replace"; from: string; to: string; tolerance: number; softness: number; keepShading: boolean;
  /** "colour": pixels close to `from`. "hue": every shade of `from`'s hue (dark, light, shaded). */
  match?: "colour" | "hue";
  /** How much of the original light and shade is kept (0–1) when keeping texture. */
  texture?: number;
}
export interface FlagLayer extends Common {
  type: "flag";
  flag: string;
  country: string;
  style: "fade" | "band" | "full" | "sleeves";
  /** Turn the flag 90°: vertical stripes become horizontal and the other way round. */
  rotate: boolean;
  flipX: boolean;
  fit: "stretch" | "cover" | "repeat";
  direction: "down" | "up" | "right" | "left";
  /** Fade: solid until `hold`, gone by `length` (fractions of the panel along `direction`). */
  hold: number;
  length: number;
  /** Band: centre height, thickness (fractions of height) and angle. */
  pos: number;
  size: number;
  angle: number;
  edgeColor: string;
  edgeWidth: number;
  panel: Panel;
}
export interface Crop { x: number; y: number; w: number; h: number }
export interface ImageLayer extends Common {
  type: "image"; src: string; x: number; y: number; scale: number; rotation: number; flipX: boolean; flipY: boolean; tint: string | null; shadow: number;
  /** "team": follows the team logo, so changing the logo updates this layer everywhere. */
  link: "team" | null;
  /** Makes the picture's background colour (read from its corners) transparent. */
  removeBg: boolean;
  bgTolerance: number;
  bgSoftness: number;
  /** Part of the picture to use, as fractions of it (after background removal the rest is trimmed). */
  crop: Crop | null;
}
/** Covers part of the kit (e.g. an old logo) with kit copied from nearby. */
export interface PatchLayer extends Common {
  type: "patch"; x: number; y: number; w: number; h: number; rotation: number;
  /** Where the cover is copied from, relative to the patch (fractions of width / height). */
  dx: number; dy: number;
  feather: number;
  ellipse: boolean;
}
export interface TextLayer extends Common {
  type: "text"; text: string; font: string; weight: number; italic: boolean; size: number; color: string;
  strokeColor: string; strokeWidth: number; spacing: number; x: number; y: number; rotation: number; shadow: number; align: "left" | "center" | "right";
}
export type ShapeKind = "rect" | "ellipse" | "triangle" | "chevron" | "star" | "diamond";
export interface ShapeLayer extends Common {
  type: "shape"; shape: ShapeKind; x: number; y: number; w: number; h: number; rotation: number;
  fill: string; fill2: string | null; gradientAngle: number; strokeColor: string; strokeWidth: number; radius: number;
}
export interface GradientLayer extends Common { type: "gradient"; kind: "linear" | "radial"; from: string; to: string; fromAlpha: number; toAlpha: number; angle: number; start: number; end: number; panel: Panel }
export type PatternKind = "stripes" | "pinstripes" | "dots" | "checks" | "zigzag";
export interface PatternLayer extends Common { type: "pattern"; kind: PatternKind; color: string; background: string | null; spacing: number; thickness: number; angle: number; panel: Panel }
export interface Stroke { color: string; size: number; erase: boolean; points: number[] }
export interface BrushLayer extends Common { type: "brush"; strokes: Stroke[] }

export type Layer = AdjustLayer | ReplaceLayer | FlagLayer | ImageLayer | TextLayer | ShapeLayer | GradientLayer | PatternLayer | BrushLayer | PatchLayer;
export type LayerType = Layer["type"];
export type Positioned = ImageLayer | TextLayer | ShapeLayer | PatchLayer;
export const isPositioned = (l: Layer): l is Positioned => l.type === "image" || l.type === "text" || l.type === "shape" || l.type === "patch";

export type Base =
  | { kind: "original" }
  /** Another part of the same kit (e.g. champion jerseys start from the main jersey), as edited. */
  | { kind: "part"; part: string }
  /** An imported picture, already scaled to the part's size. */
  | { kind: "image"; src: string };

export interface Recipe { v: 2; base: Base; layers: Layer[] }

export const emptyRecipe = (): Recipe => ({ v: 2, base: { kind: "original" }, layers: [] });
export const newId = () => Math.random().toString(36).slice(2, 10);

export const LAYER_INFO: Record<LayerType, { label: string; icon: string; hint: string }> = {
  image: { label: "Logo / image", icon: "open", hint: "PNG with transparency works best" },
  text: { label: "Text", icon: "contract", hint: "Names, numbers, slogans" },
  shape: { label: "Shape", icon: "gem", hint: "Panels, stars, chevrons" },
  flag: { label: "Flag", icon: "teams", hint: "Fade, band, sash or sleeves" },
  pattern: { label: "Pattern", icon: "rankings", hint: "Stripes, hoops, dots, checks" },
  gradient: { label: "Gradient", icon: "trend", hint: "Colour fades" },
  replace: { label: "Replace colour", icon: "reload", hint: "Swap one colour, keep the shading" },
  adjust: { label: "Adjust", icon: "eye", hint: "Hue, brightness, contrast…" },
  brush: { label: "Brush", icon: "plus", hint: "Paint and erase by hand" },
  patch: { label: "Patch / cover", icon: "close", hint: "Hide an old logo with nearby kit" },
};

export const BLENDS: { id: Blend; label: string }[] = [
  { id: "source-over", label: "Normal" },
  { id: "multiply", label: "Multiply" },
  { id: "screen", label: "Screen" },
  { id: "overlay", label: "Overlay" },
  { id: "soft-light", label: "Soft light" },
  { id: "hard-light", label: "Hard light" },
  { id: "darken", label: "Darken" },
  { id: "lighten", label: "Lighten" },
  { id: "color-dodge", label: "Colour dodge" },
  { id: "color-burn", label: "Colour burn" },
  { id: "difference", label: "Difference" },
  { id: "hue", label: "Hue" },
  { id: "saturation", label: "Saturation" },
  { id: "color", label: "Colour" },
  { id: "luminosity", label: "Luminosity" },
];

export const FONTS = ["Barlow Condensed", "Barlow", "Arial", "Arial Black", "Impact", "Segoe UI", "Verdana", "Tahoma", "Trebuchet MS", "Georgia", "Times New Roman", "Courier New", "Bahnschrift", "Franklin Gothic Medium"];

export interface LayerContext { split: boolean; color1: string; color2: string; flag: { flag: string; country: string } | null; teamName: string }

const common = (type: LayerType, name?: string, clip = true): Common => ({ id: newId(), name: name ?? LAYER_INFO[type].label, opacity: 1, blend: "source-over", clip });

export function defaultFlag(ctx: Pick<LayerContext, "flag">): FlagLayer {
  const f = ctx.flag ?? { flag: "it", country: "Italy" };
  return {
    ...common("flag", `${f.country} flag`), type: "flag", flag: f.flag, country: f.country, opacity: 0.85,
    style: "fade", rotate: false, flipX: false, fit: "stretch", direction: "down", hold: 0.12, length: 0.45,
    pos: 0.3, size: 0.09, angle: 0, edgeColor: "#d4af5a", edgeWidth: 0, panel: "both",
  };
}

export function newLayer(type: Exclude<LayerType, "image">, ctx: LayerContext): Layer {
  const cx = ctx.split ? 0.25 : 0.5, cy = ctx.split ? 0.32 : 0.5;
  switch (type) {
    case "text":
      return { ...common("text", "Text", false), type, text: ctx.teamName.toUpperCase() || "TEXT", font: "Barlow Condensed", weight: 700, italic: false, size: 0.05, color: "#ffffff", strokeColor: "#000000", strokeWidth: 0, spacing: 0.04, x: cx, y: cy, rotation: 0, shadow: 0, align: "center" };
    case "shape":
      return { ...common("shape", "Shape", false), type, shape: "rect", x: cx, y: cy, w: 0.2, h: 0.06, rotation: 0, fill: ctx.color1, fill2: null, gradientAngle: 90, strokeColor: "#ffffff", strokeWidth: 0, radius: 0 };
    case "flag":
      return defaultFlag(ctx);
    case "pattern":
      return { ...common("pattern", "Stripes"), type, kind: "stripes", color: "#ffffff", background: null, spacing: 0.04, thickness: 0.012, angle: 0, panel: "both", opacity: 0.35 };
    case "gradient":
      return { ...common("gradient"), type, kind: "linear", from: ctx.color1, to: ctx.color1, fromAlpha: 1, toAlpha: 0, angle: 180, start: 0, end: 0.6, panel: "both", opacity: 0.9 };
    case "replace":
      return { ...common("replace"), type, from: "#1d2b45", to: ctx.color1, tolerance: 22, softness: 18, keepShading: true, match: "colour", texture: 1 };
    case "adjust":
      return { ...common("adjust"), type, hue: 0, saturation: 100, brightness: 100, contrast: 100, grayscale: 0, invert: 0, blur: 0 };
    case "brush":
      return { ...common("brush", "Brush", false), type, strokes: [] };
    case "patch":
      return { ...common("patch", "Patch"), type, x: cx, y: cy, w: 0.14, h: 0.14, rotation: 0, dx: 0, dy: 0.22, feather: 0.2, ellipse: false };
  }
}

export const IMAGE_DEFAULTS = { link: null, removeBg: false, bgTolerance: 18, bgSoftness: 14, crop: null } as const;

export function imageLayer(src: string, name: string, split: boolean): ImageLayer {
  return { ...common("image", name, false), type: "image", src, x: split ? 0.25 : 0.5, y: split ? 0.3 : 0.45, scale: split ? 0.12 : 0.3, rotation: 0, flipX: false, flipY: false, tint: null, shadow: 0, ...IMAGE_DEFAULTS };
}

/** Deep copy with fresh ids, e.g. for duplicating or copying to another part. */
export function cloneLayer<T extends Layer>(l: T): T {
  return { ...structuredClone(l), id: newId() };
}

/** Upgrades recipes saved by v3.2.0 (recolour / fade / logo layers). */
export function migrate(r: unknown): Recipe {
  const rec = (r ?? {}) as { v?: number; base?: Base; layers?: Record<string, unknown>[] };
  if (rec.v === 2) {
    // Fill in options added after a recipe was saved.
    const r = rec as unknown as Recipe;
    for (const l of r.layers) if (l.type === "image") Object.assign(l, { ...IMAGE_DEFAULTS, ...l });
    return r;
  }
  const layers: Layer[] = [];
  for (const l of rec.layers ?? []) {
    if (l.type === "recolor") {
      layers.push({ ...common("adjust", "Recolour", true), type: "adjust", hue: Number(l.hue) || 0, saturation: Number(l.saturation ?? 100), brightness: Number(l.brightness ?? 100), contrast: 100, grayscale: 0, invert: 0, blur: 0 });
    } else if (l.type === "fade") {
      layers.push({ ...defaultFlag({ flag: { flag: String(l.flag), country: String(l.country) } }), hold: Number(l.hold), length: Number(l.length), opacity: Number(l.opacity) });
    } else if (l.type === "logo") {
      layers.push({ ...imageLayer(String(l.src), String(l.name ?? "Logo"), true), id: String(l.id ?? newId()), x: Number(l.x), y: Number(l.y), scale: Number(l.scale), rotation: Number(l.rotation) || 0, opacity: Number(l.opacity ?? 1) });
    }
  }
  return { v: 2, base: rec.base ?? { kind: "original" }, layers };
}
