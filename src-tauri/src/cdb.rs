//! Structural reader/writer for Cyanide `.cdb` databases (Pro Cycling Manager saves).
//!
//! On disk: `FF FF FF FF | u32 raw_len | u32 zlib_len | zlib(raw)`.
//!
//! `raw` is a tree of blocks. Every block looks like
//! `AA AA AA AA | u32 size | u32 kind | u32 0 | u32 n | n × (u32 len, bytes padded to 4) | BB BB BB BB | payload | CC CC CC CC`
//! where `size` covers the whole block. Container payloads may start with
//! `DD DD DD DD | u32 count` and then hold their child blocks back to back.
//!
//! Layout: root → table list → tables → column list → columns. A column carries its
//! index, its data type and a cell block (`rows × u32`). String columns store the byte
//! length of every row in the cell block and the concatenated NUL-terminated text in a
//! separate heap block (`u32 total_len | bytes`).

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::ops::Range;

const AA: [u8; 4] = [0xAA; 4];
const BB: [u8; 4] = [0xBB; 4];
const CC: [u8; 4] = [0xCC; 4];
const DD: [u8; 4] = [0xDD; 4];
const HEADER_LEN: usize = 12;

mod kind {
    pub const TABLE_LIST: u32 = 0x01;
    pub const TABLE: u32 = 0x10;
    pub const ROW_COUNT: u32 = 0x11;
    pub const COLUMN_LIST: u32 = 0x12;
    pub const TABLE_ID: u32 = 0x15;
    pub const COLUMN: u32 = 0x20;
    pub const COLUMN_TYPE: u32 = 0x21;
    pub const CELLS: u32 = 0x22;
    pub const HEAP: u32 = 0x23;
    pub const COLUMN_INDEX: u32 = 0x24;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColType {
    Int,
    Float,
    Str,
    IntList,
    FloatList,
    Raw(u32),
}

impl ColType {
    fn from_code(code: u32, has_heap: bool) -> Self {
        match (code, has_heap) {
            (0, _) => ColType::Int,
            (1, _) => ColType::Float,
            (2, true) => ColType::Str,
            (11, _) => ColType::IntList,
            (10, _) => ColType::FloatList,
            (other, _) => ColType::Raw(other),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ColType::Int => "int",
            ColType::Float => "float",
            ColType::Str => "string",
            ColType::IntList => "int list",
            ColType::FloatList => "float list",
            ColType::Raw(_) => "raw",
        }
    }

}

#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
    pub index: u32,
    pub ty: ColType,
    cells: Range<usize>,
    heap: Option<Range<usize>>,
}

#[derive(Clone, Debug)]
pub struct Table {
    pub name: String,
    pub id: u32,
    pub rows: usize,
    pub columns: Vec<Column>,
    by_name: HashMap<String, usize>,
}

impl Table {
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.by_name.get(name).map(|&i| &self.columns[i])
    }
}

/// Table summary for the database browser.
#[derive(Clone, Debug, Serialize)]
pub struct TableSummary {
    pub name: String,
    pub id: u32,
    pub rows: usize,
    pub columns: Vec<ColumnSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ColumnSummary {
    pub name: String,
    pub index: u32,
    pub ty: &'static str,
}

/// A single cell value, loosely typed for transport to the UI.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Cell {
    Int(i32),
    Float(f32),
    Text(String),
    Missing,
}

pub struct Cdb {
    magic: [u8; 4],
    data: Vec<u8>,
    tables: Vec<Table>,
    by_name: HashMap<String, usize>,
}

struct Block {
    kind: u32,
    size: usize,
    names: Vec<String>,
    payload: usize,
    /// Exclusive end of the payload (start of the trailing CC marker).
    end: usize,
}

fn u32_at(data: &[u8], off: usize) -> Result<u32, String> {
    data.get(off..off + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| format!("unexpected end of data at 0x{off:X}"))
}

