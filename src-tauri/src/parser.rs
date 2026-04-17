/// PCM Recon — binary .cdb parser (Rust port of extract_pcm.py)
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use flate2::read::ZlibDecoder;
use std::io::Read;

// ─── Layout constants ─────────────────────────────────────────────────────────
const NUM_CYCLISTS: usize = 0xAFE0 / 4; // 11 256
const TEAM_COUNT:   usize = 313;
const REGION_COUNT: usize = 241;
const FREE_AGENT_TEAM_ID: i32 = 119;

const AA: [u8; 4] = [0xAA, 0xAA, 0xAA, 0xAA];
const BB: [u8; 4] = [0xBB, 0xBB, 0xBB, 0xBB];

// Cyclist int-column offsets
const OFF_ID:          usize = 0x46184;
const OFF_TEAM:        usize = 0xB7B10;
const OFF_REGION:      usize = 0xC2B6C;
const OFF_BIRTHDATE:   usize = 0x102460;
const OFF_TYPE_RIDER:  usize = 0x170890;
const OFF_SIZE:        usize = 0x1919AC;
const OFF_WEIGHT:      usize = 0x19CA10;
const OFF_FLAT:        usize = 0x1D3C24;
const OFF_FLAT_P:      usize = 0x1DEC88;
const OFF_MTN:         usize = 0x1E9CF4;
const OFF_MTN_P:       usize = 0x1F4D60;
const OFF_MED_MTN:     usize = 0x1FFDCC + 0x10;
const OFF_MED_MTN_P:   usize = 0x20AE50;
const OFF_DOWNHILL:    usize = 0x215EC4;
const OFF_DOWNHILL_P:  usize = 0x220F30;
const OFF_COBBLE:      usize = 0x22BF94;
const OFF_COBBLE_P:    usize = 0x236FF8;
const OFF_TT:          usize = 0x242064;
const OFF_TT_P:        usize = 0x24D0D0;
const OFF_PROLOGUE:    usize = 0x25813C;
const OFF_PROLOGUE_P:  usize = 0x2631A8;
const OFF_SPRINT:      usize = 0x26E20C;
const OFF_SPRINT_P:    usize = 0x279270;
const OFF_ACCEL:       usize = 0x2842E4;
const OFF_ACCEL_P:     usize = 0x28F358;
const OFF_ENDURANCE:   usize = 0x29A3C4;
const OFF_ENDURANCE_P: usize = 0x2A5430;
const OFF_RESISTANCE:  usize = 0x2B049C;
const OFF_RESISTANCE_P:usize = 0x2BB508;
const OFF_RECUP:       usize = 0x2C657C;
const OFF_RECUP_P:     usize = 0x2D15F0;
const OFF_HILL:        usize = 0x2DC654;
const OFF_HILL_P:      usize = 0x2E76B8;
const OFF_BAROUDEUR:   usize = 0x2F2724;
const OFF_BAROUDEUR_P: usize = 0x2FD790;

// Float columns
const OFF_POTENTIAL:       usize = 0x12E618;
const OFF_CURRENT_ABILITY: usize = 0x13968C;

// String columns (lengths_start, data_start)
const OFF_LASTNAME_LEN:  usize = 0x511F0;
const OFF_LASTNAME_DATA: usize = 0x5C1D4;
const OFF_FIRSTNAME_LEN: usize = 0x713EC;
const OFF_FIRSTNAME_DATA:usize = 0x7C3D0;
const OFF_FULLNAME_LEN:  usize = 0x8F2C4;
const OFF_FULLNAME_DATA: usize = 0x9A2A8;

// Teams
const OFF_TEAM_REGION:      usize = 0x7BD748;
const OFF_TEAM_REGION_END:  usize = 0x7BD748 + 0x30E38;
const OFF_TEAM_NAME_LEN:    usize = 0x7BF404;
const OFF_TEAM_NAME_DATA:   usize = 0x7BF8EC;
const OFF_TEAM_SHORT_LEN:   usize = 0x7BDDE8;
const OFF_TEAM_SHORT_DATA:  usize = 0x7BE2D0;

// Countries / regions
const OFF_COUNTRY_CODES: usize = 0xA061E4;
const COUNTRY_COUNT:     usize = 145;
const OFF_REGION_POS:    usize = 0xA4CAB8;
const OFF_REGION_END:    usize = 0xA66544;
const OFF_SAVE_META_POS: usize = 0xA01C74;
const OFF_SAVE_META_END: usize = 0xA02040;

