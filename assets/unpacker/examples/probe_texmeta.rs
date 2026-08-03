//! THROWAWAY diagnostic: dump Texture2D metadata (format, colour space, mip count, filter)
//! for every texture in a dynchar bundle.
//!
//! Motivation: on FLAT colour the game renders Skadi/Virtuosa ~10% darker than we do, while
//! Mlynar matches exactly. The spine shader and material are byte-identical across skins, so
//! a per-skin difference would have to live in the texture asset itself — e.g. an sRGB flag,
//! a different compressed format, or a differing mip count.
//!
//! Usage: cargo run --release --example probe_texmeta -- <bundle.ab> [<bundle.ab>...]

use serde_json::Value;
use std::collections::HashSet;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn g<'a>(v: &'a Value, k: &str) -> Option<&'a Value> {
    v.get(k)
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        println!("\n===== {}", path.rsplit('/').next().unwrap_or(&path));
        println!(
            "  {:<52} {:>6} {:>6} {:>7} {:>5} {:>5} {:>5} {:>6}",
            "name", "w", "h", "format", "mips", "sRGB", "filt", "aniso"
        );
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let mut seen: HashSet<String> = HashSet::new();
            for obj in &sf.objects {
                if obj.class_id != 28 {
                    continue;
                }
                let Ok(v) = read_object(&sf, obj) else { continue };
                let name = g(&v, "m_Name").and_then(Value::as_str).unwrap_or("?").to_string();
                if !seen.insert(name.clone()) {
                    continue;
                }
                let n = |k: &str| g(&v, k).and_then(Value::as_i64).map(|x| x.to_string()).unwrap_or_else(|| "-".into());
                // Unity stores the sRGB flag as m_ColorSpace (1 = sRGB) on newer versions.
                let srgb = g(&v, "m_ColorSpace")
                    .or_else(|| g(&v, "m_sRGBTexture"))
                    .and_then(Value::as_i64)
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "-".into());
                let filt = g(&v, "m_TextureSettings")
                    .and_then(|s| s.get("m_FilterMode"))
                    .and_then(Value::as_i64)
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "-".into());
                let aniso = g(&v, "m_TextureSettings")
                    .and_then(|s| s.get("m_Aniso"))
                    .and_then(Value::as_i64)
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "-".into());
                let short: String = name.chars().rev().take(52).collect::<Vec<_>>().into_iter().rev().collect();
                println!(
                    "  {:<52} {:>6} {:>6} {:>7} {:>5} {:>5} {:>5} {:>6}",
                    short,
                    n("m_Width"),
                    n("m_Height"),
                    n("m_TextureFormat"),
                    n("m_MipCount"),
                    srgb,
                    filt,
                    aniso
                );
            }
        }
    }
}
