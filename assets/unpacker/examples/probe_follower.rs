//! THROWAWAY diagnostic: for every MESH renderer (class 23/33/137) in a dynchar bundle,
//! print its `GameObject` path and the nearest spine-unity `BoneFollower` in its ancestry.
//!
//! Motivation: scene-mesh quads are exported with a STATIC baked world matrix. If a quad's
//! `GameObject` rides a spine bone via `BoneFollower`, that serialized transform is only an
//! editor pose and the exported quad lands nowhere near the bone it should track.
//!
//! Usage: cargo run --release --example `probe_follower` -- <bundle.ab> [go-name-filter]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::or_fun_call,
    clippy::too_many_lines
)]

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
    let filter = args.next().unwrap_or_default().to_lowercase();
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
        let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
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
            if *cid == 4
                && let Some(go) = v.get("m_GameObject").and_then(pid)
            {
                go_to_tf.insert(go, *p);
            }
        }
        // spine-unity BoneFollower components, field-shape identified.
        let mut go_follower: HashMap<i64, (String, bool)> = HashMap::new();
        for (cid, v) in all.values() {
            if *cid == 114
                && let Some(bone) = v.get("boneName").and_then(Value::as_str)
                && !bone.is_empty()
                && v.get("followXYPosition").is_some()
                && let Some(go) = v.get("m_GameObject").and_then(pid)
            {
                let rot = v
                    .get("followBoneRotation")
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
                    != 0;
                go_follower.insert(go, (bone.to_string(), rot));
            }
        }
        let ancestry = |go: i64| -> Vec<i64> {
            let mut out = vec![go];
            let mut cur = go;
            for _ in 0..64 {
                let Some(tf) = go_to_tf.get(&cur) else { break };
                let Some(f) = all
                    .get(tf)
                    .and_then(|(_, v)| v.get("m_Father"))
                    .and_then(pid)
                    .filter(|&p| p != 0)
                else {
                    break;
                };
                let Some(g) = all
                    .get(&f)
                    .and_then(|(_, v)| v.get("m_GameObject"))
                    .and_then(pid)
                else {
                    break;
                };
                out.push(g);
                cur = g;
            }
            out
        };

        for (cid, v) in all.values() {
            if *cid != 23 && *cid != 33 && *cid != 137 {
                continue;
            }
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            let chain = ancestry(go);
            let path: Vec<String> = chain
                .iter()
                .rev()
                .map(|g| go_name.get(g).cloned().unwrap_or_default())
                .collect();
            let joined = path.join("/");
            if !filter.is_empty() && !joined.to_lowercase().contains(&filter) {
                continue;
            }
            let follower = chain
                .iter()
                .find_map(|g| go_follower.get(g).map(|f| (g, f)));
            println!(
                "renderer[{cid}] {joined}\n    follower = {}",
                follower.map_or("NONE".to_string(), |(g, (b, r))| format!(
                    "bone='{b}' rot={r} on '{}'",
                    go_name.get(g).cloned().unwrap_or_default()
                ))
            );
        }
    }
}
