//! THROWAWAY diagnostic: does a dynchar bundle carry any LIGHTING or ambient/fog state that our
//! texture-only renderer necessarily ignores?
//!
//! Motivation: Mlynar's t=4 warm deficit is BLUR-INVARIANT (a light, not missing art), warm in
//! R+G with blue flat, region-localised, and attributes to no drawn layer. A Unity Light, an
//! ambient colour, or fog would produce exactly that and would appear in NO per-layer ablation,
//! because it is not a layer. The entrance camera's PostProcessLayer also declares
//! `fog: {enabled: 1, excludeSkybox: 1}`, which only does anything if fog state exists.
//!
//! Class IDs: 108 Light, 104 RenderSettings, 157 LightmapSettings, 215 ReflectionProbe,
//! 220 LightProbeGroup, 258 LightProbes.
//!
//! Usage: cargo run --release --example probe_light -- <bundle.ab>

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    let want: HashSet<i32> = [108, 104, 157, 215, 220, 258].into_iter().collect();

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        let mut hist: HashMap<i32, usize> = HashMap::new();
        for obj in &sf.objects {
            *hist.entry(obj.class_id).or_default() += 1;
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        let mut go_name: HashMap<i64, String> = HashMap::new();
        for (p, (cls, v)) in &all {
            if *cls == 1 {
                if let Some(n) = v.get("m_Name").and_then(Value::as_str) {
                    go_name.insert(*p, n.to_string());
                }
            }
        }

        let mut found = 0;
        for (p, (cls, v)) in &all {
            if !want.contains(cls) {
                continue;
            }
            found += 1;
            let go = v
                .get("m_GameObject")
                .and_then(pid)
                .and_then(|g| go_name.get(&g))
                .cloned()
                .unwrap_or_default();
            println!("\n===== class {cls} pid={p} on GO '{go}'");
            println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
        }
        if found == 0 {
            let mut ks: Vec<_> = hist.iter().map(|(k, n)| (*k, *n)).collect();
            ks.sort();
            println!("no lighting/ambient/fog objects in {}", entry.path);
            println!("  class histogram: {ks:?}");
        }
    }
}
