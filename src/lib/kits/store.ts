import { writable, get } from "svelte/store";
import { kitImage, kitEdits, kitMetaGet, kitMetaSet, kitSaveEdit, type KitEdit, type KitStatus } from "../api";
import type { Recipe } from "./render";
import { loadImage, render, imageData, migrate, isSplit } from "./render";
import { emptyRecipe, imageLayer } from "./layers";

export const kitStatusStore = writable<KitStatus | null>(null);
export const kitEditsStore = writable<KitEdit[]>([]);
/** Bumped whenever a saved image changes, so thumbnails reload. */
export const thumbVersion = writable(0);

const cache = new Map<string, Promise<ImageData>>();

export function partImage(kit: string, part: string, edited: boolean): Promise<ImageData> {
  const key = `${kit}|${part}|${edited}`;
  let p = cache.get(key);
  if (!p) {
    p = kitImage(kit, part, edited);
    cache.set(key, p);
    p.catch(() => cache.delete(key));
  }
  return p;
}

export function invalidate(kit: string, part: string) {
  cache.delete(`${kit}|${part}|true`);
  thumbVersion.update((n) => n + 1);
}

export async function refreshEdits() {
  kitEditsStore.set(await kitEdits());
}

export const findEdit = (kit: string, part: string) => get(kitEditsStore).find((e) => e.kit === kit && e.part === part);

/** What the part looks like right now: the saved edit if there is one, else the original. */
export const currentImage = (kit: string, part: string) => partImage(kit, part, !!findEdit(kit, part));

/** The image a recipe starts from. */
export async function baseImage(kit: string, part: string, recipe: Recipe): Promise<ImageData> {
  const b = recipe.base;
  if (b.kind === "part") {
    // Start from the other part as edited (logos, recolour), minus its own flag fade,
    // so a champion jersey never stacks two flags.
    const saved = findEdit(kit, b.part)?.recipe;
    const src = saved ? migrate(saved) : null;
    if (!src || b.part === part) return partImage(kit, b.part, false);
    const srcBase = await baseImage(kit, b.part, src);
    return imageData(await render(srcBase, { ...src, layers: src.layers.filter((l) => l.type !== "flag") }, b.part));
  }
  if (b.kind === "image") {
    const img = await loadImage(b.src);
    const c = document.createElement("canvas");
    c.width = img.naturalWidth;
    c.height = img.naturalHeight;
    const ctx = c.getContext("2d")!;
    ctx.drawImage(img, 0, 0);
    return ctx.getImageData(0, 0, c.width, c.height);
  }
  return partImage(kit, part, false);
}

// ─── Team logo ────────────────────────────────────────────────────────────────
// One logo for the team. Image layers linked to it ("Follow team logo") switch together.

const TEAM_LOGO_KEY = "team-logo";
export const teamLogo = writable<string | null>(null);

export async function loadTeamLogo() {
  teamLogo.set(await kitMetaGet(TEAM_LOGO_KEY).catch(() => null));
}

const usesTeamLogo = (r: Recipe) => r.layers.some((l) => l.type === "image" && l.link === "team");

/** Re-renders and saves one part from its recipe. */
async function renderAndSave(kit: string, part: string, recipe: Recipe) {
  const img = imageData(await render(await baseImage(kit, part, recipe), recipe, part));
  await kitSaveEdit(kit, part, recipe, img);
  invalidate(kit, part);
}

/**
 * Sets the team logo and updates every saved part that follows it, plus parts built on top
 * of those (champion jerseys start from the jersey). Returns how many parts changed.
 */
export async function setTeamLogo(src: string | null, onProgress?: (msg: string) => void): Promise<number> {
  await kitMetaSet(TEAM_LOGO_KEY, src);
  teamLogo.set(src);
  if (!src) return 0;
  await refreshEdits();
  const edits = get(kitEditsStore).map((e) => ({ ...e, recipe: migrate(e.recipe) }));
  const changed = new Set<string>();
  // Parts drawn on their own first, then parts that start from another part.
  const order = [...edits].sort((a, b) => Number(a.recipe.base.kind === "part") - Number(b.recipe.base.kind === "part"));
  for (const e of order) {
    const linked = usesTeamLogo(e.recipe);
    const dependsOnChanged = e.recipe.base.kind === "part" && changed.has(`${e.kit}|${e.recipe.base.part}`);
    if (!linked && !dependsOnChanged) continue;
    for (const l of e.recipe.layers) if (l.type === "image" && l.link === "team") l.src = src;
    onProgress?.(`Updating ${e.part}…`);
    // The store must already hold the new recipes of parts this one starts from.
    kitEditsStore.update((all) => all.map((x) => (x.kit === e.kit && x.part === e.part ? { ...x, recipe: e.recipe } : x)));
    await renderAndSave(e.kit, e.part, e.recipe);
    changed.add(`${e.kit}|${e.part}`);
  }
  await refreshEdits();
  return changed.size;
}

/** Adds the team logo to a part (on top of what it has) and saves it. */
export async function placeTeamLogo(kit: string, part: string, spots: { x: number; y: number; scale: number }[]) {
  const src = get(teamLogo);
  if (!src) throw new Error("Choose a team logo first");
  const saved = findEdit(kit, part);
  const recipe = saved ? migrate(saved.recipe) : emptyRecipe();
  for (const s of spots) {
    recipe.layers.push({ ...imageLayer(src, "Team logo", isSplit(part)), ...s, link: "team", removeBg: true });
  }
  await renderAndSave(kit, part, recipe);
}
