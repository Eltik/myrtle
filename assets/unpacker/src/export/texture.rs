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

/// Decode a Texture2D object into in-memory RGBA data without writing to disk.
pub fn decode_texture_object(
    obj: &Value,
    resources: &HashMap<String, Vec<u8>>,
) -> Result<Option<DecodedTexture>, io::Error> {
    let name = obj["m_Name"].as_str().unwrap_or("unnamed");
    let width = obj["m_Width"].as_u64().unwrap_or(0) as u32;
    let height = obj["m_Height"].as_u64().unwrap_or(0) as u32;
    let format = obj["m_TextureFormat"].as_i64().unwrap_or(0) as i32;
    let image_data = obj["image data"].as_str().unwrap_or("");

    if width == 0 || height == 0 {
        return Ok(None);
    }

    let image_bytes = if !image_data.is_empty() {
        base64::engine::general_purpose::STANDARD
            .decode(image_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
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
    };

    let mut buf = vec![0u32; (width * height) as usize];
    if decode_texture(&image_bytes, width, height, format, &mut buf).is_err() {
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

/// Save a decoded texture to disk as PNG.
pub fn save_decoded_texture(tex: &DecodedTexture, output_dir: &Path) -> Result<(), io::Error> {
    let path = output_dir.join(format!("{}.png", tex.name));
    image::save_buffer(
        &path,
        &tex.rgba,
        tex.width,
        tex.height,
        image::ColorType::Rgba8,
    )
    .map_err(io::Error::other)
}

/// Decode and save a Texture2D object as PNG (convenience wrapper).
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
                buf[i] = ((alpha as u32) << 24) | 0x00FFFFFF;
            }
        }
        3 => {
            // RGB24
            for (i, chunk) in data.chunks(3).enumerate() {
                if i >= buf.len() {
                    break;
                }
                let r = chunk[0] as u32;
                let g = chunk[1] as u32;
                let b = chunk[2] as u32;
                buf[i] = (255 << 24) | (r << 16) | (g << 8) | b;
            }
        }
        4 => {
            // RGBA32
            for (i, chunk) in data.chunks(4).enumerate() {
                if i >= buf.len() {
                    break;
                }
                let r = chunk[0] as u32;
                let g = chunk[1] as u32;
                let b = chunk[2] as u32;
                let a = chunk[3] as u32;
                buf[i] = (a << 24) | (r << 16) | (g << 8) | b;
            }
        }
        5 => {
            // ARGB32
            for (i, chunk) in data.chunks(4).enumerate() {
                if i >= buf.len() {
                    break;
                }
                let a = chunk[0] as u32;
                let r = chunk[1] as u32;
                let g = chunk[2] as u32;
                let b = chunk[3] as u32;
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
                let r = ((r5 as u32) * 255 + 15) / 31;
                let g = ((g6 as u32) * 255 + 31) / 63;
                let b = ((b5 as u32) * 255 + 15) / 31;
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
                let r = ((r4 as u32) * 255 + 7) / 15;
                let g = ((g4 as u32) * 255 + 7) / 15;
                let b = ((b4 as u32) * 255 + 7) / 15;
                let a = ((a4 as u32) * 255 + 7) / 15;
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
            // ASTC variants
            let (bw, bh) = match format {
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
            texture2ddecoder::decode_astc(data, w as usize, h as usize, bw, bh, buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
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
