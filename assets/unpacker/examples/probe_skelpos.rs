//! Diagnostic: the WORLD placement of the spine skeleton's GameObject in a dynchar bundle.
//!
//! The exporter normalises the skeleton root to the origin (`origin at skeleton root`), so
//! the scene JSON carries no equivalent of the game's `Skeleton at 0, -9`. This walks the
//! Transform (typeID 4) of the GameObject that owns the `skeletonDataAsset` MonoBehaviour
//! and composes `m_LocalPosition` / `m_LocalScale` up the `m_Father` chain.
//!
//! Usage: cargo run --release --example `probe_skelpos` -- <bundle.ab> [<bundle.ab>...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> i64 {
    v.get("m_PathID").and_then(Value::as_i64).unwrap_or(0)
}

fn xyz(v: Option<&Value>) -> [f64; 3] {
    let g = |k: &str| {
        v.and_then(|o| o.get(k))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    [g("x"), g("y"), g("z")]
}

fn quat(v: Option<&Value>) -> [f64; 4] {
    let g = |k: &str| {
        v.and_then(|o| o.get(k))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    [g("x"), g("y"), g("z"), g("w")]
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        let short = path.rsplit('/').next().unwrap_or(&path).to_string();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let mut gos: HashMap<i64, String> = HashMap::new();
            let mut xforms: HashMap<i64, Value> = HashMap::new(); // keyed by the transform's own path id
            let mut xform_of_go: HashMap<i64, i64> = HashMap::new();
            let mut skel_gos: Vec<i64> = Vec::new();
            for obj in &sf.objects {
                match obj.class_id {
                    1 | 4 | 114 => {}
                    _ => continue,
                }
                let Ok(v) = read_object(&sf, obj) else {
                    continue;
                };
                match obj.class_id {
                    1 => {
                        gos.insert(
                            obj.path_id,
                            v.get("m_Name")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                        );
                    }
                    4 => {
                        let go = v.get("m_GameObject").map(pid).unwrap_or(0);
                        xform_of_go.insert(go, obj.path_id);
                        xforms.insert(obj.path_id, v);
                    }
                    _ => {
                        if v.get("skeletonDataAsset").is_some() {
                            skel_gos.push(v.get("m_GameObject").map(pid).unwrap_or(0));
                        }
                    }
                }
            }
            for go in skel_gos {
                let Some(&tid) = xform_of_go.get(&go) else {
                    println!(
                        "{short}\tGO '{}'\tNO TRANSFORM",
                        gos.get(&go).cloned().unwrap_or_default()
                    );
                    continue;
                };
                // Walk to the root, collecting the chain leaf -> root.
                let mut chain: Vec<i64> = Vec::new();
                let mut cur = tid;
                while cur != 0 {
                    chain.push(cur);
                    cur = xforms
                        .get(&cur)
                        .and_then(|t| t.get("m_Father"))
                        .map(pid)
                        .unwrap_or(0);
                    if chain.len() > 64 {
                        break;
                    }
                }
                // Compose root -> leaf: world = parent_world + parent_scale * local (rotation
                // reported, assumed identity for the composition; a non-identity quaternion
                // is printed so it cannot pass silently).
                let mut world = [0.0f64; 3];
                let mut scale = [1.0f64; 3];
                let mut rot_flag = String::new();
                let mut desc: Vec<String> = Vec::new();
                for &t in chain.iter().rev() {
                    let tv = &xforms[&t];
                    let lp = xyz(tv.get("m_LocalPosition"));
                    let ls = xyz(tv.get("m_LocalScale"));
                    let q = quat(tv.get("m_LocalRotation"));
                    if (q[3] - 1.0).abs() > 1e-6 {
                        rot_flag = format!(" ROT{q:?}");
                    }
                    for i in 0..3 {
                        world[i] += scale[i] * lp[i];
                        scale[i] *= ls[i];
                    }
                    let name = gos
                        .get(&tv.get("m_GameObject").map(pid).unwrap_or(0))
                        .cloned()
                        .unwrap_or_default();
                    desc.push(format!(
                        "{name}[{:.4},{:.4},{:.4} s{:.4}]",
                        lp[0], lp[1], lp[2], ls[0]
                    ));
                }
                println!(
                    "{short}\tGO '{}'\tworld=({:.5},{:.5},{:.5}) scale=({:.6},{:.6}){rot_flag}\tchain: {}",
                    gos.get(&go).cloned().unwrap_or_default(),
                    world[0],
                    world[1],
                    world[2],
                    scale[0],
                    scale[1],
                    desc.join(" > ")
                );
            }
        }
    }
}
