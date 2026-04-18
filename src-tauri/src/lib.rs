mod parser;

use parser::SaveData;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
fn load_save(path: String) -> Result<SaveData, String> {
    eprintln!("[load_save] path = {:?}", path);
    let result = parser::extract(&path);
    match &result {
        Ok(d)  => eprintln!("[load_save] OK: {} cyclists, {} teams", d.cyclists.len(), d.teams.len()),
        Err(e) => eprintln!("[load_save] ERR: {}", e),
    }
    result
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
        .invoke_handler(tauri::generate_handler![
            load_save,
            open_save_dialog,
            save_notes,
            load_notes,
            export_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error running PCM Recon");
}
