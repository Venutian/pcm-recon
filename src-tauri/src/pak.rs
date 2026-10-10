//! Unreal Engine `.pak` files (versions 10 and 11, as used by PCM 2026 and its mods).
//!
//! Reads the index and individual files (uncompressed or Oodle), and writes small
//! uncompressed paks for `Content/Paks/~mods`. The layout follows repak
//! (https://github.com/trumank/repak, MIT/Apache-2.0).
//!
//! Paths are game paths: the mount point without its leading `../../../`, joined with the
//! entry path, e.g. `pcm25_mod1/Plugins/Mod/Content/Jersey/Team/x/x_maillot.uexp`.
use crate::oodle::Oodle;
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const MAGIC: u32 = 0x5A6F12E1;
/// v11 footer: encryption guid (16) + encrypted (1) + magic (4) + version (4) + offset (8)
/// + size (8) + hash (20) + five 32-byte compression names.
const FOOTER_LEN: u64 = 16 + 1 + 4 + 4 + 8 + 8 + 20 + 5 * 32;
const MOUNT_PREFIX: &str = "../../../";

#[derive(Debug, Clone)]
struct Block {
    start: u64,
    end: u64,
}

#[derive(Debug, Clone)]
pub struct Entry {
    offset: u64,
    compressed: u64,
    pub uncompressed: u64,
    /// Index into the footer's compression names; `None` when stored as-is.
    compression: Option<u32>,
    encrypted: bool,
    block_size: u32,
    blocks: Vec<Block>,
}

pub struct Pak {
    pub path: PathBuf,
    compression: Vec<String>,
    pub entries: HashMap<String, Entry>,
}

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let s = self.buf.get(self.pos..self.pos + n).ok_or("Pak index is truncated")?;
        self.pos += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, String> {
        Ok(self.u32()? as i32)
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    /// Unreal FString: i32 length (negative = UTF-16), NUL-terminated.
    fn string(&mut self) -> Result<String, String> {
        let len = self.i32()?;
        if len < 0 {
            let raw = self.take((-len) as usize * 2)?;
            let chars: Vec<u16> = raw.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&c| c != 0).collect();
            Ok(String::from_utf16_lossy(&chars))
        } else {
            let raw = self.take(len as usize)?;
            let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
            Ok(String::from_utf8_lossy(&raw[..end]).into_owned())
        }
    }
}

/// Size of the entry header stored in front of each file's data.
fn header_size(compressed: bool, blocks: u32) -> u64 {
    8 + 8 + 8 + 4 + 20 + if compressed { 4 + 16 * blocks as u64 } else { 0 } + 1 + 4
}

fn read_encoded(c: &mut Cursor) -> Result<Entry, String> {
    let bits = c.u32()?;
    let compression = match (bits >> 23) & 0x3f {
        0 => None,
        n => Some(n - 1),
    };
    let encrypted = bits & (1 << 22) != 0;
    let block_count = (bits >> 6) & 0xffff;
    let block_size = match bits & 0x3f {
        0x3f => c.u32()?,
        n => n << 11,
    };
    let mut var = |bit: u32| -> Result<u64, String> { if bits & (1 << bit) != 0 { Ok(c.u32()? as u64) } else { c.u64() } };
    let offset = var(31)?;
    let uncompressed = var(30)?;
    let compressed = if compression.is_some() { var(29)? } else { uncompressed };
    let base = header_size(compression.is_some(), block_count);
    let blocks = if block_count == 1 && !encrypted {
        vec![Block { start: base, end: base + compressed }]
    } else {
        let mut at = base;
        (0..block_count)
            .map(|_| {
                let size = c.u32()? as u64;
                let b = Block { start: at, end: at + size };
                at += if encrypted { (size + 15) & !15 } else { size };
                Ok(b)
            })
            .collect::<Result<_, String>>()?
    };
    Ok(Entry { offset, compressed, uncompressed, compression, encrypted, block_size, blocks })
}

