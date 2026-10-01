//! Save editing: numeric cell edits, backups and safe writes.
use crate::cdb::Cdb;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BACKUPS: usize = 30;

/// One numeric cell change, addressed by a key column (e.g. `IDteam = 4`,
/// or `CONSTANT = "SOLDE"`) rather than by row number so it survives reloads.
#[derive(Debug, Deserialize, Clone)]
pub struct CellEdit {
    pub table: String,
    pub column: String,
    pub key_column: String,
    pub key: serde_json::Value,
    pub value: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct EditOutcome {
    pub table: String,
    pub column: String,
    pub key: String,
    pub old: f64,
    pub new: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct BackupInfo {
    pub path: String,
    pub file_name: String,
    pub created_ms: u64,
    pub size: u64,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn locate_row(db: &Cdb, e: &CellEdit) -> Result<usize, String> {
    let row = match &e.key {
        serde_json::Value::Number(n) => n.as_i64().and_then(|k| db.find_row_int(&e.table, &e.key_column, k)),
        serde_json::Value::String(s) => db.find_row_str(&e.table, &e.key_column, s),
        _ => return Err("Edit key must be a number or a string".into()),
    };
    row.ok_or_else(|| format!("No row in {} where {} = {}", e.table, e.key_column, e.key))
}

/// Applies edits in memory. Identifier columns are protected because changing them
/// would break relations between tables.
pub fn apply(db: &mut Cdb, edits: &[CellEdit]) -> Result<Vec<EditOutcome>, String> {
    let mut out = Vec::with_capacity(edits.len());
    for e in edits {
        if e.column.starts_with("ID") || e.column.starts_with("fkID") {
            return Err(format!("{}.{} is an identifier and cannot be edited", e.table, e.column));
        }
        let row = locate_row(db, e)?;
        let old = db.set_number(&e.table, &e.column, row, e.value)?;
        out.push(EditOutcome {
            table: e.table.clone(),
            column: e.column.clone(),
            key: e.key.to_string().trim_matches('"').to_string(),
            old,
            new: e.value,
        });
    }
    Ok(out)
}

/// Re-reads `bytes`, confirming the edited values landed and the file still parses.
fn verify(bytes: &[u8], edits: &[CellEdit]) -> Result<(), String> {
    let db = Cdb::from_file_bytes(bytes).map_err(|e| format!("Verification failed: {e}"))?;
    for e in edits {
        let row = locate_row(&db, e)?;
        let got = match db.cell(&e.table, &e.column, row) {
            crate::cdb::Cell::Int(v) => v as f64,
            crate::cdb::Cell::Float(v) => v as f64,
            _ => return Err(format!("Verification failed for {}.{}", e.table, e.column)),
        };
        let expected = if db.floats(&e.table, &e.column).is_some() { e.value as f32 as f64 } else { e.value };
        if (got - expected).abs() > 0.5 {
            return Err(format!("Verification failed for {}.{}: wrote {expected}, read back {got}", e.table, e.column));
        }
    }
    Ok(())
}

/// Writes `bytes` next to `path` and swaps it in, so a crash never leaves a half-written save.
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("pcmrecon-tmp");
    std::fs::write(&tmp, bytes).map_err(|e| format!("Cannot write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("Cannot replace {}: {e}. Is the save open in another program?", path.display())
    })
}

fn path_tag(save: &Path) -> String {
    // Same file name can exist for several games (PCM 2025/2026), so tag by full path.
    let mut h: u32 = 0x811C9DC5;
    for b in save.to_string_lossy().to_lowercase().bytes() {
        h = (h ^ b as u32).wrapping_mul(0x01000193);
    }
    let stem = save.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "save".into());
    format!("{stem}-{h:08x}")
}

pub fn backup_dir(app_data: &Path, save: &Path) -> PathBuf {
    app_data.join("backups").join(path_tag(save))
}

pub(crate) fn create_backup(app_data: &Path, save: &Path, raw: &[u8]) -> Result<PathBuf, String> {
    let dir = backup_dir(app_data, save);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create backup folder: {e}"))?;
    let stem = save.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "save".into());
    let target = dir.join(format!("{stem}__{}.cdb", now_ms()));
    std::fs::write(&target, raw).map_err(|e| format!("Cannot write backup: {e}"))?;
    let cdi = save.with_extension("cdi");
    if cdi.is_file() {
        let _ = std::fs::copy(&cdi, target.with_extension("cdi"));
    }
    prune(&dir);
    Ok(target)
}

fn prune(dir: &Path) {
    let mut all = list_dir(dir);
    if all.len() <= MAX_BACKUPS {
        return;
    }
    all.sort_by(|a, b| b.created_ms.cmp(&a.created_ms));
    for old in &all[MAX_BACKUPS..] {
        let p = PathBuf::from(&old.path);
        let _ = std::fs::remove_file(&p);
        let _ = std::fs::remove_file(p.with_extension("cdi"));
    }
}

fn list_dir(dir: &Path) -> Vec<BackupInfo> {
    let Ok(rd) = std::fs::read_dir(dir) else { return vec![] };
    rd.filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "cdb").unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let created_ms = name.rsplit("__").next()?.trim_end_matches(".cdb").parse().ok()?;
            Some(BackupInfo {
                path: e.path().to_string_lossy().to_string(),
                file_name: name,
                created_ms,
                size: e.metadata().map(|m| m.len()).unwrap_or(0),
            })
        })
        .collect()
}

