//! Diagnostic: dump the entrance DIRECTOR MonoBehaviour's `_params` (duration,
//! charVoiceOffset, fadeColor) and its `_effects` list.
//!
//! `fadeColor` is documented in `spine.rs` but never read by the exporter — this prints it
//! so a director-driven screen fade can be checked against the recordings.
//!
//! Usage: cargo run --release --example probe_director -- <bundle.ab>

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
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
        // Every authored `_delayTime` in the prefab, with its owner.
        let mut delays: Vec<(f64, String)> = Vec::new();
        for (_, (cid, v)) in &all {
            if *cid == 114
                && let Some(dt) = v.get("_delayTime").and_then(Value::as_f64)
            {
                let nm = v
                    .get("m_GameObject")
                    .and_then(pid)
                    .and_then(|g| go_name.get(&g))
                    .cloned()
                    .unwrap_or_default();
                delays.push((dt, nm));
            }
        }
        delays.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        println!("\n=== authored _delayTime beats ({}) ===", delays.len());
        for (d, n) in &delays {
            println!("   {d:8.3}s  {n}");
        }
        for (p, (cid, v)) in &all {
            if *cid != 114 || v.get("_mainCamera").is_none() {
                continue;
            }
            let owner = v
                .get("m_GameObject")
                .and_then(pid)
                .and_then(|g| go_name.get(&g))
                .cloned()
                .unwrap_or_default();
            println!("\n=== DIRECTOR pathID={p} on '{owner}' ===");
            println!(
                "_params  = {}",
                serde_json::to_string(v.get("_params").unwrap_or(&Value::Null)).unwrap_or_default()
            );
            let n = v
                .get("_effects")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            println!("_effects = {n} entries");
            // Every effect entry's own `_delayTime`, so a director-scheduled beat is visible.
            if let Some(arr) = v.get("_effects").and_then(Value::as_array) {
                for e in arr {
                    let Some(ep) = pid(e) else { continue };
                    let Some((_, ev)) = all.get(&ep) else {
                        println!("   effect pathID={ep} (external)");
                        continue;
                    };
                    let nm = ev
                        .get("m_GameObject")
                        .and_then(pid)
                        .and_then(|g| go_name.get(&g))
                        .cloned()
                        .unwrap_or_default();
                    println!(
                        "   effect '{nm}' _delayTime={:?} keys={:?}",
                        ev.get("_delayTime"),
                        ev.as_object().map(|o| o.keys().collect::<Vec<_>>())
                    );
                }
            }
        }
    }
}
