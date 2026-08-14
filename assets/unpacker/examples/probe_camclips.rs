//! THROWAWAY diagnostic: which `AnimationClips` actually drive the ENTRANCE CAMERA, and would the
//! exporter's NAME gate (`is_entrance_clip`: "start" | "entrance" | "enter") admit them?
//!
//! Motivation: Eyjafjalla the Hvit Aska animates her Main Camera's `orthographic size` from a clip
//! called `char_1016_agoat2#_camera_01`. The name gate misses it, so `entrance_ortho_curve` and
//! `entrance_camera_track` both return None and her entrance is framed on the IDLE's tight bounds.
//! She scores 81.506 with a luma correlation of 0.248 against the capture.
//!
//! Usage: cargo run --release --example `probe_camclips` -- <bundle.ab>...
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::too_many_lines
)]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn crc32(s: &str) -> u32 {
    let mut c: u32 = 0xFFFF_FFFF;
    for b in s.as_bytes() {
        c ^= u32::from(*b);
        for _ in 0..8 {
            c = if c & 1 != 0 {
                (c >> 1) ^ 0xEDB8_8320
            } else {
                c >> 1
            };
        }
    }
    !c
}
fn pid(v: &Value, k: &str) -> Option<i64> {
    v.get(k)?.get("m_PathID")?.as_i64().filter(|p| *p != 0)
}

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
            // camera-chain GameObject subpaths, as CRC32 of the path Unity hashes
            let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
            let mut tf_go: HashMap<i64, i64> = HashMap::new();
            let mut tf_father: HashMap<i64, i64> = HashMap::new();
            let mut cam_go: Option<i64> = None;
            for (p, (cid, v)) in &all {
                match cid {
                    4 | 224 => {
                        if let Some(g) = pid(v, "m_GameObject") {
                            go_to_tf.insert(g, *p);
                            tf_go.insert(*p, g);
                        }
                        if let Some(f) = pid(v, "m_Father") {
                            tf_father.insert(*p, f);
                        }
                    }
                    20 => cam_go = pid(v, "m_GameObject").or(cam_go),
                    _ => {}
                }
            }
            let Some(cg) = cam_go else { continue };
            // walk up from the camera, collecting names to build subpaths
            let mut names: Vec<String> = Vec::new();
            let mut cur = go_to_tf.get(&cg).copied();
            for _ in 0..64 {
                let Some(tf) = cur else { break };
                let go = tf_go.get(&tf).copied().unwrap_or(0);
                names.push(
                    all.get(&go)
                        .and_then(|(_, v)| v.get("m_Name"))
                        .and_then(Value::as_str)
                        .unwrap_or("?")
                        .to_string(),
                );
                cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
            }
            names.reverse();
            let mut hashes: HashSet<u32> = HashSet::new();
            for i in 0..names.len() {
                for j in i..names.len() {
                    hashes.insert(crc32(&names[i..=j].join("/")));
                }
            }
            let mut rows: Vec<String> = Vec::new();
            for (cid, v) in all.values() {
                if *cid != 74 {
                    continue;
                }
                let name = v
                    .get("m_Name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let Some(b) = v
                    .get("m_ClipBindingConstant")
                    .and_then(|c| c.get("genericBindings"))
                    .and_then(Value::as_array)
                else {
                    continue;
                };
                let mut hits: Vec<String> = Vec::new();
                for bd in b {
                    let path = bd.get("path").and_then(Value::as_u64).unwrap_or(0) as u32;
                    let attr = bd.get("attribute").and_then(Value::as_u64).unwrap_or(0);
                    let tid = bd
                        .get("typeID")
                        .or_else(|| bd.get("classID"))
                        .and_then(Value::as_i64)
                        .unwrap_or(0);
                    if !hashes.contains(&path) {
                        continue;
                    }
                    hits.push(match (tid, attr) {
                        (20, _) => "ortho".into(),
                        (4, 1) => "pos".into(),
                        (4, 2) => "quat".into(),
                        (4, 4) => "euler".into(),
                        (4, 3) => "scale".into(),
                        _ => format!("t{tid}a{attr}"),
                    });
                }
                if hits.is_empty() {
                    continue;
                }
                hits.sort();
                hits.dedup();
                let l = name.to_ascii_lowercase();
                let gated = l.contains("start") || l.contains("entrance") || l.contains("enter");
                rows.push(format!(
                    "      {:<52} [{}]  {}",
                    name,
                    hits.join(","),
                    if gated {
                        "admitted"
                    } else {
                        "** DROPPED by the name gate **"
                    }
                ));
            }
            if rows.is_empty() {
                continue;
            }
            rows.sort();
            println!("== {short}");
            for r in rows {
                println!("{r}");
            }
        }
    }
}
