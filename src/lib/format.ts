import type { Cyclist } from "./types";

export function flagEmoji(iso: string): string {
  if (!iso || iso.length < 2) return "";
  // Map 3-letter ISO to alpha-2 via same table as Rust
  const a2 = toAlpha2(iso);
  if (a2.length < 2) return "";
  const offset = 0x1F1E6 - 65;
  try {
    return String.fromCodePoint(a2.toUpperCase().charCodeAt(0) + offset) +
           String.fromCodePoint(a2.toUpperCase().charCodeAt(1) + offset);
  } catch { return ""; }
}

const ALPHA2: Record<string,string> = {
  alb:"AL",and:"AD",arg:"AR",arm:"AM",aus:"AU",aut:"AT",aze:"AZ",bel:"BE",
  blr:"BY",bol:"BO",bra:"BR",bul:"BG",can:"CA",chl:"CL",cmr:"CM",col:"CO",
  crc:"CR",cro:"HR",cub:"CU",cyp:"CY",cze:"CZ",den:"DK",dom:"DO",dza:"DZ",
  ecu:"EC",egy:"EG",eri:"ER",esp:"ES",est:"EE",eth:"ET",fin:"FI",fra:"FR",
  gab:"GA",gbr:"GB",geo:"GE",ger:"DE",gha:"GH",gre:"GR",hun:"HU",idn:"ID",
  ind:"IN",irl:"IE",irn:"IR",isl:"IS",isr:"IL",ita:"IT",jam:"JM",jpn:"JP",
  kaz:"KZ",ken:"KE",kgz:"KG",kor:"KR",kos:"XK",lat:"LV",lie:"LI",ltu:"LT",
  lux:"LU",mar:"MA",mas:"MY",mco:"MC",mex:"MX",mkd:"MK",mlt:"MT",mne:"ME",
  mol:"MD",ned:"NL",nga:"NG",nor:"NO",nzl:"NZ",pak:"PK",per:"PE",pol:"PL",
  por:"PT",qat:"QA",rou:"RO",rsa:"ZA",rus:"RU",rwa:"RW",sau:"SA",sen:"SN",
  ser:"RS",sgp:"SG",slo:"SI",smr:"SM",svk:"SK",swe:"SE",swi:"CH",tha:"TH",
  tto:"TT",tun:"TN",tur:"TR",twn:"TW",uae:"AE",uga:"UG",ukr:"UA",uru:"UY",
  usa:"US",uzb:"UZ",ven:"VE",vnm:"VN",zim:"ZW",civ:"CI",ben:"BJ",bfa:"BF",cod:"CD",
};
export function toAlpha2(iso3: string): string { return ALPHA2[iso3.toLowerCase()] ?? ""; }

export function fmtStars(v: number): string {
  if (!v) return "—";
  return v.toFixed(1).replace(/\.0$/, "");
}
export function fmtUpside(v: number): string {
  if (!v || v <= 0) return "—";
  return `+${v.toFixed(1).replace(/\.0$/, "")}`;
}
export function fmtNat(c: Cyclist): string {
  const flag = flagEmoji(c.iso);
  return flag ? `${flag} ${c.nationality}` : c.nationality;
}
export function rowClass(ca: number, free: boolean): string {
  if (ca >= 87) return "row-elite";
  if (ca >= 78) return "row-great";
  if (ca >= 68) return "row-good";
  if (free)     return "row-free";
  return "row-avg";
}
export function valFor(c: Cyclist, key: string): string {
  switch (key) {
    case "nat_flag":    return fmtNat(c);
    case "potential":   return fmtStars(c.potential);
    case "growth":      return fmtUpside(c.growth);
    case "top_skill_text":
      return c.top_skills.slice(0,2).map(s => `${s.label} ${s.value}`).join(" · ");
    default:
      return String((c as Record<string,unknown>)[key] ?? "");
  }
}

export function inText(hay: unknown, needle: string): boolean {
  return String(hay ?? "").toLowerCase().includes(needle);
}
