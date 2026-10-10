// `cdb` and `edit` are public so local scripts in examples/ can reuse them.
pub mod cdb;
pub mod edit;
mod kits;
mod oodle;
mod pak;
mod parser;
mod texture;

#[cfg(target_os = "windows")]
mod dwm {
    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attr: u32, data: *const std::ffi::c_void, sz: u32) -> i32;
    }
    pub fn tint_window(hwnd: isize) {
        // DWMWA_BORDER_COLOR=34, DWMWA_CAPTION_COLOR=35, DWMWA_TEXT_COLOR=36 (COLORREF = 0x00BBGGRR)
        let caption: u32 = 0x170F0B; // #0b0f17
        let border: u32 = 0x2A2018; // #18202a
        let text: u32 = 0xF2ECE6; // #e6ecf2
        unsafe {
            DwmSetWindowAttribute(hwnd, 35, &caption as *const _ as _, 4);
            DwmSetWindowAttribute(hwnd, 34, &border as *const _ as _, 4);
            DwmSetWindowAttribute(hwnd, 36, &text as *const _ as _, 4);
        }
    }
}

use cdb::{Cdb, Cell, TableSummary};
use edit::{BackupInfo, CellEdit, EditOutcome};
use parser::SaveData;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;
use tauri::Manager;

struct Loaded {
    path: PathBuf,
    db: Cdb,
}

#[derive(Default)]
struct AppState {
    loaded: Mutex<Option<Loaded>>,
    /// Kit catalog for the save it was built from (reading pak indexes takes a few seconds).
    kits: Mutex<Option<(PathBuf, std::sync::Arc<kits::Catalog>)>>,
}

fn candidate_parents() -> Vec<PathBuf> {
    let mut bases = Vec::new();
    if let Ok(dir) = std::env::current_dir() {
        bases.push(dir.clone());
        if let Some(parent) = dir.parent() {
            bases.push(parent.to_path_buf());
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        for _ in 0..3 {
            if let Some(d) = dir {
                bases.push(d.clone());
                dir = d.parent().map(Path::to_path_buf);
            }
        }
    }
    bases.sort();
    bases.dedup();
    bases
}

fn resolve_save_path(path: &str) -> Result<PathBuf, String> {
    let requested = PathBuf::from(path);
    if requested.is_file() {
        return Ok(requested.canonicalize().map(strip_unc).unwrap_or(requested));
    }
    if !requested.is_absolute() {
        for base in candidate_parents() {
            let candidate = base.join(&requested);
            if candidate.is_file() {
                return Ok(candidate.canonicalize().map(strip_unc).unwrap_or(candidate));
            }
        }
    }
    Err(format!("Save file not found: {path}"))
}

fn strip_unc(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) => PathBuf::from(rest),
        None => p,
    }
}

fn app_data(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn modified_ms(path: &Path) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

/// Runs `f` against the cached database for `path`, loading it first if needed.
fn with_db<T>(state: &AppState, path: &str, f: impl FnOnce(&Cdb) -> T) -> Result<T, String> {
    let resolved = resolve_save_path(path)?;
    let mut guard = state.loaded.lock().map_err(|_| "State lock poisoned")?;
    if guard.as_ref().map(|l| l.path != resolved).unwrap_or(true) {
        let db = Cdb::open(&resolved.to_string_lossy())?;
        *guard = Some(Loaded { path: resolved, db });
    }
    Ok(f(&guard.as_ref().unwrap().db))
}

// ─── Loading ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct LoadResponse {
    #[serde(flatten)]
    data: SaveData,
    modified_ms: u64,
}

#[tauri::command]
async fn load_save(state: tauri::State<'_, AppState>, path: String) -> Result<LoadResponse, String> {
    let resolved = resolve_save_path(&path)?;
    let (db, data, modified) = blocking(move || {
        let p = resolved.to_string_lossy().to_string();
        let db = Cdb::open(&p)?;
        let data = parser::extract(&db, &p)?;
        Ok((db, data, (resolved, modified_ms(Path::new(&p)))))
    })
    .await?;
    let (resolved, modified_ms) = modified;
    eprintln!("[load_save] {} → {} riders, {} teams", resolved.display(), data.cyclists.len(), data.teams.len());
    *state.loaded.lock().map_err(|_| "State lock poisoned")? = Some(Loaded { path: resolved, db });
    Ok(LoadResponse { data, modified_ms })
}

#[tauri::command]
fn save_modified(path: String) -> u64 {
    resolve_save_path(&path).map(|p| modified_ms(&p)).unwrap_or(0)
}

