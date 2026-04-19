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
    pub color1: String,
    pub color2: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub cyclists:  Vec<Cyclist>,
    pub scout_reports: Vec<Cyclist>,
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

fn count_sequential_ids(data: &[u8], start: usize, max_rows: usize) -> usize {
    if start == 0 || start + 4 > data.len() {
        return 0;
    }

    let mut count = 0usize;
    for i in 0..max_rows {
        let off = start + i * 4;
        if off + 4 > data.len() {
            break;
        }
        if read_i32(data, off) != (i as i32) + 1 {
            break;
        }
        count += 1;
    }
    count
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

fn find_cols(data: &[u8], start: usize, end: usize, name: &[u8]) -> Vec<usize> {
    let mut pos = start;
    let mut out = Vec::new();
    while pos < end {
        let Some(idx) = find_pattern(data, &AA, pos, end) else { break };
        if idx + 24 < data.len() {
            let block_type = read_u32(data, idx + 8);
            let header_num = read_u32(data, idx + 16);
            if block_type == 0x20 && header_num == 1 {
                let name_len = read_u32(data, idx + 20) as usize;
                if name_len > 0 && name_len < 100 && idx + 24 + name_len <= data.len() {
                    let found = &data[idx + 24 .. idx + 24 + name_len];
                    let found = found.split(|&b| b == 0).next().unwrap_or(found);
                    if found == name {
                        if let Some(col) = find_type22_data(data, idx, 800) {
                            out.push(col);
                        }
                    }
                }
            }
        }
        pos = idx + 4;
    }
    out
}

fn pick_col(cols: &[usize], min_after: usize, fallback: usize) -> usize {
    cols.iter()
        .copied()
        .find(|&col| col > min_after)
        .or_else(|| cols.first().copied())
        .unwrap_or(fallback)
}

fn string_quality_score(s: &str) -> isize {
    let s = s.trim();
    if s.is_empty() { return -12; }
    if looks_like_garbage(s) { return -25; }

    let mut score = 0isize;
    if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) { score += 6; }
    if s.len() >= 2 { score += 4; }
    if s.len() >= 4 { score += 4; }
    if s.chars().all(|c| c.is_alphabetic() || " .'-".contains(c)) { score += 8; }
    if s.starts_with(' ') || s.ends_with(' ') { score -= 8; }
    if s.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) { score -= 6; }
    score
}

fn is_good_string_start(db: &[u8], len_start: usize, data_start: usize) -> bool {
    let mut pos = data_start;
    let mut good = 0usize;
    let mut total = 0usize;
    for i in 0..40 {
        if len_start + i * 4 + 4 > db.len() { break; }
        let len = read_u32(db, len_start + i * 4) as usize;
        if len == 0 { continue; }
        total += 1;
        if len > 60 || pos + len > db.len() { return false; }
        // PCM stores null-terminated strings: last byte of the len slot must be 0x00.
        // This is a structural check that's much more reliable than capitalization heuristics.
        if db[pos + len - 1] != 0 { return false; }
        let s = decode_pcm_text(&db[pos..pos + len]);
        pos += len;
        if s.len() >= 2 && !looks_like_garbage(&s) {
            good += 1;
        }
        if total >= 20 { break; }
    }
    total >= 8 && good * 100 / total >= 70
}

fn string_start_score(db: &[u8], len_start: usize, data_start: usize) -> isize {
    let mut pos = data_start;
    let mut total = 0usize;
    let mut score = 0isize;

    for i in 0..40 {
        if len_start + i * 4 + 4 > db.len() { break; }
        let len = read_u32(db, len_start + i * 4) as usize;
        if len == 0 { continue; }
        if len > 80 || pos + len > db.len() || db[pos + len - 1] != 0 {
            return -10_000;
        }
        let s = decode_pcm_text(&db[pos..pos + len]);
        pos += len;
        total += 1;
        score += string_quality_score(&s);
        if total >= 24 { break; }
    }

    if total < 8 { -10_000 } else { score }
}

fn string_data_start(db: &[u8], len_start: usize, count: usize) -> usize {
    let guessed = len_start + count * 4 + 4;
    let mut best_start = guessed.min(db.len().saturating_sub(1));
    let mut best_score = isize::MIN;

    // Most saves place string data just after the len table, but some cloud saves
    // are shifted by a few bytes. Search a small window around the estimate.
    let from = guessed.saturating_sub(96);
    let to = (guessed + 96).min(db.len().saturating_sub(1));
    for candidate in from..=to {
        let score = string_start_score(db, len_start, candidate);
        if score > best_score {
            best_score = score;
            best_start = candidate;
        }
    }

    if best_score > -10_000 {
        return best_start;
    }

    let end_of_lens = len_start + count * 4;
    if let Some(bb_pos) = find_pattern(db, &BB, end_of_lens.saturating_sub(96), end_of_lens + 512) {
        return bb_pos + 4;
    }
    guessed
}

