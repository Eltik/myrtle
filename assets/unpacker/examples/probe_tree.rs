//! Diagnostic: print the `GameObject` hierarchy of a bundle — name, `m_IsActive`, local +
//! accumulated position/scale, and the component class IDs on each node.
//!
//! Motivation: when a visible element of the in-game render has no counterpart in the
//! exported scene/particle JSON, the first question is whether the authored prefab even
//! contains it and under which parent — the exported JSONs are anonymous (indices only),
//! so there is no way to answer that from the output alone.
//!
//! Usage: cargo run --release --example `probe_tree` -- <bundle.ab> [name-filter]
#![allow(
    // mul_add/hypot change float rounding, not just spelling; never worth it for byte-exact
    // parity output, even in a throwaway diagnostic.
    clippy::suboptimal_flops,
    clippy::imprecise_flops,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::too_many_lines
)]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn vec3(v: Option<&Value>) -> [f64; 3] {
    let g = |k: &str| {
        v.and_then(|v| v.get(k))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    [g("x"), g("y"), g("z")]
}

const fn class_name(id: i32) -> &'static str {
    match id {
        1 => "GameObject",
        4 => "Transform",
        23 => "MeshRenderer",
        33 => "MeshFilter",
        114 => "MonoBehaviour",
        198 => "ParticleSystem",
        199 => "ParticleSystemRenderer",
        212 => "SpriteRenderer",
        224 => "RectTransform",
        _ => "?",
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
    let filter = std::env::args().nth(2).unwrap_or_default().to_lowercase();
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

        // Transform path_id -> (GameObject path_id, parent transform path_id).
        let mut children: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut roots: Vec<i64> = Vec::new();
        for (tp, (cid, v)) in &all {
            if *cid != 4 && *cid != 224 {
                continue;
            }
            match v.get("m_Father").and_then(pid) {
                Some(f) if f != 0 && all.contains_key(&f) => {
                    children.entry(f).or_default().push(*tp);
                }
                _ => roots.push(*tp),
            }
        }
        for v in children.values_mut() {
            v.sort_unstable();
        }
        roots.sort_unstable();

        // GameObject path_id -> component class IDs.
        let mut comps: HashMap<i64, Vec<i32>> = HashMap::new();
        for (gp, (cid, v)) in &all {
            if *cid != 1 {
                continue;
            }
            let list = v
                .get("m_Component")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|c| {
                            let p = pid(c.get("component").unwrap_or(c))?;
                            all.get(&p).map(|(c, _)| *c)
                        })
                        .collect()
                })
                .unwrap_or_default();
            comps.insert(*gp, list);
        }

        println!("=== {} ({} objects) ===", entry.path, all.len());
        let mut stack: Vec<(i64, usize, [f64; 3])> =
            roots.iter().rev().map(|t| (*t, 0usize, [0.0; 3])).collect();
        while let Some((tp, depth, acc)) = stack.pop() {
            let Some((_, tv)) = all.get(&tp) else {
                continue;
            };
            let lp = vec3(tv.get("m_LocalPosition"));
            let world = [acc[0] + lp[0], acc[1] + lp[1], acc[2] + lp[2]];
            let gp = tv.get("m_GameObject").and_then(pid).unwrap_or(0);
            let (name, active) = all.get(&gp).map_or((String::new(), -1), |(_, g)| {
                (
                    g.get("m_Name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    g.get("m_IsActive").and_then(Value::as_i64).unwrap_or(-1),
                )
            });
            let cs: Vec<&str> = comps
                .get(&gp)
                .map(|v| v.iter().map(|c| class_name(*c)).collect())
                .unwrap_or_default();
            let ls = vec3(tv.get("m_LocalScale"));
            let q = tv.get("m_LocalRotation");
            let g = |k: &str| {
                q.and_then(|v| v.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
            };
            let (qx, qy, qz, qw) = (g("x"), g("y"), g("z"), g("w"));
            // Z euler from the quaternion (these rigs are planar, so this is the whole rotation).
            let zdeg = (2.0 * (qw * qz + qx * qy))
                .atan2(1.0 - 2.0 * (qy * qy + qz * qz))
                .to_degrees();
            if filter.is_empty() || name.to_lowercase().contains(&filter) {
                println!(
                    "{:indent$}{name}  act={active} world=({:.2},{:.2},{:.2}) local=({:.2},{:.2},{:.2}) scale=({:.3},{:.3},{:.3}) rotZ={zdeg:.2} [{}] t={tp} go={gp}",
                    "",
                    world[0],
                    world[1],
                    world[2],
                    lp[0],
                    lp[1],
                    lp[2],
                    ls[0],
                    ls[1],
                    ls[2],
                    cs.join(","),
                    indent = depth * 2
                );
            }
            for c in children.get(&tp).into_iter().flatten().rev() {
                stack.push((*c, depth + 1, world));
            }
        }
    }
}
