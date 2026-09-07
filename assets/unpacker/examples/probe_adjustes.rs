//! Diagnostic: every `_adjustes` stop of every display controller in a dynchar bundle, whole.
//!
//! The exporter reads `_adjustes[0]` as the wide frame and `_adjustes[1]` as the tight one,
//! positionally, and 46 of 82 skins ship a third stop it never reads. Whether the third stop
//! explains the three-key tail of the entrance scores turns on what an entry IS: this prints
//! each entry's full serialized shape (every field, not only `offset` and `size`) so a type or
//! enum member, if one exists, cannot hide behind a positional read.
//!
//! Usage: cargo run --release --example `probe_adjustes` -- <bundle.ab> [<bundle.ab>...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> i64 {
    v.get("m_PathID").and_then(Value::as_i64).unwrap_or(0)
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        let short = path.rsplit('/').next().unwrap_or(&path).to_string();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let mut gos: HashMap<i64, String> = HashMap::new();
            let mut ctrls: Vec<(i64, Value)> = Vec::new();
            for obj in &sf.objects {
                if obj.class_id != 1 && obj.class_id != 114 {
                    continue;
                }
                let Ok(v) = read_object(&sf, obj) else {
                    continue;
                };
                if obj.class_id == 1 {
                    gos.insert(
                        obj.path_id,
                        v.get("m_Name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                } else if v.get("_adjustes").is_some() {
                    ctrls.push((obj.path_id, v));
                }
            }
            ctrls.sort_unstable_by_key(|(p, _)| *p);
            for (p, v) in &ctrls {
                let go = gos
                    .get(&v.get("m_GameObject").map(pid).unwrap_or(0))
                    .cloned()
                    .unwrap_or_default();
                let stops = v.get("_adjustes").and_then(Value::as_array);
                let n = stops.map_or(0, Vec::len);
                let cam = v.get("_cameraSize").and_then(Value::as_f64).unwrap_or(-1.0);
                println!("{short}\tctrl {p}\tGO '{go}'\t_cameraSize {cam}\tstops {n}");
                if let Some(stops) = stops {
                    for (i, s) in stops.iter().enumerate() {
                        println!("  [{i}] {s}");
                    }
                }
                // Every other field of the controller, so a sibling that names the stops
                // (an index, a mode, a default) is visible beside them.
                if let Some(obj) = v.as_object() {
                    let others: Vec<String> = obj
                        .iter()
                        .filter(|(k, _)| !k.starts_with("m_") && k.as_str() != "_adjustes")
                        .map(|(k, val)| {
                            let s = val.to_string();
                            format!(
                                "{k}={}",
                                if s.len() > 60 {
                                    format!("{}..", &s[..60])
                                } else {
                                    s
                                }
                            )
                        })
                        .collect();
                    println!("  fields: {}", others.join("  "));
                }
            }
        }
    }
}
