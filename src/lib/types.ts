export interface TopSkill { key: string; label: string; value: number; }

export interface Cyclist {
  id: number; name: string; firstname: string; lastname: string;
  team_id: number; team: string; team_short: string;
  nationality: string; continent: string; iso: string;
  birthdate: string; age: number;
  rider_type: string; rider_type_id: number;
  size: number; weight: number;
  current_ability: number; potential: number; growth: number;
  skill_average: number; skill_ceiling: number; peak_gap: number;
  specialty_rating: number; scout_grade: string; free_agent: boolean;
  flat: number; flat_p: number; mountain: number; mountain_p: number;
  med_mtn: number; med_mtn_p: number; downhill: number; downhill_p: number;
  cobble: number; cobble_p: number; timetrial: number; timetrial_p: number;
  prologue: number; prologue_p: number; sprint: number; sprint_p: number;
  acceleration: number; acceleration_p: number;
  endurance: number; endurance_p: number; resistance: number; resistance_p: number;
  recuperation: number; recuperation_p: number;
  hill: number; hill_p: number; baroudeur: number; baroudeur_p: number;
  top_skills: TopSkill[];
}

export interface Team {
  id: number; name: string; short: string;
  country_iso: string; country_name: string;
  color1: string; color2: string;
}

export interface SaveData {
  cyclists: Cyclist[]; scout_reports: Cyclist[]; teams: Team[]; game_date: string;
}

export interface Col {
  key: keyof Cyclist | string;
  label: string;
  width: number;
  align?: "left"|"center"|"right";
  sortKey?: keyof Cyclist | string;
  fmt?: (v: unknown, row: Cyclist) => string;
}

export interface Filters {
  search: string; team: string; country: string; continent: string;
  grade: string; typeId: number|null;
  minCA: number; maxCA: number; minAge: number; maxAge: number;
  minPot: number; minUpside: number;
  onlyU23: boolean; onlyFree: boolean; onlyUpside: boolean;
  onlyHighPot: boolean; onlyShortlist: boolean;
  sortKey: string; sortDir: 1|-1;
}

export const GRADES = ["Wonderkid","Elite Prospect","Late Bloomer","Ready Now","Monitor","Veteran","Past Peak"] as const;
export type Grade = typeof GRADES[number];

export const GRADE_COLOR: Record<string, string> = {
  "Wonderkid":      "#ffd700",
  "Elite Prospect": "#2ecc82",
  "Late Bloomer":   "#9470f0",
  "Ready Now":      "#24b0a8",
  "Monitor":        "#6478a0",
  "Veteran":        "#a0b8d8",
  "Past Peak":      "#7a6040",
};
export const GRADE_BG: Record<string, string> = {
  "Wonderkid":      "#1a1400",
  "Elite Prospect": "#061a0e",
  "Late Bloomer":   "#120a2a",
  "Ready Now":      "#041412",
  "Monitor":        "#0d1525",
  "Veteran":        "#0e1828",
  "Past Peak":      "#181008",
};
export const GRADE_DESC: Record<string, string> = {
  "Wonderkid":      "Exceptional youth — high CA and massive ceiling. Sign immediately.",
  "Elite Prospect": "Outstanding development curve. Invest before rivals notice.",
  "Late Bloomer":   "Below-average now but ceiling far above. Patience required.",
  "Ready Now":      "Near-peak performer. Delivers maximum impact today.",
  "Monitor":        "Solid rider, limited ceiling. Re-evaluate next season.",
  "Veteran":        "Prime years, high CA. Reliable performer with little headroom left.",
  "Past Peak":      "Age 30+ with paper ceiling — development window has closed. Stats on screen are the peak.",
};

export const TYPE_COLORS: Record<string, string> = {
  "All":          "#263756",
  "Climber":      "#e84040",
  "Sprinter":     "#2ecc82",
  "Time Trialist":"#4d88f5",
  "All-Rounder":  "#9470f0",
  "Classics":     "#d8d8cc",
  "Ardennes":     "#c87832",
  "Rouleur":      "#60a0c8",
};

export const STAT_LABELS: Record<string, string> = {
  flat:"Flat", mountain:"Mountain", med_mtn:"Med. Mountain", downhill:"Downhill",
  cobble:"Cobbles", timetrial:"Time Trial", prologue:"Prologue", sprint:"Sprint",
  acceleration:"Acceleration", endurance:"Endurance", resistance:"Resistance",
  recuperation:"Recovery", hill:"Hill Finish", baroudeur:"Baroudeur",
};

export const SORT_OPTS: { label: string; key: string }[] = [
  { label: "Current Ability", key: "current_ability" },
  { label: "Potential Stars", key: "potential" },
  { label: "Growth Upside",   key: "growth" },
  { label: "Age",             key: "age" },
  { label: "Name",            key: "name" },
  { label: "Specialty",       key: "specialty_rating" },
  { label: "Flat",            key: "flat" },
  { label: "Mountain",        key: "mountain" },
  { label: "Time Trial",      key: "timetrial" },
  { label: "Sprint",          key: "sprint" },
  { label: "Cobbles",         key: "cobble" },
  { label: "Endurance",       key: "endurance" },
];