#[tauri::command]
fn find_default_save(filename: String) -> Option<String> {
    resolve_save_path(&filename).ok().map(|p| p.to_string_lossy().to_string())
}

#[derive(Serialize)]
struct SaveFile {
    path: String,
    name: String,
    game: String,
    modified_ms: u64,
    size: u64,
}

/// Lists career saves in every installed PCM's cloud folder (newest first).
#[tauri::command]
fn find_saves() -> Vec<SaveFile> {
    let mut out = Vec::new();
    let mut push = |path: PathBuf, game: String| {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if !name.to_lowercase().starts_with("career") {
            return;
        }
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        out.push(SaveFile { modified_ms: modified_ms(&path), path: path.to_string_lossy().to_string(), name, game, size });
    };
    if let Ok(appdata) = std::env::var("APPDATA") {
        if let Ok(games) = std::fs::read_dir(&appdata) {
            for g in games.flatten() {
                let game = g.file_name().to_string_lossy().to_string();
                if !game.starts_with("Pro Cycling Manager") {
                    continue;
                }
                let Ok(users) = std::fs::read_dir(g.path().join("Cloud")) else { continue };
                for u in users.flatten().filter(|u| u.path().is_dir()) {
                    let Ok(files) = std::fs::read_dir(u.path()) else { continue };
                    for f in files.flatten() {
                        if f.path().extension().map(|e| e.eq_ignore_ascii_case("cdb")).unwrap_or(false) {
                            push(f.path(), game.replace("Pro Cycling Manager", "PCM").trim().to_string());
                        }
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    out
}

// ─── Editing ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct EditResponse {
    outcomes: Vec<EditOutcome>,
    backup: String,
    #[serde(flatten)]
    data: SaveData,
    modified_ms: u64,
}

#[tauri::command]
async fn apply_edits(app: tauri::AppHandle, state: tauri::State<'_, AppState>, path: String, edits: Vec<CellEdit>) -> Result<EditResponse, String> {
    let save = resolve_save_path(&path)?;
    let data_dir = app_data(&app)?;
    let save2 = save.clone();
    let (res, data) = blocking(move || {
        let res = edit::edit_save(&data_dir, &save2, &edits)?;
        let data = parser::extract(&res.db, &save2.to_string_lossy())?;
        Ok((res, data))
    })
    .await?;
    let modified = modified_ms(&save);
    eprintln!("[apply_edits] {} change(s) written to {}", res.outcomes.len(), save.display());
    let backup = res.backup.to_string_lossy().to_string();
    *state.loaded.lock().map_err(|_| "State lock poisoned")? = Some(Loaded { path: save, db: res.db });
    Ok(EditResponse { outcomes: res.outcomes, backup, data, modified_ms: modified })
}

#[tauri::command]
fn list_backups(app: tauri::AppHandle, path: String) -> Result<Vec<BackupInfo>, String> {
    let save = resolve_save_path(&path)?;
    Ok(edit::list_backups(&app_data(&app)?, &save))
}

#[tauri::command]
async fn restore_backup(app: tauri::AppHandle, state: tauri::State<'_, AppState>, path: String, backup: String) -> Result<LoadResponse, String> {
    let save = resolve_save_path(&path)?;
    let data_dir = app_data(&app)?;
    let save2 = save.clone();
    let (db, data) = blocking(move || {
        let db = edit::restore(&data_dir, &save2, Path::new(&backup))?;
        let data = parser::extract(&db, &save2.to_string_lossy())?;
        Ok((db, data))
    })
    .await?;
    let modified = modified_ms(&save);
    *state.loaded.lock().map_err(|_| "State lock poisoned")? = Some(Loaded { path: save, db });
    Ok(LoadResponse { data, modified_ms: modified })
}

#[tauri::command]
fn open_backup_folder(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let save = resolve_save_path(&path)?;
    let dir = edit::backup_dir(&app_data(&app)?, &save);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    reveal(&dir)
}

fn reveal(dir: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let cmd = "explorer";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let cmd = "xdg-open";
    Command::new(cmd).arg(dir).spawn().map(|_| ()).map_err(|e| e.to_string())
}

// ─── Kits ─────────────────────────────────────────────────────────────────────

fn oodle_if_ready(app: &tauri::AppHandle) -> Result<Option<&'static oodle::Oodle>, String> {
    let dll = oodle::dll_path(&app_data(app)?);
    if dll.is_file() {
        oodle::load(&dll).map(Some)
    } else {
        Ok(None)
    }
}

/// The kit catalog for `save_path`, built on first use and cached.
fn kit_catalog(state: &AppState, save_path: &str) -> Result<std::sync::Arc<kits::Catalog>, String> {
    let save = resolve_save_path(save_path)?;
    if let Some((p, cat)) = state.kits.lock().map_err(|_| "State lock poisoned")?.as_ref() {
        if *p == save {
            return Ok(cat.clone());
        }
    }
    let install = kits::find_install(&save)?;
    let mod_pak = with_db(state, save_path, kits::find_mod_pak)?;
    let cat = std::sync::Arc::new(kits::build_catalog(install, mod_pak)?);
    *state.kits.lock().map_err(|_| "State lock poisoned")? = Some((save, cat.clone()));
    Ok(cat)
}

#[derive(Serialize)]
struct KitStatus {
    game: String,
    mod_label: Option<String>,
    oodle_ready: bool,
    override_path: String,
    override_installed: bool,
    /// When the override pak was last written (0 if absent), to spot edits made since.
    override_modified_ms: u64,
    kits: Vec<kits::KitSummary>,
}

#[tauri::command]
async fn kit_status(app: tauri::AppHandle, state: tauri::State<'_, AppState>, save_path: String) -> Result<KitStatus, String> {
    let cat = kit_catalog(&state, &save_path)?;
    let p = cat.install.override_pak();
    Ok(KitStatus {
        game: cat.install.game.clone(),
        mod_label: cat.mod_label.clone(),
        oodle_ready: oodle::dll_path(&app_data(&app)?).is_file(),
        override_path: p.to_string_lossy().into(),
        override_installed: p.is_file(),
        override_modified_ms: modified_ms(&p),
        kits: cat.summaries(),
    })
}

#[tauri::command]
async fn kit_oodle_download(app: tauri::AppHandle) -> Result<(), String> {
    let dir = app_data(&app)?;
    blocking(move || oodle::ensure_downloaded(&dir).map(|_| ())).await
}

#[tauri::command]
fn kit_oodle_pick(app: tauri::AppHandle, path: String) -> Result<(), String> {
    oodle::install_from(&app_data(&app)?, Path::new(&path))
}

fn image_response(size: texture::Size, rgba: Vec<u8>) -> tauri::ipc::Response {
    let mut out = Vec::with_capacity(8 + rgba.len());
    out.extend_from_slice(&size.width.to_le_bytes());
    out.extend_from_slice(&size.height.to_le_bytes());
    out.extend_from_slice(&rgba);
    tauri::ipc::Response::new(out)
}

/// A kit part as `u32 width, u32 height, RGBA…`; `edited` returns the user's version.
#[tauri::command]
async fn kit_image(app: tauri::AppHandle, state: tauri::State<'_, AppState>, save_path: String, kit: String, part: String, edited: bool) -> Result<tauri::ipc::Response, String> {
    if edited {
        let dir = app_data(&app)?;
        let (size, px) = blocking(move || kits::read_edit(&dir, &kit, &part)).await?;
        return Ok(image_response(size, px));
    }
    let cat = kit_catalog(&state, &save_path)?;
    let oodle = oodle_if_ready(&app)?;
    let (size, px) = blocking(move || cat.read_part(&kit, &part, oodle)).await?;
    Ok(image_response(size, px))
}

#[tauri::command]
fn kit_edits(app: tauri::AppHandle, kit: Option<String>) -> Result<Vec<kits::EditInfo>, String> {
    Ok(kits::list_edits(&app_data(&app)?, kit.as_deref()))
}

/// Raw body: u32 recipe length, recipe JSON, RGBA pixels. Headers: kit, part, width, height.
#[tauri::command]
fn kit_save_edit(app: tauri::AppHandle, request: tauri::ipc::Request<'_>) -> Result<(), String> {
    let header = |k: &str| request.headers().get(k).and_then(|v| v.to_str().ok()).map(str::to_string).ok_or_else(|| format!("Missing {k}"));
    let (kit, part) = (header("kit")?, header("part")?);
    let num = |k: &str| -> Result<u32, String> { header(k)?.parse().map_err(|_| format!("Bad {k}")) };
    let size = texture::Size { width: num("width")?, height: num("height")? };
    let tauri::ipc::InvokeBody::Raw(body) = request.body() else { return Err("Expected raw image data".into()) };
    let len = u32::from_le_bytes(body.get(0..4).ok_or("Empty body")?.try_into().unwrap()) as usize;
    let recipe = std::str::from_utf8(body.get(4..4 + len).ok_or("Bad body")?).map_err(|e| e.to_string())?;
    kits::save_edit(&app_data(&app)?, &kit, &part, recipe, size, &body[4 + len..])
}

#[tauri::command]
fn kit_delete_edit(app: tauri::AppHandle, kit: String, part: String) -> Result<(), String> {
    kits::delete_edit(&app_data(&app)?, &kit, &part)
}

#[tauri::command]
fn kit_meta_get(app: tauri::AppHandle, key: String) -> Result<Option<String>, String> {
    kits::meta_get(&app_data(&app)?, &key)
}

#[tauri::command]
fn kit_meta_set(app: tauri::AppHandle, key: String, value: Option<String>) -> Result<(), String> {
    kits::meta_set(&app_data(&app)?, &key, value.as_deref())
}

#[tauri::command]
async fn kit_apply(app: tauri::AppHandle, state: tauri::State<'_, AppState>, save_path: String) -> Result<kits::ApplyReport, String> {
    let cat = kit_catalog(&state, &save_path)?;
    let oodle = oodle_if_ready(&app)?;
    let dir = app_data(&app)?;
    blocking(move || kits::apply(&cat, &dir, oodle)).await
}

#[tauri::command]
async fn kit_remove(state: tauri::State<'_, AppState>, save_path: String) -> Result<(), String> {
    let cat = kit_catalog(&state, &save_path)?;
    kits::remove(&cat.install)
}

#[tauri::command]
async fn kit_squad_nations(state: tauri::State<'_, AppState>, save_path: String, team_id: i32) -> Result<Vec<kits::Nation>, String> {
    with_db(&state, &save_path, |db| kits::squad_nations(db, team_id))
}

/// Points a team at another kit and/or changes its colours (`rrggbb`).
#[tauri::command]
async fn set_team_kit(app: tauri::AppHandle, state: tauri::State<'_, AppState>, path: String, team_id: i32, jersey: String, color1: String, color2: String) -> Result<EditResponse, String> {
    let hex = |c: &str| -> Result<String, String> {
        let c = c.trim_start_matches('#').to_lowercase();
        if c.len() == 6 && c.chars().all(|ch| ch.is_ascii_hexdigit()) { Ok(c) } else { Err(format!("{c} isn't a colour")) }
    };
    let (color1, color2) = (hex(&color1)?, hex(&color2)?);
    if jersey.is_empty() || jersey.len() > 64 {
        return Err("Pick a kit".into());
    }
    let save = resolve_save_path(&path)?;
    let data_dir = app_data(&app)?;
    let save2 = save.clone();
    let (outcomes, backup, db, data) = blocking(move || {
        let (outcomes, backup, db) = edit::transform(&data_dir, &save2, |db| {
            let t = "DYN_team";
            let row = db.find_row_int(t, "IDteam", team_id as i64).ok_or("Team not found")?;
            let mut outcomes = Vec::new();
            let mut cols = Vec::new();
            for (col, value) in [("jersey_sz_abbreviation", &jersey), ("gene_sz_color", &color1), ("gene_sz_secondary_color", &color2)] {
                let mut v = db.raw_strings(t, col).ok_or_else(|| format!("{t}.{col} missing"))?;
                let old = cdb::decode_text(&v[row]);
                if old != *value {
                    eprintln!("[set_team_kit] {col}: {old} → {value}");
                    v[row] = cdb::encode_text(value);
                    cols.push((col, cdb::Values::Text(v)));
                    outcomes.push(edit::EditOutcome { table: t.into(), column: col.into(), key: team_id.to_string(), old: 0.0, new: 0.0 });
                }
            }
            if cols.is_empty() {
                return Err("Nothing to change".into());
            }
            db.rewrite_columns(t, &cols)?;
            Ok(outcomes)
        })?;
        let data = parser::extract(&db, &save2.to_string_lossy())?;
        Ok((outcomes, backup, db, data))
    })
    .await?;
    let modified = modified_ms(&save);
    *state.loaded.lock().map_err(|_| "State lock poisoned")? = Some(Loaded { path: save, db });
    Ok(EditResponse { outcomes, backup: backup.to_string_lossy().into(), data, modified_ms: modified })
}

/// Reads a user-picked file (logos, imported kit images).
#[tauri::command]
async fn read_file_bytes(path: String) -> Result<tauri::ipc::Response, String> {
    blocking(move || std::fs::read(&path).map_err(|e| format!("Cannot read {path}: {e}"))).await.map(tauri::ipc::Response::new)
}

/// Writes a user-picked PNG (kit export). Header: path.
#[tauri::command]
fn write_png(request: tauri::ipc::Request<'_>) -> Result<(), String> {
    let path = request.headers().get("path").and_then(|v| v.to_str().ok()).ok_or("Missing path")?;
    let path = urlencoding_decode(path);
    if !path.to_lowercase().ends_with(".png") {
        return Err("Only .png files can be written".into());
    }
    let tauri::ipc::InvokeBody::Raw(body) = request.body() else { return Err("Expected raw data".into()) };
    std::fs::write(&path, body).map_err(|e| format!("Cannot write {path}: {e}"))
}

/// Headers are ASCII, so the frontend percent-encodes the path.
fn urlencoding_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ─── Database browser ─────────────────────────────────────────────────────────

#[tauri::command]
async fn db_tables(state: tauri::State<'_, AppState>, path: String) -> Result<Vec<TableSummary>, String> {
    with_db(&state, &path, |db| db.summaries())
}

#[tauri::command]
async fn db_page(state: tauri::State<'_, AppState>, path: String, table: String, offset: usize, limit: usize) -> Result<Vec<Vec<Cell>>, String> {
    with_db(&state, &path, |db| db.page(&table, offset, limit.min(2000)))?.ok_or_else(|| format!("Table {table} not found"))
}

// ─── Workspace (notes, shortlist, settings) ──────────────────────────────────

#[tauri::command]
fn load_workspace(app: tauri::AppHandle) -> serde_json::Value {
    app_data(&app)
        .ok()
        .and_then(|d| std::fs::read_to_string(d.join("workspace.json")).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Object(Default::default()))
}

#[tauri::command]
fn save_workspace(app: tauri::AppHandle, data: serde_json::Value) -> Result<(), String> {
    let dir = app_data(&app)?;
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let tmp = dir.join("workspace.json.tmp");
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, dir.join("workspace.json")).map_err(|e| e.to_string())
}

/// Legacy notes written next to the save by v2.0.x.
#[tauri::command]
fn load_notes(path: String) -> serde_json::Value {
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Object(Default::default()))
}

#[tauri::command]
fn export_csv(path: String, data: Vec<serde_json::Value>, fields: Vec<String>) -> Result<(), String> {
    let esc = |s: String| if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s };
    let mut out = fields.iter().cloned().map(esc).collect::<Vec<_>>().join(",");
    out.push('\n');
    for row in &data {
        let values: Vec<String> = fields
            .iter()
            .map(|f| match &row[f] {
                serde_json::Value::String(s) => esc(s.clone()),
                serde_json::Value::Null => String::new(),
                other => esc(other.to_string()),
            })
            .collect();
        out.push_str(&values.join(","));
        out.push('\n');
    }
    let mut bytes = vec![0xEF_u8, 0xBB, 0xBF]; // UTF-8 BOM for Excel
    bytes.extend_from_slice(out.as_bytes());
    std::fs::write(&path, bytes).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Only http(s) URLs are allowed".into());
    }
    #[cfg(target_os = "windows")]
    return Command::new("cmd").args(["/C", "start", "", &url]).spawn().map(|_| ()).map_err(|e| e.to_string());
    #[cfg(target_os = "macos")]
    return Command::new("open").arg(&url).spawn().map(|_| ()).map_err(|e| e.to_string());
    #[cfg(all(unix, not(target_os = "macos")))]
    return Command::new("xdg-open").arg(&url).spawn().map(|_| ()).map_err(|e| e.to_string());
    #[allow(unreachable_code)]
    Err("Unsupported platform".into())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            #[cfg(target_os = "windows")]
            if let Some(w) = app.get_webview_window("main") {
                if let Ok(hwnd) = w.hwnd() {
                    dwm::tint_window(hwnd.0 as isize);
                }
            }
            let _ = app;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_save,
            save_modified,
            find_default_save,
            find_saves,
            apply_edits,
            list_backups,
            restore_backup,
            open_backup_folder,
            kit_status,
            kit_oodle_download,
            kit_oodle_pick,
            kit_image,
            kit_edits,
            kit_save_edit,
            kit_delete_edit,
            kit_apply,
            kit_meta_get,
            kit_meta_set,
            kit_remove,
            kit_squad_nations,
            set_team_kit,
            read_file_bytes,
            write_png,
            db_tables,
            db_page,
            load_workspace,
            save_workspace,
            load_notes,
            export_csv,
            open_external,
        ])
        .run(tauri::generate_context!())
        .expect("error running PCM Recon");
}