// ─── Output types ─────────────────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopSkill {
    pub key:   String,
    pub label: String,
    pub value: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cyclist {
    pub id: i32,
    pub name: String,
    pub firstname: String,
    pub lastname: String,
    pub team_id: i32,
    pub team: String,
    pub team_short: String,
    pub nationality: String,
    pub continent: String,
    pub iso: String,
    pub birthdate: String,
    pub age: i32,
    pub rider_type: String,
    pub rider_type_id: i32,
    pub size: i32,
    pub weight: i32,
    pub current_ability: f32,
    pub potential: f32,
    pub growth: f32,
    pub skill_average: f32,
    pub skill_ceiling: f32,
    pub peak_gap: i32,
    pub specialty_rating: i32,
    pub scout_grade: String,
    pub free_agent: bool,
    pub flat: i32,        pub flat_p: i32,
    pub mountain: i32,    pub mountain_p: i32,
    pub med_mtn: i32,     pub med_mtn_p: i32,
    pub downhill: i32,    pub downhill_p: i32,
    pub cobble: i32,      pub cobble_p: i32,
    pub timetrial: i32,   pub timetrial_p: i32,
    pub prologue: i32,    pub prologue_p: i32,
    pub sprint: i32,      pub sprint_p: i32,
    pub acceleration: i32,pub acceleration_p: i32,
    pub endurance: i32,   pub endurance_p: i32,
    pub resistance: i32,  pub resistance_p: i32,
    pub recuperation: i32,pub recuperation_p: i32,
    pub hill: i32,        pub hill_p: i32,
    pub baroudeur: i32,   pub baroudeur_p: i32,
    pub top_skills: Vec<TopSkill>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub short: String,
    pub country_iso: String,
    pub country_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub cyclists:  Vec<Cyclist>,
    pub teams:     Vec<Team>,
    pub game_date: String,
}

// ─── Date helpers ─────────────────────────────────────────────────────────────
struct Date { year: i32, month: i32, day: i32 }

fn parse_date_int(v: i32) -> Option<Date> {
    if v <= 19000101 || v >= 22000101 { return None; }
    let year  = v / 10000;
    let month = (v % 10000) / 100;
    let day   = v % 100;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) { return None; }
    Some(Date { year, month, day })
}

fn age_at(birth: i32, ref_date: &Date) -> i32 {
    let b = match parse_date_int(birth) { Some(d) => d, None => return 0 };
    let mut age = ref_date.year - b.year;
    if ref_date.month < b.month || (ref_date.month == b.month && ref_date.day < b.day) {
        age -= 1;
    }
    if age < 0 || age >= 100 { 0 } else { age }
}

fn date_text(d: &Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

// ─── Binary reading helpers ───────────────────────────────────────────────────
fn read_i32(data: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(data[off..off+4].try_into().unwrap_or_default())
}
fn read_u32(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(data[off..off+4].try_into().unwrap_or_default())
}
fn read_f32(data: &[u8], off: usize) -> f32 {
    f32::from_le_bytes(data[off..off+4].try_into().unwrap_or_default())
}

fn read_ints(data: &[u8], start: usize, count: usize) -> Vec<i32> {
    (0..count).map(|i| read_i32(data, start + i*4)).collect()
}
fn read_uints(data: &[u8], start: usize, count: usize) -> Vec<u32> {
    (0..count).map(|i| read_u32(data, start + i*4)).collect()
}
fn read_floats(data: &[u8], start: usize, count: usize) -> Vec<f32> {
    (0..count).map(|i| read_f32(data, start + i*4)).collect()
}

fn read_strings(data: &[u8], len_start: usize, data_start: usize, count: usize) -> Vec<String> {
    let mut pos = data_start;
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let len = read_u32(data, len_start + i*4) as usize;
        if len == 0 || len > 300 || pos + len > data.len() {
            out.push(String::new());
        } else {
            out.push(decode_pcm_text(&data[pos..pos+len]));
            pos += len;
        }
    }
    out
}

fn read_nullterm(data: &[u8], start: usize, count: usize) -> Vec<String> {
    let mut pos = start;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let end = data[pos..].iter().position(|&b| b == 0).map(|e| pos+e).unwrap_or(pos+4);
        if end > data.len() { out.push(String::new()); break; }
        out.push(decode_pcm_text(&data[pos..end]));
        pos = end + 1;
    }
    out
}

// ─── Column finder (matches Python find_col / find_type22_data) ───────────────
fn find_pattern(data: &[u8], pattern: &[u8], from: usize, to: usize) -> Option<usize> {
    let to = to.min(data.len());
    data[from..to].windows(pattern.len()).position(|w| w == pattern).map(|p| p + from)
}

fn find_type22_data(data: &[u8], col_pos: usize, search_range: usize) -> Option<usize> {
    let mut pos = col_pos + 4;
    for _ in 0..25 {
        let limit = col_pos + search_range;
        let idx = find_pattern(data, &AA, pos, limit)?;
        if idx + 12 < data.len() {
            let block_type = read_u32(data, idx + 8);
            if block_type == 0x22 {
                if let Some(bb_pos) = find_pattern(data, &BB, idx, idx + 80) {
                    if bb_pos + 4 <= data.len() {
                        return Some(bb_pos + 4);
                    }
                }
            }
        }
        pos = idx + 4;
    }
    None
}

