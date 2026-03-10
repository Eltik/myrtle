use aes::Aes128;
use base64::Engine;
use cbc::Decryptor;
use cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use serde_json::Value;
use std::{fs, io, path::Path};

type Aes128CbcDec = Decryptor<Aes128>;

const AES_KEY: &[u8; 16] = b"UITpAi82pHAWwnzq";
const AES_IV: &[u8; 16] = b"HRMCwPonJLIB3WCl";

pub fn export_text_asset(
    obj: &Value,
    output_dir: &Path,
    output_name: Option<&str>,
) -> Result<(), io::Error> {
    let name = output_name.unwrap_or_else(|| obj["m_Name"].as_str().unwrap_or("unnamed"));
    let script = obj["m_Script"].as_str().unwrap_or("");

    if script.is_empty() {
        return Ok(());
    }

    // Get raw bytes - either base64-prefixed binary or plain text
    let raw = if let Some(b64) = script.strip_prefix("base64:") {
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        script.as_bytes().to_vec()
    };

    if raw.is_empty() {
        return Ok(());
    }

    // Try AES decrypt (for encrypted gamedata)
    let decrypted = try_aes_decrypt(&raw);
    let bytes = decrypted.as_deref().unwrap_or(&raw);

    // Try parsing as JSON
    if let Ok(json) = serde_json::from_slice::<Value>(bytes) {
        let path = output_dir.join(format!("{name}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(&json).unwrap().as_bytes(),
        )?;
        return Ok(());
    }

    // Try FlatBuffer decoding (gamedata tables have 128-byte RSA header + FlatBuffer)
    if raw.len() > 128 {
        let fb_data = &raw[128..];
        if crate::flatbuffers_decode::is_flatbuffer(fb_data)
            && let Ok(json) = crate::flatbuffers_decode::decode_flatbuffer(fb_data, name)
        {
            let path = output_dir.join(format!("{name}.json"));
            fs::write(
                &path,
                serde_json::to_string_pretty(&json).unwrap().as_bytes(),
            )?;
            return Ok(());
        }
    }

    // Save as text if valid UTF-8, otherwise raw bytes
    if let Ok(text) = std::str::from_utf8(bytes) {
        let path = output_dir.join(format!("{name}.txt"));
        fs::write(&path, text)?;
    } else {
        let path = output_dir.join(format!("{name}.bytes"));
        fs::write(&path, bytes)?;
    }

    Ok(())
}

fn try_aes_decrypt(data: &[u8]) -> Option<Vec<u8>> {
    // Some files have a 128-byte RSA header prefix
    let offsets = [0, 128];

    for &offset in &offsets {
        if data.len() <= offset {
            continue;
        }
        let encrypted = &data[offset..];
        if !encrypted.len().is_multiple_of(16) {
            continue;
        }

        let mut buf = encrypted.to_vec();
        let result =
            Aes128CbcDec::new(AES_KEY.into(), AES_IV.into()).decrypt_padded_mut::<Pkcs7>(&mut buf);

        if let Ok(decrypted) = result
            && !decrypted.is_empty()
            && (decrypted[0].is_ascii_graphic() || decrypted[0].is_ascii_whitespace())
        {
            return Some(decrypted.to_vec());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unity::bundle::BundleFile;
    use crate::unity::object_reader::read_object;
    use crate::unity::serialized_file::SerializedFile;
    use std::path::PathBuf;

    #[test]
    fn test_export_text_asset() {
        // gamedata excel files contain encrypted TextAssets
        let data = match std::fs::read(
            "../downloader/ArkAssets/anon/23ff0ecb17492e0883b6b98146d15bb3.bin",
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };
        let bundle = BundleFile::parse(data).unwrap();
        let sf = SerializedFile::parse(bundle.files[0].data.clone()).unwrap();

        let output_dir = PathBuf::from("test_output/text");
        std::fs::create_dir_all(&output_dir).unwrap();

        let mut exported = 0;
        for obj in &sf.objects {
            if obj.class_id != 49 {
                continue;
            } // 49 = TextAsset
            let val = read_object(&sf, obj).unwrap();
            let name = val["m_Name"].as_str().unwrap_or("?");
            println!("Exporting TextAsset: {name}");
            match export_text_asset(&val, &output_dir, None) {
                Ok(()) => exported += 1,
                Err(e) => println!("  error: {e}"),
            }
        }
        println!("Exported {exported} text assets to test_output/text/");
    }
}
