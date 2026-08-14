//! THROWAWAY diagnostic: is Executor's scope RIM quad animated, and by what?
//!
//! Motivation: her scope aperture is right while the camera is still (game width / camera scale
//! is a constant 442 for the first 2.8 s) but wrong once it moves — the ratio swings 295..521 and
//! the width jumps 305 -> 432 in 0.2 s, which is a discrete change rather than a scaling
//! relationship. Her rim exports as a STATIC 4-vertex quad with only an alpha curve, so if the
//! game animates its geometry, that animation lives in the bundle and never reaches the exporter
//! (scene layers carry a single `pos`).
//!
//! This dumps the three things needed to settle it:
//!   1. every `AnimationClip` and the object PATHS its position/scale/float curves bind,
//!   2. every mesh-drawing `GameObject` and its full hierarchy path,
//!   3. which of those paths a curve actually targets.
//!
//! Usage: cargo run --release --example `probe_rimanim` -- <bundle.ab> [name-filter]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

/// Full "a/b/c" hierarchy path of a `GameObject`, walking Transform parents.
fn go_path(all: &HashMap<i64, (i32, Value)>, go_pid: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = Some(go_pid);
    let mut guard = 0;
    while let Some(pid) = cur {
        guard += 1;
        if guard > 64 {
            break;
        }
        let Some((_, gv)) = all.get(&pid) else { break };
        parts.push(
            gv.get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string(),
        );
        // GameObject -> its Transform -> parent Transform -> that transform's GameObject
        let tf = gv
            .get("m_Component")
            .and_then(Value::as_array)
            .and_then(|comps| {
                comps.iter().find_map(|c| {
                    let p = c.get("component")?.get("m_PathID")?.as_i64()?;
                    let (cid, _) = all.get(&p)?;
                    if *cid == 4 { Some(p) } else { None }
                })
            });
        cur = tf
            .and_then(|t| all.get(&t))
            .and_then(|(_, tv)| tv.get("m_Father"))
            .and_then(|f| f.get("m_PathID"))
            .and_then(Value::as_i64)
            .filter(|p| *p != 0)
            .and_then(|p| all.get(&p))
            .and_then(|(_, pv)| pv.get("m_GameObject"))
            .and_then(|g| g.get("m_PathID"))
            .and_then(Value::as_i64);
    }
    parts.reverse();
    parts.join("/")
}

fn collect_paths(v: &Value, key: &str, out: &mut Vec<String>) {
    if let Some(arr) = v.get(key).and_then(Value::as_array) {
        for c in arr {
            if let Some(p) = c.get("path").and_then(Value::as_str) {
                let attr = c.get("attribute").and_then(Value::as_str).unwrap_or("");
                out.push(if attr.is_empty() {
                    p.to_string()
                } else {
                    format!("{p}  [{attr}]")
                });
            }
        }
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe_rimanim <bundle.ab> [filter]");
    let filter = std::env::args()
        .nth(2)
        .unwrap_or_default()
        .to_ascii_lowercase();
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

        // --- 1. AnimationClips and what they bind -------------------------------------------
        let mut clips: Vec<(i64, &Value)> = all
            .iter()
            .filter(|(_, (cid, _))| *cid == 74)
            .map(|(p, (_, v))| (*p, v))
            .collect();
        clips.sort_unstable_by_key(|(p, _)| *p);
        println!("== {} AnimationClip(s)", clips.len());
        for (_pid, v) in &clips {
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            let mut paths = Vec::new();
            for k in [
                "m_PositionCurves",
                "m_ScaleCurves",
                "m_RotationCurves",
                "m_EulerCurves",
                "m_FloatCurves",
                "m_PPtrCurves",
            ] {
                collect_paths(v, k, &mut paths);
            }
            paths.sort();
            paths.dedup();
            let shown: Vec<&String> = paths
                .iter()
                .filter(|p| filter.is_empty() || p.to_ascii_lowercase().contains(&filter))
                .collect();
            println!(
                "   [{name}]  {} bound path(s){}",
                paths.len(),
                if filter.is_empty() {
                    ""
                } else {
                    ", matching filter:"
                }
            );
            for p in shown.iter().take(40) {
                println!("        {p}");
            }
        }

        // --- 2. mesh-drawing GameObjects, with their hierarchy paths -------------------------
        println!("\n== mesh-drawing GameObjects");
        let mut rows: Vec<String> = Vec::new();
        for (cid, v) in all.values() {
            if *cid != 23 && *cid != 33 {
                continue; // MeshRenderer / MeshFilter
            }
            let Some(go) = v
                .get("m_GameObject")
                .and_then(|g| g.get("m_PathID"))
                .and_then(Value::as_i64)
            else {
                continue;
            };
            let p = go_path(&all, go);
            if !filter.is_empty() && !p.to_ascii_lowercase().contains(&filter) {
                continue;
            }
            rows.push(p);
        }
        rows.sort();
        rows.dedup();
        for r in rows.iter().take(60) {
            println!("   {r}");
        }
    }
}