fn find_col(data: &[u8], start: usize, end: usize, name: &[u8]) -> Option<usize> {
    let mut pos = start;
    while pos < end {
        let idx = find_pattern(data, &AA, pos, end)?;
        if idx + 24 < data.len() {
            let block_type  = read_u32(data, idx + 8);
            let header_num  = read_u32(data, idx + 16);
            if block_type == 0x20 && header_num == 1 {
                let name_len = read_u32(data, idx + 20) as usize;
                if name_len > 0 && name_len < 100 && idx + 24 + name_len <= data.len() {
                    let found = &data[idx + 24 .. idx + 24 + name_len];
                    let found = found.split(|&b| b == 0).next().unwrap_or(found);
                    if found == name {
                        return find_type22_data(data, idx, 800);
                    }
                }
            }
        }
        pos = idx + 4;
    }
    None
}

// ─── Text decoding ────────────────────────────────────────────────────────────
fn decode_pcm_text(raw: &[u8]) -> String {
    let raw: Vec<u8> = raw.iter().copied().take_while(|&b| b != 0).collect();
    if raw.is_empty() { return String::new(); }

    // Try UTF-8
    if let Ok(s) = std::str::from_utf8(&raw) {
        let s = s.trim();
        if !s.is_empty() && !s.contains('\u{FFFD}') {
            return sanitize(s);
        }
    }
    // Try CP1252 via manual table
    let s: String = raw.iter().map(|&b| cp1252(b)).collect();
    sanitize(&s)
}

fn sanitize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\r' | '\n' | '\t' => out.push(' '),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cp1252(b: u8) -> char {
    // CP1252 extension block (0x80–0x9F)
    const EXT: [char; 32] = [
        '€','\u{81}','‚','ƒ','„','…','†','‡','ˆ','‰','Š','‹','Œ','\u{8D}','Ž','\u{8F}',
        '\u{90}','\'','\'','"','"','•','–','—','˜','™','š','›','œ','\u{9D}','ž','Ÿ',
    ];
    if b < 0x80 || b >= 0xA0 { b as char } else { EXT[(b - 0x80) as usize] }
}

fn looks_like_garbage(s: &str) -> bool {
    if s.is_empty() { return true; }
    if s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_digit()) { return true; }
    let visible: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
    if visible.is_empty() { return true; }
    let weird = visible.iter().filter(|&&c| !(c.is_alphabetic() || ".'\\-#/&()".contains(c))).count();
    weird as f32 / visible.len() as f32 > 0.35
}

fn choose_name(first: &str, last: &str, full: &str, id: i32) -> String {
    let combined = format!("{} {}", first, last).trim().to_string();
    for candidate in &[combined.as_str(), full, first, last] {
        if !candidate.is_empty() && !looks_like_garbage(candidate) {
            return candidate.to_string();
        }
    }
    format!("#{}", id)
}

// ─── Country / continent data ─────────────────────────────────────────────────
fn normalize_iso(raw: &str) -> String {
    let s: String = raw.chars().filter(|c| c.is_alphabetic()).take(3).collect::<String>().to_lowercase();
    match s.as_str() {
        "swd" => "swe".to_string(),
        "rom" => "rou".to_string(),
        "chi" => "chl".to_string(),
        other => other.to_string(),
    }
}

fn country_name(iso: &str) -> &'static str {
    match iso {
        "ago"=>"Angola","alb"=>"Albania","and"=>"Andorra","arg"=>"Argentina",
        "arm"=>"Armenia","aus"=>"Australia","aut"=>"Austria","aze"=>"Azerbaijan",
        "bel"=>"Belgium","ben"=>"Benin","bfa"=>"Burkina Faso","bhr"=>"Bahrain",
        "bih"=>"Bosnia and Herzegovina","blr"=>"Belarus","bol"=>"Bolivia",
        "bra"=>"Brazil","bul"=>"Bulgaria","can"=>"Canada","chl"=>"Chile",
        "cmr"=>"Cameroon","cod"=>"DR Congo","col"=>"Colombia","crc"=>"Costa Rica",
        "cro"=>"Croatia","cub"=>"Cuba","civ"=>"Ivory Coast","cyp"=>"Cyprus",
        "cze"=>"Czech Republic","den"=>"Denmark","dom"=>"Dominican Republic",
        "dza"=>"Algeria","ecu"=>"Ecuador","egy"=>"Egypt","eri"=>"Eritrea",
        "esp"=>"Spain","est"=>"Estonia","eth"=>"Ethiopia","fin"=>"Finland",
        "fra"=>"France","gab"=>"Gabon","gbr"=>"Great Britain","geo"=>"Georgia",
        "ger"=>"Germany","gha"=>"Ghana","gre"=>"Greece","hun"=>"Hungary",
        "idn"=>"Indonesia","ind"=>"India","irl"=>"Ireland","irn"=>"Iran",
        "isl"=>"Iceland","isr"=>"Israel","ita"=>"Italy","jam"=>"Jamaica",
        "jpn"=>"Japan","kaz"=>"Kazakhstan","ken"=>"Kenya","kgz"=>"Kyrgyzstan",
        "kor"=>"South Korea","kos"=>"Kosovo","lat"=>"Latvia","lie"=>"Liechtenstein",
        "ltu"=>"Lithuania","lux"=>"Luxembourg","mar"=>"Morocco","mas"=>"Malaysia",
        "mco"=>"Monaco","mex"=>"Mexico","mkd"=>"North Macedonia","mlt"=>"Malta",
        "mne"=>"Montenegro","mol"=>"Moldova","ned"=>"Netherlands","nga"=>"Nigeria",
        "nor"=>"Norway","nzl"=>"New Zealand","pak"=>"Pakistan","per"=>"Peru",
        "pol"=>"Poland","por"=>"Portugal","qat"=>"Qatar","rou"=>"Romania",
        "rsa"=>"South Africa","rus"=>"Russia","rwa"=>"Rwanda","sau"=>"Saudi Arabia",
        "sen"=>"Senegal","ser"=>"Serbia","sgp"=>"Singapore","slo"=>"Slovenia",
        "smr"=>"San Marino","svk"=>"Slovakia","swe"=>"Sweden","swi"=>"Switzerland",
        "tha"=>"Thailand","tto"=>"Trinidad and Tobago","tun"=>"Tunisia",
        "tur"=>"Turkey","twn"=>"Taiwan","uae"=>"United Arab Emirates",
        "uga"=>"Uganda","ukr"=>"Ukraine","uru"=>"Uruguay","usa"=>"United States",
        "uzb"=>"Uzbekistan","ven"=>"Venezuela","vnm"=>"Vietnam","zim"=>"Zimbabwe",
        _ => "",
    }
}

