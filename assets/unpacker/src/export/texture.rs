use std::{collections::HashMap, io, path::Path};

use base64::Engine;
use serde_json::Value;

/// Decoded texture data ready for saving or combining.
pub struct DecodedTexture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// RGBA bytes, already vertically flipped (top-left origin).
    pub rgba: Vec<u8>,
}

/// Decode a `Texture2D` object into in-memory RGBA data without writing to disk.
pub fn decode_texture_object(
    obj: &Value,
    resources: &HashMap<String, Vec<u8>>,
) -> Result<Option<DecodedTexture>, io::Error> {
    let name = obj["m_Name"].as_str().unwrap_or("unnamed");
    let width = obj["m_Width"].as_u64().unwrap_or(0) as u32;
    let height = obj["m_Height"].as_u64().unwrap_or(0) as u32;

    // A pre-decoded texture injected by external-texture resolution (`fx_textures`):
    // the RGBA is already decoded + flipped, so pass it straight through.
    if let Some(b64) = obj.get("_decodedRGBA").and_then(Value::as_str) {
        let rgba = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if width == 0 || height == 0 || rgba.len() != (width * height * 4) as usize {
            return Ok(None);
        }
        return Ok(Some(DecodedTexture {
            name: name.to_string(),
            width,
            height,
            rgba,
        }));
    }

    let format = obj["m_TextureFormat"].as_i64().unwrap_or(0) as i32;
    let image_data = obj["image data"].as_str().unwrap_or("");

    // Reject zero or absurd dimensions before they drive the `width * height`
    // pixel-buffer allocations below (lines ~76/86). `m_Width`/`m_Height` come
    // straight from the object header, so a corrupt/misread texture could claim
    // e.g. 40000x40000 and allocate ~13 GB in one shot. Real AK textures top out
    // well under 16384 on a side; compute the product as u64 so the check itself
    // can't wrap (the u32 `width * height` below would).
    const MAX_DIM: u32 = 16384;
    if width == 0 || height == 0 || width > MAX_DIM || height > MAX_DIM {
        return Ok(None);
    }

    let image_bytes = if image_data.is_empty() {
        let stream = &obj["m_StreamData"];
        let offset = stream["offset"].as_u64().unwrap_or(0) as usize;
        let size = stream["size"].as_u64().unwrap_or(0) as usize;
        let path = stream["path"].as_str().unwrap_or("");

        if size == 0 || path.is_empty() {
            return Ok(None);
        }

        let filename = path.rsplit('/').next().unwrap_or(path);
        let res_data = resources.get(filename).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("resource not found: {filename}"),
            )
        })?;

        if offset + size > res_data.len() {
            return Ok(None);
        }
        res_data[offset..offset + size].to_vec()
    } else {
        base64::engine::general_purpose::STANDARD
            .decode(image_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    };

    let mut buf = vec![0u32; (width * height) as usize];
    if let Err(e) = decode_texture(&image_bytes, width, height, format, &mut buf) {
        eprintln!(
            "  decode_texture failed for {name} ({width}x{height} fmt={format}, {} bytes): {e}",
            image_bytes.len()
        );
        return Ok(None);
    }

    // Convert u32 (ABGR packed) to RGBA bytes
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for (i, &pixel) in buf.iter().enumerate() {
        let off = i * 4;
        rgba[off] = ((pixel >> 16) & 0xFF) as u8; // R
        rgba[off + 1] = ((pixel >> 8) & 0xFF) as u8; // G
        rgba[off + 2] = (pixel & 0xFF) as u8; // B
        rgba[off + 3] = ((pixel >> 24) & 0xFF) as u8; // A
    }

    // Flip vertically (Unity origin is bottom-left)
    let stride = (width * 4) as usize;
    for y in 0..height as usize / 2 {
        let top = y * stride;
        let bot = (height as usize - 1 - y) * stride;
        for x in 0..stride {
            rgba.swap(top + x, bot + x);
        }
    }

    Ok(Some(DecodedTexture {
        name: name.to_string(),
        width,
        height,
        rgba,
    }))
}

