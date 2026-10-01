//! Name packs: replace the names PCM uses for newly generated riders of a country,
//! and fix generated riders already in a career save.
//!
//! The generator draws from `STA_regen_firstname` / `STA_regen_lastname` in the game's
//! `Mod\Default\OfficialLocal.cdb`; `fkIDregion` in those tables holds the country id.
use crate::cdb::{decode_text, encode_text, Cdb, Values};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct NamePack {
    pub key: &'static str,
    pub country_iso: &'static str,
    pub label: &'static str,
    /// Modern given names used for young riders' first names.
    pub first: &'static [&'static str],
    /// Traditional names; in Eritrea the family name is the father's or grandfather's name.
    pub last: &'static [&'static str],
    /// Older given names that are fine to keep on existing riders but aren't handed to new ones.
    pub also_valid_first: &'static [&'static str],
}

pub const ERITREA_TIGRINYA: NamePack = NamePack {
    key: "eri-tigrinya",
    country_iso: "eri",
    label: "Eritrea · Tigrinya names",
    first: &[
        "Abel", "Abenezer", "Abiel", "Aklilu", "Aman", "Amanuel", "Aron", "Awet", "Barnabas", "Benhur",
        "Bereket", "Biniam", "Biruk", "Bisrat", "Daniel", "Dawit", "Efrem", "Elias", "Ermias", "Estifanos",
        "Eyob", "Eyuel", "Ezana", "Filmon", "Fitsum", "Freselam", "Gideon", "Haben", "Henok", "Hermon",
        "Huruy", "Isaias", "Kaleab", "Kibrom", "Kidus", "Luel", "Mathewos", "Medhanie", "Merhawi", "Metkel",
        "Mewael", "Michael", "Mikael", "Milkias", "Mussie", "Nahom", "Naod", "Natan", "Natnael", "Okbay",
        "Robel", "Robiel", "Ruben", "Samson", "Samuel", "Selemun", "Semere", "Senai", "Siem", "Simon",
        "Sirak", "Tedros", "Temesgen", "Tesfalem", "Tomas", "Yafet", "Yakob", "Yared", "Yemane", "Yoel",
        "Yohannes", "Yonas", "Yonatan", "Yordanos", "Yosief", "Zekarias",
        "Abraham", "Adonay", "Alazar", "Andom", "Bemnet", "Eyasu", "Eyosias", "Ezra", "Futsum", "Hiyab",
        "Jemal", "Kirubel", "Matiyas", "Meron", "Mikiel", "Paulos", "Petros", "Ruel", "Semir", "Senay",
        "Yoftahe",
    ],
    last: &[
        "Abraha", "Abrehe", "Afewerki", "Andebrhan", "Andemariam", "Andemichael", "Araya", "Arefaine", "Asfaha", "Asmelash",
        "Asmerom", "Bahta", "Berhane", "Berhe", "Beyene", "Debesai", "Desta", "Embaye", "Fessehaye", "Fikre",
        "Gebrehiwet", "Gebrekristos", "Gebremedhin", "Gebremeskel", "Gebremichael", "Gebretinsae", "Gebru", "Ghebreab", "Ghebrekidan", "Ghebremariam",
        "Ghebreyesus", "Ghebrezgabhier", "Ghirmay", "Goitom", "Habte", "Habteab", "Habtemariam", "Habtemichael", "Habtezion", "Habtom",
        "Hagos", "Haile", "Haileab", "Hailemariam", "Hailemichael", "Haileselassie", "Hailu", "Idris", "Kahsay", "Kidane",
        "Kidanemariam", "Kidanu", "Kifle", "Kiflemariam", "Kiflom", "Kiros", "Mebrahtu", "Medhanie", "Mehari", "Mehreteab",
        "Melake", "Mengis", "Mesfin", "Mihretab", "Mulugeta", "Negash", "Negassi", "Neguse", "Nerayo", "Ogbazghi",
        "Okbamichael", "Osman", "Rezene", "Russom", "Saleh", "Sebhatu", "Semere", "Seyoum", "Sium", "Tekeste",
        "Tekie", "Tekle", "Teklehaimanot", "Teklemariam", "Teklezghi", "Tesfamariam", "Tesfamichael", "Tesfay", "Tesfaldet", "Tesfatsion",
        "Tesfazghi", "Tesfu", "Tewelde", "Tewolde", "Tsegay", "Tsehaye", "Weldegiorgis", "Weldemichael", "Woldeab", "Woldegebriel",
        "Woldemariam", "Woldu", "Yemane", "Yohannes", "Zemichael", "Zerai", "Zeremariam", "Zerom", "Zeru",
        "Adhanom", "Alem", "Andom", "Asgedom", "Ayele", "Bairu", "Beraki", "Bereketab", "Debretsion", "Ekubay",
        "Estifanos", "Fessaha", "Gebrezgi", "Ghebrat", "Ghidey", "Habtu", "Hailezghi", "Iyasu", "Keleta", "Kesete",
        "Kibreab", "Kinfe", "Mengesha", "Mesghina", "Mesgun", "Nebay", "Netsereab", "Ogbagaber", "Ogbamichael", "Redae",
        "Sebhat", "Solomon", "Tecle", "Tedla", "Tekleab", "Tesfagabir", "Tesfagiorgis", "Tesfaghebriel", "Tewoldemedhin", "Tsegezab",
        "Woldetensae", "Yacob", "Zemuy", "Zere", "Zerezghi",
    ],
    also_valid_first: &["Birhane", "Emanuel", "Fikru", "Hagos", "Kidane", "Sebhat", "Selassie", "Solomon", "Teklit", "Yossief"],
};