fn continent(iso: &str) -> &'static str {
    match iso {
        "ago"|"ben"|"bfa"|"cmr"|"cod"|"civ"|"dza"|"egy"|"eri"|"eth"|"gab"|"gha"|
        "ken"|"lba"|"lso"|"mli"|"mar"|"moz"|"nam"|"nga"|"nig"|"rsa"|"rwa"|"sau"|
        "sen"|"tun"|"uga"|"zim" => "Africa",
        "arm"|"aze"|"bhr"|"brn"|"geo"|"hkg"|"idn"|"ind"|"irn"|"irq"|"isr"|"jpn"|
        "kaz"|"ken"|"khm"|"kgz"|"kor"|"kuw"|"lao"|"lka"|"mas"|"mng"|"oma"|"pak"|
        "phl"|"qat"|"sgp"|"syr"|"tha"|"tls"|"twn"|"uae"|"uzb"|"vnm" => "Asia",
        "alb"|"and"|"aut"|"bel"|"bih"|"blr"|"bul"|"cro"|"cyp"|"cze"|"den"|"esp"|
        "est"|"fin"|"fra"|"gbr"|"geo"|"ger"|"gre"|"hun"|"irl"|"isl"|"ita"|"kos"|
        "lat"|"lie"|"ltu"|"lux"|"mco"|"mkd"|"mlt"|"mne"|"mol"|"ned"|"nor"|"pol"|
        "por"|"rou"|"rus"|"ser"|"slo"|"smr"|"svk"|"swe"|"swi"|"tur"|"ukr" => "Europe",
        "can"|"crc"|"cub"|"dom"|"gtm"|"hnd"|"jam"|"mex"|"nic"|"pan"|"pri"|"usa" => "North America",
        "aus"|"nzl" => "Oceania",
        "arg"|"bol"|"bra"|"chl"|"col"|"ecu"|"guy"|"per"|"pry"|"sur"|"tto"|"uru"|"ven" => "South America",
        _ => "Unknown",
    }
}

fn flag_emoji(iso: &str) -> String {
    if iso.len() < 2 { return String::new(); }
    // Use 2-letter ISO 3166-1 alpha-2 equivalents for common cycling nations
    let alpha2 = iso_to_alpha2(iso);
    if alpha2.len() < 2 { return String::new(); }
    let bytes: Vec<char> = alpha2.to_uppercase().chars().collect();
    if bytes.len() < 2 { return String::new(); }
    let a = char::from_u32(0x1F1E6 + bytes[0] as u32 - 'A' as u32).unwrap_or(' ');
    let b = char::from_u32(0x1F1E6 + bytes[1] as u32 - 'A' as u32).unwrap_or(' ');
    format!("{}{}", a, b)
}

