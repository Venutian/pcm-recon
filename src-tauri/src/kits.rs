//! Team kits for PCM 2026: find every kit in the base game and the career's mod, read kit
//! images, keep the user's edits, and build one override pak in `Content/Paks/~mods`.
//!
//! A kit is the folder named by `DYN_team.jersey_sz_abbreviation`:
//!   mod:  `<mount>Content/Jersey/Team/<kit>/<kit>_<part>`   + `<mount>Content/MiniJersey/Team/<kit>_minimaillot`
//!   base: `PCM/Content/3D/Cyclist/Cloth/Team/<kit>/<kit>_<part>` + `PCM/Content/Gui/MiniJersey/Team/<kit>_minimaillot`
//! Parts are `maillot`, `maillot_tour`, `maillot_<country>` (team national-champion jersey),
//! `leggings`, `gloves`, `socks`, `shoes`, `bottle`, `handles`, `corsair`, `transfert_av/ar`, …
use crate::cdb::Cdb;
use crate::oodle::Oodle;
use crate::pak::Pak;
use crate::texture::{self, Size};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

pub const OVERRIDE_PAK: &str = "zz_pcmrecon_kits_P.pak";

#[derive(Debug, Clone, Serialize)]
pub struct GameInstall {
    pub game: String,
    pub paks_dir: PathBuf,
}

impl GameInstall {
    pub fn mods_dir(&self) -> PathBuf {
        self.paks_dir.join("~mods")
    }
    pub fn override_pak(&self) -> PathBuf {
        self.mods_dir().join(OVERRIDE_PAK)
    }
}

pub struct Source {
    pub is_mod: bool,
    pub pak: Pak,
}

#[derive(Debug, Clone)]
pub struct PartRef {
    pub source: usize,
    /// Game path without extension.
    pub path: String,
}

#[derive(Debug, Clone, Default)]
pub struct Kit {
    pub parts: BTreeMap<String, PartRef>,
}

pub struct Catalog {
    pub install: GameInstall,
    pub mod_label: Option<String>,
    pub sources: Vec<Source>,
    pub kits: BTreeMap<String, Kit>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KitSummary {
    pub abbr: String,
    pub from_mod: bool,
    pub parts: Vec<String>,
}

// ─── Locating the game and the mod ────────────────────────────────────────────

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
                if let Some(rest) = line.trim().strip_prefix("\"path\"") {
                    libs.push(PathBuf::from(rest.trim().trim_matches('"').replace("\\\\", "\\")));
                }
            }
        }
    }
    libs.sort();
    libs.dedup();
    libs
}

/// The Unreal-based PCM install matching the save (by the year in its folder), else the newest.
pub fn find_install(save_path: &Path) -> Result<GameInstall, String> {
    let mut found: Vec<(String, PathBuf)> = Vec::new();
    for lib in steam_libraries() {
        let Ok(games) = std::fs::read_dir(lib.join(r"steamapps\common")) else { continue };
        for g in games.flatten() {
            let name = g.file_name().to_string_lossy().to_string();
            let paks = g.path().join(r"PCM\Content\Paks");
            if name.starts_with("Pro Cycling Manager") && paks.is_dir() {
                found.push((name, paks));
            }
        }
    }
    let save = save_path.to_string_lossy();
    found.sort_by(|a, b| b.0.cmp(&a.0));
    let pick = found.iter().find(|(n, _)| save.contains(n.as_str())).or(found.first());
    pick.map(|(n, p)| GameInstall { game: n.replace("Pro Cycling Manager", "PCM").trim().to_string(), paks_dir: p.clone() })
        .ok_or_else(|| "No PCM 2026 (or newer) installation found. Kits need the Unreal Engine version of PCM.".into())
}

/// The career's Steam Workshop mod pak and its title, if it uses one.
pub fn find_mod_pak(save: &Cdb) -> Option<(PathBuf, String)> {
    let modid = save.strings("GAM_config", "gene_sz_modid")?.into_iter().next()?;
    if modid.is_empty() {
        return None;
    }
    for lib in steam_libraries() {
        let Ok(apps) = std::fs::read_dir(lib.join(r"steamapps\workshop\content")) else { continue };
        for app in apps.flatten() {
            let dir = app.path().join(&modid);
            let Ok(files) = std::fs::read_dir(&dir) else { continue };
            for f in files.flatten() {
                if f.path().extension().is_some_and(|e| e.eq_ignore_ascii_case("pak")) {
                    let title = std::fs::read_to_string(dir.join("mod.xml"))
                        .ok()
                        .and_then(|x| Some(x.split("<Title>").nth(1)?.split("</Title>").next()?.trim().to_string()))
                        .unwrap_or_else(|| format!("Workshop mod {modid}"));
                    return Some((f.path(), title));
                }
            }
        }
    }
    None
}

