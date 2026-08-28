//! THROWAWAY: for a named Shader, print every TEXTURE property with the DEFAULT Unity
//! substitutes when a material leaves it UNBOUND (`m_DefTexture.m_DefaultName`).
//!
//! Motivation: whitw2's Ram materials ship `_RamTex` with pid 0. What the game then samples
//! is not a fit, it is whatever ShaderLab declared, so this reads it off the shader.
//!
//! Usage: cargo run --release --example probe_texdefault -- <shaders.ab> <name-filter>
use serde_json::Value;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    let path = std::env::args().nth(1).expect("shaders.ab");
    let filter = std::env::args().nth(2).unwrap_or_default().to_ascii_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") { continue }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        for obj in &sf.objects {
            if obj.class_id != 48 { continue }
            let Ok(v) = read_object(&sf, obj) else { continue };
            let name = v.get("m_ParsedForm").and_then(|p| p.get("m_Name"))
                .or_else(|| v.get("m_Name")).and_then(Value::as_str).unwrap_or("");
            if name.is_empty() || !name.to_ascii_lowercase().contains(&filter) { continue }
            let Some(props) = v.get("m_ParsedForm").and_then(|p| p.get("m_PropInfo"))
                .and_then(|p| p.get("m_Props")).and_then(Value::as_array) else { continue };
            println!("\n=== {name}");
            for p in props {
                let pn = p.get("m_Name").and_then(Value::as_str).unwrap_or("");
                let Some(dt) = p.get("m_DefTexture") else { continue };
                let dn = dt.get("m_DefaultName").and_then(Value::as_str).unwrap_or("");
                let ty = p.get("m_Type").and_then(Value::as_i64).unwrap_or(-1);
                // m_Type 4 is the texture kind in SerializedProperty; print it so a
                // non-texture property that happens to carry the field is visible as such.
                println!("  {pn:<26} type={ty} defaultName={dn:?}");
            }
        }
    }
}