fn block_at(data: &[u8], off: usize) -> Result<Block, String> {
    if data.get(off..off + 4) != Some(&AA[..]) {
        return Err(format!("expected block marker at 0x{off:X}"));
    }
    let size = u32_at(data, off + 4)? as usize;
    let kind = u32_at(data, off + 8)?;
    let count = u32_at(data, off + 16)? as usize;
    if size < 28 || off + size > data.len() || count > 64 {
        return Err(format!("corrupt block header at 0x{off:X}"));
    }
    let mut p = off + 20;
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        let len = u32_at(data, p)? as usize;
        let bytes = data
            .get(p + 4..p + 4 + len)
            .ok_or_else(|| format!("corrupt block name at 0x{p:X}"))?;
        names.push(decode_text(bytes));
        p += 4 + ((len + 3) & !3);
    }
    if data.get(p..p + 4) != Some(&BB[..]) {
        return Err(format!("missing data marker in block at 0x{off:X}"));
    }
    let end = off + size - 4;
    if data.get(end..end + 4) != Some(&CC[..]) {
        return Err(format!("missing end marker in block at 0x{off:X}"));
    }
    Ok(Block { kind, size, names, payload: p + 4, end })
}

fn children(data: &[u8], parent: &Block) -> Result<Vec<Block>, String> {
    let mut p = parent.payload;
    if data.get(p..p + 4) == Some(&DD[..]) {
        p += 8;
    }
    let mut out = Vec::new();
    while p + 4 <= parent.end && data[p..p + 4] == AA {
        let child = block_at(data, p)?;
        p += child.size;
        out.push(child);
    }
    Ok(out)
}

fn leaf_u32(data: &[u8], block: &Block) -> Result<u32, String> {
    u32_at(data, block.payload)
}

impl Cdb {
    pub fn open(path: &str) -> Result<Self, String> {
        let raw = std::fs::read(path).map_err(|e| format!("Cannot read file: {e}"))?;
        Self::from_file_bytes(&raw)
    }

