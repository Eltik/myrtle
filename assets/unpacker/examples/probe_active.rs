//! THROWAWAY: dump every `GameObject` with an animated `m_IsActive` window in a dynchar
//! prefab — does the entrance clip ACTIVATE the camera rig a few frames in?
//!
//! Usage: cargo run --release --example `probe_active` -- <bundle.ab> [go-substr]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::export::anim::active_windows;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};
fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("bundle");
    let want = a.next().unwrap_or_default().to_lowercase();
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
        let mut gname: HashMap<i64, String> = HashMap::new();
        let mut gtf: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid != 1 {
                continue;
            }
            gname.insert(
                *p,
                v.get("m_Name").and_then(Value::as_str).unwrap_or("").into(),
            );
            if let Some(cs) = v.get("m_Component").and_then(Value::as_array) {
                for c in cs {
                    if let Some(cp) = c
                        .get("component")
                        .and_then(pid)
                        .or_else(|| c.get("second").and_then(pid))
                        && let Some((4, _)) = all.get(&cp)
                    {
                        gtf.insert(*p, cp);
                    }
                }
            }
        }
        let w = active_windows(&all);
        println!(
            "GameObjects with an animated m_IsActive window: {}",
            w.len()
        );
        let mut rows: Vec<(String, Option<f32>, Option<f32>, i64)> = w
            .iter()
            .flat_map(|(go, windows)| {
                windows
                    .iter()
                    .map(|(f, u)| (gname.get(go).cloned().unwrap_or_default(), *f, *u, *go))
            })
            .collect();
        rows.sort_by(|a, b| a.1.unwrap_or(0.0).partial_cmp(&b.1.unwrap_or(0.0)).unwrap());
        for (n, f, u, go) in &rows {
            if !want.is_empty() && !n.to_lowercase().contains(&want) {
                continue;
            }
            // ancestor chain for context
            let mut chain = Vec::new();
            let mut t = gtf.get(go).copied();
            for _ in 0..6 {
                let Some(tt) = t else { break };
                let Some(fa) = all
                    .get(&tt)
                    .and_then(|(_, x)| x.get("m_Father").and_then(pid))
                    .filter(|p| *p != 0)
                else {
                    break;
                };
                if let Some(g) = all
                    .get(&fa)
                    .and_then(|(_, x)| x.get("m_GameObject").and_then(pid))
                {
                    chain.push(gname.get(&g).cloned().unwrap_or_default());
                }
                t = Some(fa);
            }
            println!(
                "  '{n}'  activeFrom={f:?} activeUntil={u:?}   under: {}",
                chain.join(" < ")
            );
        }
    }
}