pub const PACKS: &[&NamePack] = &[&ERITREA_TIGRINYA];

pub fn pack(key: &str) -> Option<&'static NamePack> {
    PACKS.iter().copied().find(|p| p.key == key)
}

#[derive(Debug, Serialize, Clone)]
pub struct PackInfo {
    pub key: String,
    pub label: String,
    pub first: Vec<String>,
    pub last: Vec<String>,
}

pub fn pack_info(p: &NamePack) -> PackInfo {
    PackInfo {
        key: p.key.into(),
        label: p.label.into(),
        first: p.first.iter().map(|s| s.to_string()).collect(),
        last: p.last.iter().map(|s| s.to_string()).collect(),
    }
}

/// Country id for an ISO code, read from a database that has `STA_country`.
pub fn country_id(db: &Cdb, iso: &str) -> Option<i32> {
    let ids = db.ints("STA_country", "IDcountry")?;
    let codes = db.strings("STA_country", "CONSTANT")?;
    codes.iter().position(|c| c.eq_ignore_ascii_case(iso)).map(|i| ids[i])
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct ListReport {
    pub removed_first: Vec<String>,
    pub removed_last: Vec<String>,
    pub added_first: usize,
    pub added_last: usize,
}

/// Current names in the generator lists for a country.
pub fn current_lists(db: &Cdb, country: i32) -> (Vec<String>, Vec<String>) {
    let read = |t: &str| -> Vec<String> {
        let regions = db.ints(t, "fkIDregion").unwrap_or_default();
        let names = db.strings(t, "gene_sz_name").unwrap_or_default();
        regions.iter().zip(names).filter(|(r, _)| **r == country).map(|(_, n)| n).collect()
    };
    (read("STA_regen_firstname"), read("STA_regen_lastname"))
}

/// Replaces the country's entries in both generator tables with the pack.
pub fn apply_to_lists(db: &mut Cdb, country: i32, p: &NamePack) -> Result<ListReport, String> {
    let mut report = ListReport::default();
    for (table, id_col, names, is_first) in [
        ("STA_regen_firstname", "IDregen_firstname", p.first, true),
        ("STA_regen_lastname", "IDregen_lastname", p.last, false),
    ] {
        if db.table(table).is_none() {
            return Err(format!("{table} not found — is this the game's OfficialLocal.cdb?"));
        }
        let ids = db.ints(table, id_col).ok_or(format!("{table}.{id_col} missing"))?;
        let regions = db.ints(table, "fkIDregion").ok_or(format!("{table}.fkIDregion missing"))?;
        let texts = db.raw_strings(table, "gene_sz_name").ok_or(format!("{table}.gene_sz_name missing"))?;
        let mut next_id = ids.iter().copied().max().unwrap_or(0) + 1;
        let (mut new_ids, mut new_regions, mut new_texts) = (Vec::new(), Vec::new(), Vec::new());
        let mut removed = Vec::new();
        for i in 0..ids.len() {
            if regions[i] == country {
                removed.push(decode_text(&texts[i]));
            } else {
                new_ids.push(ids[i]);
                new_regions.push(regions[i]);
                new_texts.push(texts[i].clone());
            }
        }
        // IDs stay sorted: the new names get fresh ids after the current maximum.
        for n in names {
            new_ids.push(next_id);
            next_id += 1;
            new_regions.push(country);
            new_texts.push(encode_text(n));
        }
        db.rewrite_columns(table, &[(id_col, Values::Int(new_ids)), ("fkIDregion", Values::Int(new_regions)), ("gene_sz_name", Values::Text(new_texts))])?;
        if is_first {
            report.removed_first = removed;
            report.added_first = names.len();
        } else {
            report.removed_last = removed;
            report.added_last = names.len();
        }
    }
    Ok(report)
}

#[derive(Debug, Serialize, Clone)]
pub struct Rename {
    pub id: i32,
    pub from: String,
    pub to: String,
}

/// Deterministic pick so re-running gives the same names.
fn pick<'a>(list: &[&'a str], seed: u64, taken: &dyn Fn(&str) -> bool) -> &'a str {
    let mut h = seed.wrapping_mul(0x9E3779B97F4A7C15) ^ 0xD1B54A32D192ED03;
    for _ in 0..list.len() * 2 {
        h ^= h >> 29;
        h = h.wrapping_mul(0xBF58476D1CE4E5B9);
        let cand = list[(h % list.len() as u64) as usize];
        if !taken(cand) {
            return cand;
        }
    }
    list[(seed % list.len() as u64) as usize]
}

