//! Oodle decompression for PCM 2026's pak files.
//!
//! The game links Oodle statically, so we load Epic's redistributable `oo2core_9_win64.dll`
//! (the same build repak and FModel use). It is fetched once into the app's data folder and
//! checked against a pinned SHA-256; a copy the user picks by hand is accepted as-is.
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const DLL_NAME: &str = "oo2core_9_win64.dll";
const DLL_URL: &str = "https://github.com/WorkingRobot/OodleUE/raw/refs/heads/main/Engine/Source/Programs/Shared/EpicGames.Oodle/Sdk/2.9.10/win/redist/oo2core_9_win64.dll";
const DLL_SHA256: &str = "6f5d41a7892ea6b2db420f2458dad2f84a63901c9a93ce9497337b16c195f457";

#[allow(non_snake_case)]
type Decompress = unsafe extern "system" fn(
    compBuf: *const u8,
    compBufSize: usize,
    rawBuf: *mut u8,
    rawLen: usize,
    fuzzSafe: u32,
    checkCRC: u32,
    verbosity: u32,
    decBufBase: u64,
    decBufSize: usize,
    fpCallback: u64,
    callbackUserData: u64,
    decoderMemory: *mut u8,
    decoderMemorySize: usize,
    threadPhase: u32,
) -> isize;

pub struct Oodle {
    _lib: libloading::Library,
    decompress: Decompress,
}

static OODLE: OnceLock<Oodle> = OnceLock::new();

pub fn dll_path(data_dir: &Path) -> PathBuf {
    data_dir.join(DLL_NAME)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Downloads the DLL into `data_dir` unless it is already there.
pub fn ensure_downloaded(data_dir: &Path) -> Result<PathBuf, String> {
    let path = dll_path(data_dir);
    if path.is_file() {
        return Ok(path);
    }
    let resp = ureq::get(DLL_URL).call().map_err(|e| format!("Couldn't download Oodle ({e}). Check your connection, or pick {DLL_NAME} by hand."))?;
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut resp.into_reader(), &mut bytes).map_err(|e| e.to_string())?;
    if sha256_hex(&bytes) != DLL_SHA256 {
        return Err("The downloaded Oodle library didn't match its expected checksum, so it wasn't used".into());
    }
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Copies a user-picked DLL into `data_dir`.
pub fn install_from(data_dir: &Path, picked: &Path) -> Result<(), String> {
    let bytes = std::fs::read(picked).map_err(|e| e.to_string())?;
    if !bytes.starts_with(b"MZ") {
        return Err(format!("{} isn't a DLL", picked.display()));
    }
    std::fs::write(dll_path(data_dir), bytes).map_err(|e| e.to_string())
}

/// Loads Oodle once per run.
pub fn load(dll: &Path) -> Result<&'static Oodle, String> {
    if let Some(o) = OODLE.get() {
        return Ok(o);
    }
    let lib = unsafe { libloading::Library::new(dll) }.map_err(|e| format!("Cannot load {}: {e}", dll.display()))?;
    let decompress = *unsafe { lib.get::<Decompress>(b"OodleLZ_Decompress\0") }.map_err(|e| format!("{} has no OodleLZ_Decompress: {e}", dll.display()))?;
    Ok(OODLE.get_or_init(|| Oodle { _lib: lib, decompress }))
}

impl Oodle {
    /// Decompresses one block into `out`, which must be exactly the block's raw size.
    pub fn decompress(&self, input: &[u8], out: &mut [u8]) -> Result<(), String> {
        let n = unsafe {
            (self.decompress)(input.as_ptr(), input.len(), out.as_mut_ptr(), out.len(), 1, 1, 0, 0, 0, 0, 0, std::ptr::null_mut(), 0, 3)
        };
        if n as usize == out.len() {
            Ok(())
        } else {
            Err(format!("Oodle decompression failed ({n} of {} bytes)", out.len()))
        }
    }
}
