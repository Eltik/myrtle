//! Diagnostic: list every TRANSFORM (typeID 4) binding in the entrance clips — position,
//! scale, rotation — with the GameObjects each path hash resolves to.
//!
//! Motivation: Skadi the Corrupting Heart's background fish are baked at the SEATED form's
//! location and sit far below frame once she reforms standing. They carry no BoneFollower,
//! so if the game moves them it must animate their rig root's Transform — which the
//! exporter would have to capture as an effect-host curve.
//!
//! Usage: cargo run --release --example probe_xform -- <bundle.ab> [go-name-filter]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

/// Unity Transform binding attributes.
fn attr_name(a: u64) -> &'static str {
    match a {
        1 => "position",
        2 => "rotation",
        3 => "scale",
        4 => "eulerAngles",
        _ => "?",
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
    let filter = std::env::args().nth(2).unwrap_or_default().to_lowercase();
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
        let mut tf_go: HashMap<i64, i64> = HashMap::new();
        let mut go_tf: HashMap<i64, i64> = HashMap::new();
        let mut tf_father: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            match cid {
                1 => {
                    go_name.insert(
                        *p,
                        v.get("m_Name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                }
                4 | 224 => {
                    if let Some(g) = v.get("m_GameObject").and_then(pid) {
                        tf_go.insert(*p, g);
                        go_tf.insert(g, *p);
                    }
                    if let Some(f) = v.get("m_Father").and_then(pid).filter(|&x| x != 0) {
                        tf_father.insert(*p, f);
                    }
                }
                _ => {}
            }
        }
        let full_path = |go: i64| -> String {
            let mut parts = Vec::new();
            let mut cur = go_tf.get(&go).copied();
            while let Some(tf) = cur {
                if let Some(g) = tf_go.get(&tf) {
                    parts.push(go_name.get(g).cloned().unwrap_or_default());
                }
                cur = tf_father.get(&tf).copied();
            }
            parts.reverse();
            parts.join("/")
        };
        // hash -> every GO whose ancestor-relative subpath hashes to it
        let mut hash_to_gos: HashMap<u32, Vec<i64>> = HashMap::new();
        for (p, (cid, _)) in &all {
            if *cid != 1 {
                continue;
            }
            let fp = full_path(*p);
            for start in 0..fp.split('/').count() {
                let sub = fp.split('/').skip(start).collect::<Vec<_>>().join("/");
                hash_to_gos
                    .entry(crc32(sub.as_bytes()))
                    .or_default()
                    .push(*p);
            }
        }

        println!("\n########## {} ##########", entry.path);
        for (cid, v) in all.values() {
            if *cid != 74 {
                continue;
            }
            let clip = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            if !clip.to_lowercase().contains("start") {
                continue;
            }
            let Some(gb) = v
                .get("m_ClipBindingConstant")
                .and_then(|b| b.get("genericBindings"))
                .and_then(Value::as_array)
            else {
                continue;
            };
            let mut rows: Vec<String> = Vec::new();
            for b in gb {
                let tid = b
                    .get("typeID")
                    .or_else(|| b.get("classID"))
                    .and_then(Value::as_i64)
                    .unwrap_or(-1);
                if tid != 4 {
                    continue;
                }
                let a = b.get("attribute").and_then(Value::as_u64).unwrap_or(0);
                let h = b.get("path").and_then(Value::as_u64).unwrap_or(0) as u32;
                let names: Vec<String> = hash_to_gos
                    .get(&h)
                    .map(|gos| gos.iter().map(|&g| full_path(g)).collect())
                    .unwrap_or_default();
                let shown: Vec<&String> = names
                    .iter()
                    .filter(|n| filter.is_empty() || n.to_lowercase().contains(&filter))
                    .collect();
                if shown.is_empty() && !filter.is_empty() {
                    continue;
                }
                rows.push(format!(
                    "   {:11} -> {} candidate(s){}",
                    attr_name(a),
                    names.len(),
                    if shown.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "\n        {}",
                            shown
                                .iter()
                                .take(6)
                                .map(|s| s.as_str())
                                .collect::<Vec<_>>()
                                .join("\n        ")
                        )
                    }
                ));
            }
            if !rows.is_empty() {
                println!(
                    "\n=== clip '{clip}' — {} transform binding(s) ===",
                    rows.len()
                );
                for r in rows {
                    println!("{r}");
                }
            }
        }
    }
}