/// Renames riders the game generated for `country` whose first or last name isn't in the pack.
/// Real riders (those whose name exists in the mod's base database, or born before
/// `generated_from_year`) are never touched.
pub fn rename_generated(save: &mut Cdb, p: &NamePack, real_names: &HashSet<(String, String)>, generated_from_year: i32) -> Result<Vec<Rename>, String> {
    let country = country_id(save, p.country_iso).ok_or("Country not found in this save")?;
    let region_country: HashMap<i32, i32> = save.ints("STA_region", "IDregion").unwrap_or_default().into_iter()
        .zip(save.ints("STA_region", "fkIDcountry").unwrap_or_default()).collect();
    let t = "DYN_cyclist";
    let ids = save.ints(t, "IDcyclist").ok_or("DYN_cyclist missing")?;
    let regions = save.ints(t, "fkIDregion").ok_or("fkIDregion missing")?;
    let births = save.ints(t, "gene_i_birthdate").ok_or("birthdate missing")?;
    let mut first = save.raw_strings(t, "gene_sz_firstname").ok_or("firstname missing")?;
    let mut last = save.raw_strings(t, "gene_sz_lastname").ok_or("lastname missing")?;
    let mut short = save.raw_strings(t, "gene_sz_firstlastname");

    // Existing riders keep any genuine given name, modern or traditional; only wrong ones change.
    let first_ok: HashSet<&str> = p.first.iter().chain(p.last).chain(p.also_valid_first).copied().collect();
    let last_ok: HashSet<&str> = p.last.iter().copied().collect();
    let mut used: HashSet<(String, String)> = first.iter().zip(&last).map(|(f, l)| (decode_text(f), decode_text(l))).collect();
    let mut renames = Vec::new();

    for i in 0..ids.len() {
        if region_country.get(&regions[i]) != Some(&country) || births[i] / 10000 < generated_from_year {
            continue;
        }
        let (f, l) = (decode_text(&first[i]), decode_text(&last[i]));
        if real_names.contains(&(f.clone(), l.clone())) {
            continue;
        }
        let (fix_first, fix_last) = (!first_ok.contains(f.as_str()), !last_ok.contains(l.as_str()));
        if !fix_first && !fix_last {
            continue;
        }
        let seed = ids[i] as u64;
        let nf = if fix_first { pick(p.first, seed, &|c| c == l) .to_string() } else { f.clone() };
        let nl = if fix_last {
            pick(p.last, seed.rotate_left(17) ^ 0xA5A5, &|c| c == nf || used.contains(&(nf.clone(), c.to_string()))).to_string()
        } else {
            l.clone()
        };
        used.remove(&(f.clone(), l.clone()));
        used.insert((nf.clone(), nl.clone()));
        first[i] = encode_text(&nf);
        last[i] = encode_text(&nl);
        if let Some(s) = short.as_mut() {
            let initial = nf.chars().next().map(|c| format!(" {c}.")).unwrap_or_default();
            s[i] = encode_text(&format!("{nl}{initial}"));
        }
        renames.push(Rename { id: ids[i], from: format!("{f} {l}"), to: format!("{nf} {nl}") });
    }
    if renames.is_empty() {
        return Ok(renames);
    }

    let mut cols = vec![("gene_sz_firstname", Values::Text(first)), ("gene_sz_lastname", Values::Text(last))];
    if let Some(s) = short {
        cols.push(("gene_sz_firstlastname", Values::Text(s)));
    }
    save.rewrite_columns(t, &cols)?;

    // Results history keeps its own copy of the full name.
    let pt = "DYN_palmares_cyclist";
    if let (Some(pc), Some(mut full)) = (save.ints(pt, "fkIDcyclist"), save.raw_strings(pt, "gene_sz_full_name")) {
        let by_id: HashMap<i32, &Rename> = renames.iter().map(|r| (r.id, r)).collect();
        let mut changed = false;
        for (i, cid) in pc.iter().enumerate() {
            if let Some(r) = by_id.get(cid) {
                full[i] = encode_text(&r.to);
                changed = true;
            }
        }
        if changed {
            save.rewrite_columns(pt, &[("gene_sz_full_name", Values::Text(full))])?;
        }
    }
    Ok(renames)
}

