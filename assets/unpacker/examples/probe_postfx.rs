//! THROWAWAY diagnostic: find the PostProcessing volume/profile shipped with a dyn-illust bundle.
//!
//! Motivation: cet's residual is a broad, smooth, near-zero-mean field that survives every global
//! geometric and photometric correction (flip/shift/scale/blur/gain all measured out). A
//! post-process chain we do not reproduce — bloom, vignette, colour grading — has exactly that
//! signature. Her bundle ships Hidden/PostProcessing/{Uber,Bloom,MobileBlur,FinalPass}, and
//! Whislash's `_Start` clip animates a MonoBehaviour named `pp`, so the cinematics clearly drive
//! one. Print any MonoBehaviour whose fields look like post-process settings.
//!
//! Usage: cargo run --release --example probe_postfx -- <bundle.ab>...
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

const KEYS: &[&str] = &[
    "bloom", "vignette", "grading", "exposure", "intensity", "threshold", "softKnee", "diffusion",
    "colorFilter", "temperature", "tint", "contrast", "saturation", "gamma", "gain", "lift",
    "chromatic", "grain", "ambient", "postExposure", "hueShift", "brightness",
];

fn walk(v: &Value, depth: usize, out: &mut Vec<String>, path: &str) {
    if depth > 4 { return; }
    match v {
        Value::Object(m) => {
            for (k, vv) in m {
                let kl = k.to_ascii_lowercase();
                if KEYS.iter().any(|s| kl.contains(s)) {
                    let t = format!("{vv}");
                    out.push(format!("      {path}.{k} = {}", if t.len() > 150 { format!("{}…", &t[..150]) } else { t }));
                }
                walk(vv, depth + 1, out, &format!("{path}.{k}"));
            }
        }
        Value::Array(a) => {
            for (i, vv) in a.iter().enumerate().take(8) { walk(vv, depth + 1, out, &format!("{path}[{i}]")); }
        }
        _ => {}
    }
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        println!("== {}", path.rsplit('/').next().unwrap_or(&path));
        for entry in &bundle.files {
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let mut names: HashMap<i64, String> = HashMap::new();
            for obj in &sf.objects {
                if obj.class_id == 1 {
                    if let Ok(v) = read_object(&sf, obj) {
                        names.insert(obj.path_id, v["m_Name"].as_str().unwrap_or("").to_string());
                    }
                }
            }
            for obj in &sf.objects {
                if obj.class_id != 114 { continue; }
                let Ok(v) = read_object(&sf, obj) else { continue };
                let mut out = Vec::new();
                walk(&v, 0, &mut out, "");
                // Also surface volume/profile wiring even when it carries no numeric fields.
                let raw = format!("{v}");
                let is_vol = raw.contains("sharedProfile") || raw.contains("profile") || raw.contains("isGlobal") || raw.contains("blendDistance");
                if out.len() < 3 && !is_vol { continue; }
                if is_vol {
                    let go2 = v.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64).unwrap_or(0);
                    println!("   VOLUME-ish on GO '{}': {}", names.get(&go2).cloned().unwrap_or_default(),
                        if raw.len() > 400 { format!("{}…", &raw[..400]) } else { raw });
                }
                let go = v.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64).unwrap_or(0);
                let nm = names.get(&go).cloned().unwrap_or_default();
                let own = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
                println!("   MonoBehaviour on GO '{nm}' (m_Name '{own}') — {} post-fx-ish fields:", out.len());
                for l in out.iter().take(24) { println!("{l}"); }
            }
        }
    }
}