    pub fn from_file_bytes(raw: &[u8]) -> Result<Self, String> {
        if raw.len() < HEADER_LEN || raw[..4] != [0xFF; 4] {
            return Err("Not a PCM database file (bad header)".into());
        }
        let expected = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]) as usize;
        let mut data = Vec::with_capacity(expected);
        ZlibDecoder::new(&raw[HEADER_LEN..])
            .read_to_end(&mut data)
            .map_err(|e| format!("Decompression failed: {e}"))?;
        if data.len() != expected {
            return Err(format!("Size mismatch: header says {expected} bytes, got {}", data.len()));
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&raw[..4]);
        Self::from_raw(magic, data)
    }

    fn from_raw(magic: [u8; 4], data: Vec<u8>) -> Result<Self, String> {
        let root = block_at(&data, 0)?;
        let mut tables = Vec::new();
        for list in children(&data, &root)?.iter().filter(|b| b.kind == kind::TABLE_LIST) {
            for tb in children(&data, list)?.iter().filter(|b| b.kind == kind::TABLE) {
                tables.push(Self::parse_table(&data, tb)?);
            }
        }
        if tables.is_empty() {
            return Err("No tables found in database".into());
        }
        let by_name = tables.iter().enumerate().map(|(i, t)| (t.name.clone(), i)).collect();
        Ok(Cdb { magic, data, tables, by_name })
    }

    fn parse_table(data: &[u8], tb: &Block) -> Result<Table, String> {
        let name = tb.names.first().cloned().unwrap_or_default();
        let mut table = Table { name, id: 0, rows: 0, columns: Vec::new(), by_name: HashMap::new() };
        for child in children(data, tb)? {
            match child.kind {
                kind::TABLE_ID => table.id = leaf_u32(data, &child)?,
                kind::ROW_COUNT => table.rows = leaf_u32(data, &child)? as usize,
                kind::COLUMN_LIST => {
                    for cb in children(data, &child)?.iter().filter(|b| b.kind == kind::COLUMN) {
                        table.columns.push(Self::parse_column(data, cb)?);
                    }
                }
                _ => {}
            }
        }
        table.columns.sort_by_key(|c| c.index);
        table.by_name = table.columns.iter().enumerate().map(|(i, c)| (c.name.clone(), i)).collect();
        Ok(table)
    }

    fn parse_column(data: &[u8], cb: &Block) -> Result<Column, String> {
        let name = cb.names.first().cloned().unwrap_or_default();
        let (mut index, mut code, mut cells, mut heap) = (0, u32::MAX, 0..0, None);
        for child in children(data, cb)? {
            match child.kind {
                kind::COLUMN_INDEX => index = leaf_u32(data, &child)?,
                kind::COLUMN_TYPE => code = leaf_u32(data, &child)?,
                kind::CELLS => cells = child.payload..child.end,
                kind::HEAP => heap = Some(child.payload..child.end),
                _ => {}
            }
        }
        let ty = ColType::from_code(code, heap.is_some());
        Ok(Column { name, index, ty, cells, heap })
    }

    // ─── Lookup ──────────────────────────────────────────────────────────────

    pub fn table(&self, name: &str) -> Option<&Table> {
        self.by_name.get(name).map(|&i| &self.tables[i])
    }

    pub fn rows(&self, table: &str) -> usize {
        self.table(table).map(|t| t.rows).unwrap_or(0)
    }

    fn column(&self, table: &str, col: &str) -> Option<(&Table, &Column)> {
        let t = self.table(table)?;
        Some((t, t.column(col)?))
    }

    fn word(&self, c: &Column, row: usize) -> Option<[u8; 4]> {
        let off = c.cells.start + row * 4;
        if off + 4 > c.cells.end {
            return None;
        }
        let b = &self.data[off..off + 4];
        Some([b[0], b[1], b[2], b[3]])
    }

    // ─── Typed column readers ────────────────────────────────────────────────

    pub fn ints(&self, table: &str, col: &str) -> Option<Vec<i32>> {
        let (t, c) = self.column(table, col)?;
        if c.ty != ColType::Int {
            return None;
        }
        Some((0..t.rows).map(|r| self.word(c, r).map(i32::from_le_bytes).unwrap_or(0)).collect())
    }

    pub fn floats(&self, table: &str, col: &str) -> Option<Vec<f32>> {
        let (t, c) = self.column(table, col)?;
        if c.ty != ColType::Float {
            return None;
        }
        Some((0..t.rows).map(|r| self.word(c, r).map(f32::from_le_bytes).unwrap_or(0.0)).collect())
    }

    pub fn strings(&self, table: &str, col: &str) -> Option<Vec<String>> {
        let (t, c) = self.column(table, col)?;
        if c.ty != ColType::Str {
            return None;
        }
        let heap = c.heap.clone()?;
        let mut pos = heap.start + 4; // skip total length
        let mut out = Vec::with_capacity(t.rows);
        for r in 0..t.rows {
            let len = self.word(c, r).map(u32::from_le_bytes).unwrap_or(0) as usize;
            let end = (pos + len).min(heap.end);
            out.push(decode_text(&self.data[pos.min(end)..end]));
            pos = end;
        }
        Some(out)
    }

    pub fn cell(&self, table: &str, col: &str, row: usize) -> Cell {
        let Some((t, c)) = self.column(table, col) else { return Cell::Missing };
        if row >= t.rows {
            return Cell::Missing;
        }
        match c.ty {
            ColType::Int => self.word(c, row).map(|w| Cell::Int(i32::from_le_bytes(w))).unwrap_or(Cell::Missing),
            ColType::Float => self.word(c, row).map(|w| Cell::Float(f32::from_le_bytes(w))).unwrap_or(Cell::Missing),
            _ => Cell::Missing,
        }
    }

    /// Reads a page of rows from any table for the database browser.
    pub fn page(&self, table: &str, offset: usize, limit: usize) -> Option<Vec<Vec<Cell>>> {
        let t = self.table(table)?;
        let end = (offset + limit).min(t.rows);
        let start = offset.min(end);
        let cols: Vec<Vec<Cell>> = t
            .columns
            .iter()
            .map(|c| match c.ty {
                ColType::Str => self
                    .strings(table, &c.name)
                    .map(|v| v[start..end].iter().cloned().map(Cell::Text).collect())
                    .unwrap_or_default(),
                ColType::Int | ColType::Float => (start..end).map(|r| self.cell(table, &c.name, r)).collect(),
                _ => (start..end).map(|_| Cell::Missing).collect(),
            })
            .collect();
        Some(
            (0..end - start)
                .map(|i| cols.iter().map(|col| col.get(i).cloned().unwrap_or(Cell::Missing)).collect())
                .collect(),
        )
    }

    pub fn summaries(&self) -> Vec<TableSummary> {
        self.tables
            .iter()
            .map(|t| TableSummary {
                name: t.name.clone(),
                id: t.id,
                rows: t.rows,
                columns: t
                    .columns
                    .iter()
                    .map(|c| ColumnSummary { name: c.name.clone(), index: c.index, ty: c.ty.label() })
                    .collect(),
            })
            .collect()
    }

    // ─── Row lookup ──────────────────────────────────────────────────────────

    pub fn find_row_int(&self, table: &str, key_col: &str, key: i64) -> Option<usize> {
        self.ints(table, key_col)?.iter().position(|&v| v as i64 == key)
    }

    pub fn find_row_str(&self, table: &str, key_col: &str, key: &str) -> Option<usize> {
        self.strings(table, key_col)?.iter().position(|v| v == key)
    }

    // ─── Writing ─────────────────────────────────────────────────────────────

    /// Overwrites one numeric cell in place and returns the previous value.
    /// Integer columns require a whole number within `i32` range.
    pub fn set_number(&mut self, table: &str, col: &str, row: usize, value: f64) -> Result<f64, String> {
        if !value.is_finite() {
            return Err("Value must be a finite number".into());
        }
        let (t, c) = self
            .column(table, col)
            .ok_or_else(|| format!("Column {table}.{col} not found"))?;
        if row >= t.rows {
            return Err(format!("Row {row} out of range for {table} ({} rows)", t.rows));
        }
        let (ty, off) = (c.ty, c.cells.start + row * 4);
        if off + 4 > c.cells.end {
            return Err(format!("Cell {table}.{col}[{row}] is outside its data block"));
        }
        let old_bytes = [self.data[off], self.data[off + 1], self.data[off + 2], self.data[off + 3]];
        let (old, new_bytes) = match ty {
            ColType::Int => {
                if value.fract() != 0.0 || value < i32::MIN as f64 || value > i32::MAX as f64 {
                    return Err(format!("{table}.{col} needs a whole number between {} and {}", i32::MIN, i32::MAX));
                }
                (i32::from_le_bytes(old_bytes) as f64, (value as i32).to_le_bytes())
            }
            ColType::Float => (f32::from_le_bytes(old_bytes) as f64, (value as f32).to_le_bytes()),
            other => return Err(format!("{table}.{col} is a {} column and cannot be edited", other.label())),
        };
        self.data[off..off + 4].copy_from_slice(&new_bytes);
        Ok(old)
    }

    /// Raw bytes of every string in a column (without the trailing NUL), for lossless rewrites.
    pub fn raw_strings(&self, table: &str, col: &str) -> Option<Vec<Vec<u8>>> {
        let (t, c) = self.column(table, col)?;
        if c.ty != ColType::Str {
            return None;
        }
        let heap = c.heap.clone()?;
        let mut pos = heap.start + 4;
        let mut out = Vec::with_capacity(t.rows);
        for r in 0..t.rows {
            let len = self.word(c, r).map(u32::from_le_bytes).unwrap_or(0) as usize;
            let end = (pos + len).min(heap.end);
            let bytes = &self.data[pos.min(end)..end];
            out.push(bytes.split(|&b| b == 0).next().unwrap_or(bytes).to_vec());
            pos = end;
        }
        Some(out)
    }

    /// Replaces whole columns of `table`. When the new length differs from the current row
    /// count, every column of the table must be supplied (rows are added or removed).
    /// All enclosing block sizes are recomputed, so text may grow or shrink freely.
    pub fn rewrite_columns(&mut self, table: &str, cols: &[(&str, Values)]) -> Result<(), String> {
        let t = self.table(table).ok_or_else(|| format!("Table {table} not found"))?;
        let n = cols.first().map(|(_, v)| v.len()).ok_or("No columns to rewrite")?;
        if cols.iter().any(|(_, v)| v.len() != n) {
            return Err(format!("{table}: columns have different lengths"));
        }
        if n != t.rows {
            let missing: Vec<&str> = t.columns.iter().map(|c| c.name.as_str()).filter(|c| !cols.iter().any(|(n, _)| n == c)).collect();
            if !missing.is_empty() {
                return Err(format!("{table}: changing the row count needs every column; missing {}", missing.join(", ")));
            }
        }
        let mut repl: HashMap<(String, String, u32), Vec<u8>> = HashMap::new();
        for (name, values) in cols {
            let c = t.column(name).ok_or_else(|| format!("Column {table}.{name} not found"))?;
            let ok = matches!((c.ty, values), (ColType::Int, Values::Int(_)) | (ColType::Float, Values::Float(_)) | (ColType::Str, Values::Text(_)));
            if !ok {
                return Err(format!("{table}.{name} is a {} column; wrong value type", c.ty.label()));
            }
            let (cells, heap) = values.encode();
            repl.insert((table.into(), name.to_string(), kind::CELLS), cells);
            if let Some(h) = heap {
                repl.insert((table.into(), name.to_string(), kind::HEAP), h);
            }
        }
        repl.insert((table.into(), String::new(), kind::ROW_COUNT), (n as u32).to_le_bytes().to_vec());
        let mut out = Vec::with_capacity(self.data.len() + 4096);
        self.emit(0, "", "", &repl, &mut out)?;
        *self = Self::from_raw(self.magic, out)?;
        Ok(())
    }

    /// Re-serialises the block at `off`, substituting replaced leaf payloads and fixing sizes.
    fn emit(&self, off: usize, table: &str, col: &str, repl: &HashMap<(String, String, u32), Vec<u8>>, out: &mut Vec<u8>) -> Result<(), String> {
        let data = &self.data;
        let b = block_at(data, off)?;
        let start = out.len();
        out.extend_from_slice(&data[off..b.payload]);
        let is_container = matches!(b.kind, 0 | kind::TABLE_LIST | kind::TABLE | kind::COLUMN_LIST | kind::COLUMN);
        if is_container {
            let name = b.names.first().map(String::as_str).unwrap_or("");
            let (table, col) = match b.kind {
                kind::TABLE => (name, ""),
                kind::COLUMN => (table, name),
                _ => (table, col),
            };
            let mut p = b.payload;
            if data.get(p..p + 4) == Some(&DD[..]) {
                out.extend_from_slice(&data[p..p + 8]);
                p += 8;
            }
            while p + 4 <= b.end && data[p..p + 4] == AA {
                let child = block_at(data, p)?;
                self.emit(p, table, col, repl, out)?;
                p += child.size;
            }
            out.extend_from_slice(&data[p..off + b.size]);
        } else {
            let key_col = if b.kind == kind::ROW_COUNT { "" } else { col };
            match repl.get(&(table.to_string(), key_col.to_string(), b.kind)) {
                Some(payload) => {
                    out.extend_from_slice(payload);
                    while (out.len() - start) % 4 != 0 {
                        out.push(0);
                    }
                    out.extend_from_slice(&CC);
                }
                None => out.extend_from_slice(&data[b.payload..off + b.size]),
            }
        }
        let size = (out.len() - start) as u32;
        out[start + 4..start + 8].copy_from_slice(&size.to_le_bytes());
        Ok(())
    }

    /// Re-encodes the database into the on-disk `.cdb` format.
    pub fn to_file_bytes(&self) -> Result<Vec<u8>, String> {
        let mut enc = ZlibEncoder::new(Vec::with_capacity(self.data.len() / 3), Compression::default());
        enc.write_all(&self.data).map_err(|e| format!("Compression failed: {e}"))?;
        let compressed = enc.finish().map_err(|e| format!("Compression failed: {e}"))?;
        let mut out = Vec::with_capacity(HEADER_LEN + compressed.len());
        out.extend_from_slice(&self.magic);
        out.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        out.extend_from_slice(&compressed);
        Ok(out)
    }
}

