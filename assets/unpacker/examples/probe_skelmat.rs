//! THROWAWAY diagnostic: for every `GameObject` that owns a spine skeleton
//! (`skeletonDataAsset`), dump its `MeshRenderer`'s material shader + `_Color`/`_TintColor`.
//!
//! Motivation: the exporter reads the spine renderer's `m_SortingOrder` but never its
//! material colour, so a skeleton-wide tint would be silently dropped.
//!
//! Usage: cargo run --release --example `probe_skelmat` -- <bundle.ab> [<bundle.ab>...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn colors(mat: &Value) -> Vec<(String, Vec<f64>)> {
    let mut out = Vec::new();
    let Some(arr) = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(Value::as_array)
    else {
        return out;
    };
    for e in arr {
        let (Some(k), Some(v)) = (
            e.get("first")
                .and_then(Value::as_str)
                .or_else(|| e.get(0).and_then(Value::as_str)),
            e.get("second").or_else(|| e.get(1)),
        ) else {
            continue;
        };
        let rgba: Vec<f64> = ["r", "g", "b", "a"]
            .iter()
            .map(|c| v.get(c).and_then(Value::as_f64).unwrap_or(f64::NAN))
            .collect();
        out.push((k.to_string(), rgba));
    }
    out
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        println!("\n===== {path}");
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for obj in &sf.objects {
                if skip.contains(&obj.class_id) {
                    continue;
                }
                if let Ok(v) = read_object(&sf, obj) {
                    all.insert(obj.path_id, (obj.class_id, v));
                }
            }
            let mut go_name: HashMap<i64, String> = HashMap::new();
            for (p, (cid, v)) in &all {
                if *cid == 1 {
                    go_name.insert(
                        *p,
                        v.get("m_Name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                }
            }
            let spine_gos: HashSet<i64> = all
                .values()
                .filter(|(cid, v)| *cid == 114 && v.get("skeletonDataAsset").is_some())
                .filter_map(|(_, v)| v.get("m_GameObject").and_then(pid))
                .collect();
            for (rp, (cid, v)) in &all {
                if *cid != 23 {
                    continue;
                }
                let Some(go) = v.get("m_GameObject").and_then(pid) else {
                    continue;
                };
                if !spine_gos.contains(&go) {
                    continue;
                }
                let name = go_name.get(&go).cloned().unwrap_or_default();
                let sort = v
                    .get("m_SortingOrder")
                    .and_then(Value::as_i64)
                    .unwrap_or(-999);
                println!("  renderer pid={rp} GO='{name}' sort={sort}");
                for m in v
                    .get("m_Materials")
                    .and_then(Value::as_array)
                    .unwrap_or(&vec![])
                {
                    let Some(mp) = pid(m) else { continue };
                    match all.get(&mp) {
                        Some((21, mv)) => {
                            let nm = mv.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                            println!("    mat pid={mp} name='{nm}'");
                            for (k, c) in colors(mv) {
                                println!("      {k} = {c:?}");
                            }
                        }
                        Some((c, _)) => println!("    mat pid={mp} is class {c}"),
                        None => println!("    mat pid={mp} EXTERNAL (not in bundle)"),
                    }
                }
            }
        }
    }
}
