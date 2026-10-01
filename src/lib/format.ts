import type { Cyclist, StatKey } from "./types";
import { TERRAINS } from "./types";

const eur0 = new Intl.NumberFormat("en-GB", { maximumFractionDigits: 0 });

/** Full euro amount: €1,234,567 / −€368,630 */
export function eur(v: number): string {
  const sign = v < 0 ? "−" : "";
  return `${sign}€${eur0.format(Math.abs(Math.round(v)))}`;
}

/** Compact euro amount: €2.27M, −€369k, €850 */
export function eurShort(v: number): string {
  const sign = v < 0 ? "−" : "";
  const a = Math.abs(v);
  if (a >= 1_000_000) return `${sign}€${(a / 1_000_000).toFixed(a >= 10_000_000 ? 1 : 2)}M`;
  if (a >= 10_000) return `${sign}€${Math.round(a / 1000)}k`;
  if (a >= 1_000) return `${sign}€${(a / 1000).toFixed(1)}k`;
  return `${sign}€${Math.round(a)}`;
}

export function signedEur(v: number): string {
  return (v > 0 ? "+" : "") + eurShort(v);
}

/** Parses "1.5m", "250k", "-368,630", "€2 000 000" into euros. */
export function parseMoney(input: string): number | null {
  const s = input.trim().toLowerCase().replace(/[€\s,_]/g, "").replace(/−/g, "-");
  const m = s.match(/^([+-]?\d+(?:\.\d+)?)([km]?)$/);
  if (!m) return null;
  const n = parseFloat(m[1]) * (m[2] === "m" ? 1_000_000 : m[2] === "k" ? 1_000 : 1);
  return Number.isFinite(n) ? Math.round(n) : null;
}

export function stars(v: number): string {
  if (!v) return "–";
  return v.toFixed(1).replace(/\.0$/, "");
}

export function fmtDate(iso: string): string {
  if (!iso) return "–";
  const d = new Date(iso + "T00:00:00");
  return d.toLocaleDateString("en-GB", { day: "numeric", month: "short", year: "numeric" });
}

export function timeAgo(ms: number): string {
  const s = (Date.now() - ms) / 1000;
  if (s < 60) return "just now";
  if (s < 3600) return `${Math.floor(s / 60)} min ago`;
  if (s < 86400) return `${Math.floor(s / 3600)} h ago`;
  const d = new Date(ms);
  return d.toLocaleString("en-GB", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
}

export function stat(c: Cyclist, key: string): number {
  return (c as unknown as Record<string, number>)[key] ?? 0;
}

export function terrainScore(c: Cyclist, stats: StatKey[], ceiling = false): number {
  const vals = stats.map((k) => stat(c, ceiling ? `${k}_p` : k));
  return vals.reduce((a, b) => a + b, 0) / vals.length;
}

export function terrainProfile(c: Cyclist, ceiling = false): number[] {
  return TERRAINS.map((t) => terrainScore(c, t.stats, ceiling));
}

/** CA bib colour band: the bib is white; the stripe under the number signals level. */
export function caBand(ca: number): string {
  if (ca >= 80) return "#f5c518";
  if (ca >= 75) return "#3dbe6e";
  if (ca >= 70) return "#5aa8e6";
  if (ca >= 65) return "#8692a3";
  return "#4f5a6a";
}

export function statColor(v: number): string {
  if (v >= 80) return "#f5c518";
  if (v >= 75) return "#3dbe6e";
  if (v >= 70) return "#5aa8e6";
  if (v >= 62) return "#8692a3";
  return "#4f5a6a";
}

/** Gap between true potential and what the scout report shows. Positive = underrated. */
export function gemGap(c: Cyclist): number {
  if (!c.scout_estimate) return 0;
  return Math.round((c.potential - c.scout_estimate) * 10) / 10;
}

export function matches(c: Cyclist, q: string): boolean {
  if (!q) return true;
  const s = q.toLowerCase();
  return (
    c.name.toLowerCase().includes(s) ||
    c.team.toLowerCase().includes(s) ||
    c.nationality.toLowerCase().includes(s) ||
    c.rider_type.toLowerCase().includes(s)
  );
}

export function teamColor(color: string | undefined, id: number): string {
  if (color && color !== "#000000") return color;
  const hue = Math.abs((id * 137) % 360);
  return `hsl(${hue} 55% 55%)`;
}

export function clamp(v: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, v));
}
