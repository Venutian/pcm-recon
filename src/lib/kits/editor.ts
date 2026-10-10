import { getContext, setContext } from "svelte";
import { writable } from "svelte/store";

/** What the kit editor shares with its panels and fields. */
export interface EditorCtx {
  /**
   * Mutate the recipe inside `fn`; the editor re-renders and records undo history.
   * Changes sharing a `gesture` key (one slider drag, one canvas drag) form a single undo step.
   */
  change(fn: () => void, gesture?: string): void;
  /** Ask the user to click a colour on the kit; `cb` receives `#rrggbb`. */
  pickColor(cb: (hex: string) => void): void;
  /** Team colours offered as swatches. */
  teamColors: string[];
}

const KEY = Symbol("kit-editor");
export const provideEditor = (ctx: EditorCtx) => setContext(KEY, ctx);
export const useEditor = () => getContext<EditorCtx>(KEY);

/** Colours used recently, most recent first. */
export const recentColors = writable<string[]>([]);
export function rememberColor(hex: string) {
  recentColors.update((r) => [hex, ...r.filter((c) => c !== hex)].slice(0, 8));
}

/** The browser's own eyedropper (picks from anywhere on screen), when available. */
export async function screenEyedropper(): Promise<string | null> {
  const ED = (window as unknown as { EyeDropper?: new () => { open(): Promise<{ sRGBHex: string }> } }).EyeDropper;
  if (!ED) return null;
  try { return (await new ED().open()).sRGBHex; } catch { return null; }
}

// ─── Undo history ─────────────────────────────────────────────────────────────
// Snapshots are JSON, with big strings (imported pictures, logos as data URLs) stored once
// and referenced, so a long session doesn't hold hundreds of copies of the same image.

const assets = new Map<string, string>();
const assetIds = new Map<string, string>();

export function snapshot(value: unknown): string {
  return JSON.stringify(value, (_k, v) => {
    if (typeof v === "string" && v.length > 4096) {
      let id = assetIds.get(v);
      if (!id) {
        id = `\u0000asset:${assets.size}`;
        assets.set(id, v);
        assetIds.set(v, id);
      }
      return id;
    }
    return v;
  });
}

export function restore<T>(snap: string): T {
  return JSON.parse(snap, (_k, v) => (typeof v === "string" && v.startsWith("\u0000asset:") ? assets.get(v) ?? v : v));
}

export class History {
  private undo: string[] = [];
  private redo: string[] = [];
  private last = 0;
  private lastKey: string | undefined;
  constructor(private limit = 120) {}

  /** Records `current` before a change. Consecutive changes with the same `key` merge into one step. */
  record(current: unknown, key?: string, mergeMs = 1500) {
    const now = Date.now();
    if (!key || key !== this.lastKey || now - this.last > mergeMs) {
      this.undo.push(snapshot(current));
      if (this.undo.length > this.limit) this.undo.shift();
      this.redo = [];
    }
    this.last = now;
    this.lastKey = key;
  }
  /** Ends the merge window, so the next change starts a new undo step. */
  cut() { this.last = 0; this.lastKey = undefined; }
  back<T>(current: T): T | null {
    const s = this.undo.pop();
    if (!s) return null;
    this.redo.push(snapshot(current));
    this.last = 0;
    return restore<T>(s);
  }
  forward<T>(current: T): T | null {
    const s = this.redo.pop();
    if (!s) return null;
    this.undo.push(snapshot(current));
    this.last = 0;
    return restore<T>(s);
  }
  get canUndo() { return this.undo.length > 0; }
  get canRedo() { return this.redo.length > 0; }
}
