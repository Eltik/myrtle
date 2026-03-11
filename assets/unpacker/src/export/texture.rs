use std::{collections::HashMap, io, path::Path};

use base64::Engine;
use serde_json::Value;

pub fn export_texture(
    obj: &Value,
    output_dir: &Path,
    resources: &HashMap<String, Vec<u8>>,
) -> Result<(), io::Error> {
    let name = obj["m_Name"].as_str().unwrap_or("unnamed");
    let width = obj["m_Width"].as_u64().unwrap_or(0) as u32;
    let height = obj["m_Height"].as_u64().unwrap_or(0) as u32;
    let format = obj["m_TextureFormat"].as_i64().unwrap_or(0) as i32;
    let image_data = obj["image data"].as_str().unwrap_or(""); // base64 string

    if width == 0 || height == 0 {
        return Ok(());
    }

    let image_bytes = if !image_data.is_empty() {
        base64::engine::general_purpose::STANDARD
            .decode(image_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        // Try m_StreamData (.resS)
        let stream = &obj["m_StreamData"];
        let offset = stream["offset"].as_u64().unwrap_or(0) as usize;
        let size = stream["size"].as_u64().unwrap_or(0) as usize;
        let path = stream["path"].as_str().unwrap_or("");

        if size == 0 || path.is_empty() {
            return Ok(());
        }

        let filename = path.rsplit('/').next().unwrap_or(path);
        let res_data = resources.get(filename).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("resource not found: {filename}"),
            )
        })?;

        if offset + size > res_data.len() {
            return Ok(());
        }
        res_data[offset..offset + size].to_vec()
    };

    let mut buf = vec![0u32; (width * height) as usize];
    if decode_texture(&image_bytes, width, height, format, &mut buf).is_err() {
        return Ok(());
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

    // Save PNG
    let path = output_dir.join(format!("{name}.png"));
    image::save_buffer(&path, &rgba, width, height, image::ColorType::Rgba8)
        .map_err(io::Error::other)?;

    Ok(())
}

