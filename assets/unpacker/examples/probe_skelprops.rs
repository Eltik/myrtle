//! THROWAWAY diagnostic: dump EVERY saved property (floats, colors, texEnvs) and the shader
//! reference of the material on each spine-skeleton `MeshRenderer`.
//!
//! Motivation: our render is systematically too bright in the midtones and the error fits a
//! per-skin GAMMA (Skadi 1.12, Virtuosa 1.07, Mlynar 1.02). A per-skin tone transform would
//! most plausibly live as a shader float on the character material.
//!
//! Usage: cargo run --release --example `probe_skelprops` -- <bundle.ab> [<bundle.ab>...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn kv(e: &Value) -> Option<(&str, &Value)> {
    let k = e
        .get("first")
        .and_then(Value::as_str)
        .or_else(|| e.get(0).and_then(Value::as_str))?;
    let v = e.get("second").or_else(|| e.get(1))?;
    Some((k, v))
}

fn dump_props(mv: &Value) {
    let Some(sp) = mv.get("m_SavedProperties") else {
        println!("      (no m_SavedProperties)");
        return;
    };
    for (label, key) in [
        ("float", "m_Floats"),
        ("color", "m_Colors"),
        ("tex", "m_TexEnvs"),
    ] {
        let Some(arr) = sp.get(key).and_then(Value::as_array) else {
            continue;
        };
        if arr.is_empty() {
            continue;
        }
        for e in arr {
            let Some((k, v)) = kv(e) else { continue };
            match label {
                "float" => println!(
                    "      {label:<5} {k:<26} = {}",
                    v.as_f64().unwrap_or(f64::NAN)
                ),
                "color" => {
                    let c: Vec<String> = ["r", "g", "b", "a"]
                        .iter()
                        .map(|c| {
                            format!(
                                "{:.4}",
                                v.get(c).and_then(Value::as_f64).unwrap_or(f64::NAN)
                            )
                        })
                        .collect();
                    println!("      {label:<5} {k:<26} = [{}]", c.join(", "));
                }
                _ => {
                    let sc = v.get("m_Scale");
                    let of = v.get("m_Offset");
                    println!(
                        "      {label:<5} {k:<26} = scale {:?} offset {:?}",
                        sc.map(|s| (
                            s.get("x").and_then(Value::as_f64),
                            s.get("y").and_then(Value::as_f64)
                        )),
                        of.map(|s| (
                            s.get("x").and_then(Value::as_f64),
                            s.get("y").and_then(Value::as_f64)
                        ))
                    );
                }
            }
        }
    }
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        println!("\n===== {}", path.rsplit('/').next().unwrap_or(&path));
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
            let mut seen: HashSet<i64> = HashSet::new();
            for (cid, v) in all.values() {
                if *cid != 23 {
                    continue;
                }
                let Some(go) = v.get("m_GameObject").and_then(pid) else {
                    continue;
                };
                if !spine_gos.contains(&go) {
                    continue;
                }
                println!("  GO '{}'", go_name.get(&go).cloned().unwrap_or_default());
                for m in v
                    .get("m_Materials")
                    .and_then(Value::as_array)
                    .unwrap_or(&vec![])
                {
                    let Some(mp) = pid(m) else { continue };
                    if !seen.insert(mp) {
                        continue;
                    }
                    match all.get(&mp) {
                        Some((21, mv)) => {
                            println!(
                                "    mat '{}'  shaderRef {:?}  keywords {:?}",
                                mv.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                                mv.get("m_Shader"),
                                mv.get("m_ShaderKeywords")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                            );
                            dump_props(mv);
                        }
                        Some((c, _)) => println!("    mat pid={mp} is class {c}"),
                        None => println!("    mat pid={mp} EXTERNAL"),
                    }
                }
            }
        }
    }
}