// ─── Catalog ──────────────────────────────────────────────────────────────────

fn is_kit_path(p: &str) -> bool {
    (p.ends_with(".uasset") || p.ends_with(".uexp")) && (p.contains("/Jersey/Team/") || p.contains("/MiniJersey/Team/") || p.contains("/Cloth/Team/"))
}

/// (kit, part) for a kit asset path without extension.
fn classify(path: &str) -> Option<(String, String)> {
    let file = path.rsplit('/').next()?;
    if path.contains("/MiniJersey/Team/") {
        let at = file.find("_minimaillot")?;
        return Some((file[..at].to_string(), file[at + 1..].to_string()));
    }
    // .../Team/<kit>/<kit>_<part>
    let mut segs = path.rsplit('/');
    let (_, kit) = (segs.next()?, segs.next()?);
    if segs.next()? != "Team" {
        return None;
    }
    let part = file.strip_prefix(kit)?.strip_prefix('_')?;
    Some((kit.to_string(), part.to_string()))
}

pub fn build_catalog(install: GameInstall, mod_pak: Option<(PathBuf, String)>) -> Result<Catalog, String> {
    let mut sources = Vec::new();
    let mod_label = mod_pak.as_ref().map(|(_, t)| t.clone());
    if let Some((path, _)) = mod_pak {
        sources.push(Source { is_mod: true, pak: Pak::open(&path, is_kit_path)? });
    }
    let mut base: Vec<PathBuf> = std::fs::read_dir(&install.paks_dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|f| f.path())
        .filter(|p| p.extension().is_some_and(|e| e == "pak"))
        .collect();
    base.sort();
    for path in base {
        let pak = Pak::open(&path, is_kit_path)?;
        if !pak.entries.is_empty() {
            sources.push(Source { is_mod: false, pak });
        }
    }
    let mut kits: BTreeMap<String, Kit> = BTreeMap::new();
    for (si, s) in sources.iter().enumerate() {
        for p in s.pak.entries.keys() {
            let Some(stem) = p.strip_suffix(".uasset") else { continue };
            let Some((kit, part)) = classify(stem) else { continue };
            // The mod is listed first, so its parts win over the base game's.
            kits.entry(kit.to_lowercase()).or_default().parts.entry(part).or_insert(PartRef { source: si, path: stem.to_string() });
        }
    }
    kits.retain(|_, k| k.parts.contains_key("maillot") || k.parts.contains_key("minimaillot"));
    Ok(Catalog { install, mod_label, sources, kits })
}

impl Catalog {
    pub fn summaries(&self) -> Vec<KitSummary> {
        self.kits
            .iter()
            .map(|(abbr, k)| KitSummary {
                abbr: abbr.clone(),
                from_mod: k.parts.values().any(|p| self.sources[p.source].is_mod),
                parts: k.parts.keys().cloned().collect(),
            })
            .collect()
    }

    fn kit(&self, abbr: &str) -> Result<&Kit, String> {
        self.kits.get(&abbr.to_lowercase()).ok_or_else(|| format!("Kit {abbr} isn't in the game or the career's mod"))
    }

    fn read_asset(&self, r: &PartRef, oodle: Option<&Oodle>) -> Result<(Vec<u8>, Vec<u8>), String> {
        let pak = &self.sources[r.source].pak;
        Ok((pak.read(&format!("{}.uasset", r.path), oodle)?, pak.read(&format!("{}.uexp", r.path), oodle)?))
    }

    /// Original image of a part as RGBA.
    pub fn read_part(&self, abbr: &str, part: &str, oodle: Option<&Oodle>) -> Result<(Size, Vec<u8>), String> {
        let r = self.kit(abbr)?.parts.get(part).ok_or_else(|| format!("{abbr} has no {part}"))?;
        let (uasset, uexp) = self.read_asset(r, oodle)?;
        let size = texture::parse(&uasset, &uexp)?;
        let mut px = texture::pixels(&uexp, size).to_vec();
        texture::swap_rb(&mut px);
        Ok((size, px))
    }

