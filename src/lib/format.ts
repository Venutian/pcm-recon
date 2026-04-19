import type { Cyclist } from "./types";

export function flagEmoji(iso: string): string {
  const a2 = toAlpha2(iso).toLowerCase();
  if (!a2) return "";
  return `<span class="fi fi-${a2}" style="width:20px;height:15px;display:inline-block;vertical-align:middle;margin-right:6px;background-size:cover;background-position:center"></span>`;
}

const ALPHA2: Record<string, string> = {
  alb: "AL",
  and: "AD",
  arg: "AR",
  arm: "AM",
  aus: "AU",
  aut: "AT",
  aze: "AZ",
  bel: "BE",
  blr: "BY",
  bol: "BO",
  bra: "BR",
  bul: "BG",
  can: "CA",
  chl: "CL",
  cmr: "CM",
  col: "CO",
  crc: "CR",
  cro: "HR",
  cub: "CU",
  cyp: "CY",
  cze: "CZ",
  den: "DK",
  dom: "DO",
  dza: "DZ",
  ecu: "EC",
  egy: "EG",
  eri: "ER",
  esp: "ES",
  est: "EE",
  eth: "ET",
  fin: "FI",
  fra: "FR",
  gab: "GA",
  gbr: "GB",
  geo: "GE",
  ger: "DE",
  gha: "GH",
  gre: "GR",
  hun: "HU",
  idn: "ID",
  ind: "IN",
  irl: "IE",
  irn: "IR",
  isl: "IS",
  isr: "IL",
  ita: "IT",
  jam: "JM",
  jpn: "JP",
  kaz: "KZ",
  ken: "KE",
  kgz: "KG",
  kor: "KR",
  kos: "XK",
  lat: "LV",
  lie: "LI",
  ltu: "LT",
  lux: "LU",
  mar: "MA",
  mas: "MY",
  mco: "MC",
  mex: "MX",
  mkd: "MK",
  mlt: "MT",
  mne: "ME",
  mol: "MD",
  ned: "NL",
  nga: "NG",
  nor: "NO",
  nzl: "NZ",
  pak: "PK",
  per: "PE",
  pol: "PL",
  por: "PT",
  qat: "QA",
  rou: "RO",
  rsa: "ZA",
  rus: "RU",
  rwa: "RW",
  sau: "SA",
  sen: "SN",
  ser: "RS",
  sgp: "SG",
  slo: "SI",
  smr: "SM",
  svk: "SK",
  swe: "SE",
  swi: "CH",
  tha: "TH",
  tto: "TT",
  tun: "TN",
  tur: "TR",
  twn: "TW",
  uae: "AE",
  uga: "UG",
  ukr: "UA",
  uru: "UY",
  usa: "US",
  uzb: "UZ",
  ven: "VE",
  vnm: "VN",
  zim: "ZW",
  civ: "CI",
  ben: "BJ",
  bfa: "BF",
  cod: "CD",
};

export function toAlpha2(iso3: string): string {
  return ALPHA2[iso3.toLowerCase()] ?? "";
}

export function fmtStars(v: number): string {
  if (!v) return "-";
  return v.toFixed(1).replace(/\.0$/, "");
}

export function fmtUpside(v: number): string {
  if (!v || v <= 0) return "-";
  return `+${v.toFixed(1).replace(/\.0$/, "")}`;
}

export function fmtNatParts(c: Cyclist): { flagHtml: string; nationality: string } {
  return {
    flagHtml: flagEmoji(c.iso),
    nationality: c.nationality || "Unknown",
  };
}

export function fmtNat(c: Cyclist): string {
  return fmtNatParts(c).nationality;
}

export function rowClass(ca: number, free: boolean): string {
  if (ca >= 87) return "row-elite";
  if (ca >= 78) return "row-great";
  if (ca >= 68) return "row-good";
  if (free) return "row-free";
  return "row-avg";
}

export function valFor(c: Cyclist, key: string): string {
  switch (key) {
    case "nat_flag":
      return fmtNat(c);
    case "potential":
      return fmtStars(c.potential);
    case "growth":
      return fmtUpside(c.growth);
    case "top_skill_text":
      return c.top_skills.slice(0, 2).map((s) => `${s.label} ${s.value}`).join(" | ");
    default:
      return String((c as Record<string, unknown>)[key] ?? "");
  }
}

export function inText(hay: unknown, needle: string): boolean {
  return String(hay ?? "").toLowerCase().includes(needle);
}

export function teamColorFromId(id: number): string {
  if (!id || id <= 0) return "#4a5e80";
  const hue = Math.abs((id * 137) % 360);
  return `hsl(${hue}, 58%, 52%)`;
}

export function teamColorBgFromId(id: number): string {
  if (!id || id <= 0) return "#111c30";
  const hue = Math.abs((id * 137) % 360);
  return `hsl(${hue}, 32%, 13%)`;
}

export function resolveTeamColor(color1: string, teamId: number): string {
  if (color1 && color1 !== "#000000") return color1;
  return teamColorFromId(teamId);
}
