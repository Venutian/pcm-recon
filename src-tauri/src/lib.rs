mod cdb;
mod edit;
mod names;
mod parser;

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

// ─── Name packs ───────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct NameDb {
    game: String,
    path: String,
    current_first: Vec<String>,
    current_last: Vec<String>,
}

#[derive(Serialize)]
struct NamePacksInfo {
    packs: Vec<names::PackInfo>,
    databases: Vec<NameDb>,
    copies_folder: String,
    /// Country id of the pack's nation, taken from the loaded save (the name database has no country table).
    country_id: i32,
}

fn pack_or_err(key: &str) -> Result<&'static names::NamePack, String> {
    names::pack(key).ok_or_else(|| format!("Unknown name pack {key}"))
}

fn copies_folder(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .document_dir()
        .unwrap_or_else(|_| app_data(app).unwrap_or_default())
        .join("PCM Recon")
        .join("Name packs")
}

#[tauri::command]
async fn name_packs(app: tauri::AppHandle, state: tauri::State<'_, AppState>, pack: String, save_path: String) -> Result<NamePacksInfo, String> {
    let p = pack_or_err(&pack)?;
    let folder = copies_folder(&app);
    let country_id = with_db(&state, &save_path, |db| names::country_id(db, p.country_iso))?
        .ok_or_else(|| format!("{} isn't a country in this save", p.country_iso.to_uppercase()))?;
    blocking(move || {
        let mut databases = Vec::new();
        for (game, path) in names::find_name_databases() {
            let db = Cdb::open(&path.to_string_lossy())?;
            let (current_first, current_last) = names::current_lists(&db, country_id);
            databases.push(NameDb { game, path: path.to_string_lossy().into(), current_first, current_last });
        }
        Ok(NamePacksInfo { packs: names::PACKS.iter().map(|p| names::pack_info(p)).collect(), databases, copies_folder: folder.to_string_lossy().into(), country_id })
    })
    .await
}

#[derive(Serialize)]
struct GameListResult {
    report: names::ListReport,
    backup: String,
    copy: String,
}

/// Rewrites the game's generator name lists and keeps copies (original + edited) in Documents,
/// so the change can be restored or re-applied after a game update.
#[tauri::command]
async fn apply_name_pack_to_game(app: tauri::AppHandle, pack: String, path: String, country_id: i32) -> Result<GameListResult, String> {
    let p = pack_or_err(&pack)?;
    let file = PathBuf::from(&path);
    if file.file_name().map(|f| f != "OfficialLocal.cdb").unwrap_or(true) {
        return Err("Pick the game's OfficialLocal.cdb".into());
    }
    let data_dir = app_data(&app)?;
    let folder = copies_folder(&app);
    blocking(move || {
        std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let original = folder.join("OfficialLocal.original.cdb");
        if !original.exists() {
            std::fs::copy(&file, &original).map_err(|e| format!("Cannot keep the original copy: {e}"))?;
        }
        let (report, backup, db) = edit::transform(&data_dir, &file, |db| names::apply_to_lists(db, country_id, p))?;
        let copy = folder.join(format!("OfficialLocal.{}.cdb", p.key));
        std::fs::write(&copy, db.to_file_bytes()?).map_err(|e| e.to_string())?;
        let note = format!(
            "PCM Recon name pack: {}

OfficialLocal.original.cdb  the game's file before any change
{}  the file with the name pack applied

After a game update, open PCM Recon > Name packs and press \"Apply to the game\" again.
That keeps any new content from the update. Copying the edited file back by hand also works, but it
undoes whatever the update changed in that file.

The game reads: {}
",
            p.label, copy.file_name().unwrap().to_string_lossy(), file.display()
        );
        let _ = std::fs::write(folder.join("README.txt"), note);
        Ok(GameListResult { report, backup: backup.to_string_lossy().into(), copy: copy.to_string_lossy().into() })
    })
    .await
}

#[derive(Serialize)]
struct SaveRenameResult {
    renames: Vec<names::Rename>,
    backup: String,
    #[serde(flatten)]
    data: SaveData,
    modified_ms: u64,
}

/// Fixes the names of riders the game already generated in this career.
#[tauri::command]
async fn apply_name_pack_to_save(app: tauri::AppHandle, state: tauri::State<'_, AppState>, pack: String, path: String) -> Result<SaveRenameResult, String> {
    let p = pack_or_err(&pack)?;
    let save = resolve_save_path(&path)?;
    let data_dir = app_data(&app)?;
    let save2 = save.clone();
    let (renames, backup, db, data) = blocking(move || {
        let (renames, backup, db) = edit::transform(&data_dir, &save2, |db| {
            let start = db.ints("GAM_config", "game_i_starting_year").and_then(|v| v.first().copied()).unwrap_or(2000);
            let real = names::base_database_for(db).map(|b| names::real_rider_names(&b)).unwrap_or_default();
            // Riders born before the career's first junior class can't have been generated.
            names::rename_generated(db, p, &real, start - 19)
        })?;
        let data = parser::extract(&db, &save2.to_string_lossy())?;
        Ok((renames, backup, db, data))
    })
    .await?;
    let modified = modified_ms(&save);
    *state.loaded.lock().map_err(|_| "State lock poisoned")? = Some(Loaded { path: save, db });
    Ok(SaveRenameResult { renames, backup: backup.to_string_lossy().into(), data, modified_ms: modified })
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
            name_packs,
            apply_name_pack_to_game,
            apply_name_pack_to_save,
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