/// Full (non-encoded) entry record, used for the rare entries the encoder couldn't pack.
fn read_full(c: &mut Cursor) -> Result<Entry, String> {
    let offset = c.u64()?;
    let compressed = c.u64()?;
    let uncompressed = c.u64()?;
    let compression = match c.u32()? {
        0 => None,
        n => Some(n - 1),
    };
    c.take(20)?;
    let blocks = if compression.is_some() {
        (0..c.u32()?).map(|_| Ok(Block { start: c.u64()?, end: c.u64()? })).collect::<Result<_, String>>()?
    } else {
        Vec::new()
    };
    let encrypted = c.take(1)?[0] & 1 != 0;
    let block_size = c.u32()?;
    Ok(Entry { offset, compressed, uncompressed, compression, encrypted, block_size, blocks })
}

impl Pak {
    /// Reads the index of `path`, keeping only entries whose game path passes `keep`.
    pub fn open(path: &Path, keep: impl Fn(&str) -> bool) -> Result<Self, String> {
        let mut f = std::fs::File::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
        let len = f.metadata().map_err(|e| e.to_string())?.len();
        if len < FOOTER_LEN {
            return Err(format!("{} is too small to be a pak", path.display()));
        }
        let mut footer = vec![0u8; FOOTER_LEN as usize];
        f.seek(SeekFrom::Start(len - FOOTER_LEN)).and_then(|_| f.read_exact(&mut footer)).map_err(|e| e.to_string())?;
        let mut c = Cursor::new(&footer);
        c.take(16)?;
        let encrypted_index = c.take(1)?[0] != 0;
        if c.u32()? != MAGIC {
            return Err(format!("{} is not a supported pak (expected version 11)", path.display()));
        }
        let version = c.u32()?;
        if !(10..=11).contains(&version) {
            return Err(format!("{} uses pak version {version}; only 10 and 11 are supported", path.display()));
        }
        if encrypted_index {
            return Err(format!("{} has an encrypted index", path.display()));
        }
        let (index_offset, index_size) = (c.u64()?, c.u64()?);
        c.take(20)?;
        let compression: Vec<String> = (0..5)
            .map(|_| c.take(32).map(|n| String::from_utf8_lossy(n).trim_end_matches('\0').to_string()))
            .collect::<Result<_, _>>()?;

        let read_at = |f: &mut std::fs::File, at: u64, n: u64| -> Result<Vec<u8>, String> {
            let mut buf = vec![0u8; n as usize];
            f.seek(SeekFrom::Start(at)).and_then(|_| f.read_exact(&mut buf)).map_err(|e| e.to_string())?;
            Ok(buf)
        };
        let index = read_at(&mut f, index_offset, index_size)?;
        let mut c = Cursor::new(&index);
        let mount = c.string()?;
        let mount = mount.strip_prefix(MOUNT_PREFIX).unwrap_or(&mount).to_string();
        let _count = c.u32()?;
        let _seed = c.u64()?;
        if c.u32()? != 0 {
            c.take(8 + 8 + 20)?; // path hash index: not needed when the directory index exists
        }
        if c.u32()? == 0 {
            return Err(format!("{} has no directory index", path.display()));
        }
        let (fdi_offset, fdi_size) = (c.u64()?, c.u64()?);
        c.take(20)?;
        let encoded_len = c.u32()? as usize;
        let encoded = c.take(encoded_len)?;
        let full: Vec<Entry> = (0..c.u32()?).map(|_| read_full(&mut c)).collect::<Result<_, _>>()?;

        let fdi = read_at(&mut f, fdi_offset, fdi_size)?;
        let mut d = Cursor::new(&fdi);
        let mut entries = HashMap::new();
        for _ in 0..d.u32()? {
            let dir = d.string()?;
            let dir = dir.strip_prefix('/').unwrap_or(&dir).to_string();
            for _ in 0..d.u32()? {
                let name = d.string()?;
                let at = d.i32()?;
                let game_path = format!("{mount}{dir}{name}");
                if !keep(&game_path) {
                    continue;
                }
                let entry = if at >= 0 {
                    read_encoded(&mut Cursor { buf: encoded, pos: at as usize })?
                } else {
                    full.get((-at) as usize - 1).cloned().ok_or("Bad pak entry reference")?
                };
                entries.insert(game_path, entry);
            }
        }
        Ok(Pak { path: path.to_path_buf(), compression, entries })
    }

