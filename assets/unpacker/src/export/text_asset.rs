use aes::Aes128;
use base64::Engine;
use cbc::Decryptor;
use cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use serde_json::Value;
use std::{fs, io, path::Path};

type Aes128CbcDec = Decryptor<Aes128>;

/// Arknights AES-CBC mask (chat_mask v2)
const MASK: &[u8; 32] = b"UITpAi82pHAWwnzqHRMCwPonJLIB3WCl";

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

    // Try FlatBuffer decoding on raw data (128-byte RSA header + FlatBuffer)
    if let Some(json) = try_flatbuffer_decode(&raw, name) {
        let path = output_dir.join(format!("{name}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(&json).unwrap().as_bytes(),
        )?;
        return Ok(());
    }

    // Try FlatBuffer decoding on AES-decrypted data (some files are AES-encrypted FlatBuffers)
    if decrypted.is_some()
        && let Some(json) = try_flatbuffer_decode(bytes, name)
    {
        let path = output_dir.join(format!("{name}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(&json).unwrap().as_bytes(),
        )?;
        return Ok(());
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

/// Try FlatBuffer decode at various offsets (raw, skip 128-byte RSA header)
fn try_flatbuffer_decode(data: &[u8], name: &str) -> Option<Value> {
    for &offset in &[128usize, 0] {
        if data.len() <= offset {
            continue;
        }
        let fb_data = &data[offset..];
        if crate::flatbuffers_decode::is_flatbuffer(fb_data)
            && let Ok(json) = crate::flatbuffers_decode::decode_flatbuffer(fb_data, name)
        {
            return Some(json);
        }
    }
    None
}

/// Decrypt AES-CBC encrypted Arknights data.
/// Format: [128-byte RSA header][16-byte IV seed][AES-CBC encrypted data]
/// Key = MASK[0..16], IV = data[0..16] XOR MASK[16..32]
fn try_aes_decrypt(data: &[u8]) -> Option<Vec<u8>> {
    // Try with and without 128-byte RSA header
    for &offset in &[128usize, 0] {
        if data.len() <= offset + 16 {
            continue;
        }
        let payload = &data[offset..];

        // First 16 bytes are the IV seed
        let iv: [u8; 16] = std::array::from_fn(|i| payload[i] ^ MASK[16 + i]);
        let key = &MASK[..16];

        let encrypted = &payload[16..];
        if encrypted.is_empty() || !encrypted.len().is_multiple_of(16) {
            continue;
        }

        let mut buf = encrypted.to_vec();
        let result =
            Aes128CbcDec::new(key.into(), &iv.into()).decrypt_padded_mut::<Pkcs7>(&mut buf);

        if let Ok(decrypted) = result
            && !decrypted.is_empty()
        {
            // Verify decrypted data looks plausible
            let check_len = decrypted.len().min(16);
            let printable = decrypted[..check_len]
                .iter()
                .filter(|b| b.is_ascii_graphic() || b.is_ascii_whitespace())
                .count();
            if printable >= check_len * 3 / 4 {
                return Some(decrypted.to_vec());
            }
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
