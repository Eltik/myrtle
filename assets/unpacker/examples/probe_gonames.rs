//! THROWAWAY diagnostic: the full `GameObject` inventory of a dynchar bundle.
//!
//! Motivation: Civilight Eterna's entrance renders into a hard 1920x1080 (16:9) aperture inside
//! a 2340x1080 screen, reproduced from a cold boot. Everything structured has been cleared --
//! Camera fields are byte-identical to Eyjafjalla's apart from the `GameObject` id, `_maxSize` is
//! genuinely 2048x2048 on both, neither ships a `RenderTexture`, the entrance director carries no
//! rendering params, `skin_table` has no flag, and no exported scene field separates her from the
//! seven full-frame skins categorically. So the aperture must come from an object we never
//! export at all. The entrance director's `_effects[]` is the one structural difference (cet 1,
//! Eyjafjalla 3), which makes the object inventory worth reading directly.
//!
//! Usage: cargo run --release --example `probe_gonames` -- <bundle.ab> [<bundle.ab> ...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    assert!(!paths.is_empty(), "usage: probe_gonames <bundle.ab> [...]");

    for path in &paths {
        let short = path.rsplit('/').next().unwrap_or(path);
        println!("\n######## {short}");
        let Ok(data) = std::fs::read(path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
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
            let mut names: Vec<(String, i64, usize)> = all
                .iter()
                .filter(|(_, (cid, _))| *cid == 1) // GameObject
                .map(|(pid, (_, v))| {
                    let n = v
                        .get("m_Name")
                        .and_then(Value::as_str)
                        .unwrap_or("?")
                        .to_string();
                    let comps = v
                        .get("m_Component")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len);
                    (n, *pid, comps)
                })
                .collect();
            names.sort();
            println!("  {} GameObject(s)", names.len());
            for (n, pid, c) in &names {
                println!("   {n:52} pid={pid:>21} components={c}");
            }
        }
    }
}