fn iso_to_alpha2(iso3: &str) -> &'static str {
    match iso3 {
        "alb"=>"AL","and"=>"AD","arg"=>"AR","arm"=>"AM","aus"=>"AU","aut"=>"AT",
        "aze"=>"AZ","bel"=>"BE","blr"=>"BY","bol"=>"BO","bra"=>"BR","bul"=>"BG",
        "can"=>"CA","chl"=>"CL","cmr"=>"CM","col"=>"CO","crc"=>"CR","cro"=>"HR",
        "cub"=>"CU","cyp"=>"CY","cze"=>"CZ","den"=>"DK","dom"=>"DO","dza"=>"DZ",
        "ecu"=>"EC","egy"=>"EG","eri"=>"ER","esp"=>"ES","est"=>"EE","eth"=>"ET",
        "fin"=>"FI","fra"=>"FR","gab"=>"GA","gbr"=>"GB","geo"=>"GE","ger"=>"DE",
        "gha"=>"GH","gre"=>"GR","hun"=>"HU","idn"=>"ID","ind"=>"IN","irl"=>"IE",
        "irn"=>"IR","isl"=>"IS","isr"=>"IL","ita"=>"IT","jam"=>"JM","jpn"=>"JP",
        "kaz"=>"KZ","ken"=>"KE","kgz"=>"KG","kor"=>"KR","kos"=>"XK","lat"=>"LV",
        "lie"=>"LI","ltu"=>"LT","lux"=>"LU","mar"=>"MA","mas"=>"MY","mco"=>"MC",
        "mex"=>"MX","mkd"=>"MK","mlt"=>"MT","mne"=>"ME","mol"=>"MD","ned"=>"NL",
        "nga"=>"NG","nor"=>"NO","nzl"=>"NZ","pak"=>"PK","per"=>"PE","pol"=>"PL",
        "por"=>"PT","qat"=>"QA","rou"=>"RO","rsa"=>"ZA","rus"=>"RU","rwa"=>"RW",
        "sau"=>"SA","sen"=>"SN","ser"=>"RS","sgp"=>"SG","slo"=>"SI","smr"=>"SM",
        "svk"=>"SK","swe"=>"SE","swi"=>"CH","tha"=>"TH","tto"=>"TT","tun"=>"TN",
        "tur"=>"TR","twn"=>"TW","uae"=>"AE","uga"=>"UG","ukr"=>"UA","uru"=>"UY",
        "usa"=>"US","uzb"=>"UZ","ven"=>"VE","vnm"=>"VN","zim"=>"ZW","civ"=>"CI",
        "eth"=>"ET","ben"=>"BJ","bfa"=>"BF","cod"=>"CD",_ => "",
    }
}

// ─── Derived metrics ──────────────────────────────────────────────────────────
const STAT_KEYS: &[(&str, &str, bool)] = &[
    // (field_name, label, is_potential)
    ("flat",         "Flat",          false),
    ("mountain",     "Mountain",      false),
    ("med_mtn",      "Med. Mountain", false),
    ("downhill",     "Downhill",      false),
    ("cobble",       "Cobbles",       false),
    ("timetrial",    "Time Trial",    false),
    ("prologue",     "Prologue",      false),
    ("sprint",       "Sprint",        false),
    ("acceleration", "Acceleration",  false),
    ("endurance",    "Endurance",     false),
    ("resistance",   "Resistance",    false),
    ("recuperation", "Recuperation",  false),
    ("hill",         "Hill",          false),
    ("baroudeur",    "Baroudeur",     false),
];

fn get_stat(c: &Cyclist, key: &str) -> f32 {
    match key {
        "flat"=>c.flat as f32,"mountain"=>c.mountain as f32,"med_mtn"=>c.med_mtn as f32,
        "downhill"=>c.downhill as f32,"cobble"=>c.cobble as f32,"timetrial"=>c.timetrial as f32,
        "prologue"=>c.prologue as f32,"sprint"=>c.sprint as f32,"acceleration"=>c.acceleration as f32,
        "endurance"=>c.endurance as f32,"resistance"=>c.resistance as f32,
        "recuperation"=>c.recuperation as f32,"hill"=>c.hill as f32,"baroudeur"=>c.baroudeur as f32,
        _ => 0.0,
    }
}
fn get_stat_p(c: &Cyclist, key: &str) -> f32 {
    match key {
        "flat"=>c.flat_p as f32,"mountain"=>c.mountain_p as f32,"med_mtn"=>c.med_mtn_p as f32,
        "downhill"=>c.downhill_p as f32,"cobble"=>c.cobble_p as f32,"timetrial"=>c.timetrial_p as f32,
        "prologue"=>c.prologue_p as f32,"sprint"=>c.sprint_p as f32,
        "acceleration"=>c.acceleration_p as f32,"endurance"=>c.endurance_p as f32,
        "resistance"=>c.resistance_p as f32,"recuperation"=>c.recuperation_p as f32,
        "hill"=>c.hill_p as f32,"baroudeur"=>c.baroudeur_p as f32,
        _ => 0.0,
    }
}

fn stat_avg(c: &Cyclist) -> f32 {
    STAT_KEYS.iter().map(|(k,_,_)| get_stat(c,k)).sum::<f32>() / STAT_KEYS.len() as f32
}
fn stat_ceil(c: &Cyclist) -> f32 {
    STAT_KEYS.iter().map(|(k,_,_)| get_stat_p(c,k)).sum::<f32>() / STAT_KEYS.len() as f32
}
fn peak_gap(c: &Cyclist) -> i32 {
    STAT_KEYS.iter().map(|(k,_,_)| (get_stat_p(c,k) - get_stat(c,k)) as i32).max().unwrap_or(0)
}
fn top_skills(c: &Cyclist) -> Vec<TopSkill> {
    let mut ranked: Vec<TopSkill> = STAT_KEYS.iter().map(|(k,l,_)| TopSkill {
        key: k.to_string(), label: l.to_string(), value: get_stat(c,k) as i32,
    }).collect();
    ranked.sort_by(|a,b| b.value.cmp(&a.value));
    ranked.truncate(3);
    ranked
}