/// THE PNG DEFLATE LEVEL (`DYNCHAR_PNG_LEVEL`, missing = the crate's `Fast`, today's bytes).
///
/// Every page the exporter writes goes through `image::save_buffer`, whose PNG default is
/// `CompressionType::Fast`: zlib header 78 01, FLEVEL 0, on every file in the tree. The
/// same decoded RGBA re-deflated at a higher level is pixel-identical by construction and
/// measurably smaller: on the ten largest dynchar pages (102.7 MB) the crate's `Default`
/// (balanced) level writes 89.3 MB, 13.0 percent less, at 0.8 s a page against 0.05, and
/// `Best` (high) 89.0 MB at 2.3 s a page; on ten random pages 20.0 and 20.4 percent
/// (2026-09-08, `probe_pngbest`). Default `default` since 2026-09-08 (Ian's ruling after the
/// delivery gate: 19.3 percent at 139 s, 2979 of 2979 pages pixel-identical); `fast` restores
/// the previous bytes exactly, which is the harness's control arm, and `best` is the high level.
#[must_use]
pub fn png_compression() -> image::codecs::png::CompressionType {
    use image::codecs::png::CompressionType;
    match std::env::var("DYNCHAR_PNG_LEVEL").as_deref() {
        Ok("fast") => CompressionType::Fast,
        Ok("best") => CompressionType::Best,
        _ => CompressionType::Default,
    }
}

/// Write one RGBA8 image as PNG at the configured level. At `Fast` this is exactly
/// `image::save_buffer`, byte for byte; otherwise the crate's encoder at that level with
/// adaptive filters.
pub fn write_png(path: &Path, rgba: &[u8], w: u32, h: u32) -> Result<(), io::Error> {
    use image::ImageEncoder;
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    let level = png_compression();
    if matches!(level, CompressionType::Fast) {
        return image::save_buffer(path, rgba, w, h, image::ColorType::Rgba8)
            .map_err(io::Error::other);
    }
    let file = std::fs::File::create(path)?;
    let mut out = std::io::BufWriter::new(file);
    PngEncoder::new_with_quality(&mut out, level, FilterType::Adaptive)
        .write_image(rgba, w, h, image::ExtendedColorType::Rgba8)
        .map_err(io::Error::other)
}

/// Save a decoded texture to disk as PNG.
pub fn save_decoded_texture(tex: &DecodedTexture, output_dir: &Path) -> Result<(), io::Error> {
    let path = output_dir.join(format!("{}.png", tex.name));
    if output_case_insensitive(output_dir) && !claim_texture_path(&path, tex) {
        return Ok(());
    }
    write_png(&path, &tex.rgba, tex.width, tex.height)
}

/// Content hash and byte count of the RGBA kept at a folded path, and the texture's name.
type ClaimedTex = (u64, usize, String);
/// Every texture path written this run, keyed by the CASE-FOLDED path, so two textures whose
/// names differ only in case are detected on a filesystem that folds them onto one file.
static CLAIMED_TEX: std::sync::LazyLock<std::sync::Mutex<HashMap<String, ClaimedTex>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static TEX_COLLISION_REPORT: std::sync::LazyLock<std::sync::Mutex<Vec<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));
static OUTPUT_CASE_INSENSITIVE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// Whether the output filesystem folds letter case, probed once on the first texture
/// directory written (every texture lands on the one output volume). On a case-sensitive
/// filesystem (the Linux VPS) two textures named `BG` and `bg` are two files and nothing
/// here runs; on the default macOS volume they are one file, and which of the two survives
/// was the write order of a parallel walk (the determinism instrument's first nonzero read,
/// 2026-09-10: `[uc]multiv3` and `[uc]recalrune` each ship a distinct `BG` and `bg`).
fn output_case_insensitive(dir: &Path) -> bool {
    *OUTPUT_CASE_INSENSITIVE.get_or_init(|| {
        let pid = std::process::id();
        let probe = dir.join(format!(".case-probe-{pid}"));
        if std::fs::write(&probe, b"").is_err() {
            return false;
        }
        let folded = std::fs::metadata(dir.join(format!(".CASE-PROBE-{pid}"))).is_ok();
        let _ = std::fs::remove_file(&probe);
        folded
    })
}

