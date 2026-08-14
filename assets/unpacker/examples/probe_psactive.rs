//! THROWAWAY diagnostic: for every `ParticleSystem`, print its ROOT and its `GameObject`'s
//! serialized `m_IsActive`, plus whether any ancestor ships inactive.
//!
//! Motivation: cross-root admission is closed on the evidence that the entrance DIRECTOR's
//! `_effects` list is 142/142 entrance-root — but that list governs ACTIVATION of rigs that ship
//! `m_IsActive = 0`. A rig on the IDLE root that ships ACTIVE needs no director entry and would
//! simply be running from scene load, including during the entrance. That is a different question,
//! and it is the only single mechanism left that could give Skadi's crown shoal both ~1.7x the
//! count and larger apparent blobs (two overlapping rings merging).
//!
//! Usage: cargo run --release --example `probe_psactive` -- <bundle.ab>
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::for_kv_map,
    clippy::too_many_lines,
    clippy::useless_let_if_seq
)]

use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
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
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        let mut go_tr: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid != 1 {
                continue;
            }
            if let Some(cs) = v.get("m_Component").and_then(Value::as_array) {
                for c in cs {
                    if let Some(cp) = c.get("component").and_then(pid)
                        && matches!(all.get(&cp), Some((4, _)))
                    {
                        go_tr.insert(*p, cp);
                    }
                }
            }
        }
        let parent_go = |go: i64| -> Option<i64> {
            let tr = *go_tr.get(&go)?;
            let f = pid(all.get(&tr)?.1.get("m_Father")?)?;
            (f != 0)
                .then(|| pid(all.get(&f)?.1.get("m_GameObject")?))
                .flatten()
        };
        let active_of = |go: i64| -> i64 {
            all.get(&go)
                .and_then(|(_, v)| v.get("m_IsActive"))
                .map_or(-1, |x| {
                    x.as_bool()
                        .map(i64::from)
                        .or_else(|| x.as_i64())
                        .unwrap_or(-1)
                })
        };
        let name_of = |p: i64| -> String {
            all.get(&p)
                .and_then(|(_, v)| v.get("m_Name").and_then(Value::as_str))
                .unwrap_or("?")
                .to_string()
        };
        let mut rows: Vec<(String, String, i64, i64)> = Vec::new();
        for (_p, (cid, v)) in &all {
            if *cid != 198 {
                continue;
            } // ParticleSystem
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            // walk to root, tracking whether ANY ancestor (incl. self) ships inactive
            let mut r = go;
            let mut any_inactive = 0i64;
            if active_of(go) == 0 {
                any_inactive = 1;
            }
            for _ in 0..64 {
                match parent_go(r) {
                    Some(p2) => {
                        r = p2;
                        if active_of(r) == 0 {
                            any_inactive = 1;
                        }
                    }
                    None => break,
                }
            }
            rows.push((name_of(go), name_of(r), active_of(go), any_inactive));
        }
        rows.sort();
        let mut idle_active = 0;
        let mut idle_inactive = 0;
        let mut ent = 0;
        for (n, root, act, chain) in &rows {
            let is_ent = root.starts_with("dyn_entrance");
            if is_ent {
                ent += 1;
            } else if *chain == 0 {
                idle_active += 1;
            } else {
                idle_inactive += 1;
            }
            if n.contains("fish") || n.contains("xiaoyu") {
                println!("  {n:<26} root={root:<44} m_IsActive={act} anyAncestorInactive={chain}");
            }
        }
        if !rows.is_empty() {
            println!(
                "\n  {} particle systems: {} entrance-root, IDLE-root {} fully ACTIVE / {} with an inactive ancestor",
                rows.len(),
                ent,
                idle_active,
                idle_inactive
            );
        }
    }
}