    /// Reads one file, decompressing it with Oodle when needed.
    pub fn read(&self, game_path: &str, oodle: Option<&Oodle>) -> Result<Vec<u8>, String> {
        let e = self.entries.get(game_path).ok_or_else(|| format!("{game_path} is not in {}", self.path.display()))?;
        if e.encrypted {
            return Err(format!("{game_path} is encrypted"));
        }
        let mut f = std::fs::File::open(&self.path).map_err(|err| err.to_string())?;
        let Some(slot) = e.compression else {
            let mut out = vec![0u8; e.uncompressed as usize];
            f.seek(SeekFrom::Start(e.offset + header_size(false, 0))).and_then(|_| f.read_exact(&mut out)).map_err(|err| err.to_string())?;
            return Ok(out);
        };
        let method = self.compression.get(slot as usize).map(String::as_str).unwrap_or("");
        if !method.eq_ignore_ascii_case("oodle") {
            return Err(format!("{game_path} uses unsupported compression \"{method}\""));
        }
        let oodle = oodle.ok_or("Oodle is needed to read this file")?;
        let first = e.blocks.first().ok_or("Compressed entry without blocks")?.start;
        let last = e.blocks.last().unwrap().end;
        let mut raw = vec![0u8; (last - first) as usize];
        f.seek(SeekFrom::Start(e.offset + first)).and_then(|_| f.read_exact(&mut raw)).map_err(|err| err.to_string())?;
        let mut out = vec![0u8; e.uncompressed as usize];
        let chunk = if e.blocks.len() == 1 { e.uncompressed as usize } else { e.block_size as usize };
        for (b, dst) in e.blocks.iter().zip(out.chunks_mut(chunk.max(1))) {
            oodle.decompress(&raw[(b.start - first) as usize..(b.end - first) as usize], dst)?;
        }
        let _ = e.compressed;
        Ok(out)
    }
}

// ─── Writing ──────────────────────────────────────────────────────────────────

fn put_string(w: &mut Vec<u8>, s: &str) {
    w.extend_from_slice(&(s.len() as u32 + 1).to_le_bytes());
    w.extend_from_slice(s.as_bytes());
    w.push(0);
}

