//! THROWAWAY diagnostic: how many display controllers does a dynchar bundle ship, and what
//! `_cameraSize` does each carry?
//!
//! Motivation: the clean skin-preview capture pins the preview's visible height to
//! `0.9565 x cameraSizePx`. Skadi and Virtuosa match to five significant figures (implied ortho
//! 10.0000 vs authored 10.000; 10.4997 vs 10.500) but Mlynar implies **10.7768** against an
//! authored **11.110** — a real 3% miss. `spine.rs` already documents that a bundle can ship
//! SEVERAL controllers and that picking via `HashMap` iteration once made amiya2's cameraSize flip
//! between 10.0 and 10.5; it now takes the own-prefab-root one, else the lowest `path_id`. If
//! Mlynar ships a second controller at ~10.78, we are simply reading the wrong one.
//!
//! Usage: cargo run --release --example `probe_camsize` -- <bundle.ab>
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe_camsize <bundle.ab>");
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
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for o in &sf.objects {
            if let Ok(v) = read_object(&sf, o) {
                all.insert(o.path_id, (o.class_id, v));
            }
        }
        // Every MonoBehaviour carrying `_cameraSize`, in the same deterministic path_id order
        // the exporter uses, plus its owning GameObject name so the two can be told apart.
        let mut ctrls: Vec<(i64, &Value)> = all
            .iter()
            .filter(|(_, (cid, _))| *cid == 114)
            .filter(|(_, (_, v))| v.get("_cameraSize").is_some())
            .map(|(pid, (_, v))| (*pid, v))
            .collect();
        ctrls.sort_unstable_by_key(|(pid, _)| *pid);
        if ctrls.is_empty() {
            continue;
        }
        println!(
            "== {} : {} controller(s) with _cameraSize",
            entry.path,
            ctrls.len()
        );
        for (pid, v) in &ctrls {
            let go_name = v
                .get("m_GameObject")
                .and_then(|g| g.get("m_PathID"))
                .and_then(Value::as_i64)
                .and_then(|go| all.get(&go))
                .and_then(|(_, gv)| gv.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            let cs = v
                .get("_cameraSize")
                .and_then(Value::as_f64)
                .unwrap_or(f64::NAN);
            let view = v.get("_adjustes").and_then(Value::as_array).map(|a| {
                a.iter()
                    .map(|s| s.get("size").and_then(Value::as_f64).unwrap_or(f64::NAN))
                    .collect::<Vec<_>>()
            });
            let ms = v.get("_maxSize").map(|m| {
                (
                    m.get("x").and_then(Value::as_f64),
                    m.get("y").and_then(Value::as_f64),
                )
            });
            println!(
                "   pathID {pid:>8}  GO {go_name:<34} _cameraSize {cs:.6}  _adjustes sizes {view:?}"
            );
            println!("      _maxSize {ms:?}");
            let sizes: Vec<f64> = v
                .get("_adjustes")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|s| {
                            s.get("size")
                                .and_then(|z| z.get("x"))
                                .and_then(Value::as_f64)
                                .unwrap_or(f64::NAN)
                        })
                        .collect()
                })
                .unwrap_or_default();
            println!("ROW\t{}\t{cs:.4}\t{sizes:?}", sizes.len());
        }
        // Also every Camera component (class 20), which is where an orthographic size would
        // live if the preview used a camera rather than the controller's authored number.
        for (pid, (cid, v)) in &all {
            if *cid != 20 {
                continue;
            }
            let os = v.get("orthographic size").and_then(Value::as_f64);
            let go_name = v
                .get("m_GameObject")
                .and_then(|g| g.get("m_PathID"))
                .and_then(Value::as_i64)
                .and_then(|go| all.get(&go))
                .and_then(|(_, gv)| gv.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            println!("   CAMERA pathID {pid:>8}  GO {go_name:<30} orthographic size {os:?}");
        }
    }
}
