//! THROWAWAY diagnostic: dump the RAW material JSON for every renderer on a named GO,
//! following external material refs, so we can see exactly what `_MainTex`/`_Color` hold.
//!
//! Usage: cargo run --release --example `probe_mat` -- <bundle.ab> <go-name> [<go-name>...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let names: Vec<String> = args.map(|s| s.to_lowercase()).collect();
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
        for (cid, v) in all.values() {
            if *cid != 23 && *cid != 137 && *cid != 199 {
                continue;
            }
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            let name = go_name.get(&go).cloned().unwrap_or_default();
            if !names.iter().any(|w| name.to_lowercase() == *w) {
                continue;
            }
            println!("\n########## renderer on GO '{name}' ##########");
            println!(
                "m_Materials = {}",
                serde_json::to_string(v.get("m_Materials").unwrap_or(&Value::Null))
                    .unwrap_or_default()
            );
            if let Some(mats) = v.get("m_Materials").and_then(Value::as_array) {
                for m in mats {
                    let Some(mp) = pid(m) else { continue };
                    match all.get(&mp) {
                        Some((21, mv)) => {
                            println!("--- material pathID={mp} ---");
                            println!("{}", serde_json::to_string_pretty(mv).unwrap_or_default());
                        }
                        Some((c, _)) => {
                            println!("--- pathID={mp} is class {c}, not a material ---");
                        }
                        None => println!("--- material pathID={mp} NOT IN BUNDLE (external) ---"),
                    }
                }
            }
        }
    }
}