/// Claim `path` for `tex` on a case-folding filesystem. Returns whether to write.
///
/// Two textures folding onto one path is upstream's naming, and the export cannot keep
/// both without renaming one, which would break the game's own case-sensitive address. So
/// exactly one survives, and the choice is made by the CONTENT rather than by which thread
/// arrived first: the larger RGBA wins, a size tie falls to the byte-smaller name (`BG`
/// before `bg`), a trade for reproducibility rather than a judgement of the art. The loser
/// is reported (`texture_collision_report`) and the winner's own name casing is what lands
/// on disk: the incumbent file is removed before the winner is written, because a folding
/// filesystem keeps the FIRST creator's spelling on an overwrite.
fn claim_texture_path(path: &Path, tex: &DecodedTexture) -> bool {
    let key = path.to_string_lossy().to_lowercase();
    let hash = fnv1a64(&tex.rgba);
    let mut claimed = CLAIMED_TEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match claimed.get(&key) {
        None => {
            claimed.insert(key, (hash, tex.rgba.len(), tex.name.clone()));
            true
        }
        Some((h, len, _)) if *h == hash && *len == tex.rgba.len() => true,
        Some((_, kept_len, kept_name)) => {
            let (kept_len, kept_name) = (*kept_len, kept_name.clone());
            let we_win = match tex.rgba.len().cmp(&kept_len) {
                std::cmp::Ordering::Greater => true,
                std::cmp::Ordering::Less => false,
                std::cmp::Ordering::Equal => tex.name.as_str() < kept_name.as_str(),
            };
            let (winner, wlen, loser, llen) = if we_win {
                (
                    tex.name.as_str(),
                    tex.rgba.len(),
                    kept_name.as_str(),
                    kept_len,
                )
            } else {
                (
                    kept_name.as_str(),
                    kept_len,
                    tex.name.as_str(),
                    tex.rgba.len(),
                )
            };
            TEX_COLLISION_REPORT
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(format!(
                    "{}: the output filesystem folds `{loser}` onto `{winner}`; kept {winner} ({wlen} RGBA bytes), dropped {loser} ({llen})",
                    path.display()
                ));
            if we_win {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::remove_file(parent.join(format!("{kept_name}.png")));
                }
                claimed.insert(key, (hash, tex.rgba.len(), tex.name.clone()));
            }
            we_win
        }
    }
}

/// One line per texture whose name the output filesystem folded onto another's, sorted.
#[must_use]
pub fn texture_collision_report() -> Vec<String> {
    let mut lines = TEX_COLLISION_REPORT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    lines.sort_unstable();
    lines
}

/// FNV-1a over the decoded pixels, the same identity the skeleton claim uses.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Decode and save a `Texture2D` object as PNG (convenience wrapper).
pub fn export_texture(
    obj: &Value,
    output_dir: &Path,
    resources: &HashMap<String, Vec<u8>>,
) -> Result<(), io::Error> {
    let Some(decoded) = decode_texture_object(obj, resources)? else {
        return Ok(());
    };
    save_decoded_texture(&decoded, output_dir)
}