fn scout_grade(c: &Cyclist) -> &'static str {
    let age     = c.age;
    let stars   = c.potential;
    let upside  = c.growth;
    let ca      = c.current_ability;
    if age > 0 && age <= 20 && stars >= 5.5 && upside >= 6.0 { return "Wonderkid"; }
    if age > 0 && age <= 22 && stars >= 5.0 && upside >= 4.5 { return "Elite Prospect"; }
    if age > 0 && age <= 23 && upside >= 5.0 && ca < 72.0    { return "Late Bloomer"; }
    if age > 0 && age <= 23 && ca >= 72.0                     { return "Ready Now"; }
    "Monitor"
}

fn is_free_agent(team_id: i32) -> bool {
    team_id == FREE_AGENT_TEAM_ID || team_id <= 0
}

fn is_placeholder(c: &Cyclist) -> bool {
    let name_placeholder = c.name.starts_with('#') && c.name[1..].chars().all(|ch| ch.is_ascii_digit());
    let team_placeholder = c.team.starts_with("Team #") || (c.team_short.starts_with('#') && c.team_short[1..].chars().all(|ch| ch.is_ascii_digit()));
    let broken = looks_like_garbage(&c.name) && (c.team.is_empty() || team_placeholder) && c.nationality == "Unknown";
    let impossible = c.age == 0 && c.current_ability <= 0.0
        && (name_placeholder || c.team_id > 10000 || c.nationality == "Unknown" || team_placeholder);
    (name_placeholder && team_placeholder && c.nationality == "Unknown") || broken || impossible
}

// ─── Rider type ───────────────────────────────────────────────────────────────
fn rider_type(id: i32) -> &'static str {
    match id {
        1=>"All-Rounder",2=>"Climber",3=>"Time Trialist",4=>"Sprinter",
        5=>"Ardennes",6=>"Classics",7=>"Rouleur",_=>"?",
    }
}

