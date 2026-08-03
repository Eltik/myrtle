//! THROWAWAY corpus scan: for every dynchar bundle, list statically-inactive rigs under the
//! entrance root that are NOT in the entrance director's `_effects` activation list.
//!
//! Motivation: entrance rigs ship `m_IsActive = 0` and the director switches them on at runtime,
//! so the prefab flag alone cannot gate them (honouring it deletes whole cinematics). The
//! director's `_effects` array IS the activation set — on cello all 13 inactive rigs are in it.
//! A rig that is inactive AND absent from that list is content the game never switches on, yet
//! the exporter admits it today. This counts them, and — per the standing rule that a corpus scan
//! must count CONSEQUENCES, not layers — also reports how many ParticleSystems and MeshRenderers
//! actually live under each such rig.
//!
//! Usage: cargo run --release --example scan_effects -- <dir-of-bundles>...

use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn scan(path: &std::path::Path) {
    let Ok(data) = std::fs::read(path) else { return };
    let Ok(bundle) = BundleFile::parse(data) else { return };
    let skin = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();

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
        // descendants of a GO, via the transform tree
        let children_of = {
            let mut m: HashMap<i64, Vec<i64>> = HashMap::new();
            for (g, _) in go_tr.iter() {
                if let Some(p) = parent_go(*g) {
                    m.entry(p).or_default().push(*g);
                }
            }
            m
        };
        let count_renderers = |root: i64| -> (usize, usize) {
            let (mut ps, mut mr) = (0usize, 0usize);
            let mut stack = vec![root];
            let mut guard = 0;
            while let Some(g) = stack.pop() {
                guard += 1;
                if guard > 20000 {
                    break;
                }
                if let Some((1, gv)) = all.get(&g)
                    && let Some(cs) = gv.get("m_Component").and_then(Value::as_array)
                {
                    for c in cs {
                        if let Some(cp) = c.get("component").and_then(pid) {
                            match all.get(&cp) {
                                Some((198, _)) => ps += 1,
                                Some((23, _)) => mr += 1,
                                _ => {}
                            }
                        }
                    }
                }
                if let Some(kids) = children_of.get(&g) {
                    stack.extend(kids);
                }
            }
            (ps, mr)
        };

        let Some((_, dir)) = all.values().find(|(cid, v)| *cid == 114 && v.get("_mainCamera").is_some()) else {
            continue;
        };
        let members: Vec<i64> = dir
            .get("_effects")
            .and_then(Value::as_array)
            .map(|l| l.iter().filter_map(|e| pid(e).or_else(|| e.get("m_GameObject").and_then(pid))).filter(|&g| g != 0).collect())
            .unwrap_or_default();

        let mut listed = 0usize;
        let mut orphans: Vec<(String, usize, usize)> = Vec::new();
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
            let par_name = parent_go(*gp).map(name_of).unwrap_or_default();
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
            if inlist {
                listed += 1;
            } else {
                let (ps, mr) = count_renderers(*gp);
                orphans.push((name_of(*gp), ps, mr));
            }
        }
        if listed + orphans.len() == 0 {
            continue;
        }
        orphans.sort();
        let tot_ps: usize = orphans.iter().map(|o| o.1).sum();
        let tot_mr: usize = orphans.iter().map(|o| o.2).sum();
        println!(
            "{:<44} effects={:<3} inactive={:<3} listed={:<3} NOT-LISTED={:<3} (ps={} mr={})",
            skin,
            members.len(),
            listed + orphans.len(),
            listed,
            orphans.len(),
            tot_ps,
            tot_mr
        );
        for (n, ps, mr) in orphans.iter().filter(|o| o.1 + o.2 > 0) {
            println!("      NOT-LISTED  {n:<52} ps={ps} mr={mr}");
        }
    }
}

fn main() {
    for dir in std::env::args().skip(1) {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        let mut files: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "ab")).collect();
        files.sort();
        eprintln!("scanning {} bundles in {dir}", files.len());
        for f in files {
            scan(&f);
        }
    }
}
