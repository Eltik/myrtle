//! THROWAWAY diagnostic: resolve every MATERIAL's external shader reference to its shader
//! NAME, and print it with the material's `_MainColor` / `_Color` / `_TintColor` and blend.
//!
//! Motivation: `ram_tint_scale` keys the ×2 `_MainColor` convention off the shader name, so
//! two visually identical wind sheets can take completely different tint paths. `probe_mat`
//! only sees `m_Shader = <external PPtr>` and cannot tell them apart.
//!
//! Usage: cargo run --release --example probe_shader -- <bundle.ab> <shaders-dir> [name-filter]

use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pptr(v: &Value) -> Option<(i64, i64)> {
    Some((v.get("m_FileID")?.as_i64()?, v.get("m_PathID")?.as_i64()?))
}

fn col(mat: &Value, key: &str) -> String {
    mat.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.get(key))
        .map_or_else(
            || "-".to_string(),
            |c| {
                let g = |k: &str| c.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN);
                format!("({:.4},{:.4},{:.4},{:.4})", g("r"), g("g"), g("b"), g("a"))
            },
        )
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
    let shader_dir = std::env::args().nth(2).expect("shaders dir");
    let filter = std::env::args().nth(3).unwrap_or_default().to_lowercase();

    let mut files = Vec::new();
    for e in walkdir(&std::path::PathBuf::from(&shader_dir)) {
        files.push(e);
    }
    let map = build_shader_map(&files);
    eprintln!("shader map entries: {}", map.len());

    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let mut mats: HashMap<i64, Value> = HashMap::new();
        for obj in &sf.objects {
            if obj.class_id != 21 {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                mats.insert(obj.path_id, v);
            }
        }
        let mut names: Vec<_> = mats.iter().collect();
        names.sort_by_key(|(p, _)| **p);
        for (pid, m) in names {
            let name = m.get("m_Name").and_then(Value::as_str).unwrap_or("");
            if !filter.is_empty() && !name.to_lowercase().contains(&filter) {
                continue;
            }
            let shader = m
                .get("m_Shader")
                .and_then(pptr)
                .and_then(|(f, p)| resolve_shader(&sf.externals, f, p, &map))
                .unwrap_or("<unresolved>");
            let f = |k: &str| {
                m.get("m_SavedProperties")
                    .and_then(|sp| sp.get("m_Floats"))
                    .and_then(|c| c.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::NAN)
            };
            println!(
                "{name}  pid={pid}\n    shader={shader}\n    _MainColor={} _Color={} _TintColor={}\n    src={} dst={} MainColorACtrl={} Opacity={}",
                col(m, "_MainColor"),
                col(m, "_Color"),
                col(m, "_TintColor"),
                f("_SrcBlend"),
                f("_DstBlend"),
                f("_MainColorACtrl"),
                f("_Opacity"),
            );
        }
    }
}

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
