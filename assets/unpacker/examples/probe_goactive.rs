//! THROWAWAY diagnostic: print the STATIC `m_IsActive` of named GameObjects and their whole
//! ancestor chain.
//!
//! Motivation: Virtuosa's two biggest prewarm-sensitive emitters (`rainbow_large_01`,
//! `spark_small_01`) hang under a parent named `largeOnly`, which reads like a view variant
//! that should be off in this composition — and a 1360 px additive flare rendering when the
//! game hides it would swamp any phase argument. `probe_active` only reports ANIMATED
//! `m_IsActive` windows; this reports the static flag, which is what group gating uses.
//!
//! Usage: cargo run --release --example probe_goactive -- <bundle.ab> <go-substr>...

use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("usage: probe_goactive <bundle.ab> <go-substr>...");
    let want: Vec<String> = a.map(|s| s.to_lowercase()).collect();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for o in &sf.objects {
            if let Ok(v) = read_object(&sf, o) {
                all.insert(o.path_id, (o.class_id, v));
            }
        }
        // Transform pathID -> its GameObject pathID, so a parent Transform can be named.
        let mut tf_go: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid == 4 && let Some(g) = v.get("m_GameObject").and_then(pid) {
                tf_go.insert(*p, g);
            }
        }
        let name = |g: i64| -> String {
            all.get(&g)
                .and_then(|(_, v)| v.get("m_Name").and_then(Value::as_str))
                .unwrap_or("?")
                .to_string()
        };
        let active = |g: i64| -> i64 {
            all.get(&g)
                .and_then(|(_, v)| v.get("m_IsActive"))
                .map(|x| x.as_bool().map(i64::from).or_else(|| x.as_i64()).unwrap_or(-1))
                .unwrap_or(-1)
        };
        for (p, (cid, v)) in &all {
            if *cid != 1 {
                continue;
            }
            let n = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            if !want.iter().any(|w| n.to_lowercase().contains(w)) {
                continue;
            }
            // Walk up via the GameObject's Transform component.
            let mut chain: Vec<String> = Vec::new();
            let tf = v
                .get("m_Component")
                .and_then(Value::as_array)
                .and_then(|cs| {
                    cs.iter().find_map(|c| {
                        let q = c.get("component").unwrap_or(c);
                        let id = pid(q)?;
                        matches!(all.get(&id), Some((4, _))).then_some(id)
                    })
                });
            let mut cur = tf;
            while let Some(t) = cur {
                let Some((_, tv)) = all.get(&t) else { break };
                let parent = tv.get("m_Father").and_then(pid).filter(|&x| x != 0);
                let Some(pt) = parent else { break };
                let Some(&pg) = tf_go.get(&pt) else { break };
                chain.push(format!("{}[active={}]", name(pg), active(pg)));
                cur = Some(pt);
            }
            println!(
                "GO '{n}' (pathID {p})  m_IsActive={}  m_Layer={:?}  m_Tag={:?}",
                active(*p),
                v.get("m_Layer"),
                v.get("m_Tag")
            );
            println!("   ancestors: {}", chain.join(" < "));
        }
    }
}
