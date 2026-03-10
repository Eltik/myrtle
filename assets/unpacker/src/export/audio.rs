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
    let ext = if bytes.len() >= 4 && &bytes[0..4] == b"OggS" {
        "ogg"
    } else if bytes.len() >= 4 && &bytes[0..4] == b"RIFF" {
        "wav"
    } else if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        "m4a"
    } else {
        "bytes"
    };

    let path = output_dir.join(format!("{name}.{ext}"));
    std::fs::write(&path, &bytes)?;

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
    fn test_export_audio() {
        let data = match std::fs::read(
            "../downloader/ArkAssets/audio/sound_beta_2/voice_en/char_002_amiya.ab",
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };
        let bundle = BundleFile::parse(data).unwrap();

        // Build resource map from .resS entries
        let mut resources = HashMap::new();
        for entry in &bundle.files {
            if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
                let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
                resources.insert(filename.to_string(), entry.data.clone());
            }
        }

        let sf = SerializedFile::parse(bundle.files[0].data.clone()).unwrap();
        let output_dir = PathBuf::from("test_output/audio");
        std::fs::create_dir_all(&output_dir).unwrap();

        let mut exported = 0;
        for obj in &sf.objects {
            if obj.class_id != 83 {
                continue;
            } // 83 = AudioClip
            let val = read_object(&sf, obj).unwrap();
            let name = val["m_Name"].as_str().unwrap_or("?");
            println!("Exporting AudioClip: {name}");
            match export_audio(&val, &output_dir, &resources) {
                Ok(()) => exported += 1,
                Err(e) => println!("  skip: {e}"),
            }
        }
        println!("Exported {exported} audio clips to test_output/audio/");
        assert!(exported > 0, "should export at least one audio clip");
    }
}
