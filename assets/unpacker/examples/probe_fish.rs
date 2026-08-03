//! THROWAWAY diagnostic: enumerate every ParticleSystem GameObject whose name matches a
//! filter, with its ACTIVE state, parent chain and material — to tell "the exporter dropped
//! it" from "the data says it never plays".
//!
//! Usage: cargo run --release --example probe_fish -- <bundle.ab> <name-substr>
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> { v.get("m_PathID").and_then(Value::as_i64) }

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("bundle");
    let want = a.next().unwrap_or_default().to_lowercase();
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
        // GameObject name + active + its Transform
        let mut go_name: HashMap<i64, String> = HashMap::new();
        let mut go_active: HashMap<i64, i64> = HashMap::new();
        let mut go_tf: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid != 1 { continue; }
            go_name.insert(*p, v.get("m_Name").and_then(Value::as_str).unwrap_or("").into());
            go_active.insert(*p, v.get("m_IsActive").and_then(Value::as_i64).unwrap_or(-1));
            if let Some(comps) = v.get("m_Component").and_then(Value::as_array) {
                for c in comps {
                    let cp = c.get("component").and_then(pid).or_else(|| c.get("second").and_then(pid));
                    if let Some(cp) = cp && let Some((4, _)) = all.get(&cp) { go_tf.insert(*p, cp); }
                }
            }
        }
        let tf_parent = |t: i64| -> Option<i64> { all.get(&t).and_then(|(_, v)| v.get("m_Father").and_then(pid)).filter(|p| *p != 0) };
        let tf_go = |t: i64| -> Option<i64> { all.get(&t).and_then(|(_, v)| v.get("m_GameObject").and_then(pid)) };
        for (cid, v) in all.values() {
            if *cid != 198 { continue; } // ParticleSystem
            let Some(go) = v.get("m_GameObject").and_then(pid) else { continue };
            let name = go_name.get(&go).cloned().unwrap_or_default();
            if !want.is_empty() && !name.to_lowercase().contains(&want) { continue; }
            let mut chain = vec![format!("{name}[active={}]", go_active.get(&go).copied().unwrap_or(-1))];
            let mut t = go_tf.get(&go).copied();
            let mut depth = 0;
            while let Some(tt) = t.and_then(tf_parent) {
                if depth > 8 { break; }
                if let Some(g) = tf_go(tt) {
                    chain.push(format!("{}[active={}]", go_name.get(&g).cloned().unwrap_or_default(), go_active.get(&g).copied().unwrap_or(-1)));
                }
                t = Some(tt); depth += 1;
            }
            println!("PS on {}", chain.join("  <  "));
        }
    }
}
