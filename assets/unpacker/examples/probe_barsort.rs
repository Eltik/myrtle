//! THROWAWAY diagnostic: print the `m_SortingOrder` of every MeshRenderer in a bundle, keyed by
//! its GameObject name.
//!
//! Motivation: the entrance LETTERBOX is detected geometrically (`find_letterbox`) and exported
//! as a bare rect, so the renderer draws it as four synthetic quads ON TOP of everything. That is
//! wrong at the end of Civilight Eterna's cinematic: her white transition plane (scene layer sort
//! 100) paints OVER the bars in the game, and our bars hide it, leaving the pillarbox black
//! through a fade the game takes to 0.99 alpha. The bars have a real sorting order like every
//! other quad; this prints it so it can be exported and honoured.
//!
//! Usage: cargo run --release --example probe_barsort -- <bundle.ab> [name-substring]

use std::collections::HashMap;

use unpacker::export::mesh::Mat4;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn path_id(v: &serde_json::Value) -> Option<i64> {
    v.get("m_PathID").and_then(serde_json::Value::as_i64)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let want = args.next().unwrap_or_default().to_ascii_lowercase();

    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        // GameObject pathID -> name, active
        let mut go: HashMap<i64, (String, bool)> = HashMap::new();
        let mut rends: Vec<(i64, i64, i64, i32)> = Vec::new(); // (go, sortingOrder, sortingLayerID, class)
        // GameObject -> Transform, Transform -> (father, local TRS)
        let mut go_tf: HashMap<i64, i64> = HashMap::new();
        let mut tf: HashMap<i64, (i64, Mat4)> = HashMap::new();
        for obj in &sf.objects {
            let Ok(v) = read_object(&sf, obj) else { continue };
            if matches!(obj.class_id, 4 | 224) {
                if let Some(g) = v.get("m_GameObject").and_then(path_id) {
                    go_tf.insert(g, obj.path_id);
                }
                let vec3 = |field: &str, d: f32| {
                    let g = |k: &str| {
                        v.get(field)
                            .and_then(|x| x.get(k))
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(f64::from(d)) as f32
                    };
                    [g("x"), g("y"), g("z")]
                };
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
                        Mat4::trs(vec3("m_LocalPosition", 0.0), q, vec3("m_LocalScale", 1.0)),
                    ),
                );
            }
            match obj.class_id {
                1 => {
                    go.insert(
                        obj.path_id,
                        (
                            v["m_Name"].as_str().unwrap_or("").to_string(),
                            v.get("m_IsActive")
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(true),
                        ),
                    );
                }
                // 23 MeshRenderer, 137 SkinnedMeshRenderer, 199 ParticleSystemRenderer
                23 | 137 | 199 => {
                    let Some(g) = v.get("m_GameObject").and_then(path_id) else {
                        continue;
                    };
                    rends.push((
                        g,
                        v.get("m_SortingOrder")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0),
                        v.get("m_SortingLayerID")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0),
                        obj.class_id,
                    ));
                }
                _ => {}
            }
        }
        rends.sort_by_key(|r| r.1);
        for (g, order, layer, class) in rends {
            let (name, active) = go
                .get(&g)
                .cloned()
                .unwrap_or_else(|| ("?".into(), true));
            if !want.is_empty() && !name.to_ascii_lowercase().contains(&want) {
                continue;
            }
            // WORLD z the same way the exporter derives `BgQuad.z`: compose the full TRS chain
            // root-first and transform the origin. Summing local z is NOT equivalent — these
            // rigs are rotated, and a rotated parent changes both the magnitude and the SIGN.
            let mut stack: Vec<i64> = Vec::new();
            let mut cur = go_tf.get(&g).copied().unwrap_or(0);
            for _ in 0..64 {
                let Some(&(father, _)) = tf.get(&cur) else { break };
                stack.push(cur);
                if father == 0 {
                    break;
                }
                cur = father;
            }
            let mut m = Mat4::identity();
            for &t in stack.iter().rev() {
                if let Some((_, lm)) = tf.get(&t) {
                    m = m.mul(lm);
                }
            }
            let z = f64::from(m.point([0.0, 0.0, 0.0])[2]);
            println!(
                "sort {order:>5}  z {z:>9.3}  layerID {layer:>6}  class {class:>3}  active {active:>5}  {name}"
            );
        }
    }
}