// find_cols can return a position INSIDE the len array (e.g. if find_type22_data
// finds the wrong BB marker). Back up by multiples of 4 until strings look valid.
fn validated_len_col(db: &[u8], raw_col: usize, count: usize) -> usize {
    let data_fwd = string_data_start(db,raw_col, count);
    if is_good_string_start(db, raw_col, data_fwd) { return raw_col; }
    for k in 1..=512usize {
        let candidate = match raw_col.checked_sub(k * 4) { Some(c) => c, None => break };
        let data_try = string_data_start(db,candidate, count);
        if is_good_string_start(db, candidate, data_try) { return candidate; }
    }
    raw_col
}

fn country_code_start_score(db: &[u8], len_start: usize, data_start: usize) -> isize {
    let mut pos = data_start;
    let mut total = 0usize;
    let mut score = 0isize;

    for i in 0..40 {
        if len_start + i * 4 + 4 > db.len() { break; }
        let len = read_u32(db, len_start + i * 4) as usize;
        if len == 0 {
            total += 1;
            if total >= 24 { break; }
            continue;
        }
        if len > 8 || pos + len > db.len() || db[pos + len - 1] != 0 {
            return -10_000;
        }
        let iso = normalize_iso(&decode_pcm_text(&db[pos..pos + len]));
        pos += len;
        total += 1;
        if iso.is_empty() {
            score += 1;
        } else if iso.len() == 3 && iso.chars().all(|c| c.is_ascii_lowercase()) {
            score += 8;
            if country_name(&iso) != "" {
                score += 8;
            }
        } else {
            score -= 10;
        }
        if total >= 24 { break; }
    }

    if total < 8 { -10_000 } else { score }
}

fn country_code_data_start(db: &[u8], len_start: usize, count: usize) -> usize {
    let guessed = len_start + count * 4 + 4;
    let mut best_start = guessed.min(db.len().saturating_sub(1));
    let mut best_score = isize::MIN;

    let from = guessed.saturating_sub(96);
    let to = (guessed + 96).min(db.len().saturating_sub(1));
    for candidate in from..=to {
        let score = country_code_start_score(db, len_start, candidate);
        if score > best_score {
            best_score = score;
            best_start = candidate;
        }
    }

    if best_score > -10_000 {
        return best_start;
    }

    guessed
}

fn score_iso_code_block(db: &[u8], len_col: usize, count: usize) -> isize {
    let codes = read_strings(db, len_col, country_code_data_start(db, len_col, count), count);
    let mut score = 0isize;
    for code in codes.iter().take(24) {
        let iso = normalize_iso(code);
        if iso.len() == 3 && iso.chars().all(|c| c.is_ascii_lowercase()) {
            score += 4;
            if country_name(&iso) != "" {
                score += 6;
            }
        } else if !iso.is_empty() {
            score -= 6;
        }
    }
    score
}

fn pick_country_code_col(db: &[u8], cols: &[usize], min_after: usize, fallback: usize, count: usize) -> usize {
    let mut best_col = fallback;
    let mut best_score = isize::MIN;

    for &col in cols {
        if col <= min_after { continue; }
        let score = score_iso_code_block(db, col, count);
        if score > best_score {
            best_score = score;
            best_col = col;
        }
    }

    if best_score > isize::MIN {
        best_col
    } else {
        fallback
    }
}

