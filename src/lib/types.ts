export interface TopSkill { key: string; label: string; value: number; }

export interface Cyclist {
  id: number; name: string; firstname: string; lastname: string;
  team_id: number; team: string; team_short: string; division: string;
  nationality: string; continent: string; iso: string; flag: string;
  birthdate: string; age: number; signable: boolean; signable_from: number;
  rider_type: string; rider_type_id: number;
  size: number; weight: number;
  current_ability: number; potential: number; growth: number;
  skill_average: number; skill_ceiling: number; peak_gap: number;
  specialty_rating: number; scout_grade: string; free_agent: boolean; is_mine: boolean;
  flat: number; flat_p: number; mountain: number; mountain_p: number;
  med_mtn: number; med_mtn_p: number; downhill: number; downhill_p: number;
  cobble: number; cobble_p: number; timetrial: number; timetrial_p: number;
  prologue: number; prologue_p: number; sprint: number; sprint_p: number;
  acceleration: number; acceleration_p: number;
  endurance: number; endurance_p: number; resistance: number; resistance_p: number;
  recuperation: number; recuperation_p: number;
  hill: number; hill_p: number; baroudeur: number; baroudeur_p: number;
  top_skills: TopSkill[];
  wage: number; contract_start: number; contract_end: number;
  popularity: number; wins: number; tour_rating: number; classic_rating: number;
  injured: boolean; will_retire: boolean; on_market: boolean;
  scout_reports: number; my_report: boolean; scout_report_date: string; scout_estimate: number;
  scout_tour_potential: number; scout_mountain_potential: number; scout_timetrial_potential: number;
  scout_sprint_potential: number; scout_ardennes_potential: number; scout_cobble_potential: number;
  scout_flat_potential: number;
}

export interface Team {
  id: number; name: string; short: string; abbreviation: string; jersey: string;
  country_iso: string; country_name: string; flag: string;
  color1: string; color2: string;
  division_id: number; division: string; tier: number;
  budget: number; sponsor_id: number; sponsor_deal_id: number; sponsor: string;
  sponsor_budget: number; sponsor_budget_next: number; sponsor_contract_end: number;
  payroll: number; staff_payroll: number; riders: number;
  avg_ca: number; top_ca: number; evaluation: number; is_mine: boolean;
}

export interface Staff { id: number; role: string; name: string; wage: number; contract_end: number; }
export interface BrandDeal { id: number; brand: string; category: string; budget: number; year: number; years_left: number; }
export interface SponsorOffer { sponsor_id: number; sponsor: string; budget: number; duration: number; contact_date: string; deadline: string; state: number; }
export interface LedgerEntry { id: number; date: string; amount: number; category: string; label: string; detail: string; }

export interface Finance {
  team_id: number; balance: number; balance_editable: boolean; season_budget: number;
  sponsor_id: number; sponsor_deal_id: number; sponsor: string; sponsor_budget: number; sponsor_budget_next: number;
  sponsor_contract_start: number; sponsor_contract_end: number;
  monthly_rider_wages: number; monthly_staff_wages: number;
  staff: Staff[]; brands: BrandDeal[]; offers: SponsorOffer[]; ledger: LedgerEntry[];
}

export interface SaveMeta {
  path: string; file_name: string; game_date: string; season: number;
  mod_name: string; game_version: string; user_team_id: number; user_team: string; manager: string;
}

export interface SaveData {
  meta: SaveMeta; cyclists: Cyclist[]; teams: Team[]; finance: Finance | null;
  game_date: string; modified_ms: number;
}

export interface CellEdit { table: string; column: string; key_column: string; key: number | string; value: number; }
export interface EditOutcome { table: string; column: string; key: string; old: number; new: number; }
export interface EditResponse extends SaveData { outcomes: EditOutcome[]; backup: string; }
export interface BackupInfo { path: string; file_name: string; created_ms: number; size: number; }
export interface SaveFile { path: string; name: string; game: string; modified_ms: number; size: number; }
export interface TableSummary { name: string; id: number; rows: number; columns: { name: string; index: number; ty: string }[]; }

/** How a column renders in RiderTable. */
export type ColKind =
  | "text" | "rider" | "team" | "ca" | "stars" | "upside" | "grade" | "type"
  | "money" | "contract" | "stat" | "scout" | "gem" | "age" | "nation" | "delta" | "sign";

export interface Col {
  key: string;
  label: string;
  width: number;
  kind?: ColKind;
  align?: "left" | "center" | "right";
  title?: string;
  value?: (r: Cyclist) => number | string;
}

