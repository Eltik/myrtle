use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::Path;

/// Check if data is likely a FlatBuffer
pub fn is_flatbuffer(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }

    // FlatBuffer starts with root table offset (4 bytes, little endian)
    let root_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    // Root offset should point within the buffer
    if root_offset >= data.len() || root_offset < 4 {
        return false;
    }

    // At root offset, there should be a vtable offset (signed)
    if root_offset + 4 > data.len() {
        return false;
    }

    let vtable_offset = i32::from_le_bytes([
        data[root_offset],
        data[root_offset + 1],
        data[root_offset + 2],
        data[root_offset + 3],
    ]);

    // vtable should be before the root table
    let vtable_pos = (root_offset as i32 - vtable_offset) as usize;
    if vtable_pos >= data.len() || vtable_pos < 4 {
        return false;
    }

    // vtable starts with its size (2 bytes)
    let vtable_size = u16::from_le_bytes([data[vtable_pos], data[vtable_pos + 1]]) as usize;

    // Sanity check vtable size
    vtable_size >= 4 && vtable_size < 1000 && vtable_pos + vtable_size <= data.len()
}

/// Guess the root type from filename
fn guess_root_type(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();

    if lower.contains("enemy_database") {
        "enemy_database"
    } else if lower.contains("character_table") || lower.contains("char_table") {
        "character_table"
    } else if lower.contains("skill_table") {
        "skill_table"
    } else if lower.contains("stage_table") {
        "stage_table"
    } else if lower.contains("item_table") {
        "item_table"
    } else if lower.contains("gacha_table") {
        "gacha_table"
    } else if lower.contains("enemy_handbook") {
        "enemy_handbook"
    } else {
        "unknown"
    }
}

/// Decode enemy database to clean JSON using direct field access
fn decode_enemy_database(data: &[u8]) -> Result<Value> {
    // Use the new macro-based implementation for fast, clean JSON output
    crate::fb_json_impl_enemy::decode_enemy_database_json(data)
}

/// Decode character table to clean JSON using direct field access
fn decode_character_table(data: &[u8]) -> Result<Value> {
    // Use the new macro-based implementation for fast, clean JSON output
    crate::fb_json_impl_character::decode_character_table_json(data)
}

/// Decode FlatBuffer with schema-based decoding
pub fn decode_flatbuffer_generic(data: &[u8]) -> Result<Value> {
    decode_flatbuffer(data, "")
}

/// Main decode function - uses schema-based decoding when possible
pub fn decode_flatbuffer(data: &[u8], filename: &str) -> Result<Value> {
    if !is_flatbuffer(data) {
        anyhow::bail!("Data is not a valid FlatBuffer");
    }

    let schema_type = guess_root_type(filename);

    // Only try matching schemas to avoid slow FlatBuffer parsing on mismatched data
    let result = match schema_type {
        "enemy_database" => decode_enemy_database(data),
        "character_table" => decode_character_table(data),
        _ => Err(anyhow::anyhow!("Unknown schema type")),
    };

    if let Ok(value) = result {
        return Ok(value);
    }

    // For unknown schemas, just extract strings (fast fallback)
    Ok(json!({
        "type": "unknown",
        "strings": extract_strings(data)
    }))
}

/// Extract strings from FlatBuffer (fallback for unknown types)
pub fn extract_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut i = 0;

    while i + 4 < data.len() {
        let potential_len =
            u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;

        if potential_len > 0 && potential_len < 1000 && i + 4 + potential_len <= data.len() {
            let str_data = &data[i + 4..i + 4 + potential_len];

            if let Ok(s) = std::str::from_utf8(str_data) {
                if s.chars()
                    .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace() || !c.is_ascii())
                {
                    if s.len() >= 2 {
                        strings.push(s.to_string());
                    }
                }
            }
        }
        i += 1;
    }

    strings
}

/// Decode and save FlatBuffer file
pub fn decode_flatbuffer_file(input: &Path, output: &Path) -> Result<()> {
    let data = std::fs::read(input).with_context(|| format!("Failed to read {:?}", input))?;

    let filename = input.file_name().and_then(|s| s.to_str()).unwrap_or("");

    let decoded = decode_flatbuffer(&data, filename)?;

    let json_str = serde_json::to_string_pretty(&decoded)?;
    std::fs::write(output, json_str).with_context(|| format!("Failed to write {:?}", output))?;

    Ok(())
}