fn fnv64_path(path: &str, seed: u64) -> u64 {
    let mut h = 0xcbf29ce484222325u64.wrapping_add(seed);
    for b in path.to_lowercase().encode_utf16().flat_map(u16::to_le_bytes) {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Splits `a/b/c` into (`a/b/`, `c`); the root is `/`.
fn split_child(path: &str) -> Option<(&str, &str)> {
    if path.is_empty() || path == "/" {
        return None;
    }
    let p = path.strip_suffix('/').unwrap_or(path);
    Some(match p.rfind('/') {
        Some(i) => p.split_at(i + 1),
        None => ("/", p),
    })
}

/// Writes an uncompressed v11 pak mounted at `../../../`, keyed by game path.
pub fn write(out: &Path, files: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    let mut data = Vec::new();
    let mut offsets = Vec::new();
    for bytes in files.values() {
        let offset = data.len() as u64;
        offsets.push(offset);
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        data.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&Sha1::digest(bytes));
        data.push(0); // flags
        data.extend_from_slice(&0u32.to_le_bytes()); // compression block size
        data.extend_from_slice(bytes);
    }

    // Encoded records: flags + u32 offset + u32 size (32-bit safe, uncompressed).
    let mut encoded = Vec::new();
    let mut record_at = Vec::new();
    for (bytes, &offset) in files.values().zip(&offsets) {
        record_at.push(encoded.len() as u32);
        if offset > u32::MAX as u64 || bytes.len() as u64 > u32::MAX as u64 {
            return Err("Override pak is too large".into());
        }
        encoded.extend_from_slice(&((1u32 << 29) | (1 << 30) | (1 << 31)).to_le_bytes());
        encoded.extend_from_slice(&(offset as u32).to_le_bytes());
        encoded.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    }

    let seed = 0u64;
    let mut phi = Vec::new();
    phi.extend_from_slice(&(files.len() as u32).to_le_bytes());
    for (path, at) in files.keys().zip(&record_at) {
        phi.extend_from_slice(&fnv64_path(path, seed).to_le_bytes());
        phi.extend_from_slice(&at.to_le_bytes());
    }
    phi.extend_from_slice(&0u32.to_le_bytes());

    let mut dirs: BTreeMap<&str, BTreeMap<&str, u32>> = BTreeMap::new();
    for (path, at) in files.keys().zip(&record_at) {
        let mut p = path.as_str();
        while let Some((parent, _)) = split_child(p) {
            p = parent;
            dirs.entry(p).or_default();
        }
        let (dir, name) = split_child(path).ok_or("Empty path in pak")?;
        dirs.entry(dir).or_default().insert(name, *at);
    }
    let mut fdi = Vec::new();
    fdi.extend_from_slice(&(dirs.len() as u32).to_le_bytes());
    for (dir, names) in &dirs {
        put_string(&mut fdi, dir);
        fdi.extend_from_slice(&(names.len() as u32).to_le_bytes());
        for (name, at) in names {
            put_string(&mut fdi, name);
            fdi.extend_from_slice(&at.to_le_bytes());
        }
    }

    let index_offset = data.len() as u64;
    let header_len = 4 + MOUNT_PREFIX.len() as u64 + 1 + 4 + 8 + 4 + 36 + 4 + 36 + 4 + encoded.len() as u64 + 4;
    let phi_offset = index_offset + header_len;
    let fdi_offset = phi_offset + phi.len() as u64;
    let mut index = Vec::new();
    put_string(&mut index, MOUNT_PREFIX);
    index.extend_from_slice(&(files.len() as u32).to_le_bytes());
    index.extend_from_slice(&seed.to_le_bytes());
    for (at, buf) in [(phi_offset, &phi), (fdi_offset, &fdi)] {
        index.extend_from_slice(&1u32.to_le_bytes());
        index.extend_from_slice(&at.to_le_bytes());
        index.extend_from_slice(&(buf.len() as u64).to_le_bytes());
        index.extend_from_slice(&Sha1::digest(buf));
    }
    index.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
    index.extend_from_slice(&encoded);
    index.extend_from_slice(&0u32.to_le_bytes());
    debug_assert_eq!(index.len() as u64, header_len);

    let mut footer = Vec::new();
    footer.extend_from_slice(&[0u8; 16]);
    footer.push(0);
    footer.extend_from_slice(&MAGIC.to_le_bytes());
    footer.extend_from_slice(&11u32.to_le_bytes());
    footer.extend_from_slice(&index_offset.to_le_bytes());
    footer.extend_from_slice(&(index.len() as u64).to_le_bytes());
    footer.extend_from_slice(&Sha1::digest(&index));
    footer.extend_from_slice(&[0u8; 5 * 32]);

    let tmp = out.with_extension("pak.tmp");
    let mut f = std::fs::File::create(&tmp).map_err(|e| format!("Cannot write {}: {e}", tmp.display()))?;
    for part in [&data, &index, &phi, &fdi, &footer] {
        f.write_all(part).map_err(|e| e.to_string())?;
    }
    drop(f);
    std::fs::rename(&tmp, out).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("Cannot replace {} ({e}). Close PCM first, it keeps the file open while running.", out.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_reads_back() {
        let dir = std::env::temp_dir().join(format!("pcmrecon-pak-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("t_P.pak");
        let mut files = BTreeMap::new();
        files.insert("pcm25_mod1/Plugins/Mod/Content/Jersey/Team/x/x_maillot.uasset".to_string(), b"asset".to_vec());
        files.insert("pcm25_mod1/Plugins/Mod/Content/Jersey/Team/x/x_maillot.uexp".to_string(), vec![7u8; 100_000]);
        files.insert("PCM/Content/Gui/MiniJersey/Team/x_minimaillot.uexp".to_string(), vec![1, 2, 3]);
        write(&out, &files).unwrap();
        let pak = Pak::open(&out, |_| true).unwrap();
        assert_eq!(pak.entries.len(), 3);
        for (path, bytes) in &files {
            assert_eq!(&pak.read(path, None).unwrap(), bytes, "{path}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn split_child_matches_repak() {
        assert_eq!(split_child("a/really/long/path"), Some(("a/really/long/", "path")));
        assert_eq!(split_child("a/really/long/"), Some(("a/really/", "long")));
        assert_eq!(split_child("a"), Some(("/", "a")));
        assert_eq!(split_child("/"), None);
    }
}