export const GRADES = ["Wonderkid", "Elite Prospect", "Late Bloomer", "Ready Now", "Monitor", "Veteran", "Past Peak"] as const;

export const GRADE_COLOR: Record<string, string> = {
  "Wonderkid": "#f5c518",
  "Elite Prospect": "#3dbe6e",
  "Late Bloomer": "#b48cf0",
  "Ready Now": "#5aa8e6",
  "Monitor": "#8692a3",
  "Veteran": "#c9d2dd",
  "Past Peak": "#b08a5c",
};

export const GRADE_DESC: Record<string, string> = {
  "Wonderkid": "Young, a huge ceiling and already moving. Sign before anyone else does.",
  "Elite Prospect": "Steep development curve. Worth a long contract now.",
  "Late Bloomer": "Modest today, much higher ceiling. Needs patience and race days.",
  "Ready Now": "Young and already strong enough to race at the top level.",
  "Monitor": "Solid with a limited ceiling. Re-check next season.",
  "Veteran": "In their prime and reliable, with little room left to grow.",
  "Past Peak": "Over 30. The ceiling on paper is unlikely to be reached.",
};

/** Rider types in PCM order (STA_type_rider), each with a stable colour. */
export const RIDER_TYPES: { id: number; label: string; color: string }[] = [
  { id: 1, label: "Stage Racer", color: "#f5c518" },
  { id: 2, label: "Climber", color: "#e8524a" },
  { id: 3, label: "Time Trialist", color: "#5aa8e6" },
  { id: 4, label: "Sprinter", color: "#3dbe6e" },
  { id: 5, label: "Puncheur", color: "#e8913a" },
  { id: 6, label: "Cobbles", color: "#b48cf0" },
  { id: 7, label: "Rouleur", color: "#7fc4c0" },
];
export const TYPE_COLOR: Record<string, string> = Object.fromEntries(RIDER_TYPES.map((t) => [t.label, t.color]));

export const STAT_KEYS = [
  "mountain", "med_mtn", "hill", "timetrial", "prologue", "flat", "cobble",
  "sprint", "acceleration", "endurance", "resistance", "recuperation", "downhill", "baroudeur",
] as const;
export type StatKey = typeof STAT_KEYS[number];

export const STAT_LABELS: Record<string, string> = {
  flat: "Flat", mountain: "Mountain", med_mtn: "Medium mountain", downhill: "Downhill",
  cobble: "Cobbles", timetrial: "Time trial", prologue: "Prologue", sprint: "Sprint",
  acceleration: "Acceleration", endurance: "Stamina", resistance: "Resistance",
  recuperation: "Recovery", hill: "Hills", baroudeur: "Breakaway",
};
export const STAT_SHORT: Record<string, string> = {
  flat: "FL", mountain: "MO", med_mtn: "MM", downhill: "DH", cobble: "COB", timetrial: "TT",
  prologue: "PRL", sprint: "SP", acceleration: "ACC", endurance: "STA", resistance: "RES",
  recuperation: "REC", hill: "HIL", baroudeur: "BRK",
};

/** Terrain groups used for squad analysis and the compare radar. */
export const TERRAINS: { key: string; label: string; stats: StatKey[] }[] = [
  { key: "climb", label: "Climbing", stats: ["mountain", "med_mtn"] },
  { key: "hills", label: "Hills", stats: ["hill", "acceleration"] },
  { key: "tt", label: "Time trial", stats: ["timetrial", "prologue"] },
  { key: "sprint", label: "Sprint", stats: ["sprint", "acceleration"] },
  { key: "cobbles", label: "Cobbles", stats: ["cobble", "flat"] },
  { key: "stamina", label: "Stamina", stats: ["endurance", "resistance", "recuperation"] },
  { key: "break", label: "Breakaway", stats: ["baroudeur", "flat"] },
];

/** Scout report categories (DYN_scout_report potentials). */
export const SCOUT_CATS: { key: keyof Cyclist; label: string; short: string }[] = [
  { key: "scout_tour_potential", label: "Stage races", short: "Tour" },
  { key: "scout_mountain_potential", label: "Mountains", short: "Mtn" },
  { key: "scout_ardennes_potential", label: "Hills", short: "Hill" },
  { key: "scout_timetrial_potential", label: "Time trial", short: "TT" },
  { key: "scout_sprint_potential", label: "Sprint", short: "Spr" },
  { key: "scout_cobble_potential", label: "Cobbles", short: "Cob" },
  { key: "scout_flat_potential", label: "Flat", short: "Flat" },
];
