mod parser;

#[cfg(target_os = "windows")]
mod dwm {
    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attr: u32, data: *const std::ffi::c_void, sz: u32) -> i32;
    }
    pub fn tint_window(hwnd: isize) {
        // DWMWA_BORDER_COLOR=34, DWMWA_CAPTION_COLOR=35, DWMWA_TEXT_COLOR=36
        // Colors are COLORREF = 0x00BBGGRR
        let caption: u32 = 0x22140B; // #0b1422
        let border:  u32 = 0x482D1C; // #1c2d48
        let text:    u32 = 0xFFE8DC; // #dce8ff
        unsafe {
            DwmSetWindowAttribute(hwnd, 35, &caption as *const _ as _, 4);
            DwmSetWindowAttribute(hwnd, 34, &border  as *const _ as _, 4);
            DwmSetWindowAttribute(hwnd, 36, &text    as *const _ as _, 4);
        }
    }
}

use parser::SaveData;
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;

fn candidate_parents() -> Vec<PathBuf> {
    let mut bases = Vec::new();

    if let Ok(dir) = std::env::current_dir() {
        bases.push(dir);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let dir = dir.to_path_buf();
            bases.push(dir.clone());
            if let Some(parent) = dir.parent() {
                bases.push(parent.to_path_buf());
                if let Some(grandparent) = parent.parent() {
                    bases.push(grandparent.to_path_buf());
                }
            }
        }
    }

    bases.sort();
    bases.dedup();
    bases
}

fn resolve_save_path(path: &str) -> Result<PathBuf, String> {
    let requested = PathBuf::from(path);
    if requested.is_absolute() && requested.is_file() {
        return Ok(requested);
    }

    if requested.is_file() {
        return requested
            .canonicalize()
            .or(Ok(requested));
    }

    for base in candidate_parents() {
        let candidate = base.join(&requested);
        if candidate.is_file() {
            return candidate
                .canonicalize()
                .or(Ok(candidate));
        }
    }

    Err(format!(
        "Cannot read file: {}",
        std::io::Error::from(std::io::ErrorKind::NotFound)
    ))
}

#[tauri::command]
fn load_save(path: String) -> Result<SaveData, String> {
    let resolved = resolve_save_path(&path)?;
    eprintln!("[load_save] path = {:?} -> {:?}", path, resolved);
    let result = parser::extract(&resolved.to_string_lossy());
    match &result {
        Ok(d)  => eprintln!("[load_save] OK: {} cyclists, {} teams", d.cyclists.len(), d.teams.len()),
        Err(e) => eprintln!("[load_save] ERR: {}", e),
    }
    result
}

#[tauri::command]
fn find_default_save(filename: String) -> Option<String> {
    resolve_save_path(&filename)
        .ok()
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
fn open_save_dialog(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("PCM Save File", &["cdb"])
        .blocking_pick_file()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn save_notes(path: String, notes: serde_json::Value) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&notes).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_notes(path: String) -> serde_json::Value {
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Object(Default::default()))
}

#[tauri::command]
fn export_csv(path: String, data: Vec<serde_json::Value>, fields: Vec<String>) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&fields.join(","));
    out.push('\n');
    for row in &data {
        let values: Vec<String> = fields.iter().map(|f| {
            let v = &row[f];
            let s = match v {
                serde_json::Value::String(s) => s.replace('"', "\"\""),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b)   => b.to_string(),
                serde_json::Value::Null      => String::new(),
                other => other.to_string().replace('"', "\"\""),
            };
            if s.contains(',') || s.contains('"') || s.contains('\n') {
                format!("\"{}\"", s)
            } else {
                s
            }
        }).collect();
        out.push_str(&values.join(","));
        out.push('\n');
    }
    // UTF-8 BOM for Excel compatibility
    let mut bytes = vec![0xEF_u8, 0xBB, 0xBF];
    bytes.extend_from_slice(out.as_bytes());
    std::fs::write(&path, bytes).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                if let Some(w) = app.get_webview_window("main") {
                    if let Ok(hwnd) = w.hwnd() {
                        dwm::tint_window(hwnd.0 as isize);
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_save,
            find_default_save,
            open_save_dialog,
            save_notes,
            load_notes,
            export_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error running PCM Recon");
}
