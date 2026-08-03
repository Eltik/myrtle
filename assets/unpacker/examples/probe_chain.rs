//! THROWAWAY: which AnimationClip(s) carry TRANSFORM curves for each GameObject in a named
//! chain? If the camera's parent chain is driven by a clip OTHER than the camera clip, two
//! clocks get mixed into one baked curve.
//!
//! Usage: cargo run --release --example probe_chain -- <bundle.ab> <go1,go2,...>
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};
fn pid(v: &Value) -> Option<i64> { v.get("m_PathID").and_then(Value::as_i64) }
fn crc32(b: &[u8]) -> u32 {
    let mut c: u32 = !0;
    for &x in b {
        c ^= u32::from(x);
        for _ in 0..8 { c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 }; }
    }
    !c
}
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("bundle");
    let wants: HashSet<String> = a.next().unwrap_or_default().split(',').map(|s| s.to_lowercase()).collect();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") { continue; }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if skip.contains(&obj.class_id) { continue; }
            if let Ok(v) = read_object(&sf, obj) { all.insert(obj.path_id, (obj.class_id, v)); }
        }
        let mut gname: HashMap<i64, String> = HashMap::new();
        let mut gtf: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid != 1 { continue; }
            gname.insert(*p, v.get("m_Name").and_then(Value::as_str).unwrap_or("").into());
            if let Some(cs) = v.get("m_Component").and_then(Value::as_array) {
                for c in cs {
                    if let Some(cp) = c.get("component").and_then(pid).or_else(|| c.get("second").and_then(pid))
                        && let Some((4, _)) = all.get(&cp) { gtf.insert(*p, cp); }
                }
            }
        }
        let tf_parent = |t: i64| all.get(&t).and_then(|(_, v)| v.get("m_Father").and_then(pid)).filter(|p| *p != 0);
        let tf_go = |t: i64| all.get(&t).and_then(|(_, v)| v.get("m_GameObject").and_then(pid));
        // hash -> GO, for every RELATIVE path (GO under each ancestor level)
        let mut hash_go: HashMap<u32, Vec<i64>> = HashMap::new();
        for (&go, &tf) in &gtf {
            let mut segs = vec![gname.get(&go).cloned().unwrap_or_default()];
            let mut cur = tf;
            for _ in 0..10 {
                let rel = segs.iter().rev().cloned().collect::<Vec<_>>().join("/");
                hash_go.entry(crc32(rel.as_bytes())).or_default().push(go);
                let Some(p) = tf_parent(cur) else { break };
                let Some(pg) = tf_go(p) else { break };
                segs.push(gname.get(&pg).cloned().unwrap_or_default());
                cur = p;
            }
        }
        println!("{:<52} {:>10} {:>12}   targets in chain", "clip", "transforms", "maxKeyTime");
        for (cid, v) in all.values() {
            if *cid != 74 { continue; }
            let cname = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            let Some(bs) = v.get("m_ClipBindingConstant").and_then(|b| b.get("genericBindings")).and_then(Value::as_array) else { continue };
            let mut hits: Vec<String> = Vec::new();
            let mut n_tf = 0;
            for b in bs {
                let ty = b.get("typeID").and_then(Value::as_i64).unwrap_or(-1);
                if ty != 4 { continue; }
                n_tf += 1;
                let h = b.get("path").and_then(Value::as_u64).unwrap_or(0) as u32;
                if let Some(gos) = hash_go.get(&h) {
                    for g in gos {
                        let n = gname.get(g).cloned().unwrap_or_default();
                        if wants.contains(&n.to_lowercase()) && !hits.contains(&n) { hits.push(n); }
                    }
                }
            }
            let stop = v.get("m_MuscleClip").and_then(|m| m.get("m_StopTime")).and_then(Value::as_f64).unwrap_or(f64::NAN);
            if !hits.is_empty() {
                println!("{cname:<52} {n_tf:>10} {stop:>12.4}   {hits:?}");
            }
        }
    }
}
