//! THROWAWAY diagnostic: resolve the entrance DIRECTOR's `_effects` list to GameObject NAMES,
//! and check every statically-inactive entrance rig for membership.
//!
//! Motivation: entrance rigs ship `m_IsActive = 0` and something activates them at runtime, so
//! the prefab flag alone cannot say which ones actually draw (honouring it deletes the whole
//! cinematic — cello goes from 58 particle systems to 0). The director MonoBehaviour that owns
//! `_mainCamera` / `_params.duration` also carries an `_effects` array of GameObject refs. If
//! that list is the activation set, then a rig ABSENT from it is one the game never switches on
//! — which is exactly the shape needed to explain Virtuosa's ten `..._Wing_*_start(Clone)`
//! particle rigs painting a pink glow the game does not show.
//!
//! Usage: cargo run --release --example probe_effects -- <bundle.ab>

use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

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
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        let name_of = |p: i64| -> String {
            all.get(&p)
                .and_then(|(_, v)| v.get("m_Name").and_then(Value::as_str))
                .unwrap_or("?")
                .to_string()
        };
        // GameObject -> its Transform, and Transform -> parent Transform, for ancestor walks.
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
            (f != 0).then(|| pid(all.get(&f)?.1.get("m_GameObject")?)).flatten()
        };

        for (p, (cid, v)) in &all {
            if *cid != 114 || v.get("_mainCamera").is_none() {
                continue;
            }
            println!("DIRECTOR pathID={p}");
            let Some(list) = v.get("_effects").and_then(Value::as_array) else { continue };
            println!("  _effects: {} entries", list.len());
            let mut members: Vec<i64> = Vec::new();
            for e in list {
                // Each entry is a GameObject PPtr (or an inline object carrying one).
                let gp = pid(e).or_else(|| e.get("m_GameObject").and_then(pid));
                match gp {
                    Some(g) if g != 0 => {
                        members.push(g);
                        let act = all
                            .get(&g)
                            .and_then(|(_, gv)| gv.get("m_IsActive"))
                            .map(|x| x.as_bool().map(i64::from).or_else(|| x.as_i64()).unwrap_or(-1))
                            .unwrap_or(-1);
                        println!("    '{}'  pathID={g}  m_IsActive={act}", name_of(g));
                    }
                    _ => println!("    (unresolved: {})", serde_json::to_string(e).unwrap_or_default()),
                }
            }

            // Every statically-inactive GameObject directly under the entrance root, and whether
            // it (or an ancestor) is in the _effects list.
            println!("\n  entrance-root rigs with m_IsActive=0, and their _effects membership:");
            let mut rows: Vec<(String, bool)> = Vec::new();
            for (gp, (gcid, gv)) in &all {
                if *gcid != 1 {
                    continue;
                }
                let act = gv
                    .get("m_IsActive")
                    .map(|x| x.as_bool().unwrap_or_else(|| x.as_i64().unwrap_or(1) != 0))
                    .unwrap_or(true);
                if act {
                    continue;
                }
                // only rigs whose parent is the entrance root
                let par = parent_go(*gp);
                let par_name = par.map(name_of).unwrap_or_default();
                if !par_name.starts_with("dyn_entrance") {
                    continue;
                }
                let mut cur = Some(*gp);
                let mut inlist = false;
                for _ in 0..32 {
                    match cur {
                        Some(c) if members.contains(&c) => {
                            inlist = true;
                            break;
                        }
                        Some(c) => cur = parent_go(c),
                        None => break,
                    }
                }
                rows.push((name_of(*gp), inlist));
            }
            rows.sort();
            for (n, inlist) in &rows {
                println!("    {:<58} in _effects: {}", n, if *inlist { "YES" } else { "no" });
            }
            println!("\n  {} inactive entrance rigs, {} in the list", rows.len(), rows.iter().filter(|r| r.1).count());
        }
    }
}
