use std::{collections::HashMap, io, path::Path};

use base64::Engine;
use serde_json::Value;

pub fn export_audio(
    obj: &Value,
    output_dir: &Path,
    resources: &HashMap<String, Vec<u8>>,
) -> Result<(), io::Error> {
    let name = obj["m_Name"].as_str().unwrap_or("unnamed");

    // Try m_AudioData first (inline base64)
    let audio_data = obj["m_AudioData"].as_str().unwrap_or("");
    let bytes = if !audio_data.is_empty() {
        if let Some(b64) = audio_data.strip_prefix("base64:") {
            base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            audio_data.as_bytes().to_vec()
        }
    } else {
        // Try m_Resource for external .resS data
        let res = &obj["m_Resource"];
        let source_raw = res["m_Source"].as_str().unwrap_or("");
        let source = source_raw.rsplit('/').next().unwrap_or(source_raw);
        let offset = res["m_Offset"].as_u64().unwrap_or(0) as usize;
        let size = res["m_Size"].as_u64().unwrap_or(0) as usize;

        if !source.is_empty() && size > 0 {
            if let Some(res_data) = resources.get(source) {
                if offset + size <= res_data.len() {
                    res_data[offset..offset + size].to_vec()
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    };

    if bytes.is_empty() {
        return Ok(());
    }

    // Detect format by magic bytes
    if bytes.len() >= 4 && &bytes[0..4] == b"OggS" {
        let path = output_dir.join(format!("{name}.ogg"));
        std::fs::write(&path, &bytes)?;
    } else if bytes.len() >= 4 && &bytes[0..4] == b"RIFF" {
        let path = output_dir.join(format!("{name}.wav"));
        std::fs::write(&path, &bytes)?;
    } else if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        let path = output_dir.join(format!("{name}.m4a"));
        std::fs::write(&path, &bytes)?;
    } else if bytes.len() >= 4 && &bytes[0..4] == b"FSB5" {
        // Decode FSB5 Vorbis to OGG
        match super::fsb5::fsb5_to_ogg(&bytes) {
            Ok(samples) => {
                for (_, ogg_data) in samples {
                    let path = output_dir.join(format!("{name}.ogg"));
                    std::fs::write(&path, &ogg_data)?;
                }
            }
            Err(_) => {
                // Fallback: save raw FSB5
                let path = output_dir.join(format!("{name}.fsb"));
                std::fs::write(&path, &bytes)?;
            }
        }
    } else {
        let path = output_dir.join(format!("{name}.bytes"));
        std::fs::write(&path, &bytes)?;
    }

    Ok(())
}
