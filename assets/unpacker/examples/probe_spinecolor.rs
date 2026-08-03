//! THROWAWAY diagnostic: does the `_Start` entrance clip animate a MATERIAL COLOUR on the
//! SPINE renderer's GameObject? The scene exporter replays these curves for background
//! layers, but the spine GO is excluded from the layer set, so such a curve would be
//! dropped silently — and it would dim the whole character.
//!
//! Usage: cargo run --release --example probe_spinecolor -- <bundle.ab> [<bundle.ab>...]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::export::anim::entrance_material_color_channels;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        println!("\n===== {}", path.rsplit('/').next().unwrap_or(&path));
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") { continue; }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for obj in &sf.objects {
                if skip.contains(&obj.class_id) { continue; }
                if let Ok(v) = read_object(&sf, obj) { all.insert(obj.path_id, (obj.class_id, v)); }
            }
            let mut names: HashMap<i64, String> = HashMap::new();
            for (p, (cid, v)) in &all {
                if *cid == 1 { names.insert(*p, v.get("m_Name").and_then(Value::as_str).unwrap_or("").into()); }
            }
            let spine_gos: HashSet<i64> = all.values()
                .filter(|(cid, v)| *cid == 114 && v.get("skeletonDataAsset").is_some())
                .filter_map(|(_, v)| v.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64))
                .collect();
            let chans = entrance_material_color_channels(&all);
            if chans.is_empty() { continue; }
            println!("  GOs with ANIMATED material colour: {}", chans.len());
            for (go, cs) in &chans {
                let nm = names.get(go).cloned().unwrap_or_default();
                let is_spine = spine_gos.contains(go);
                if !is_spine && !nm.to_lowercase().contains("illust") && !nm.to_lowercase().contains("entrance") { continue; }
                println!("   {} GO '{}' (spine={})", if is_spine { ">>> SPINE" } else { "        " }, nm, is_spine);
                for c in cs {
                    let n = c.curve.len();
                    let first = c.curve.first().copied().unwrap_or((0.0, 0.0));
                    let last = c.curve.last().copied().unwrap_or((0.0, 0.0));
                    let mn = c.curve.iter().map(|s| s.1).fold(f32::INFINITY, f32::min);
                    let mx = c.curve.iter().map(|s| s.1).fold(f32::NEG_INFINITY, f32::max);
                    println!("       crc28=0x{:07X} ch={} n={} t[{:.2}..{:.2}] v[{:.3}..{:.3}] first={:.3} last={:.3}",
                        c.prop_crc28, c.channel, n, first.0, last.0, mn, mx, first.1, last.1);
                }
            }
            // also report spine GOs that have NO channel
            for go in &spine_gos {
                if !chans.contains_key(go) {
                    println!("        spine GO '{}' has NO animated material colour", names.get(go).cloned().unwrap_or_default());
                }
            }
        }
    }
}
