//! THROWAWAY diagnostic: every display controller's `_maxSize`, `_cameraSize` and any other
//! aspect-bearing field, per bundle.
//!
//! Motivation: Civilight Eterna's entrance provably renders into a hard 1920x1080 aperture inside
//! the 2340x1080 screen (cold-boot reproduced). Simulating that on our render — cropping the
//! horizontal FOV to 1920/2340 = 0.8205 and blacking the bars — moves her MADC 38.578 -> 33.076
//! and her luma correlation 0.541 -> 0.720, a SHARP optimum at exactly that ratio. The open
//! question is which skins get the aperture. `_maxSize` is where the exporter already reads a
//! render-target aspect, so it is the first place to look for a data-derived gate.
//!
//! Usage: cargo run --release --example `probe_maxaspect` -- <bundle.ab>...
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
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
        let short = path
            .rsplit('/')
            .next()
            .unwrap_or(&path)
            .trim_end_matches(".ab")
            .to_string();
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
            let mut pids: Vec<i64> = all.keys().copied().collect();
            pids.sort_unstable();
            for pid in pids {
                let (cid, v) = &all[&pid];
                if *cid != 114 {
                    continue;
                }
                let has = v.get("_maxSize").is_some() || v.get("_cameraSize").is_some();
                if !has {
                    continue;
                }
                let g = |k: &str, a: &str| v.get(k).and_then(|x| x.get(a)).and_then(Value::as_f64);
                let mx = g("_maxSize", "x");
                let my = g("_maxSize", "y");
                let cs = v.get("_cameraSize").and_then(Value::as_f64);
                let asp = match (mx, my) {
                    (Some(a), Some(b)) if b != 0.0 => format!("{:.4}", a / b),
                    _ => "-".into(),
                };
                println!(
                    "{short:34} pid={pid:<21} _maxSize=({mx:?},{my:?}) aspect={asp:>7} _cameraSize={cs:?}"
                );
            }
        }
    }
}