/// Real rider names from the mod's (or game's) base database, used to protect real riders.
pub fn real_rider_names(base_db: &Path) -> HashSet<(String, String)> {
    let Ok(db) = Cdb::open(&base_db.to_string_lossy()) else { return HashSet::new() };
    match (db.strings("DYN_cyclist", "gene_sz_firstname"), db.strings("DYN_cyclist", "gene_sz_lastname")) {
        (Some(f), Some(l)) => f.into_iter().zip(l).collect(),
        _ => HashSet::new(),
    }
}

fn steam_libraries() -> Vec<PathBuf> {
    let mut libs = Vec::new();
    for base in [r"C:\Program Files (x86)\Steam", r"C:\Program Files\Steam"] {
        let base = PathBuf::from(base);
        if !base.is_dir() {
            continue;
        }
        libs.push(base.clone());
        if let Ok(vdf) = std::fs::read_to_string(base.join(r"steamapps\libraryfolders.vdf")) {
            for line in vdf.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("\"path\"") {
                    let p = rest.trim().trim_matches('"').replace("\\\\", "\\");
                    libs.push(PathBuf::from(p));
                }
            }
        }
    }
    libs.sort();
    libs.dedup();
    libs
}

/// The base database the career was started from: the Steam Workshop mod if one is set,
/// otherwise nothing (birth year then protects real riders).
pub fn base_database_for(save: &Cdb) -> Option<PathBuf> {
    let modid = save.strings("GAM_config", "gene_sz_modid")?.into_iter().next()?;
    if modid.is_empty() {
        return None;
    }
    for lib in steam_libraries() {
        let Ok(apps) = std::fs::read_dir(lib.join(r"steamapps\workshop\content")) else { continue };
        for app in apps.flatten() {
            let cand = app.path().join(&modid).join("OfficialRelease.cdb");
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    None
}

/// Game name-list databases (`Mod\Default\OfficialLocal.cdb`) for each installed PCM.
pub fn find_name_databases() -> Vec<(String, PathBuf)> {
    let Ok(appdata) = std::env::var("APPDATA") else { return vec![] };
    let Ok(dirs) = std::fs::read_dir(&appdata) else { return vec![] };
    let mut out: Vec<(String, PathBuf)> = dirs
        .flatten()
        .filter(|d| d.file_name().to_string_lossy().starts_with("Pro Cycling Manager"))
        .map(|d| (d.file_name().to_string_lossy().replace("Pro Cycling Manager", "PCM").trim().to_string(), d.path().join(r"Mod\Default\OfficialLocal.cdb")))
        .filter(|(_, p)| p.is_file())
        .collect();
    out.sort_by(|a, b| b.0.cmp(&a.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_has_no_duplicates_and_is_ascii() {
        for p in PACKS {
            for list in [p.first, p.last] {
                let set: HashSet<_> = list.iter().collect();
                assert_eq!(set.len(), list.len(), "{}: duplicate names", p.key);
                assert!(list.iter().all(|n| n.is_ascii() && !n.contains(' ')));
            }
        }
    }

    #[test]
    fn renames_only_generated_riders_in_copy() {
        let src = concat!(env!("CARGO_MANIFEST_DIR"), "/../Career_2 copy.cdb");
        if !Path::new(src).is_file() {
            return;
        }
        let mut save = Cdb::open(src).unwrap();
        let real = base_database_for(&save).map(|p| real_rider_names(&p)).unwrap_or_default();
        let before_real: Vec<String> = save.strings("DYN_cyclist", "gene_sz_lastname").unwrap();
        let renames = rename_generated(&mut save, &ERITREA_TIGRINYA, &real, 2007).unwrap();
        assert!(!renames.is_empty());
        let after = Cdb::from_file_bytes(&save.to_file_bytes().unwrap()).unwrap();
        let names: Vec<String> = after.strings("DYN_cyclist", "gene_sz_lastname").unwrap();
        let ids = after.ints("DYN_cyclist", "IDcyclist").unwrap();
        // Real riders untouched
        for (id, real_last) in [(7110, "Girmay"), (7372, "Tesfatsion"), (6357, "Ghebreigzabhier")] {
            let i = ids.iter().position(|&x| x == id).unwrap();
            assert_eq!(names[i], real_last);
        }
        assert_eq!(names.len(), before_real.len());
        assert!(!names.iter().any(|n| n == "Mewaelsimon" || n == "Zecahrias"));
        for r in &renames {
            println!("{:>6}  {:<28} → {}", r.id, r.from, r.to);
        }
    }
}

#[cfg(test)]
mod live_copy_tests {
    use super::*;

    /// Runs the full game-list flow on a temp copy of the real OfficialLocal.cdb.
    #[test]
    fn applies_pack_to_copy_of_game_list() {
        let Some((_, real)) = find_name_databases().into_iter().find(|(g, _)| g.contains("2026")) else { return };
        let tmp = std::env::temp_dir().join(format!("pcmrecon-names-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let copy = tmp.join("OfficialLocal.cdb");
        std::fs::copy(&real, &copy).unwrap();
        let before = Cdb::open(&copy.to_string_lossy()).unwrap();
        let (f0, l0) = current_lists(&before, 41);
        let (report, _backup, _) = crate::edit::transform(&tmp, &copy, |db| apply_to_lists(db, 41, &ERITREA_TIGRINYA)).unwrap();
        let after = Cdb::open(&copy.to_string_lossy()).unwrap();
        let (f1, l1) = current_lists(&after, 41);
        assert_eq!(report.removed_first.len(), f0.len());
        assert_eq!(report.removed_last.len(), l0.len());
        assert_eq!(f1.len(), ERITREA_TIGRINYA.first.len());
        assert_eq!(l1.len(), ERITREA_TIGRINYA.last.len());
        for t in ["STA_regen_firstname", "STA_regen_lastname", "LOC"] {
            let extra = if t == "STA_regen_firstname" { f1.len() as isize - f0.len() as isize } else if t == "STA_regen_lastname" { l1.len() as isize - l0.len() as isize } else { 0 };
            assert_eq!(after.rows(t) as isize, before.rows(t) as isize + extra, "{t}");
        }
        let id_col = |t: &str| if t.contains("first") { "IDregen_firstname" } else { "IDregen_lastname" };
        for t in ["STA_regen_firstname", "STA_regen_lastname"] {
            let ids = after.ints(t, id_col(t)).unwrap();
            assert!(ids.windows(2).all(|w| w[0] < w[1]), "{t} ids not sorted");
            // Other nations untouched
            let other = |db: &Cdb| db.strings(t, "gene_sz_name").unwrap().into_iter().zip(db.ints(t, "fkIDregion").unwrap()).filter(|(_, r)| *r != 41).collect::<Vec<_>>();
            assert_eq!(other(&before), other(&after));
        }
        println!("game list: removed {} + {}, now {} + {}", f0.len(), l0.len(), f1.len(), l1.len());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