fn decode_texture(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unity::bundle::BundleFile;
    use crate::unity::object_reader::read_object;
    use crate::unity::serialized_file::SerializedFile;
    use std::path::PathBuf;

    #[test]
    fn test_export_texture() {
        // Pick an AB file that contains a Texture2D (class 28)
        let data = match std::fs::read("../downloader/ArkAssets/chararts/char_003_kalts.ab") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };
        let bundle = BundleFile::parse(data).unwrap();
        // Build resource map from non-serialized entries (.resS files)
        let mut resources = HashMap::new();
        for entry in &bundle.files {
            if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
                let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
                resources.insert(filename.to_string(), entry.data.clone());
            }
        }

        let sf = SerializedFile::parse(bundle.files[0].data.clone()).unwrap();
        let output_dir = PathBuf::from("test_output");
        std::fs::create_dir_all(&output_dir).unwrap();

        let mut exported = 0;
        for obj in &sf.objects {
            if obj.class_id != 28 {
                continue;
            } // 28 = Texture2D
            let val = read_object(&sf, obj).unwrap();
            let name = val["m_Name"].as_str().unwrap_or("?");
            println!("Exporting Texture2D: {name}");
            match export_texture(&val, &output_dir, &resources) {
                Ok(()) => exported += 1,
                Err(e) => println!("  skip: {e}"),
            }
        }
        println!("Exported {exported} textures to test_output/");
        assert!(exported > 0, "should export at least one texture");
    }

    /// Bug: decode_texture rejects format 1 (Alpha8) with "unsupported texture format"
    #[test]
    fn test_decode_alpha8() {
        let data = vec![0u8, 128, 255]; // 3 alpha pixels
        let mut buf = vec![0u32; 3];
        decode_texture(&data, 3, 1, 1, &mut buf).expect("Alpha8 (format 1) should be supported");
        // Alpha8 should produce white pixels with varying alpha
        assert_eq!(buf[0] & 0xFF000000, 0x00000000, "first pixel alpha=0");
        assert_eq!(buf[1] >> 24, 128, "second pixel alpha=128");
        assert_eq!(buf[2] >> 24, 255, "third pixel alpha=255");
    }

    /// Bug: decode_texture rejects format 7 (RGB565) with "unsupported texture format"
    #[test]
    fn test_decode_rgb565() {
        // Pure red in RGB565: 11111_000000_00000 = 0xF800
        let data = vec![0x00, 0xF8]; // little-endian 0xF800
        let mut buf = vec![0u32; 1];
        decode_texture(&data, 1, 1, 7, &mut buf).expect("RGB565 (format 7) should be supported");
        // Should decode to full red
        let r = (buf[0] >> 16) & 0xFF;
        assert!(r > 240, "red channel should be near 255, got {r}");
    }

    /// Bug: decode_texture rejects format 51 (ASTC 7x7) with "unsupported texture format"
    #[test]
    fn test_decode_astc_7x7_accepted() {
        // ASTC block is 16 bytes; for 7x7 block size on a 7x7 image we need 1 block
        let data = vec![0u8; 16];
        let mut buf = vec![0u32; 7 * 7];
        // Should not return "unsupported texture format"
        let result = decode_texture(&data, 7, 7, 51, &mut buf);
        assert!(
            !result
                .as_ref()
                .is_err_and(|e| e.to_string().contains("unsupported")),
            "ASTC 7x7 (format 51) should be supported, got: {result:?}"
        );
    }

    /// Bug: export_texture panics when m_StreamData offset+size exceeds resource data length.
    /// Should skip silently instead.
    #[test]
    fn test_export_texture_stream_data_out_of_bounds() {
        use serde_json::json;

        let mut resources = HashMap::new();
        resources.insert("test.resS".to_string(), vec![0u8; 100]);

        // m_StreamData references offset=50, size=200 — exceeds the 100-byte resource
        let obj = json!({
            "m_Name": "test_oob",
            "m_Width": 4,
            "m_Height": 4,
            "m_TextureFormat": 4,
            "image data": "",
            "m_StreamData": {
                "offset": 50,
                "size": 200,
                "path": "archive:/CAB-abc/test.resS"
            }
        });

        let dir = PathBuf::from("test_output/texture_oob");
        std::fs::create_dir_all(&dir).unwrap();

        // Should not panic and should skip silently (Ok(()))
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            export_texture(&obj, &dir, &resources)
        }));
        assert!(
            result.is_ok(),
            "export_texture should not panic on out-of-bounds stream data"
        );
        assert!(
            result.unwrap().is_ok(),
            "out-of-bounds stream data should be silently skipped"
        );
    }

    /// Bug: "Not enough data to decode image!" spams stderr for textures with incomplete stream data.
    /// These should be silently skipped (Ok(())) since incomplete data is expected for
    /// lightmaps and other textures whose .resS data isn't fully bundled.
    #[test]
    fn test_export_texture_insufficient_data_skips_silently() {
        use serde_json::json;

        let mut resources = HashMap::new();
        // ASTC 4x4 for 1024x1024 needs ~1MB, provide only 16 bytes
        resources.insert("small.resS".to_string(), vec![0u8; 16]);

        let obj = json!({
            "m_Name": "test_small_astc",
            "m_Width": 1024,
            "m_Height": 1024,
            "m_TextureFormat": 48,
            "image data": "",
            "m_StreamData": {
                "offset": 0,
                "size": 16,
                "path": "small.resS"
            }
        });

        let dir = PathBuf::from("test_output/texture_small");
        std::fs::create_dir_all(&dir).unwrap();

        // Should skip silently — Ok(()) with no file written, not Err
        let result = export_texture(&obj, &dir, &resources);
        assert!(
            result.is_ok(),
            "insufficient texture data should be silently skipped, got: {result:?}"
        );
        assert!(
            !dir.join("test_small_astc.png").exists(),
            "should not write a PNG for insufficient data"
        );
    }

    /// Bug: RGBA32 panics with "index out of bounds: the len is 10000 but the index is 10000"
    /// This happens when data contains exactly w*h*4 bytes but buf is also w*h,
    /// and chunks(4) yields exactly w*h items indexed 0..w*h-1 — but a texture whose
    /// compressed data decodes to slightly more pixels than w*h will overflow.
    /// Reproduce with data that has MORE chunks than buf entries.
    #[test]
    fn test_decode_rgba32_overflow_no_panic() {
        // buf has 4 pixels, but data has 5 RGBA pixels (20 bytes) — index 4 overflows
        let data = vec![255u8; 20];
        let mut buf = vec![0u32; 4];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            decode_texture(&data, 2, 2, 4, &mut buf)
        }));
        assert!(
            result.is_ok(),
            "decode_texture should not panic when data has more pixels than w*h"
        );
    }
}
