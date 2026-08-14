//! THROWAWAY diagnostic: every display controller's `_maxSize` in a dynchar bundle.
//!
//! Motivation: the exported scene field `aspect` is documented as "render-target aspect
//! (`maxSize.x / maxSize.y`)" and reads **1.0 for all 13 entrance skins** -- a suspiciously
//! constant value for a field that is supposed to vary. Civilight Eterna's entrance provably
//! renders into a 16:9 target (a hard 1920x1080 aperture inside 2340x1080, reproduced from a
//! cold boot), so a correct `aspect` for her would be 1.7778, not 1.0.
//!
//! `spine.rs` picks it with `controller_mbs.iter().find_map(|(_, v)| v.get("_maxSize") ...)`,
//! i.e. the FIRST controller that carries one. The same `find_map`-takes-the-first pattern has
//! already bitten on `_adjustes`, where the first controller's entries all parse to None and the
//! exporter never looks at another. A bundle can ship several controllers, so this dumps them
//! ALL -- with the owning `GameObject` and whether it is the own-prefab-root one -- so the picked
//! controller can be compared against the rest.
//!
//! Usage: cargo run --release --example `probe_maxsize` -- <bundle.ab> [<bundle.ab> ...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    assert!(!paths.is_empty(), "usage: probe_maxsize <bundle.ab> [...]");

    for path in &paths {
        let short = path.rsplit('/').next().unwrap_or(path);
        let Ok(data) = std::fs::read(path) else {
            println!("{short:44} (unreadable)");
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            println!("{short:44} (not a bundle)");
            continue;
        };
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
            // Same deterministic path_id order the exporter iterates in, so the FIRST row printed
            // is the one `find_map` would take.
            let mut ctrls: Vec<(i64, &Value)> = all
                .iter()
                .filter(|(_, (cid, _))| *cid == 114)
                .filter(|(_, (_, v))| v.get("_maxSize").is_some() || v.get("_cameraSize").is_some())
                .map(|(pid, (_, v))| (*pid, v))
                .collect();
            ctrls.sort_unstable_by_key(|(pid, _)| *pid);
            if ctrls.is_empty() {
                continue;
            }
            println!("== {short}  ({} controller(s))", ctrls.len());
            for (i, (pid, v)) in ctrls.iter().enumerate() {
                let go = v
                    .get("m_GameObject")
                    .and_then(|g| g.get("m_PathID"))
                    .and_then(Value::as_i64)
                    .and_then(|g| all.get(&g))
                    .and_then(|(_, gv)| gv.get("m_Name"))
                    .and_then(Value::as_str)
                    .unwrap_or("?");
                let ms = v.get("_maxSize");
                let x = ms.and_then(|m| m.get("x")).and_then(Value::as_f64);
                let y = ms.and_then(|m| m.get("y")).and_then(Value::as_f64);
                let cs = v.get("_cameraSize").and_then(Value::as_f64);
                let asp = match (x, y) {
                    (Some(x), Some(y)) if y > 0.0 => format!("{:.4}", x / y),
                    _ => "-".to_string(),
                };
                let picked = if i == 0 && ms.is_some() {
                    "  <== find_map picks this"
                } else {
                    ""
                };
                println!(
                    "   [{i}] pid={pid:>21} go={go:34} _maxSize={x:?}x{y:?} aspect={asp:>7} _cameraSize={cs:?}{picked}"
                );
            }
        }
    }
}
