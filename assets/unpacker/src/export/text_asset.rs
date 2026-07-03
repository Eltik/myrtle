use aes::Aes128;
use base64::Engine;
use cbc::Decryptor;
use cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use serde_json::Value;
use std::{fs, io, path::Path};

type Aes128CbcDec = Decryptor<Aes128>;

/// Arknights AES-CBC mask (`chat_mask` v2)
const MASK: &[u8; 32] = b"UITpAi82pHAWwnzqHRMCwPonJLIB3WCl";

/// Decode a `TextAsset`'s `m_Script` into a JSON `Value` (base64, AES-CBC,
/// `FlatBuffer` with optional 128-byte RSA header). Returns `None` if no
/// decode path yields structured data — e.g. plain text or invalid payloads.
///
/// Used by callers that need the parsed JSON in-memory rather than written
/// to disk — for example the mappreview pre-scan that reads each level's
/// tile grid dimensions to compute its display aspect.
#[must_use]
pub fn parse_text_asset_json(obj: &Value) -> Option<Value> {
    let script = obj.get("m_Script")?.as_str()?;
    if script.is_empty() {
        return None;
    }
    let raw = if let Some(b64) = script.strip_prefix("base64:") {
        base64::engine::general_purpose::STANDARD.decode(b64).ok()?
    } else {
        script.as_bytes().to_vec()
    };
    if raw.is_empty() {
        return None;
    }

    let decrypted = try_aes_decrypt(&raw);
    let name = obj.get("m_Name").and_then(|v| v.as_str()).unwrap_or("");
    try_structured_decode(&raw, decrypted.as_deref(), name)
}

/// Try every structured-decode path over a `TextAsset` payload, in order:
/// plain JSON (on AES-decrypted bytes if decryption succeeded, else raw),
/// `FlatBuffer` on raw (most level data: 128-byte RSA header + `FlatBuffer`),
/// then `FlatBuffer` on the AES-decrypted bytes.
fn try_structured_decode(raw: &[u8], decrypted: Option<&[u8]>, name: &str) -> Option<Value> {
    let bytes = decrypted.unwrap_or(raw);

    if let Ok(json) = serde_json::from_slice::<Value>(bytes) {
        return Some(json);
    }

    if let Some(json) = try_flatbuffer_decode(raw, name) {
        return Some(json);
    }

    if decrypted.is_some()
        && let Some(json) = try_flatbuffer_decode(bytes, name)
    {
        return Some(json);
    }

    None
}

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

    if let Some(json) = try_structured_decode(&raw, decrypted.as_deref(), name) {
        let path = output_dir.join(format!("{name}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(&json).unwrap().as_bytes(),
        )?;
        return Ok(());
    }

    // Save as text if valid UTF-8, otherwise raw bytes. Lua scripts already
    // carry their real extension from the manifest path — keep it instead of
    // appending .txt.
    if let Ok(text) = std::str::from_utf8(bytes) {
        let path = if std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lua"))
        {
            output_dir.join(name)
        } else {
            output_dir.join(format!("{name}.txt"))
        };
        fs::write(&path, text)?;
    } else {
        let path = output_dir.join(format!("{name}.bytes"));
        fs::write(&path, bytes)?;
    }

    Ok(())
}

/// Try `FlatBuffer` decode at various offsets (raw, skip 128-byte RSA header)
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