pub fn list_backups(app_data: &Path, save: &Path) -> Vec<BackupInfo> {
    let mut v = list_dir(&backup_dir(app_data, save));
    v.sort_by(|a, b| b.created_ms.cmp(&a.created_ms));
    v
}

pub struct WriteResult {
    pub outcomes: Vec<EditOutcome>,
    pub backup: PathBuf,
    pub db: Cdb,
}

/// Reads the save fresh from disk, backs it up, applies the edits, verifies and writes it back.
pub fn edit_save(app_data: &Path, save: &Path, edits: &[CellEdit]) -> Result<WriteResult, String> {
    if edits.is_empty() {
        return Err("Nothing to change".into());
    }
    let raw = std::fs::read(save).map_err(|e| format!("Cannot read save: {e}"))?;
    let mut db = Cdb::from_file_bytes(&raw)?;
    let outcomes = apply(&mut db, edits)?;
    let bytes = db.to_file_bytes()?;
    verify(&bytes, edits)?;
    let backup = create_backup(app_data, save, &raw)?;
    atomic_write(save, &bytes)?;

    // The .cdi beside the save caches the team summary shown on the load screen.
    // Keep its team budget in sync; it is cosmetic, so failures are ignored.
    let cdi_edits: Vec<CellEdit> = edits.iter().filter(|e| e.table == "DYN_team").cloned().collect();
    let cdi = save.with_extension("cdi");
    if !cdi_edits.is_empty() && cdi.is_file() {
        if let Ok(mut small) = Cdb::open(&cdi.to_string_lossy()) {
            let applicable: Vec<CellEdit> = cdi_edits.into_iter().filter(|e| locate_row(&small, e).is_ok()).collect();
            if !applicable.is_empty() && apply(&mut small, &applicable).is_ok() {
                if let Ok(b) = small.to_file_bytes() {
                    let _ = atomic_write(&cdi, &b);
                }
            }
        }
    }
    Ok(WriteResult { outcomes, backup, db })
}

/// Reads a database fresh from disk, lets `change` modify it, verifies the result parses,
/// backs up the original and writes the new file atomically.
pub fn transform<T>(app_data: &Path, file: &Path, change: impl FnOnce(&mut Cdb) -> Result<T, String>) -> Result<(T, PathBuf, Cdb), String> {
    let raw = std::fs::read(file).map_err(|e| format!("Cannot read {}: {e}", file.display()))?;
    let mut db = Cdb::from_file_bytes(&raw)?;
    let result = change(&mut db)?;
    let bytes = db.to_file_bytes()?;
    Cdb::from_file_bytes(&bytes).map_err(|e| format!("Verification failed: {e}"))?;
    let backup = create_backup(app_data, file, &raw)?;
    atomic_write(file, &bytes)?;
    Ok((result, backup, db))
}

/// Restores a backup over the save, after backing up the current state.
pub fn restore(app_data: &Path, save: &Path, backup: &Path) -> Result<Cdb, String> {
    let dir = backup_dir(app_data, save);
    if backup.parent() != Some(dir.as_path()) {
        return Err("That backup does not belong to this save".into());
    }
    let bytes = std::fs::read(backup).map_err(|e| format!("Cannot read backup: {e}"))?;
    let db = Cdb::from_file_bytes(&bytes)?;
    if let Ok(current) = std::fs::read(save) {
        create_backup(app_data, save, &current)?;
    }
    atomic_write(save, &bytes)?;
    let cdi_backup = backup.with_extension("cdi");
    if cdi_backup.is_file() {
        let _ = std::fs::copy(&cdi_backup, save.with_extension("cdi"));
    }
    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_copy_with_backup() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
        let src = format!("{root}/Career_2 copy.cdb");
        if !Path::new(&src).is_file() {
            return;
        }
        let tmp = std::env::temp_dir().join(format!("pcmrecon-test-{}", now_ms()));
        std::fs::create_dir_all(&tmp).unwrap();
        let save = tmp.join("Career_T.cdb");
        std::fs::copy(&src, &save).unwrap();

        let edits = vec![
            CellEdit { table: "GAM_career_data".into(), column: "value".into(), key_column: "CONSTANT".into(), key: "SOLDE".into(), value: 1_500_000.0 },
            CellEdit { table: "DYN_team".into(), column: "value_i_budget".into(), key_column: "IDteam".into(), key: 4.into(), value: 3_000_000.0 },
        ];
        let res = edit_save(&tmp, &save, &edits).unwrap();
        assert_eq!(res.outcomes[0].old, -368_630.0);
        let db = Cdb::open(&save.to_string_lossy()).unwrap();
        assert_eq!(crate::parser::career_balance(&db), Some(1_500_000.0));
        assert_eq!(list_backups(&tmp, &save).len(), 1);

        let bad = vec![CellEdit { table: "DYN_team".into(), column: "IDteam".into(), key_column: "IDteam".into(), key: 4.into(), value: 5.0 }];
        assert!(edit_save(&tmp, &save, &bad).is_err());

        let backup = PathBuf::from(&list_backups(&tmp, &save)[0].path);
        restore(&tmp, &save, &backup).unwrap();
        let db = Cdb::open(&save.to_string_lossy()).unwrap();
        assert_eq!(crate::parser::career_balance(&db), Some(-368_630.0));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