    /// Asset files for an edited part: the original with new pixels, or, for a part the kit
    /// doesn't have yet (a new national-champion jersey), a renamed copy of a same-shaped one.
    fn build_part(&self, abbr: &str, part: &str, size: Size, bgra: &[u8], oodle: Option<&Oodle>) -> Result<Vec<(String, Vec<u8>)>, String> {
        let kit = self.kit(abbr)?;
        if let Some(r) = kit.parts.get(part) {
            let (uasset, uexp) = self.read_asset(r, oodle)?;
            let have = texture::parse(&uasset, &uexp)?;
            if have != size {
                return Err(format!("{abbr} {part} is {}×{}, the edit is {}×{}", have.width, have.height, size.width, size.height));
            }
            return Ok(vec![(format!("{}.uasset", r.path), uasset), (format!("{}.uexp", r.path), texture::with_pixels(&uexp, size, bgra)?)]);
        }
        // New part: needs the kit's own folder, and a template from the same pak whose kit
        // and part names have the same lengths, so renaming keeps every offset in place.
        let home = kit.parts.get("maillot").ok_or_else(|| format!("{abbr} has no main jersey to place {part} next to"))?;
        let family = part.rsplit_once('_').map(|(f, _)| format!("{f}_")).unwrap_or_default();
        let mut candidates: Vec<(&String, &String, &PartRef)> = Vec::new();
        for (k_abbr, k) in &self.kits {
            if k_abbr.len() != abbr.len() {
                continue;
            }
            for (p, r) in &k.parts {
                if r.source == home.source && p.len() == part.len() && p.starts_with(&family) && !family.is_empty() {
                    candidates.push((k_abbr, p, r));
                }
            }
        }
        // Prefer the kit's own variants, then any other kit.
        candidates.sort_by_key(|(k, _, _)| *k != &abbr.to_lowercase());
        for (k_abbr, p, r) in candidates {
            let Ok((uasset, uexp)) = self.read_asset(r, oodle) else { continue };
            if texture::parse(&uasset, &uexp).ok() != Some(size) {
                continue;
            }
            // Use the template's real spelling of its kit name (folder case can differ).
            let t_kit = r.path.rsplit('/').nth(1).unwrap_or(k_abbr);
            let h_kit = home.path.rsplit('/').nth(1).unwrap_or(abbr);
            let new_asset = texture::renamed(
                &uasset,
                &[
                    (&format!("/{t_kit}/{t_kit}_{p}\0"), &format!("/{h_kit}/{h_kit}_{part}\0")),
                    (&format!("{t_kit}_{p}\0"), &format!("{h_kit}_{part}\0")),
                ],
            );
            let Ok(new_asset) = new_asset else { continue };
            let dir = home.path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
            let path = format!("{dir}/{h_kit}_{part}");
            return Ok(vec![(format!("{path}.uasset"), new_asset), (format!("{path}.uexp"), texture::with_pixels(&uexp, size, bgra)?)]);
        }
        Err(format!("Couldn't find a template to create {abbr} {part}"))
    }
}

// ─── Edits ────────────────────────────────────────────────────────────────────

/// Edits live in `<app data>/kits/<kit>/<part>.json` (the editor's recipe, opaque here)
/// and `<part>.bin` (u32 width, u32 height, BGRA pixels as rendered).
pub fn edits_dir(app_data: &Path) -> PathBuf {
    app_data.join("kits")
}

fn safe_name(s: &str) -> Result<&str, String> {
    if s.is_empty() || s.contains(['/', '\\', ':', '.']) {
        return Err(format!("Invalid name {s}"));
    }
    Ok(s)
}

#[derive(Debug, Clone, Serialize)]
pub struct EditInfo {
    pub kit: String,
    pub part: String,
    pub recipe: serde_json::Value,
    pub modified_ms: u64,
}

