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

    let image_bytes;

    if !image_data.is_empty() {
        image_bytes = base64::engine::general_purpose::STANDARD
            .decode(image_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
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

        image_bytes = res_data[offset..offset + size].to_vec();
    }

    let mut buf = vec![0u32; (width * height) as usize];
    decode_texture(&image_bytes, width, height, format, &mut buf)?;

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
        3 => {
            // RGB24
            for (i, chunk) in data.chunks(3).enumerate() {
                let r = chunk[0] as u32;
                let g = chunk[1] as u32;
                let b = chunk[2] as u32;
                buf[i] = (255 << 24) | (r << 16) | (g << 8) | b;
            }
        }
        4 => {
            // RGBA32
            for (i, chunk) in data.chunks(4).enumerate() {
                let r = chunk[0] as u32;
                let g = chunk[1] as u32;
                let b = chunk[2] as u32;
                let a = chunk[3] as u32;
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
        48 | 49 | 50 | 52 | 54 | 56 => {
            // ASTC variants
            let (bw, bh) = match format {
                48 => (4, 4),
                49 => (5, 5),
                50 => (6, 6),
                52 => (8, 8),
                54 => (10, 10),
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
}
