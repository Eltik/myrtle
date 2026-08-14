//! THROWAWAY diagnostic: dump the entrance camera's POST-PROCESS stack — the `PostProcessVolume`
//! on the `pp` `GameObject`, the `pp Profile` asset it points at, and every effect-settings
//! `MonoBehaviour` the profile holds.
//!
//! Motivation: Mlynar's t=4 warm deficit and t=13 bottom deficit both reduce to "a localised
//! brightening the exported scene does not contain", and the residual is BLUR-INVARIANT — a
//! light or a colour transform, not missing art. `probe_grade` found a full `PostProcessLayer` on
//! `.../03/Dummy002/Main Camera` plus a `pp` volume and a `pp Profile`, none of which the
//! exporter reads. A colour-grading profile is exactly the mechanism that produces a warm,
//! low-frequency, region-weighted difference with no corresponding drawn layer.
//!
//! Usage: cargo run --release --example `probe_pp` -- <bundle.ab>
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

/// Walk a `GameObject`'s parent chain into a readable path.
fn go_path(all: &HashMap<i64, (i32, Value)>, go: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = Some(go);
    let mut guard = 0;
    while let Some(g) = cur {
        guard += 1;
        if guard > 64 {
            break;
        }
        let Some((_, gv)) = all.get(&g) else { break };
        parts.push(
            gv.get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string(),
        );
        // GameObject -> its Transform -> parent Transform -> that transform's GameObject
        let tr = gv
            .get("m_Component")
            .and_then(Value::as_array)
            .and_then(|cs| {
                cs.iter().find_map(|c| {
                    let p = pid(c.get("component")?)?;
                    matches!(all.get(&p), Some((4, _))).then_some(p)
                })
            });
        cur = tr
            .and_then(|t| all.get(&t))
            .and_then(|(_, tv)| pid(tv.get("m_Father")?))
            .filter(|&p| p != 0)
            .and_then(|p| all.get(&p))
            .and_then(|(_, pv)| pid(pv.get("m_GameObject")?));
    }
    parts.reverse();
    parts.join("/")
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
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

        // Every MonoBehaviour that looks like part of a post-process stack: the volume itself
        // (has `sharedProfile`), the profile (has `settings`), or an effect settings block
        // (has `enabled` + `active` and no GameObject).
        for (p, (cls, v)) in &all {
            if *cls != 114 {
                continue;
            }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            let has_profile = v.get("sharedProfile").is_some();
            let has_settings = v.get("settings").is_some();
            if !has_profile && !has_settings && !name.to_lowercase().contains("pp") {
                continue;
            }
            let go = pid(v.get("m_GameObject").unwrap_or(&Value::Null)).filter(|&g| g != 0);
            println!("\n===== MonoBehaviour pid={p} name='{name}'");
            if let Some(g) = go {
                println!("  on GO: {}", go_path(&all, g));
            }
            println!("  {}", serde_json::to_string_pretty(v).unwrap_or_default());

            // Follow `settings` (profile -> effect blocks) and dump each one.
            if let Some(list) = v.get("settings").and_then(Value::as_array) {
                for s in list {
                    let Some(sp) = pid(s) else { continue };
                    match all.get(&sp) {
                        Some((_, sv)) => println!(
                            "\n  ---- setting pid={sp}\n{}",
                            serde_json::to_string_pretty(sv).unwrap_or_default()
                        ),
                        None => println!("\n  ---- setting pid={sp}  NOT IN THIS FILE (external)"),
                    }
                }
            }
        }
    }
}