pub fn list_edits(app_data: &Path, kit: Option<&str>) -> Vec<EditInfo> {
    let mut out = Vec::new();
    let Ok(kits) = std::fs::read_dir(edits_dir(app_data)) else { return out };
    for k in kits.flatten().filter(|k| k.path().is_dir()) {
        let name = k.file_name().to_string_lossy().to_string();
        if kit.is_some_and(|want| !want.eq_ignore_ascii_case(&name)) {
            continue;
        }
        let Ok(files) = std::fs::read_dir(k.path()) else { continue };
        for f in files.flatten() {
            let p = f.path();
            if p.extension().is_some_and(|e| e == "json") && p.with_extension("bin").is_file() {
                let recipe = std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(serde_json::Value::Null);
                let modified_ms = f.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64).unwrap_or(0);
                out.push(EditInfo { kit: name.clone(), part: p.file_stem().unwrap().to_string_lossy().to_string(), recipe, modified_ms });
            }
        }
    }
    out.sort_by(|a, b| (&a.kit, &a.part).cmp(&(&b.kit, &b.part)));
    out
}

pub fn save_edit(app_data: &Path, kit: &str, part: &str, recipe: &str, size: Size, rgba: &[u8]) -> Result<(), String> {
    let dir = edits_dir(app_data).join(safe_name(&kit.to_lowercase())?);
    safe_name(part)?;
    if rgba.len() != size.width as usize * size.height as usize * 4 {
        return Err("Image size doesn't match its dimensions".into());
    }
    serde_json::from_str::<serde_json::Value>(recipe).map_err(|e| format!("Bad recipe: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut bin = Vec::with_capacity(8 + rgba.len());
    bin.extend_from_slice(&size.width.to_le_bytes());
    bin.extend_from_slice(&size.height.to_le_bytes());
    bin.extend_from_slice(rgba);
    texture::swap_rb(&mut bin[8..]);
    std::fs::write(dir.join(format!("{part}.bin")), bin).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(format!("{part}.json")), recipe).map_err(|e| e.to_string())
}

pub fn delete_edit(app_data: &Path, kit: &str, part: &str) -> Result<(), String> {
    let dir = edits_dir(app_data).join(safe_name(&kit.to_lowercase())?);
    for ext in ["bin", "json"] {
        let _ = std::fs::remove_file(dir.join(format!("{}.{ext}", safe_name(part)?)));
    }
    let _ = std::fs::remove_dir(&dir); // only succeeds once empty
    Ok(())
}

/// Rendered image of an edit as RGBA.
pub fn read_edit(app_data: &Path, kit: &str, part: &str) -> Result<(Size, Vec<u8>), String> {
    let path = edits_dir(app_data).join(safe_name(&kit.to_lowercase())?).join(format!("{}.bin", safe_name(part)?));
    let mut bin = std::fs::read(&path).map_err(|_| format!("No edit for {kit} {part}"))?;
    let size = Size { width: u32::from_le_bytes(bin[0..4].try_into().unwrap()), height: u32::from_le_bytes(bin[4..8].try_into().unwrap()) };
    let mut px = bin.split_off(8);
    texture::swap_rb(&mut px);
    Ok((size, px))
}

/// Small named values kept next to the edits (e.g. the team logo), at `<app data>/kits/<key>.meta`.
pub fn meta_get(app_data: &Path, key: &str) -> Result<Option<String>, String> {
    let p = edits_dir(app_data).join(format!("{}.meta", safe_name(key)?));
    Ok(std::fs::read_to_string(p).ok())
}

pub fn meta_set(app_data: &Path, key: &str, value: Option<&str>) -> Result<(), String> {
    let dir = edits_dir(app_data);
    let p = dir.join(format!("{}.meta", safe_name(key)?));
    match value {
        Some(v) => {
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            std::fs::write(p, v).map_err(|e| e.to_string())
        }
        None => {
            let _ = std::fs::remove_file(p);
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyReport {
    pub parts: usize,
    pub path: String,
    pub skipped: Vec<String>,
}

/// Builds the override pak from every saved edit, or removes it when there are none.
pub fn apply(cat: &Catalog, app_data: &Path, oodle: Option<&Oodle>) -> Result<ApplyReport, String> {
    let out = cat.install.override_pak();
    let edits = list_edits(app_data, None);
    if edits.is_empty() {
        remove(&cat.install)?;
        return Ok(ApplyReport { parts: 0, path: out.to_string_lossy().into(), skipped: vec![] });
    }
    let mut files = BTreeMap::new();
    let mut skipped = Vec::new();
    let mut parts = 0;
    for e in &edits {
        let built = read_edit(app_data, &e.kit, &e.part).and_then(|(size, mut px)| {
            texture::swap_rb(&mut px);
            cat.build_part(&e.kit, &e.part, size, &px, oodle)
        });
        match built {
            Ok(list) => {
                parts += 1;
                files.extend(list);
            }
            Err(err) => skipped.push(format!("{} {}: {err}", e.kit, e.part)),
        }
    }
    if files.is_empty() {
        return Err(format!("Nothing could be applied:\n{}", skipped.join("\n")));
    }
    std::fs::create_dir_all(cat.install.mods_dir()).map_err(|e| e.to_string())?;
    crate::pak::write(&out, &files)?;
    remove_legacy(&cat.install);
    Ok(ApplyReport { parts, path: out.to_string_lossy().into(), skipped })
}

pub fn remove(install: &GameInstall) -> Result<(), String> {
    remove_legacy(install);
    let p = install.override_pak();
    if p.exists() {
        std::fs::remove_file(&p).map_err(|e| format!("Cannot remove {} ({e}). Close PCM first.", p.display()))?;
    }
    Ok(())
}

/// The green test jersey pak from the kit feasibility check; it would fight the real override.
fn remove_legacy(install: &GameInstall) {
    let _ = std::fs::remove_file(install.mods_dir().join("zz_pcmrecon_kittest_P.pak"));
}

// ─── Squad nationalities ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct Nation {
    /// The game's country code as used in jersey names (`ita`, `swd`, …).
    pub code: String,
    pub name: String,
    pub flag: String,
    pub riders: usize,
}

/// Nationalities of a team's riders, most riders first.
pub fn squad_nations(db: &Cdb, team_id: i32) -> Vec<Nation> {
    let countries: HashMap<i32, String> = db.ints("STA_country", "IDcountry").unwrap_or_default().into_iter().zip(db.strings("STA_country", "CONSTANT").unwrap_or_default()).collect();
    let region_country: HashMap<i32, i32> = db.ints("STA_region", "IDregion").unwrap_or_default().into_iter().zip(db.ints("STA_region", "fkIDcountry").unwrap_or_default()).collect();
    let mut counts: HashMap<String, usize> = HashMap::new();
    let teams = db.ints("DYN_cyclist", "fkIDteam").unwrap_or_default();
    let regions = db.ints("DYN_cyclist", "fkIDregion").unwrap_or_default();
    for (t, r) in teams.iter().zip(&regions) {
        if *t != team_id {
            continue;
        }
        if let Some(code) = region_country.get(r).and_then(|c| countries.get(c)) {
            let code: String = code.chars().filter(|c| c.is_ascii_alphabetic()).collect::<String>().to_lowercase();
            if !code.is_empty() {
                *counts.entry(code).or_default() += 1;
            }
        }
    }
    let mut out: Vec<Nation> = counts
        .into_iter()
        .map(|(code, riders)| {
            let (name, flag) = crate::parser::country_display(&code);
            Nation { code, name, flag, riders }
        })
        .collect();
    out.sort_by(|a, b| b.riders.cmp(&a.riders).then(a.name.cmp(&b.name)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_kit_paths() {
        assert_eq!(classify("pcm25_mod1/Plugins/Mod/Content/Jersey/Team/x-abc/x-abc_maillot_ita"), Some(("x-abc".into(), "maillot_ita".into())));
        assert_eq!(classify("PCM/Content/3D/Cyclist/Cloth/Team/abc/abc_leggings"), Some(("abc".into(), "leggings".into())));
        assert_eq!(classify("PCM/Content/Gui/MiniJersey/Team/abc_minimaillot_tour"), Some(("abc".into(), "minimaillot_tour".into())));
        assert_eq!(classify("pcm25_mod1/Plugins/Mod/Content/Jersey/Country/ITA_maillot_champion"), None);
    }

    /// Full round trip against the installed game and the WorldDB mod, when present:
    /// read a kit, build an override pak with an edited jersey and a brand-new champion
    /// variant, then read both back.
    #[test]
    fn builds_override_from_installed_game() {
        let Some(save_file) = crate::cdb::sample_path() else { return };
        let save_path = Path::new(&save_file);
        let Ok(install) = find_install(save_path) else { return };
        // Exercises the real download + checksum, into a temp folder.
        let dll_dir = std::env::temp_dir().join("pcmrecon-oodle-test");
        std::fs::create_dir_all(&dll_dir).unwrap();
        let oodle = crate::oodle::load(&crate::oodle::ensure_downloaded(&dll_dir).unwrap()).unwrap();
        let save = Cdb::open(&save_path.to_string_lossy()).unwrap();
        let cat = build_catalog(install, find_mod_pak(&save)).unwrap();
        // The save's own team kit.
        let data = crate::parser::extract(&save, &save_file).unwrap();
        let kit = data.teams.iter().find(|t| t.is_mine).map(|t| t.jersey.clone()).unwrap();
        let kit = kit.as_str();
        assert!(cat.kits.len() > 100, "{} kits", cat.kits.len());
        let (size, rgba) = cat.read_part(kit, "maillot", Some(oodle)).unwrap();
        assert_eq!((size.width, size.height), (1200, 826));
        let (mini, _) = cat.read_part(kit, "minimaillot", Some(oodle)).unwrap();
        assert_eq!((mini.width, mini.height), (256, 256));

        let mut bgra = rgba.clone();
        texture::swap_rb(&mut bgra);
        let edited = cat.build_part(kit, "maillot", size, &bgra, Some(oodle)).unwrap();
        let maillot_uexp = edited.iter().map(|(p, _)| p.clone()).find(|p| p.ends_with(".uexp")).unwrap();
        let fresh = cat.build_part(kit, "maillot_zzz", size, &bgra, Some(oodle)).unwrap();
        assert!(fresh[0].0.ends_with(&format!("/{kit}/{kit}_maillot_zzz.uasset")), "{}", fresh[0].0);
        assert!(String::from_utf8_lossy(&fresh[0].1).contains(&format!("/{kit}/{kit}_maillot_zzz\0")));

        let dir = std::env::temp_dir().join(format!("pcmrecon-kits-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("t_P.pak");
        crate::pak::write(&out, &edited.into_iter().chain(fresh).collect()).unwrap();
        if let Ok(keep) = std::env::var("PCMRECON_KEEP_PAK") {
            std::fs::copy(&out, keep).unwrap(); // for checking the output with other pak tools
        }
        let back = Pak::open(&out, |_| true).unwrap();
        assert_eq!(back.entries.len(), 4);
        for (path, _) in back.entries.iter().filter(|(p, _)| p.ends_with(".uexp")) {
            let uexp = back.read(path, None).unwrap();
            let uasset = back.read(&path.replace(".uexp", ".uasset"), None).unwrap();
            assert_eq!(texture::parse(&uasset, &uexp).unwrap(), size);
        }

        // Saved edits → Apply → Remove, writing into a temp ~mods instead of the game's.
        let mut cat = cat;
        cat.install.paks_dir = dir.join("Paks");
        let app_data = dir.join("appdata");
        let mut red = rgba.clone();
        red.chunks_exact_mut(4).for_each(|p| p[0] = 255);
        save_edit(&app_data, kit, "maillot", r#"{"v":1}"#, size, &red).unwrap();
        save_edit(&app_data, kit, "maillot_zzz", r#"{"v":1}"#, size, &rgba).unwrap();
        assert_eq!(list_edits(&app_data, None).len(), 2);
        assert_eq!(read_edit(&app_data, kit, "maillot").unwrap().1, red);
        assert!(save_edit(&app_data, "../evil", "maillot", "{}", size, &rgba).is_err());
        let report = apply(&cat, &app_data, Some(oodle)).unwrap();
        assert_eq!((report.parts, report.skipped.len()), (2, 0), "{:?}", report.skipped);
        let applied = Pak::open(&cat.install.override_pak(), |_| true).unwrap();
        assert_eq!(applied.entries.len(), 4);
        let uexp = applied.read(&maillot_uexp, None).unwrap();
        let mut back_px = texture::pixels(&uexp, size).to_vec();
        texture::swap_rb(&mut back_px);
        assert_eq!(back_px, red);
        delete_edit(&app_data, kit, "maillot").unwrap();
        delete_edit(&app_data, kit, "maillot_zzz").unwrap();
        apply(&cat, &app_data, Some(oodle)).unwrap(); // no edits left: removes the pak
        assert!(!cat.install.override_pak().exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

