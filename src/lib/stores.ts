import { writable, derived } from "svelte/store";
import type { Cyclist, Team, Filters } from "./types";
import { inText } from "./format";

export const allCyclists   = writable<Cyclist[]>([]);
export const teams         = writable<Team[]>([]);
export const teamRosters   = writable<Record<number, Cyclist[]>>({});
export const gameDate      = writable("");
export const currentPath   = writable("");
export const activeSection = writable("Dashboard");
export const shortlistIds  = writable<Set<number>>(new Set());
export const notes         = writable<Record<string, string>>({});
export const selectedRider = writable<Cyclist | null>(null);
export const isLoading     = writable(false);
export const loadStatus    = writable("");

export const defaultFilters = (): Filters => ({
  search: "", team: "All Teams", country: "All Countries",
  continent: "All Continents", grade: "All Grades", typeId: null,
  minCA: 0, maxCA: 100, minAge: 16, maxAge: 45,
  minPot: 0, minUpside: 0,
  onlyU23: false, onlyFree: false, onlyUpside: false,
  onlyHighPot: false, onlyShortlist: false,
  sortKey: "current_ability", sortDir: -1,
});

export const filters = writable<Filters>(defaultFilters());

export const filtered = derived([allCyclists, filters, shortlistIds], ([$all, $f, $sl]) => {
  const q = $f.search.trim().toLowerCase();
  let out = $all.filter(c => {
    if (q && !["name","team","team_short","nationality","rider_type","continent"]
        .some(k => inText((c as Record<string,unknown>)[k], q))) return false;
    if ($f.team    !== "All Teams"      && c.team        !== $f.team)       return false;
    if ($f.country !== "All Countries"  && c.nationality !== $f.country)    return false;
    if ($f.continent !== "All Continents" && c.continent !== $f.continent)  return false;
    if ($f.grade   !== "All Grades"     && c.scout_grade !== $f.grade)      return false;
    if (c.current_ability < $f.minCA || c.current_ability > $f.maxCA)       return false;
    if (c.potential < $f.minPot)   return false;
    if (c.growth    < $f.minUpside) return false;
    if (c.age && (c.age < $f.minAge || c.age > $f.maxAge))                 return false;
    if ($f.typeId !== null && c.rider_type_id !== $f.typeId)               return false;
    if ($f.onlyU23    && (!c.age || c.age > 23))   return false;
    if ($f.onlyFree   && !c.free_agent)            return false;
    if ($f.onlyUpside && c.growth < 5)             return false;
    if ($f.onlyHighPot&& c.potential < 5)          return false;
    if ($f.onlyShortlist && !$sl.has(c.id))        return false;
    return true;
  });
  const k = $f.sortKey;
  const dir = $f.sortDir;
  out.sort((a, b) => {
    const av = (a as Record<string,unknown>)[k];
    const bv = (b as Record<string,unknown>)[k];
    if (typeof av === "number" && typeof bv === "number") return (av - bv) * dir;
    return String(av).localeCompare(String(bv)) * dir;
  });
  return out;
});

export const dropdownTeams    = derived(allCyclists, $c =>
  ["All Teams",    ...new Set($c.map(r => r.team).filter(t => t && !t.startsWith("Free")))].sort());
export const dropdownCountries = derived(allCyclists, $c =>
  ["All Countries",...new Set($c.map(r => r.nationality).filter(n => n && n !== "Unknown"))].sort());
export const dropdownConts     = derived(allCyclists, $c =>
  ["All Continents",...new Set($c.map(r => r.continent).filter(Boolean))].sort());
