//! THROWAWAY diagnostic: which entrance transforms carry a POSITION curve but NO scale curve?
//!
//! Motivation: `entrance_transform_curves` documents "Only SCALE-animated transforms are returned
//! (a varying scale is the driver of an effect host's scale-in; a pure position curve is the camera
//! dolly, which owns no particles)". That was written for the PARTICLE consumer. Scene layers now
//! read the same map (they got `scaleCurve` from it), so any scene object animated purely by
//! POSITION is invisible to the exporter and exports as a frozen static quad.
//!
//! Prints, per bundle, every entrance-clip transform binding grouped by path: whether it has a
//! position curve, a scale curve, or both, and the GameObject the path resolves to.
//!
//! Usage: cargo run --release --example probe_poscurves -- <bundle.ab>...
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn crc32(s: &str) -> u32 {
    let mut c: u32 = 0xFFFF_FFFF;
    for b in s.as_bytes() {
        c ^= u32::from(*b);
        for _ in 0..8 { c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 }; }
    }
    !c
}
fn pid(v: &Value, k: &str) -> Option<i64> { v.get(k)?.get("m_PathID")?.as_i64().filter(|p| *p != 0) }

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        let short = path.rsplit('/').next().unwrap_or(&path).trim_end_matches(".ab").to_string();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") { continue; }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for o in &sf.objects {
                if let Ok(v) = read_object(&sf, o) { all.insert(o.path_id, (o.class_id, v)); }
            }
            // subpath hash -> GameObject name, for every GO in the file
            let mut go_parent: HashMap<i64, i64> = HashMap::new();
            let mut tf_go: HashMap<i64, i64> = HashMap::new();
            let mut go_tf: HashMap<i64, i64> = HashMap::new();
            for (p, (cid, v)) in &all {
                if matches!(cid, 4 | 224) {
                    if let Some(g) = pid(v, "m_GameObject") { tf_go.insert(*p, g); go_tf.insert(g, *p); }
                }
            }
            for (p, (cid, v)) in &all {
                if matches!(cid, 4 | 224)
                    && let (Some(g), Some(f)) = (pid(v, "m_GameObject"), pid(v, "m_Father"))
                    && let Some(pg) = tf_go.get(&f)
                { go_parent.insert(g, *pg); let _ = p; }
            }
            let name = |g: i64| all.get(&g).and_then(|(_, v)| v.get("m_Name")).and_then(Value::as_str).unwrap_or("?").to_string();
            let mut hash_name: HashMap<u32, String> = HashMap::new();
            for g in go_tf.keys() {
                let mut parts = vec![name(*g)];
                let mut cur = *g;
                for _ in 0..32 {
                    let Some(p) = go_parent.get(&cur) else { break };
                    parts.push(name(*p));
                    cur = *p;
                }
                for i in 0..parts.len() {
                    let sub: Vec<String> = parts[..=i].iter().rev().cloned().collect();
                    hash_name.entry(crc32(&sub.join("/"))).or_insert_with(|| parts[0].clone());
                }
            }
            // renderers, so a path can be reported as "owns a mesh" (a scene layer) or not
            let mut has_renderer: HashSet<i64> = HashSet::new();
            for (_, (cid, v)) in &all {
                if *cid == 23 && let Some(g) = pid(v, "m_GameObject") { has_renderer.insert(g); }
            }
            let mut rows: Vec<String> = Vec::new();
            for (_, (cid, v)) in &all {
                if *cid != 74 { continue; }
                let cname = v.get("m_Name").and_then(Value::as_str).unwrap_or("").to_ascii_lowercase();
                if !(cname.contains("start") || cname.contains("entrance") || cname.contains("enter")) { continue; }
                let Some(b) = v.get("m_ClipBindingConstant").and_then(|c| c.get("genericBindings")).and_then(Value::as_array) else { continue };
                let mut per: HashMap<u32, (bool, bool)> = HashMap::new();
                for bd in b {
                    let p = bd.get("path").and_then(Value::as_u64).unwrap_or(0) as u32;
                    let a = bd.get("attribute").and_then(Value::as_u64).unwrap_or(0);
                    let t = bd.get("typeID").or_else(|| bd.get("classID")).and_then(Value::as_i64).unwrap_or(0);
                    if t != 4 { continue; }
                    let e = per.entry(p).or_insert((false, false));
                    if a == 1 { e.0 = true; }
                    if a == 3 { e.1 = true; }
                }
                for (p, (has_pos, has_scale)) in per {
                    if has_pos && !has_scale {
                        let n = hash_name.get(&p).cloned().unwrap_or_else(|| format!("<unresolved {p:#x}>"));
                        let owns = go_tf.keys().any(|g| name(*g) == n && has_renderer.contains(g));
                        rows.push(format!("      {:38} pos-only{}", n, if owns { "   ** OWNS A RENDERER **" } else { "" }));
                    }
                }
            }
            if rows.is_empty() { continue; }
            rows.sort(); rows.dedup();
            println!("== {short}   ({} pos-only transform(s))", rows.len());
            for r in rows { println!("{r}"); }
        }
    }
}
