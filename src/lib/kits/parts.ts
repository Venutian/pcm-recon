// Friendly names for kit parts (the game's own names are French-ish file suffixes).

const NAMES: Record<string, string> = {
  maillot: "Jersey",
  maillot_tour: "Jersey, white alternate",
  maillot_world: "World champion jersey",
  maillot_world_itt: "World TT champion jersey",
  maillot_euro: "European champion jersey",
  maillot_euro_itt: "European TT champion jersey",
  minimaillot: "Small jersey (menus)",
  minimaillot_tour: "Small jersey, alternate (menus)",
  leggings: "Shorts",
  corsair: "Bib shorts, 3/4",
  gloves: "Gloves",
  gloves_tour: "Gloves, alternate",
  socks: "Socks",
  socks_tour: "Socks, alternate",
  shoes: "Shoes",
  bottle: "Bottle",
  handles: "Handlebar tape",
  transfert_av: "Decal, front",
  transfert_ar: "Decal, back",
};

/** `maillot_ita` → `ita`, for team national-champion jerseys. */
export function championCode(part: string): string | null {
  const m = /^maillot_([a-z]{3})$/.exec(part);
  return m ? m[1] : null;
}

export function partLabel(part: string, countryName?: (code: string) => string | undefined): string {
  if (NAMES[part]) return NAMES[part];
  const code = championCode(part);
  if (code) return `${countryName?.(code) ?? code.toUpperCase()} champion jersey`;
  return part.replace(/_/g, " ");
}

/** Main parts first, then champion jerseys, then the rest. */
export function partOrder(part: string): number {
  const fixed = ["minimaillot", "maillot", "maillot_tour", "minimaillot_tour", "leggings", "corsair", "socks", "socks_tour", "gloves", "gloves_tour", "shoes", "bottle", "handles", "transfert_av", "transfert_ar", "maillot_world", "maillot_world_itt", "maillot_euro", "maillot_euro_itt"];
  const i = fixed.indexOf(part);
  if (i >= 0) return i;
  return championCode(part) ? 100 : 200;
}