fn country_id_base(raw_country_codes: &[String]) -> i32 {
    let first = raw_country_codes
        .first()
        .map(|code| normalize_iso(code))
        .unwrap_or_default();
    let second = raw_country_codes
        .get(1)
        .map(|code| normalize_iso(code))
        .unwrap_or_default();

    // PCM saves usually keep an empty sentinel at index 0, so country id 2 maps to
    // the second entry ("ita" in the stock DB). If that sentinel is missing, fall
    // back to the older +2 layout.
    if first.is_empty() && second.len() == 3 {
        1
    } else {
        2
    }
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

fn name_quality(s: &str) -> usize {
    let trimmed = s.trim();
    if trimmed.is_empty() || looks_like_garbage(trimmed) {
        return 0;
    }
    let words = trimmed.split_whitespace().count();
    let letters = trimmed.chars().filter(|c| c.is_alphabetic()).count();
    let separators = trimmed.chars().filter(|c| matches!(c, ' ' | '-' | '\'')).count();
    words * 1000 + letters * 10 + separators
}

fn choose_name(first: &str, last: &str, full: &str, id: i32) -> String {
    let f = first.trim();
    let l = last.trim();
    let full = full.trim();
    let combined = format!("{} {}", f, l).trim().to_string();
    // Only trust fullname if it actually shares a token with first or last name.
    // If fullname comes from the wrong table it'll be someone else's name.
    let full_correlated = !full.is_empty() && !looks_like_garbage(full) && (
        (f.len() >= 2 && full.to_lowercase().contains(&f.to_lowercase())) ||
        (l.len() >= 2 && full.to_lowercase().contains(&l.to_lowercase()))
    );
    let candidates: &[&str] = if full_correlated {
        &[full, combined.as_str(), f, l]
    } else {
        &[combined.as_str(), f, l]
    };
    let best = candidates.iter()
        .copied()
        .max_by_key(|s| name_quality(s))
        .unwrap_or("");
    if best.is_empty() { format!("#{}", id) } else { best.to_string() }
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
        "ken"|"lba"|"lso"|"mli"|"mar"|"moz"|"nam"|"nga"|"nig"|"rsa"|"rwa"|
        "sen"|"tun"|"uga"|"zim" => "Africa",
        "arm"|"aze"|"bhr"|"brn"|"hkg"|"idn"|"ind"|"irn"|"irq"|"isr"|"jpn"|
        "kaz"|"khm"|"kgz"|"kor"|"kuw"|"lao"|"lka"|"mas"|"mng"|"oma"|"pak"|
        "phl"|"qat"|"sau"|"sgp"|"syr"|"tha"|"tls"|"twn"|"uae"|"uzb"|"vnm" => "Asia",
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
        "ben"=>"BJ","bfa"=>"BF","cod"=>"CD",_ => "",
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
    let age    = c.age;
    let stars  = c.potential;
    let upside = c.growth;
    let ca     = c.current_ability;
    // Youth tiers (genuine development potential)
    if age > 0 && age <= 20 && stars >= 5.5 && upside >= 6.0 { return "Wonderkid"; }
    if age > 0 && age <= 22 && stars >= 5.0 && upside >= 4.5 { return "Elite Prospect"; }
    if age > 0 && age <= 23 && upside >= 5.0 && ca < 72.0    { return "Late Bloomer"; }
    if age > 0 && age <= 23 && ca >= 72.0                     { return "Ready Now"; }
    // Adult tiers — guard against misleading paper potential for older riders
    if age >= 30 && upside >= 3.0  { return "Past Peak"; } // ceiling exists but age makes it unreachable
    if age >= 27 && ca >= 78.0     { return "Veteran"; }
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
    let raw = std::fs::read(path).map_err(|e| format!("Cannot read file: {e}"))?;
    if raw.len() < 0x0C { return Err("File too short".into()); }
    let mut decompressed = Vec::new();
    ZlibDecoder::new(&raw[0x0C..])
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("Decompression failed: {e}"))?;
    let db = &decompressed;

    let save_date_col = find_col(db, 0, db.len(), b"gene_i_date_resolve")
        .or_else(|| find_col(db, OFF_SAVE_META_POS, OFF_SAVE_META_END, b"gene_i_date"))
        .or_else(|| find_col(db, 0, db.len(), b"gene_i_date"));
    let game_date = save_date_col
        .and_then(|col| (0..8).map(|i| parse_date_int(read_i32(db, col + i * 4))).find(|d| d.is_some()))
        .flatten()
        .unwrap_or(Date { year: 2033, month: 8, day: 5 });

    let cyclist_id_col = pick_col(&find_cols(db, 0, db.len(), b"IDcyclist"), 100_000, OFF_ID);
    let team_ref_col = pick_col(&find_cols(db, 0, db.len(), b"fkIDteam"), cyclist_id_col, OFF_TEAM);
    let region_ref_col = pick_col(&find_cols(db, 0, db.len(), b"fkIDregion"), cyclist_id_col, OFF_REGION);
    let birthdate_col = pick_col(&find_cols(db, 0, db.len(), b"gene_i_birthdate"), cyclist_id_col, OFF_BIRTHDATE);
    let rider_type_col = pick_col(&find_cols(db, 0, db.len(), b"fkIDtype_rider"), cyclist_id_col, OFF_TYPE_RIDER);
    let size_col = pick_col(&find_cols(db, 0, db.len(), b"gene_i_size"), cyclist_id_col, OFF_SIZE);
    let weight_col = pick_col(&find_cols(db, 0, db.len(), b"gene_i_weight"), cyclist_id_col, OFF_WEIGHT);
    let potential_col = pick_col(&find_cols(db, 0, db.len(), b"value_f_potentiel"), cyclist_id_col, OFF_POTENTIAL);
    let ability_col = pick_col(&find_cols(db, 0, db.len(), b"value_f_current_ability"), cyclist_id_col, OFF_CURRENT_ABILITY);

    let lastname_len_col = validated_len_col(db, pick_col(&find_cols(db, 0, db.len(), b"gene_sz_lastname"), cyclist_id_col, OFF_LASTNAME_LEN), NUM_CYCLISTS);
    let firstname_len_col = validated_len_col(db, pick_col(&find_cols(db, 0, db.len(), b"gene_sz_firstname"), cyclist_id_col, OFF_FIRSTNAME_LEN), NUM_CYCLISTS);
    let fullname_len_col = validated_len_col(db, pick_col(&find_cols(db, 0, db.len(), b"gene_sz_firstlastname"), cyclist_id_col, OFF_FULLNAME_LEN), NUM_CYCLISTS);
    let lastname_data_col = string_data_start(db,lastname_len_col, NUM_CYCLISTS);
    let firstname_data_col = string_data_start(db,firstname_len_col, NUM_CYCLISTS);
    let fullname_data_col = string_data_start(db,fullname_len_col, NUM_CYCLISTS);

    let s_flat_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_plain"), cyclist_id_col, OFF_FLAT);
    let s_flat_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_plain"), cyclist_id_col, OFF_FLAT_P);
    let s_mtn_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_mountain"), cyclist_id_col, OFF_MTN);
    let s_mtn_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_mountain"), cyclist_id_col, OFF_MTN_P);
    let s_med_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_medium_mountain"), cyclist_id_col, OFF_MED_MTN);
    let s_med_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_medium_mountain"), cyclist_id_col, OFF_MED_MTN_P);
    let s_dh_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_downhilling"), cyclist_id_col, OFF_DOWNHILL);
    let s_dh_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_downhilling"), cyclist_id_col, OFF_DOWNHILL_P);
    let s_cob_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_cobble"), cyclist_id_col, OFF_COBBLE);
    let s_cob_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_cobble"), cyclist_id_col, OFF_COBBLE_P);
    let s_tt_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_timetrial"), cyclist_id_col, OFF_TT);
    let s_tt_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_timetrial"), cyclist_id_col, OFF_TT_P);
    let s_pro_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_prologue"), cyclist_id_col, OFF_PROLOGUE);
    let s_pro_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_prologue"), cyclist_id_col, OFF_PROLOGUE_P);
    let s_spr_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_sprint"), cyclist_id_col, OFF_SPRINT);
    let s_spr_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_sprint"), cyclist_id_col, OFF_SPRINT_P);
    let s_acc_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_acceleration"), cyclist_id_col, OFF_ACCEL);
    let s_acc_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_acceleration"), cyclist_id_col, OFF_ACCEL_P);
    let s_end_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_endurance"), cyclist_id_col, OFF_ENDURANCE);
    let s_end_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_endurance"), cyclist_id_col, OFF_ENDURANCE_P);
    let s_res_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_resistance"), cyclist_id_col, OFF_RESISTANCE);
    let s_res_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_resistance"), cyclist_id_col, OFF_RESISTANCE_P);
    let s_rec_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_recuperation"), cyclist_id_col, OFF_RECUP);
    let s_rec_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_recuperation"), cyclist_id_col, OFF_RECUP_P);
    let s_hil_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_hill"), cyclist_id_col, OFF_HILL);
    let s_hil_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_hill"), cyclist_id_col, OFF_HILL_P);
    let s_bar_col = pick_col(&find_cols(db, 0, db.len(), b"charac_i_baroudeur"), cyclist_id_col, OFF_BAROUDEUR);
    let s_bar_p_col = pick_col(&find_cols(db, 0, db.len(), b"limit_i_baroudeur"), cyclist_id_col, OFF_BAROUDEUR_P);

    let team_id_col = pick_col(&find_cols(db, 0, db.len(), b"IDteam"), 8_000_000, 0);
    let team_short_len_col = validated_len_col(db, pick_col(&find_cols(db, 0, db.len(), b"gene_sz_shortname"), team_id_col, OFF_TEAM_SHORT_LEN), TEAM_COUNT);
    let team_name_len_col = validated_len_col(db, pick_col(&find_cols(db, 0, db.len(), b"gene_sz_name"), team_id_col, OFF_TEAM_NAME_LEN), TEAM_COUNT);
    let team_country_col = pick_col(&find_cols(db, 0, db.len(), b"fkIDcountry"), team_id_col, 0);
    let team_short_data_col = string_data_start(db,team_short_len_col, TEAM_COUNT);
    let team_name_data_col = string_data_start(db,team_name_len_col, TEAM_COUNT);

    let country_id_col = pick_col(&find_cols(db, 0, db.len(), b"IDcountry"), team_id_col, 0);
    let country_code_len_col = pick_country_code_col(
        db,
        &find_cols(db, 0, db.len(), b"CONSTANT"),
        country_id_col,
        OFF_COUNTRY_CODES - COUNTRY_COUNT * 4 - 4,
        COUNTRY_COUNT,
    );
    let country_code_data_col = country_code_data_start(db, country_code_len_col, COUNTRY_COUNT);
    let raw_country_codes = read_strings(db, country_code_len_col, country_code_data_col, COUNTRY_COUNT);
    let country_base = country_id_base(&raw_country_codes);
    let country_map = raw_country_codes.iter().enumerate()
        .map(|(i, code)| ((i as i32) + country_base, normalize_iso(code)))
        .collect::<HashMap<i32, String>>();

    let region_id_col = pick_col(&find_cols(db, 0, db.len(), b"IDregion"), country_id_col, 0);
    let region_country_col = pick_col(&find_cols(db, 0, db.len(), b"fkIDcountry"), region_id_col, 0);
    let region_ids = if region_id_col > 0 { read_ints(db, region_id_col, REGION_COUNT) } else { vec![] };
    let region_countries = if region_country_col > 0 { read_ints(db, region_country_col, REGION_COUNT) } else { vec![] };
    let region_to_country = region_ids.iter().zip(region_countries.iter())
        .filter(|(&r, &c)| r > 0 && c > 0)
        .map(|(&r, &c)| (r, c))
        .collect::<HashMap<i32, i32>>();

    let team_ids = if team_id_col > 0 { read_uints(db, team_id_col, TEAM_COUNT) } else { vec![0; TEAM_COUNT] };
    let team_countries = if team_country_col > 0 { read_ints(db, team_country_col, TEAM_COUNT) } else { vec![0; TEAM_COUNT] };
    let team_names = read_strings(db, team_name_len_col, team_name_data_col, TEAM_COUNT);
    let team_shorts = read_strings(db, team_short_len_col, team_short_data_col, TEAM_COUNT);

    // Try several known PCM color field names; use first that resolves to a position past team_id_col
    let find_team_col = |names: &[&[u8]]| -> usize {
        for &n in names {
            let cols = find_cols(db, 0, db.len(), n);
            let c = pick_col(&cols, team_id_col, 0);
            if c > 0 { return c; }
        }
        0
    };
    let team_c1_col = find_team_col(&[b"gene_i_color1", b"gene_i_couleur1", b"color_i_1", b"gene_i_colorkit1"]);
    let team_c2_col = find_team_col(&[b"gene_i_color2", b"gene_i_couleur2", b"color_i_2", b"gene_i_colorkit2"]);
    let team_c1 = if team_c1_col > 0 { read_ints(db, team_c1_col, TEAM_COUNT) } else { vec![0; TEAM_COUNT] };
    let team_c2 = if team_c2_col > 0 { read_ints(db, team_c2_col, TEAM_COUNT) } else { vec![0; TEAM_COUNT] };
    eprintln!("[colors] c1_col={team_c1_col} c2_col={team_c2_col} first5={:?}", team_c1.iter().take(5).collect::<Vec<_>>());

    let bgr_to_hex = |v: i32| -> String {
        if v <= 0 { return String::new(); }
        let r = (v & 0xFF) as u8;
        let g = ((v >> 8) & 0xFF) as u8;
        let b = ((v >> 16) & 0xFF) as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    };

    let mut teams_map = HashMap::new();
    for i in 0..TEAM_COUNT {
        let tid = team_ids[i] as i32;
        if tid <= 0 || tid >= 10000 || team_names[i].is_empty() { continue; }
        let iso = country_map.get(&team_countries[i]).cloned().unwrap_or_default();
        let c1 = bgr_to_hex(team_c1[i]);
        let c2 = bgr_to_hex(team_c2[i]);
        teams_map.insert(tid, (team_names[i].clone(), team_shorts[i].clone(), iso, c1, c2));
    }

    let ids = read_ints(db, cyclist_id_col, NUM_CYCLISTS);
    let team_refs = read_ints(db, team_ref_col, NUM_CYCLISTS);
    let regions = read_ints(db, region_ref_col, NUM_CYCLISTS);
    let birthdates = read_ints(db, birthdate_col, NUM_CYCLISTS);
    let type_ids = read_ints(db, rider_type_col, NUM_CYCLISTS);
    let sizes = read_ints(db, size_col, NUM_CYCLISTS);
    let weights = read_ints(db, weight_col, NUM_CYCLISTS);
    let potentials = read_floats(db, potential_col, NUM_CYCLISTS);
    let abilities = read_floats(db, ability_col, NUM_CYCLISTS);

    let s_flat = read_ints(db, s_flat_col, NUM_CYCLISTS); let s_flat_p = read_ints(db, s_flat_p_col, NUM_CYCLISTS);
    let s_mtn = read_ints(db, s_mtn_col, NUM_CYCLISTS); let s_mtn_p = read_ints(db, s_mtn_p_col, NUM_CYCLISTS);
    let s_med = read_ints(db, s_med_col, NUM_CYCLISTS); let s_med_p = read_ints(db, s_med_p_col, NUM_CYCLISTS);
    let s_dh = read_ints(db, s_dh_col, NUM_CYCLISTS); let s_dh_p = read_ints(db, s_dh_p_col, NUM_CYCLISTS);
    let s_cob = read_ints(db, s_cob_col, NUM_CYCLISTS); let s_cob_p = read_ints(db, s_cob_p_col, NUM_CYCLISTS);
    let s_tt = read_ints(db, s_tt_col, NUM_CYCLISTS); let s_tt_p = read_ints(db, s_tt_p_col, NUM_CYCLISTS);
    let s_pro = read_ints(db, s_pro_col, NUM_CYCLISTS); let s_pro_p = read_ints(db, s_pro_p_col, NUM_CYCLISTS);
    let s_spr = read_ints(db, s_spr_col, NUM_CYCLISTS); let s_spr_p = read_ints(db, s_spr_p_col, NUM_CYCLISTS);
    let s_acc = read_ints(db, s_acc_col, NUM_CYCLISTS); let s_acc_p = read_ints(db, s_acc_p_col, NUM_CYCLISTS);
    let s_end = read_ints(db, s_end_col, NUM_CYCLISTS); let s_end_p = read_ints(db, s_end_p_col, NUM_CYCLISTS);
    let s_res = read_ints(db, s_res_col, NUM_CYCLISTS); let s_res_p = read_ints(db, s_res_p_col, NUM_CYCLISTS);
    let s_rec = read_ints(db, s_rec_col, NUM_CYCLISTS); let s_rec_p = read_ints(db, s_rec_p_col, NUM_CYCLISTS);
    let s_hil = read_ints(db, s_hil_col, NUM_CYCLISTS); let s_hil_p = read_ints(db, s_hil_p_col, NUM_CYCLISTS);
    let s_bar = read_ints(db, s_bar_col, NUM_CYCLISTS); let s_bar_p = read_ints(db, s_bar_p_col, NUM_CYCLISTS);

    let lastnames  = read_strings(db, lastname_len_col, lastname_data_col, NUM_CYCLISTS);
    let firstnames = read_strings(db, firstname_len_col, firstname_data_col, NUM_CYCLISTS);
    let fullnames  = read_strings(db, fullname_len_col, fullname_data_col, NUM_CYCLISTS);

    let clamp = |v: i32| v.max(0).min(100);
    let mut cyclists: Vec<Cyclist> = Vec::new();
    let mut cyclist_rows: Vec<Option<Cyclist>> = vec![None; NUM_CYCLISTS];

    for i in 0..NUM_CYCLISTS {
        let id = ids[i];
        if id <= 0 || id > 200_000 { continue; }

        let team_id = team_refs[i];
        let (team_name, team_short, team_iso) = if let Some(t) = teams_map.get(&team_id) {
            (t.0.clone(), t.1.clone(), t.2.clone())
        } else if is_free_agent(team_id) {
            ("Free Agent Pool".into(), "Free Agent".into(), String::new())
        } else {
            (format!("Team #{team_id}"), format!("#{team_id}"), String::new())
        };


        let region_id = regions[i];
        let country_id = region_to_country.get(&region_id).copied()
            .or_else(|| if region_id > 100 { Some(region_id / 100) } else { None })
            .unwrap_or(0);
        let iso_raw = country_map
            .get(&country_id)
            .cloned()
            .filter(|iso| !iso.is_empty())
            .or_else(|| (!team_iso.is_empty()).then_some(team_iso.clone()))
            .unwrap_or_default();
        let iso = normalize_iso(&iso_raw);

        let cname = if country_name(&iso).is_empty() {
            if iso.is_empty() { "Unknown".to_string() } else { iso.to_uppercase() }
        } else {
            country_name(&iso).to_string()
        };

        let age = age_at(birthdates[i], &game_date);
        let name = choose_name(&firstnames[i], &lastnames[i], &fullnames[i], id);
        let ca = (abilities[i] * 10.0).round() / 10.0;
        let pot = (potentials[i] * 100.0).round() / 100.0;

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
            flat: clamp(s_flat[i]), flat_p: clamp(s_flat_p[i]),
            mountain: clamp(s_mtn[i]), mountain_p: clamp(s_mtn_p[i]),
            med_mtn: clamp(s_med[i]), med_mtn_p: clamp(s_med_p[i]),
            downhill: clamp(s_dh[i]), downhill_p: clamp(s_dh_p[i]),
            cobble: clamp(s_cob[i]), cobble_p: clamp(s_cob_p[i]),
            timetrial: clamp(s_tt[i]), timetrial_p: clamp(s_tt_p[i]),
            prologue: clamp(s_pro[i]), prologue_p: clamp(s_pro_p[i]),
            sprint: clamp(s_spr[i]), sprint_p: clamp(s_spr_p[i]),
            acceleration: clamp(s_acc[i]), acceleration_p: clamp(s_acc_p[i]),
            endurance: clamp(s_end[i]), endurance_p: clamp(s_end_p[i]),
            resistance: clamp(s_res[i]), resistance_p: clamp(s_res_p[i]),
            recuperation: clamp(s_rec[i]), recuperation_p: clamp(s_rec_p[i]),
            hill: clamp(s_hil[i]), hill_p: clamp(s_hil_p[i]),
            baroudeur: clamp(s_bar[i]), baroudeur_p: clamp(s_bar_p[i]),
            top_skills: vec![],
        };

        let avg = (stat_avg(&c) * 10.0).round() / 10.0;
        let ceil = (stat_ceil(&c) * 10.0).round() / 10.0;
        c.skill_average = avg;
        c.skill_ceiling = ceil;
        c.growth = ((ceil - avg).max(0.0) * 10.0).round() / 10.0;
        c.peak_gap = peak_gap(&c);
        c.top_skills = top_skills(&c);
        c.specialty_rating = c.top_skills.iter().map(|s| s.value).sum();
        c.scout_grade = scout_grade(&c).to_string();

        if !is_placeholder(&c) {
            cyclist_rows[i] = Some(c.clone());
            cyclists.push(c);
        }
    }

    let scout_row_col = pick_col(&find_cols(db, 0, db.len(), b"IDscout_cyclist"), 0, 0);
    let scout_row_count = count_sequential_ids(db, scout_row_col, NUM_CYCLISTS);
    let mut scout_reports = Vec::new();
    let mut seen_scout_ids = std::collections::HashSet::new();

    for scout_row in 0..scout_row_count {
        if let Some(c) = cyclist_rows.get(scout_row).and_then(|row| row.as_ref()) {
            if seen_scout_ids.insert(c.id) {
                scout_reports.push(c.clone());
            }
        }
    }

    cyclists.sort_by(|a, b| b.current_ability.partial_cmp(&a.current_ability).unwrap_or(std::cmp::Ordering::Equal));
    scout_reports.sort_by(|a, b| {
        b.potential
            .partial_cmp(&a.potential)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.growth.partial_cmp(&a.growth).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.age.cmp(&b.age))
    });

    let mut teams: Vec<Team> = teams_map.iter().filter_map(|(&tid, (name, short, iso, c1, c2))| {
        if name.is_empty() { return None; }
        let cname = if country_name(iso).is_empty() { iso.to_uppercase() } else { country_name(iso).to_string() };
        Some(Team { id: tid, name: name.clone(), short: short.clone(), country_iso: iso.clone(), country_name: cname, color1: c1.clone(), color2: c2.clone() })
    }).collect();
    teams.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(SaveData {
        cyclists,
        scout_reports,
        teams,
        game_date: date_text(&game_date),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diag_name_bytes() {
        let path = "C:/Users/YoelM/AppData/Roaming/Pro Cycling Manager 2025/Cloud/76561198036384740/Career_Z.cdb";
        let raw = std::fs::read(path).unwrap();
        let mut db = Vec::new();
        ZlibDecoder::new(&raw[0x0C..]).read_to_end(&mut db).unwrap();

        let cyclist_id_col = pick_col(&find_cols(&db, 0, db.len(), b"IDcyclist"), 100_000, OFF_ID);
        let raw_ln = pick_col(&find_cols(&db, 0, db.len(), b"gene_sz_lastname"), cyclist_id_col, OFF_LASTNAME_LEN);
        let raw_fn = pick_col(&find_cols(&db, 0, db.len(), b"gene_sz_firstname"), cyclist_id_col, OFF_FIRSTNAME_LEN);

        let ln_col = validated_len_col(&db, raw_ln, NUM_CYCLISTS);
        let fn_col = validated_len_col(&db, raw_fn, NUM_CYCLISTS);
        let ln_data = string_data_start(&db, ln_col, NUM_CYCLISTS);
        let fn_data = string_data_start(&db, fn_col, NUM_CYCLISTS);

        println!("ln_data=0x{:X}  fn_data=0x{:X}", ln_data, fn_data);

        // Find "Bekele" in the decompressed buffer
        for needle in &[b"Bekele" as &[u8], b"Mosca", b"Luigi", b"Eskender"] {
            if let Some(pos) = db.windows(needle.len()).position(|w| w == *needle) {
                let name_str = std::str::from_utf8(needle).unwrap();
                // How many null bytes are between ln_data and this position?
                let nulls = db[ln_data..pos].iter().filter(|&&b| b == 0).count();
                println!("{:?} found at 0x{:X}, offset from ln_data={}, nulls_before={}", name_str, pos, pos as i64 - ln_data as i64, nulls);
            }
        }

        // Show bytes at ln_data and fn_data around the problematic idx range
        let ids = read_ints(&db, cyclist_id_col, NUM_CYCLISTS);
        // Find idx of id=7708
        if let Some(idx) = ids.iter().position(|&id| id == 7708) {
            println!("\nid=7708 at idx={}", idx);
            // Compute the byte offset to this entry's name in ln_data
            let mut pos = ln_data;
            for i in 0..idx {
                let end = db[pos..].iter().position(|&b| b == 0).map(|e| pos+e).unwrap_or(pos);
                pos = end + 1;
            }
            let end = db[pos..].iter().position(|&b| b == 0).map(|e| pos+e).unwrap_or(pos);
            println!("  lastname at 0x{:X}: {:?}", pos, std::str::from_utf8(&db[pos..end]).unwrap_or("?"));
            // Show 20 bytes before and after that position in the buffer
            let show_start = pos.saturating_sub(10);
            println!("  context (lastname data around 0x{:X}): {:?}", show_start, &db[show_start..show_start.min(db.len()).max(pos+30).min(db.len())]);
        }
    }

    #[test]
    fn diag_extract_z() {
        let path = "C:/Users/YoelM/AppData/Roaming/Pro Cycling Manager 2025/Cloud/76561198036384740/Career_Z.cdb";
        let data = extract(path).unwrap();
        println!("total cyclists: {}", data.cyclists.len());
        // Find the cyclist the user sees as "Luigi Mosca" Ethiopian climber
        for target in &[7703i32, 7708, 7598] {
            match data.cyclists.iter().find(|c| c.id == *target) {
                Some(c) => println!("id={}: name={:?} nat={:?} team={:?} ca={} age={}", c.id, c.name, c.nationality, c.team_short, c.current_ability, c.age),
                None => println!("id={}: NOT FOUND", target),
            }
        }
        // Also print first 5 cyclists
        for c in data.cyclists.iter().take(5) {
            println!("  id={} name={:?} nat={:?} age={}", c.id, c.name, c.nationality, c.age);
        }
        // Search by nationality=Ethiopia
        let eth: Vec<_> = data.cyclists.iter().filter(|c| c.nationality == "Ethiopia").collect();
        println!("Ethiopian cyclists: {}", eth.len());
        for c in eth.iter().take(5) {
            println!("  id={} name={:?} team={:?} ca={}", c.id, c.name, c.team_short, c.current_ability);
        }
    }

    #[test]
    fn diag_career_z() {
        let path = "C:/Users/YoelM/AppData/Roaming/Pro Cycling Manager 2025/Cloud/76561198036384740/Career_Z.cdb";
        let raw = std::fs::read(path).unwrap();
        let mut db = Vec::new();
        ZlibDecoder::new(&raw[0x0C..]).read_to_end(&mut db).unwrap();

        let cyclist_id_col = pick_col(&find_cols(&db, 0, db.len(), b"IDcyclist"), 100_000, OFF_ID);
        println!("cyclist_id_col=0x{:X}", cyclist_id_col);

        let raw_ln = pick_col(&find_cols(&db, 0, db.len(), b"gene_sz_lastname"), cyclist_id_col, OFF_LASTNAME_LEN);
        let raw_fn = pick_col(&find_cols(&db, 0, db.len(), b"gene_sz_firstname"), cyclist_id_col, OFF_FIRSTNAME_LEN);
        let raw_fu = pick_col(&find_cols(&db, 0, db.len(), b"gene_sz_firstlastname"), cyclist_id_col, OFF_FULLNAME_LEN);
        println!("raw_lastname_col=0x{:X}  raw_firstname_col=0x{:X}  raw_fullname_col=0x{:X}", raw_ln, raw_fn, raw_fu);

        let ln_col = validated_len_col(&db, raw_ln, NUM_CYCLISTS);
        let fn_col = validated_len_col(&db, raw_fn, NUM_CYCLISTS);
        let fu_col = validated_len_col(&db, raw_fu, NUM_CYCLISTS);
        println!("validated_lastname=0x{:X}  validated_firstname=0x{:X}  validated_fullname=0x{:X}", ln_col, fn_col, fu_col);

        let ln_data = string_data_start(&db, ln_col, NUM_CYCLISTS);
        let fn_data = string_data_start(&db, fn_col, NUM_CYCLISTS);
        let fu_data = string_data_start(&db, fu_col, NUM_CYCLISTS);
        println!("lastname_data=0x{:X}  firstname_data=0x{:X}  fullname_data=0x{:X}", ln_data, fn_data, fu_data);

        // Print first 10 len values at each validated col
        println!("First 5 len[i] at lastname_col:");
        for i in 0..5 { println!("  [{}]={}", i, read_u32(&db, ln_col + i*4)); }
        println!("First 5 len[i] at firstname_col:");
        for i in 0..5 { println!("  [{}]={}", i, read_u32(&db, fn_col + i*4)); }

        // Read first 10 names from each field
        let lns = read_nullterm(&db, ln_data, 15);
        let fns = read_nullterm(&db, fn_data, 15);
        let fus = read_nullterm(&db, fu_data, 15);
        println!("First 15 lastnames: {:?}", lns);
        println!("First 15 firstnames: {:?}", fns);
        println!("First 15 fullnames: {:?}", fus);

        // Look up by name: find "Bekele" or "Milan" in the data buffer
        let ids = read_ints(&db, cyclist_id_col, NUM_CYCLISTS);
        let all_lns = read_nullterm(&db, ln_data, NUM_CYCLISTS);
        let all_fns = read_nullterm(&db, fn_data, NUM_CYCLISTS);
        for (i, (ln, fn_)) in all_lns.iter().zip(all_fns.iter()).enumerate() {
            if ln.to_lowercase().contains("bekele") || fn_.to_lowercase().contains("eskender")
            || fn_.to_lowercase().contains("milan") || ln.to_lowercase().contains("mosca") {
                println!("  idx={} id={} first={:?} last={:?}", i, ids[i], fn_, ln);
            }
        }
    }
}
