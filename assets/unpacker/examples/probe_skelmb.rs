//! THROWAWAY diagnostic: dump the serialized fields of every `MonoBehaviour` that owns a
//! spine skeleton (`skeletonDataAsset`) — looking for a skeleton-level colour/alpha that
//! multiplies the whole character and that the exporter never reads.
//!
//! Usage: cargo run --release --example `probe_skelmb` -- <bundle.ab> [<bundle.ab>...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

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
            let mut names: HashMap<i64, String> = HashMap::new();
            for (p, (cid, v)) in &all {
                if *cid == 1 {
                    names.insert(
                        *p,
                        v.get("m_Name").and_then(Value::as_str).unwrap_or("").into(),
                    );
                }
            }
            for (cid, v) in all.values() {
                if *cid != 114 || v.get("skeletonDataAsset").is_none() {
                    continue;
                }
                let go = v
                    .get("m_GameObject")
                    .and_then(|g| g.get("m_PathID"))
                    .and_then(Value::as_i64)
                    .unwrap_or(0);
                println!(
                    "  --- spine MonoBehaviour on GO '{}'",
                    names.get(&go).cloned().unwrap_or_default()
                );
                if let Some(o) = v.as_object() {
                    for (k, val) in o {
                        if k.starts_with("m_") && k != "m_Enabled" {
                            continue;
                        }
                        println!(
                            "      {k} = {}",
                            serde_json::to_string(val).unwrap_or_default()
                        );
                    }
                }
            }
        }
    }
}