pub fn decode_texture(
    data: &[u8],
    w: u32,
    h: u32,
    format: i32,
    buf: &mut [u32],
) -> Result<(), io::Error> {
    match format {
        1 => {
            // Alpha8
            for (i, &alpha) in data.iter().enumerate() {
                if i >= buf.len() {
                    break;
                }
                buf[i] = (u32::from(alpha) << 24) | 0x00FFFFFF;
            }
        }
        3 => {
            // RGB24
            for (i, chunk) in data.chunks(3).enumerate() {
                if i >= buf.len() {
                    break;
                }
                let r = u32::from(chunk[0]);
                let g = u32::from(chunk[1]);
                let b = u32::from(chunk[2]);
                buf[i] = (255 << 24) | (r << 16) | (g << 8) | b;
            }
        }
        4 => {
            // RGBA32
            for (i, chunk) in data.chunks(4).enumerate() {
                if i >= buf.len() {
                    break;
                }
                let r = u32::from(chunk[0]);
                let g = u32::from(chunk[1]);
                let b = u32::from(chunk[2]);
                let a = u32::from(chunk[3]);
                buf[i] = (a << 24) | (r << 16) | (g << 8) | b;
            }
        }
        5 => {
            // ARGB32
            for (i, chunk) in data.chunks(4).enumerate() {
                if i >= buf.len() {
                    break;
                }
                let a = u32::from(chunk[0]);
                let r = u32::from(chunk[1]);
                let g = u32::from(chunk[2]);
                let b = u32::from(chunk[3]);
                buf[i] = (a << 24) | (r << 16) | (g << 8) | b;
            }
        }
        7 => {
            // RGB565
            for (i, chunk) in data.chunks(2).enumerate() {
                if i >= buf.len() {
                    break;
                }
                let pixel = u16::from_le_bytes([chunk[0], chunk[1]]);
                let r5 = (pixel >> 11) & 0x1F;
                let g6 = (pixel >> 5) & 0x3F;
                let b5 = pixel & 0x1F;
                let r = (u32::from(r5) * 255 + 15) / 31;
                let g = (u32::from(g6) * 255 + 31) / 63;
                let b = (u32::from(b5) * 255 + 15) / 31;
                buf[i] = (255 << 24) | (r << 16) | (g << 8) | b;
            }
        }
        13 => {
            // RGBA4444
            for (i, chunk) in data.chunks(2).enumerate() {
                if i >= buf.len() {
                    break;
                }
                let pixel = u16::from_le_bytes([chunk[0], chunk[1]]);
                let r4 = (pixel >> 12) & 0xF;
                let g4 = (pixel >> 8) & 0xF;
                let b4 = (pixel >> 4) & 0xF;
                let a4 = pixel & 0xF;
                let r = (u32::from(r4) * 255 + 7) / 15;
                let g = (u32::from(g4) * 255 + 7) / 15;
                let b = (u32::from(b4) * 255 + 7) / 15;
                let a = (u32::from(a4) * 255 + 7) / 15;
                buf[i] = (a << 24) | (r << 16) | (g << 8) | b;
            }
        }
        34 => {
            // ETC_RGB4
            texture2ddecoder::decode_etc1(data, w as usize, h as usize, buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        }
        45 => {
            // ETC2_RGB
            texture2ddecoder::decode_etc2_rgb(data, w as usize, h as usize, buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        }
        47 => {
            // ETC2_RGBA8
            texture2ddecoder::decode_etc2_rgba8(data, w as usize, h as usize, buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        }
        48..=56 => {
            // ASTC variants — try reported block size first, fall back if data doesn't fit
            let reported = match format {
                48 => (4, 4),
                49 => (5, 5),
                50 => (6, 6),
                51 => (7, 7),
                52 => (8, 8),
                53 => (10, 5),
                54 => (10, 10),
                55 => (12, 10),
                56 => (12, 12),
                _ => unreachable!(),
            };
            let result = texture2ddecoder::decode_astc(
                data, w as usize, h as usize, reported.0, reported.1, buf,
            );
            if result.is_err() {
                // Unity sometimes reports wrong ASTC block size; find the block size
                // that matches the actual data length: blocks = ceil(w/bw) * ceil(h/bh), bytes = blocks * 16
                let candidates: &[(usize, usize)] =
                    &[(4, 4), (5, 5), (6, 6), (7, 7), (8, 8), (10, 10), (12, 12)];
                let mut decoded = false;
                for &(bw, bh) in candidates {
                    if (bw, bh) == reported {
                        continue;
                    }
                    let blocks_w = (w as usize).div_ceil(bw);
                    let blocks_h = (h as usize).div_ceil(bh);
                    let expected = blocks_w * blocks_h * 16;
                    if expected <= data.len()
                        && texture2ddecoder::decode_astc(data, w as usize, h as usize, bw, bh, buf)
                            .is_ok()
                    {
                        decoded = true;
                        break;
                    }
                }
                if !decoded {
                    result
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                }
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported texture format: {format}"),
            ));
        }
    }
    Ok(())
}
