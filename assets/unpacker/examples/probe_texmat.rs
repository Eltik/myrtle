//! THROWAWAY diagnostic: find the Material that binds a given `_MainTex` texture NAME, and
//! print its shader plus `_TintColor` / `_MainColor`.
//!
//! Motivation: `particle_tint` skips every NON-additive material (`is_additive` early-return),
//! so an alphablend legacy port never exports its `_TintColor` — and Unity's legacy particle
//! shaders multiply by `_TintColor * 2` for alphablend exactly as they do for additive. Skadi's
//! sweeping streak (texture `skadi2_06`) measures ~4x too dim. Before touching that gate, read
//! the material's real tint: if it is the 0.5 neutral, the gate is not the cause.
//!
//! Usage: cargo run --release --example `probe_texmat` -- <bundle.ab> <shaders-dir> <tex-name>...
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::unnecessary_lazy_evaluations
)]

use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn col(mat: &Value, key: &str) -> String {
    mat.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.get(key))
        .map_or_else(
            || "-".to_string(),
            |c| {
                let g = |n: &str| c.get(n).and_then(Value::as_f64).unwrap_or(f64::NAN);
                format!("({:.4},{:.4},{:.4},{:.4})", g("r"), g("g"), g("b"), g("a"))
            },
        )
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: probe_texmat <bundle.ab> <shaders-dir> <tex-name>...");
    let shaders = args.next().expect("shaders dir");
    let wanted: Vec<String> = args.map(|s| s.to_ascii_lowercase()).collect();

    let smap = build_shader_map(&walkdir(&std::path::PathBuf::from(&shaders)));
    let data = std::fs::read(&path).expect("read bundle");
    let bundle = BundleFile::parse(data).expect("parse bundle");

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        // Texture pathID -> name, so a material's `_MainTex` ref can be named.
        let mut tex: HashMap<i64, String> = HashMap::new();
        for o in &sf.objects {
            if o.class_id == 28
                && let Ok(v) = read_object(&sf, o)
            {
                tex.insert(
                    o.path_id,
                    v.get("m_Name")
                        .and_then(Value::as_str)
                        .unwrap_or("?")
                        .to_string(),
                );
            }
        }
        for o in sf.objects.iter().filter(|o| o.class_id == 21) {
            let Ok(mat) = read_object(&sf, o) else {
                continue;
            };
            let main = mat
                .get("m_SavedProperties")
                .and_then(|sp| sp.get("m_TexEnvs"))
                .and_then(|te| te.get("_MainTex"))
                .and_then(|e| e.get("m_Texture"))
                .and_then(pid)
                .and_then(|t| tex.get(&t))
                .cloned()
                .unwrap_or_else(|| "-".into());
            if !wanted.is_empty() && !wanted.iter().any(|w| main.to_ascii_lowercase() == *w) {
                continue;
            }
            let shader = mat
                .get("m_Shader")
                .and_then(|s| Some((s.get("m_FileID")?.as_i64()?, s.get("m_PathID")?.as_i64()?)))
                .and_then(|(f, p)| resolve_shader(&sf.externals, f, p, &smap))
                .unwrap_or_else(|| "<unresolved>");
            println!(
                "MAT '{}'  _MainTex={main}\n    shader={shader}\n    _TintColor={}  _MainColor={}  _Color={}",
                mat.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                col(&mat, "_TintColor"),
                col(&mat, "_MainColor"),
                col(&mat, "_Color"),
            );
        }
    }
}

/// Flatten a directory into the file list `build_shader_map` expects.
fn walkdir(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        if d.is_file() {
            out.push(d);
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            stack.push(e.path());
        }
    }
    out
}