/// New contents for one column in [`Cdb::rewrite_columns`].
#[derive(Clone, Debug)]
pub enum Values {
    Int(Vec<i32>),
    #[allow(dead_code)]
    Float(Vec<f32>),
    /// Raw string bytes without the trailing NUL (see [`encode_text`]).
    Text(Vec<Vec<u8>>),
}

impl Values {
    fn len(&self) -> usize {
        match self {
            Values::Int(v) => v.len(),
            Values::Float(v) => v.len(),
            Values::Text(v) => v.len(),
        }
    }

    /// (cell block payload, heap block payload for strings)
    fn encode(&self) -> (Vec<u8>, Option<Vec<u8>>) {
        match self {
            Values::Int(v) => (v.iter().flat_map(|x| x.to_le_bytes()).collect(), None),
            Values::Float(v) => (v.iter().flat_map(|x| x.to_le_bytes()).collect(), None),
            Values::Text(v) => {
                let cells = v.iter().flat_map(|s| (s.len() as u32 + 1).to_le_bytes()).collect();
                let total: usize = v.iter().map(|s| s.len() + 1).sum();
                let mut heap = Vec::with_capacity(total + 4);
                heap.extend_from_slice(&(total as u32).to_le_bytes());
                for s in v {
                    heap.extend_from_slice(s);
                    heap.push(0);
                }
                (cells, Some(heap))
            }
        }
    }
}

