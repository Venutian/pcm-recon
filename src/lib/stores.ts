import { writable, derived, get } from "svelte/store";
import type { Cyclist, Team, SaveData, SaveMeta, Finance } from "./types";

export const save = writable<SaveData | null>(null);
export const meta = derived(save, ($s): SaveMeta | null => $s?.meta ?? null);
export const finance = derived(save, ($s): Finance | null => $s?.finance ?? null);
export const allCyclists = derived(save, ($s): Cyclist[] => $s?.cyclists ?? []);
export const teams = derived(save, ($s): Team[] => $s?.teams ?? []);
export const teamById = derived(teams, ($t) => new Map($t.map((t) => [t.id, t])));
export const myTeam = derived(teams, ($t) => $t.find((t) => t.is_mine) ?? null);
export const myRiders = derived(allCyclists, ($c) => $c.filter((c) => c.is_mine).sort((a, b) => b.current_ability - a.current_ability));
export const cyclistById = derived(allCyclists, ($c) => new Map($c.map((c) => [c.id, c])));
export const prospects = derived(allCyclists, ($c) => $c.filter((c) => c.scout_reports > 0));

export const activeSection = writable("Overview");
export const selectedId = writable<number | null>(null);
export const selectedRider = derived([selectedId, cyclistById], ([$id, $m]) => ($id == null ? null : $m.get($id) ?? null));
export const isLoading = writable(false);
export const loadStatus = writable("");
export const paletteOpen = writable(false);

/** Section-to-section hand-off, e.g. "find a sprinter" from My team into Scout. */
export const scoutPreset = writable<Partial<ScoutFilters> | null>(null);

// ─── Workspace (persisted by api.ts) ─────────────────────────────────────────
export const shortlist = writable<Set<number>>(new Set());
export const notes = writable<Record<string, string>>({});
export const compareIds = writable<number[]>([]);
export const lastSavePath = writable<string>("");

export function toggleShortlist(id: number) {
  shortlist.update((s) => {
    const n = new Set(s);
    n.has(id) ? n.delete(id) : n.add(id);
    return n;
  });
}

export function toggleCompare(id: number) {
  compareIds.update((ids) => (ids.includes(id) ? ids.filter((x) => x !== id) : [...ids, id].slice(-4)));
}

export function selectRider(id: number | null) {
  selectedId.set(id);
}

export function goTo(section: string) {
  activeSection.set(section);
}

// ─── Toasts ───────────────────────────────────────────────────────────────────
export interface Toast { id: number; kind: "ok" | "error" | "info"; text: string; }
export const toasts = writable<Toast[]>([]);
let toastSeq = 0;
export function toast(text: string, kind: Toast["kind"] = "ok", ms = 4200) {
  const id = ++toastSeq;
  toasts.update((t) => [...t, { id, kind, text }]);
  setTimeout(() => toasts.update((t) => t.filter((x) => x.id !== id)), ms);
}

// ─── Scout search filters ─────────────────────────────────────────────────────
export interface ScoutFilters {
  q: string;
  types: number[];
  minAge: number;
  maxAge: number;
  minCA: number;
  minPot: number;
  nation: string;
  division: string;
  status: "all" | "market" | "free" | "expiring" | "signed";
  maxWage: number;
  statKey: string;
  statMin: number;
  hideMine: boolean;
  sort: string;
  sortDir: 1 | -1;
}

export const defaultScoutFilters = (): ScoutFilters => ({
  q: "", types: [], minAge: 16, maxAge: 40, minCA: 0, minPot: 0,
  nation: "", division: "", status: "all", maxWage: 0,
  statKey: "", statMin: 0, hideMine: false,
  sort: "current_ability", sortDir: -1,
});

export const scoutFilters = writable<ScoutFilters>(defaultScoutFilters());

export const scoutResults = derived([allCyclists, scoutFilters, meta], ([$all, f, $meta]) => {
  const q = f.q.trim().toLowerCase();
  const season = $meta?.season ?? 0;
  const out = $all.filter((c) => {
    if (q && !(c.name.toLowerCase().includes(q) || c.team.toLowerCase().includes(q) || c.nationality.toLowerCase().includes(q))) return false;
    if (f.types.length && !f.types.includes(c.rider_type_id)) return false;
    if (c.age && (c.age < f.minAge || c.age > f.maxAge)) return false;
    if (c.current_ability < f.minCA) return false;
    if (c.potential < f.minPot) return false;
    if (f.nation && c.nationality !== f.nation) return false;
    if (f.division && c.division !== f.division) return false;
    if (f.hideMine && c.is_mine) return false;
    if (f.status === "market" && !c.on_market) return false;
    if (f.status === "free" && !c.free_agent) return false;
    if (f.status === "expiring" && (c.free_agent || c.contract_end > season)) return false;
    if (f.status === "signed" && c.free_agent) return false;
    if (f.maxWage > 0 && c.wage > f.maxWage) return false;
    if (f.statKey && f.statMin > 0 && ((c as unknown as Record<string, number>)[f.statKey] ?? 0) < f.statMin) return false;
    return true;
  });
  const k = f.sort, dir = f.sortDir;
  out.sort((a, b) => {
    const av = (a as unknown as Record<string, unknown>)[k], bv = (b as unknown as Record<string, unknown>)[k];
    if (typeof av === "number" && typeof bv === "number") return (av - bv) * dir;
    return String(av).localeCompare(String(bv)) * dir;
  });
  return out;
});

export function currentSave(): SaveData | null {
  return get(save);
}

// ─── Save writes (confirmed through WriteConfirm.svelte) ─────────────────────
import type { CellEdit } from "./types";
export interface PendingWrite {
  title: string;
  lines: { label: string; from: number; to: number }[];
  edits: CellEdit[];
  done?: string;
}
export const pendingWrite = writable<PendingWrite | null>(null);
export function requestWrite(w: PendingWrite) {
  pendingWrite.set(w);
}