// ─── Main extractor ───────────────────────────────────────────────────────────
pub fn extract(path: &str) -> Result<SaveData, String> {
    // ── Decompress ──────────────────────────────────────────────────────────
    let raw = std::fs::read(path).map_err(|e| format!("Cannot read file: {e}"))?;
    if raw.len() < 0x0C { return Err("File too short".into()); }
    let mut decompressed = Vec::new();
    ZlibDecoder::new(&raw[0x0C..])
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("Decompression failed: {e}"))?;
    let db = &decompressed;

    // ── Game date ────────────────────────────────────────────────────────────
    let save_date_col = find_col(db, OFF_SAVE_META_POS, OFF_SAVE_META_END, b"gene_i_date");
    let game_date = save_date_col
        .and_then(|col| (0..8).map(|i| parse_date_int(read_i32(db, col + i*4))).find(|d| d.is_some()))
        .flatten()
        .unwrap_or(Date { year: 2033, month: 8, day: 5 });

    // ── Country codes ─────────────────────────────────────────────────────────
    let raw_country_codes = read_nullterm(db, OFF_COUNTRY_CODES, COUNTRY_COUNT);
    let country_map: HashMap<i32, String> = raw_country_codes.iter().enumerate()
        .map(|(i, code)| ((i as i32) + 2, normalize_iso(code)))
        .collect();

    // ── Regions → countries ───────────────────────────────────────────────────
    let region_id_col      = find_col(db, OFF_REGION_POS, OFF_REGION_END, b"IDregion");
    let region_country_col = find_col(db, OFF_REGION_POS, OFF_REGION_END, b"fkIDcountry");
    let region_ids:      Vec<i32> = region_id_col.map(|c| read_ints(db, c, REGION_COUNT)).unwrap_or_default();
    let region_countries:Vec<i32> = region_country_col.map(|c| read_ints(db, c, REGION_COUNT)).unwrap_or_default();
    let region_to_country: HashMap<i32, i32> = region_ids.iter().zip(region_countries.iter())
        .filter(|(&r, &c)| r > 0 && c > 0)
        .map(|(&r, &c)| (r, c))
        .collect();

    // ── Teams ─────────────────────────────────────────────────────────────────
    let team_id_col      = find_col(db, OFF_TEAM_REGION, OFF_TEAM_REGION_END, b"IDteam");
    let team_country_col = find_col(db, OFF_TEAM_REGION, OFF_TEAM_REGION_END, b"fkIDcountry");
    let team_ids:        Vec<u32> = team_id_col.map(|c| read_uints(db, c, TEAM_COUNT)).unwrap_or_else(|| vec![0; TEAM_COUNT]);
    let team_countries:  Vec<i32> = team_country_col.map(|c| read_ints(db, c, TEAM_COUNT)).unwrap_or_else(|| vec![0; TEAM_COUNT]);
    let team_names  = read_strings(db, OFF_TEAM_NAME_LEN,  OFF_TEAM_NAME_DATA,  TEAM_COUNT);
    let team_shorts = read_strings(db, OFF_TEAM_SHORT_LEN, OFF_TEAM_SHORT_DATA, TEAM_COUNT);

    let mut teams_map: HashMap<i32, (String, String, String)> = HashMap::new();
    for i in 0..TEAM_COUNT {
        let tid = team_ids[i] as i32;
        if tid <= 0 || tid >= 10000 || team_names[i].is_empty() { continue; }
        let iso  = country_map.get(&team_countries[i]).cloned().unwrap_or_default();
        teams_map.insert(tid, (team_names[i].clone(), team_shorts[i].clone(), iso));
    }

    // ── Read all cyclist columns ──────────────────────────────────────────────
    let ids        = read_ints(db,   OFF_ID,          NUM_CYCLISTS);
    let team_refs  = read_ints(db,   OFF_TEAM,        NUM_CYCLISTS);
    let regions    = read_ints(db,   OFF_REGION,      NUM_CYCLISTS);
    let birthdates = read_ints(db,   OFF_BIRTHDATE,   NUM_CYCLISTS);
    let type_ids   = read_ints(db,   OFF_TYPE_RIDER,  NUM_CYCLISTS);
    let sizes      = read_ints(db,   OFF_SIZE,        NUM_CYCLISTS);
    let weights    = read_ints(db,   OFF_WEIGHT,      NUM_CYCLISTS);
    let potentials = read_floats(db, OFF_POTENTIAL,   NUM_CYCLISTS);
    let abilities  = read_floats(db, OFF_CURRENT_ABILITY, NUM_CYCLISTS);

    let s_flat   = read_ints(db, OFF_FLAT,        NUM_CYCLISTS); let s_flat_p   = read_ints(db, OFF_FLAT_P,        NUM_CYCLISTS);
    let s_mtn    = read_ints(db, OFF_MTN,         NUM_CYCLISTS); let s_mtn_p    = read_ints(db, OFF_MTN_P,         NUM_CYCLISTS);
    let s_med    = read_ints(db, OFF_MED_MTN,     NUM_CYCLISTS); let s_med_p    = read_ints(db, OFF_MED_MTN_P,     NUM_CYCLISTS);
    let s_dh     = read_ints(db, OFF_DOWNHILL,    NUM_CYCLISTS); let s_dh_p     = read_ints(db, OFF_DOWNHILL_P,    NUM_CYCLISTS);
    let s_cob    = read_ints(db, OFF_COBBLE,      NUM_CYCLISTS); let s_cob_p    = read_ints(db, OFF_COBBLE_P,      NUM_CYCLISTS);
    let s_tt     = read_ints(db, OFF_TT,          NUM_CYCLISTS); let s_tt_p     = read_ints(db, OFF_TT_P,          NUM_CYCLISTS);
    let s_pro    = read_ints(db, OFF_PROLOGUE,    NUM_CYCLISTS); let s_pro_p    = read_ints(db, OFF_PROLOGUE_P,    NUM_CYCLISTS);
    let s_spr    = read_ints(db, OFF_SPRINT,      NUM_CYCLISTS); let s_spr_p    = read_ints(db, OFF_SPRINT_P,      NUM_CYCLISTS);
    let s_acc    = read_ints(db, OFF_ACCEL,       NUM_CYCLISTS); let s_acc_p    = read_ints(db, OFF_ACCEL_P,       NUM_CYCLISTS);
    let s_end    = read_ints(db, OFF_ENDURANCE,   NUM_CYCLISTS); let s_end_p    = read_ints(db, OFF_ENDURANCE_P,   NUM_CYCLISTS);
    let s_res    = read_ints(db, OFF_RESISTANCE,  NUM_CYCLISTS); let s_res_p    = read_ints(db, OFF_RESISTANCE_P,  NUM_CYCLISTS);
    let s_rec    = read_ints(db, OFF_RECUP,       NUM_CYCLISTS); let s_rec_p    = read_ints(db, OFF_RECUP_P,       NUM_CYCLISTS);
    let s_hil    = read_ints(db, OFF_HILL,        NUM_CYCLISTS); let s_hil_p    = read_ints(db, OFF_HILL_P,        NUM_CYCLISTS);
    let s_bar    = read_ints(db, OFF_BAROUDEUR,   NUM_CYCLISTS); let s_bar_p    = read_ints(db, OFF_BAROUDEUR_P,   NUM_CYCLISTS);

    let lastnames  = read_strings(db, OFF_LASTNAME_LEN,  OFF_LASTNAME_DATA,  NUM_CYCLISTS);
    let firstnames = read_strings(db, OFF_FIRSTNAME_LEN, OFF_FIRSTNAME_DATA, NUM_CYCLISTS);
    let fullnames  = read_strings(db, OFF_FULLNAME_LEN,  OFF_FULLNAME_DATA,  NUM_CYCLISTS);

    // ── Build cyclists ────────────────────────────────────────────────────────
    let clamp = |v: i32| v.max(0).min(100);
    let mut cyclists: Vec<Cyclist> = Vec::new();

    for i in 0..NUM_CYCLISTS {
        let id = ids[i];
        if id <= 0 || id > 200_000 { continue; }

        let team_id  = team_refs[i];
        let (team_name, team_short, team_iso) = if let Some(t) = teams_map.get(&team_id) {
            (t.0.clone(), t.1.clone(), t.2.clone())
        } else if is_free_agent(team_id) {
            ("Free Agent Pool".into(), "Free Agent".into(), String::new())
        } else {
            (format!("Team #{team_id}"), format!("#{team_id}"), String::new())
        };

        let region_id  = regions[i];
        let country_id = region_to_country.get(&region_id).copied()
            .or_else(|| if region_id > 100 { Some(region_id / 100) } else { None })
            .unwrap_or(0);
        let iso_raw    = country_map.get(&country_id).cloned()
            .unwrap_or_else(|| country_map.get(&teams_map.get(&team_id).map(|_| 0).unwrap_or(0)).cloned().unwrap_or_default());
        let iso = normalize_iso(&iso_raw);

        let cname = if country_name(&iso).is_empty() {
            if iso.is_empty() { "Unknown".to_string() } else { iso.to_uppercase() }
        } else { country_name(&iso).to_string() };

        let flag  = flag_emoji(&iso);
        let nat_display = if flag.is_empty() { cname.clone() } else { format!("{} {}", flag, cname) };

        let age = age_at(birthdates[i], &game_date);

        let name = choose_name(&firstnames[i], &lastnames[i], &fullnames[i], id);

        let ca   = (abilities[i] * 10.0).round() / 10.0;
        let pot  = (potentials[i] * 100.0).round() / 100.0;

        let mut c = Cyclist {
            id, name, firstname: firstnames[i].clone(), lastname: lastnames[i].clone(),
            team_id, team: team_name, team_short,
            nationality: cname.clone(), continent: continent(&iso).to_string(),
            iso: iso.clone(), birthdate: birthdates[i].to_string(), age,
            rider_type: rider_type(type_ids[i]).to_string(), rider_type_id: type_ids[i],
            size: sizes[i], weight: weights[i],
            current_ability: ca, potential: pot,
            growth: 0.0, skill_average: 0.0, skill_ceiling: 0.0, peak_gap: 0, specialty_rating: 0,
            scout_grade: String::new(), free_agent: is_free_agent(team_id),
            flat: clamp(s_flat[i]),     flat_p: clamp(s_flat_p[i]),
            mountain: clamp(s_mtn[i]),  mountain_p: clamp(s_mtn_p[i]),
            med_mtn: clamp(s_med[i]),   med_mtn_p: clamp(s_med_p[i]),
            downhill: clamp(s_dh[i]),   downhill_p: clamp(s_dh_p[i]),
            cobble: clamp(s_cob[i]),    cobble_p: clamp(s_cob_p[i]),
            timetrial: clamp(s_tt[i]),  timetrial_p: clamp(s_tt_p[i]),
            prologue: clamp(s_pro[i]),  prologue_p: clamp(s_pro_p[i]),
            sprint: clamp(s_spr[i]),    sprint_p: clamp(s_spr_p[i]),
            acceleration: clamp(s_acc[i]), acceleration_p: clamp(s_acc_p[i]),
            endurance: clamp(s_end[i]), endurance_p: clamp(s_end_p[i]),
            resistance: clamp(s_res[i]),resistance_p: clamp(s_res_p[i]),
            recuperation: clamp(s_rec[i]), recuperation_p: clamp(s_rec_p[i]),
            hill: clamp(s_hil[i]),      hill_p: clamp(s_hil_p[i]),
            baroudeur: clamp(s_bar[i]), baroudeur_p: clamp(s_bar_p[i]),
            top_skills: vec![],
        };

        let avg  = (stat_avg(&c) * 10.0).round() / 10.0;
        let ceil = (stat_ceil(&c) * 10.0).round() / 10.0;
        c.skill_average = avg;
        c.skill_ceiling = ceil;
        c.growth = ((ceil - avg).max(0.0) * 10.0).round() / 10.0;
        c.peak_gap = peak_gap(&c);
        c.top_skills = top_skills(&c);
        c.specialty_rating = c.top_skills.iter().map(|s| s.value).sum();
        c.scout_grade = scout_grade(&c).to_string();

        if !is_placeholder(&c) {
            cyclists.push(c);
        }
    }

    cyclists.sort_by(|a, b| b.current_ability.partial_cmp(&a.current_ability).unwrap_or(std::cmp::Ordering::Equal));

    // ── Build teams list ──────────────────────────────────────────────────────
    let mut teams: Vec<Team> = teams_map.iter().filter_map(|(&tid, (name, short, iso))| {
        if name.is_empty() { return None; }
        let cname = if country_name(iso).is_empty() { iso.to_uppercase() } else { country_name(iso).to_string() };
        Some(Team { id: tid, name: name.clone(), short: short.clone(), country_iso: iso.clone(), country_name: cname })
    }).collect();
    teams.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(SaveData {
        cyclists,
        teams,
        game_date: date_text(&game_date),
    })
}
