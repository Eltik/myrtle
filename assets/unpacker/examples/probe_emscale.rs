//! THROWAWAY diagnostic: for a named emitter GameObject, print the RAW `startSize` from the
//! ParticleSystem module, the full transform chain with each local scale, and the accumulated
//! world scale — so the exported `startSize` can be reconciled against the prefab.
//!
//! Motivation: Wiš'adel's `stroke_01 (1)` exports `startSize` 1185.8 px, which on her framing
//! covers the ENTIRE screen and renders as a full-frame wash rather than a brush stroke. The
//! exporter multiplies the authored size by the emitter's world basis (`em_inv`) and the global
//! `inv_scale`, and this project already has two bugs of that family (the anisotropic basis, and
//! a 0.0025-scale emitter inflating particles ~400x). This says which factor produced 1185.8.
//!
//! Usage: cargo run --release --example probe_emscale -- <bundle.ab> <go-name>

use std::collections::HashMap;

use unpacker::export::mesh::Mat4;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn path_id(v: &serde_json::Value) -> Option<i64> {
    v.get("m_PathID").and_then(serde_json::Value::as_i64)
}

fn v3(v: &serde_json::Value, field: &str, d: f32) -> [f32; 3] {
    let g = |k: &str| {
        v.get(field)
            .and_then(|x| x.get(k))
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::from(d)) as f32
    };
    [g("x"), g("y"), g("z")]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let want = args.next().expect("go name").to_ascii_lowercase();

    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let mut go_name: HashMap<i64, String> = HashMap::new();
        let mut go_tf: HashMap<i64, i64> = HashMap::new();
        // transform pid -> (father, local scale, owning GO)
        let mut tf: HashMap<i64, (i64, [f32; 3], [f32; 3], [f32; 4], i64)> = HashMap::new();
        let mut systems: Vec<(i64, serde_json::Value)> = Vec::new();
        for obj in &sf.objects {
            let Ok(v) = read_object(&sf, obj) else { continue };
            match obj.class_id {
                1 => {
                    go_name.insert(obj.path_id, v["m_Name"].as_str().unwrap_or("").to_string());
                }
                4 | 224 => {
                    let go = v.get("m_GameObject").and_then(path_id).unwrap_or(0);
                    go_tf.insert(go, obj.path_id);
                    let q = {
                        let g = |k: &str, d: f32| {
                            v.get("m_LocalRotation")
                                .and_then(|x| x.get(k))
                                .and_then(serde_json::Value::as_f64)
                                .unwrap_or(f64::from(d)) as f32
                        };
                        [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
                    };
                    tf.insert(
                        obj.path_id,
                        (
                            v.get("m_Father").and_then(path_id).unwrap_or(0),
                            v3(&v, "m_LocalPosition", 0.0),
                            v3(&v, "m_LocalScale", 1.0),
                            q,
                            go,
                        ),
                    );
                }
                198 => {
                    let go = v.get("m_GameObject").and_then(path_id).unwrap_or(0);
                    systems.push((go, v));
                }
                _ => {}
            }
        }
        for (go, ps) in &systems {
            let name = go_name.get(go).cloned().unwrap_or_default();
            if !name.to_ascii_lowercase().contains(&want) {
                continue;
            }
            let raw = ps
                .get("InitialModule")
                .and_then(|m| m.get("startSize"))
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let scalar = raw.get("scalar").and_then(serde_json::Value::as_f64);
            let state = raw.get("minMaxState").and_then(serde_json::Value::as_i64);
            let scaling = ps
                .get("moveWithTransform")
                .and_then(serde_json::Value::as_i64);
            let smode = ps.get("scalingMode").and_then(serde_json::Value::as_i64);
            println!("\n=== {name}");
            println!("   raw startSize: scalar={scalar:?}  minMaxState={state:?}");
            println!("   scalingMode={smode:?}  moveWithTransform={scaling:?}");
            println!("   transform chain (leaf -> root):");
            let mut chain: Vec<i64> = Vec::new();
            let mut cur = go_tf.get(go).copied().unwrap_or(0);
            for _ in 0..64 {
                let Some(&(father, _, _, _, _)) = tf.get(&cur) else {
                    break;
                };
                chain.push(cur);
                if father == 0 {
                    break;
                }
                cur = father;
            }
            let mut acc = [1.0f32, 1.0, 1.0];
            for &t in &chain {
                let (_, _, sc, _, owner) = tf[&t];
                let nm = go_name.get(&owner).cloned().unwrap_or_default();
                println!("      {nm:38} localScale=({:.4}, {:.4}, {:.4})", sc[0], sc[1], sc[2]);
                acc[0] *= sc[0];
                acc[1] *= sc[1];
                acc[2] *= sc[2];
            }
            println!("   ACCUMULATED world scale = ({:.5}, {:.5}, {:.5})", acc[0], acc[1], acc[2]);
            // Full world matrix, root-first, and the screen-plane axis lengths the exporter uses.
            let mut m = Mat4::identity();
            for &t in chain.iter().rev() {
                let (_, p, sc, q, _) = tf[&t];
                m = m.mul(&Mat4::trs(p, q, sc));
            }
            let o = m.point([0.0, 0.0, 0.0]);
            let ex = m.point([1.0, 0.0, 0.0]);
            let ey = m.point([0.0, 1.0, 0.0]);
            let lx = ((ex[0] - o[0]).powi(2) + (ex[1] - o[1]).powi(2)).sqrt();
            let ly = ((ey[0] - o[0]).powi(2) + (ey[1] - o[1]).powi(2)).sqrt();
            println!("   world basis, SCREEN-PLANE axis lengths: |X|={lx:.5}  |Y|={ly:.5}");
            if let Some(s) = scalar {
                println!(
                    "   raw {s:.4}  x  meanAxis {:.5}  = {:.4}  (before the global inv_scale)",
                    (lx + ly) / 2.0,
                    s * f64::from((lx + ly) / 2.0)
                );
            }
        }
    }
}