/// Encodes text the way PCM stores it (Windows-1252); characters outside it become '?'.
pub fn encode_text(s: &str) -> Vec<u8> {
    s.chars()
        .map(|ch| {
            let code = ch as u32;
            if code < 0x80 || (0xA0..=0xFF).contains(&code) {
                return code as u8;
            }
            (0x80u8..0xA0).find(|&b| cp1252(b) == ch).unwrap_or(b'?')
        })
        .collect()
}

// ─── Text decoding ──────────────────────────────────────────────────────────

/// Decodes PCM text: UTF-8 when valid, otherwise Windows-1252. Stops at the first NUL.
pub fn decode_text(raw: &[u8]) -> String {
    let raw = raw.split(|&b| b == 0).next().unwrap_or(raw);
    if raw.is_empty() {
        return String::new();
    }
    match std::str::from_utf8(raw) {
        Ok(s) => s.trim().to_string(),
        Err(_) => raw.iter().map(|&b| cp1252(b)).collect::<String>().trim().to_string(),
    }
}

fn cp1252(b: u8) -> char {
    const EXT: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
        '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
    ];
    if (0x80..0xA0).contains(&b) {
        EXT[(b - 0x80) as usize]
    } else {
        b as char
    }
}

/// Save the tests run against: `PCM_SAVE`, else the first `*.cdb` in the repo root.
/// Tests that need a real save skip when there is none.
#[cfg(test)]
pub(crate) fn sample_path() -> Option<String> {
    if let Ok(p) = std::env::var("PCM_SAVE") {
        return Some(p);
    }
    let mut saves: Vec<_> = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("cdb")))
        .collect();
    saves.sort();
    saves.first().map(|p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tables_and_round_trips() {
        let Some(path) = sample_path() else { return };
        let db = Cdb::open(&path).unwrap();
        assert!(db.table("DYN_cyclist").is_some());
        assert!(db.rows("DYN_team") > 100);
        let names = db.strings("DYN_team", "gene_sz_name").unwrap();
        assert_eq!(names.len(), db.rows("DYN_team"));
        assert!(names.iter().filter(|n| !n.is_empty()).count() > 100);

        let constants = db.strings("GAM_career_data", "CONSTANT").unwrap();
        let solde = constants.iter().position(|c| c == "SOLDE").unwrap();

        let bytes = db.to_file_bytes().unwrap();
        let mut copy = Cdb::from_file_bytes(&bytes).unwrap();
        let old = copy.set_number("GAM_career_data", "value", solde, 1_234_567.0).unwrap();
        let reread = Cdb::from_file_bytes(&copy.to_file_bytes().unwrap()).unwrap();
        assert_eq!(reread.floats("GAM_career_data", "value").unwrap()[solde], 1_234_567.0);
        assert_eq!(db.floats("GAM_career_data", "value").unwrap()[solde] as f64, old);
        assert!(copy.set_number("DYN_team", "value_i_budget", 0, 1.5).is_err());
        assert!(copy.set_number("DYN_team", "gene_sz_name", 0, 1.0).is_err());
    }

    #[test]
    fn rewrites_text_and_rows_losslessly() {
        let Some(path) = sample_path() else { return };
        let db = Cdb::open(&path).unwrap();

        // Rewriting a column with its own contents must reproduce the file byte for byte.
        let mut same = Cdb::open(&path).unwrap();
        let names = same.raw_strings("DYN_cyclist", "gene_sz_lastname").unwrap();
        same.rewrite_columns("DYN_cyclist", &[("gene_sz_lastname", Values::Text(names.clone()))]).unwrap();
        assert!(same.data == db.data, "identity rewrite changed the database");

        // Longer text: neighbours and every other table stay intact.
        let mut edited = Cdb::open(&path).unwrap();
        let mut longer = names.clone();
        longer[0] = encode_text("Ghebremedhin-Tesfamariam");
        edited.rewrite_columns("DYN_cyclist", &[("gene_sz_lastname", Values::Text(longer))]).unwrap();
        let reread = Cdb::from_file_bytes(&edited.to_file_bytes().unwrap()).unwrap();
        let after = reread.strings("DYN_cyclist", "gene_sz_lastname").unwrap();
        assert_eq!(after[0], "Ghebremedhin-Tesfamariam");
        assert_eq!(after[1..], db.strings("DYN_cyclist", "gene_sz_lastname").unwrap()[1..]);
        assert_eq!(reread.strings("DYN_team", "gene_sz_name"), db.strings("DYN_team", "gene_sz_name"));
        assert_eq!(reread.ints("DYN_cyclist", "IDcyclist"), db.ints("DYN_cyclist", "IDcyclist"));

        // Adding a row to a small table.
        let mut grown = Cdb::open(&path).unwrap();
        let t = "GAM_career_data";
        let mut ids = grown.ints(t, "IDcareer_data").unwrap();
        let mut vals = grown.floats(t, "value").unwrap();
        let mut consts = grown.raw_strings(t, "CONSTANT").unwrap();
        ids.push(999);
        vals.push(1.0);
        consts.push(b"PCMRECON_TEST".to_vec());
        grown.rewrite_columns(t, &[("IDcareer_data", Values::Int(ids)), ("value", Values::Float(vals)), ("CONSTANT", Values::Text(consts))]).unwrap();
        let reread = Cdb::from_file_bytes(&grown.to_file_bytes().unwrap()).unwrap();
        assert_eq!(reread.rows(t), db.rows(t) + 1);
        assert_eq!(reread.strings(t, "CONSTANT").unwrap().last().unwrap(), "PCMRECON_TEST");
        assert_eq!(reread.rows("DYN_cyclist"), db.rows("DYN_cyclist"));
    }
}
