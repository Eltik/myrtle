//! What ACTIVE WINDOWS does the exporter's reveal timeline derive for a bundle?
//!
//! `spine.rs` gates every entrance scene layer on `anim::active_windows`, which admits a clip
//! only when `is_entrance_clip` name-matches "start" | "entrance" | "enter". Three entrance
//! bundles name their cinematic clip something else (Eyjafjalla `..._camera_01`, Ch'en the
//! Holungday `Take 002`, Whislash-alter `03`) — exactly the trap `camera_motion_clips` already
//! documents for the CAMERA. This prints the derived windows so the gap is visible per skin.
//!
//! Usage: cargo run --release --example `probe_windows` -- <bundle.ab> [name-filter]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::anim::active_windows;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn go_path(all: &HashMap<i64, (i32, Value)>, go_pid: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = Some(go_pid);
    for _ in 0..64 {
        let Some(pid) = cur else { break };
        let Some((_, gv)) = all.get(&pid) else { break };
        parts.push(
            gv.get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string(),
        );
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

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe_windows <bundle.ab> [name-filter]");
    let filter = std::env::args()
        .nth(2)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let l = entry.path.to_ascii_lowercase();
        if l.ends_with(".ress") || l.ends_with(".resource") {
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
        let clips: Vec<String> = all
            .values()
            .filter(|(cid, _)| *cid == 74)
            .filter_map(|(_, v)| v.get("m_Name").and_then(Value::as_str).map(str::to_string))
            .collect();
        if clips.is_empty() {
            continue;
        }
        println!("== {} : {} clip(s)", entry.path, clips.len());
        for c in &clips {
            let admitted = {
                let n = c.to_ascii_lowercase();
                n.contains("start") || n.contains("entrance") || n.contains("enter")
            };
            println!("   clip {:<40} name-admitted={admitted}", format!("[{c}]"));
        }
        let w = active_windows(&all);
        println!("   -> active_windows derived {} GO window-list(s)", w.len());
        let mut rows: Vec<(String, String)> = w
            .iter()
            .map(|(go, list)| {
                let p = go_path(&all, *go);
                let s = list
                    .iter()
                    .map(|(a, b)| format!("[{a:?}, {b:?})"))
                    .collect::<Vec<_>>()
                    .join(" ");
                (p, s)
            })
            .filter(|(p, _)| filter.is_empty() || p.to_ascii_lowercase().contains(&filter))
            .collect();
        rows.sort();
        for (p, s) in rows {
            println!("      {p:<62} {s}");
        }
    }
}
