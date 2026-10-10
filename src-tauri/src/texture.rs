//! Kit textures as cooked by PCM 2026: an uncompressed `PF_B8G8R8A8` Texture2D split into a
//! small `.uasset` header and a `.uexp` holding one mip of raw pixels.
//!
//! `.uexp` layout: 0x78 bytes of texture properties (width at 0x50, height at 0x54), then
//! width × height × 4 bytes of BGRA, then a 28-byte trailer that repeats width and height.
//! Only that exact shape is accepted, so an edit can never produce a malformed asset.

const PIXELS_AT: usize = 0x78;
const TRAILER: usize = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}

/// Validates the asset pair and returns the texture size.
pub fn parse(uasset: &[u8], uexp: &[u8]) -> Result<Size, String> {
    if !uasset.windows(11).any(|w| w == b"PF_B8G8R8A8") {
        return Err("Not an uncompressed BGRA texture".into());
    }
    let (w, h) = (u32_at(uexp, 0x50).unwrap_or(0), u32_at(uexp, 0x54).unwrap_or(0));
    let px = w as usize * h as usize * 4;
    if w == 0 || h == 0 || w > 8192 || h > 8192 || uexp.len() != PIXELS_AT + px + TRAILER {
        return Err("Unexpected texture layout".into());
    }
    if u32_at(uexp, PIXELS_AT + px) != Some(w) || u32_at(uexp, PIXELS_AT + px + 4) != Some(h) {
        return Err("Unexpected texture trailer".into());
    }
    Ok(Size { width: w, height: h })
}

/// BGRA pixels of a validated `.uexp`.
pub fn pixels(uexp: &[u8], size: Size) -> &[u8] {
    &uexp[PIXELS_AT..PIXELS_AT + size.width as usize * size.height as usize * 4]
}

/// A copy of `uexp` with its pixels replaced (same size).
pub fn with_pixels(uexp: &[u8], size: Size, bgra: &[u8]) -> Result<Vec<u8>, String> {
    let n = size.width as usize * size.height as usize * 4;
    if bgra.len() != n {
        return Err(format!("Image is {} bytes, the texture needs {n}", bgra.len()));
    }
    let mut out = uexp.to_vec();
    out[PIXELS_AT..PIXELS_AT + n].copy_from_slice(bgra);
    Ok(out)
}

/// Swaps red and blue in place (BGRA ⇄ RGBA).
pub fn swap_rb(px: &mut [u8]) {
    for p in px.chunks_exact_mut(4) {
        p.swap(0, 2);
    }
}

/// Makes a new asset from an existing one by renaming it. Every replacement keeps its byte
/// length, so no offsets in the header move; callers pick a template whose names have the
/// same lengths as the target's (e.g. `a-aalborg_maillot_den` for `a-other-kt_maillot_ita`).
pub fn renamed(uasset: &[u8], replacements: &[(&str, &str)]) -> Result<Vec<u8>, String> {
    let mut out = uasset.to_vec();
    for (from, to) in replacements {
        if from.len() != to.len() {
            return Err(format!("Cannot rename {from} to {to}: lengths differ"));
        }
        let (f, t) = (from.as_bytes(), to.as_bytes());
        let mut i = 0;
        let mut hits = 0;
        while i + f.len() <= out.len() {
            if &out[i..i + f.len()] == f {
                out[i..i + f.len()].copy_from_slice(t);
                i += f.len();
                hits += 1;
            } else {
                i += 1;
            }
        }
        if hits == 0 {
            return Err(format!("{from} not found in the template asset"));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake(w: u32, h: u32) -> (Vec<u8>, Vec<u8>) {
        let uasset = b"..../Mod/Jersey/Team/abc/abc_maillot_den....PF_B8G8R8A8...abc_maillot_den..".to_vec();
        let mut uexp = vec![0u8; PIXELS_AT];
        uexp[0x50..0x54].copy_from_slice(&w.to_le_bytes());
        uexp[0x54..0x58].copy_from_slice(&h.to_le_bytes());
        uexp.extend(std::iter::repeat(9u8).take((w * h * 4) as usize));
        let mut trailer = vec![0u8; TRAILER];
        trailer[0..4].copy_from_slice(&w.to_le_bytes());
        trailer[4..8].copy_from_slice(&h.to_le_bytes());
        uexp.extend(trailer);
        (uasset, uexp)
    }

    #[test]
    fn parses_and_replaces_pixels() {
        let (a, e) = fake(4, 2);
        let size = parse(&a, &e).unwrap();
        assert_eq!(size, Size { width: 4, height: 2 });
        let new = with_pixels(&e, size, &[1u8; 32]).unwrap();
        assert_eq!(pixels(&new, size), &[1u8; 32]);
        assert_eq!(new.len(), e.len());
        assert!(with_pixels(&e, size, &[1u8; 31]).is_err());
        assert!(parse(&a, &e[..e.len() - 1]).is_err());
    }

    #[test]
    fn renames_with_equal_lengths_only() {
        let (a, _) = fake(1, 1);
        let r = renamed(&a, &[("abc", "xyz"), ("xyz_maillot_den", "xyz_maillot_ita")]).unwrap();
        let s = String::from_utf8_lossy(&r);
        assert!(s.contains("/Team/xyz/xyz_maillot_ita") && !s.contains("abc"));
        assert_eq!(r.len(), a.len());
        assert!(renamed(&a, &[("abc", "abcd")]).is_err());
    }

    /// Real assets pulled from the installed WorldDB mod, when present.
    #[test]
    fn parses_real_kit_textures() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.kit-samples");
        let Ok(files) = std::fs::read_dir(&dir) else { return };
        for f in files.flatten().filter(|f| f.path().extension().is_some_and(|e| e == "uexp")) {
            let uexp = std::fs::read(f.path()).unwrap();
            let uasset = std::fs::read(f.path().with_extension("uasset")).unwrap();
            let size = parse(&uasset, &uexp).unwrap_or_else(|e| panic!("{}: {e}", f.path().display()));
            assert_eq!(with_pixels(&uexp, size, pixels(&uexp, size)).unwrap(), uexp);
        }
    }
}
