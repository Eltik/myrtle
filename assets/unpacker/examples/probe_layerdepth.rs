//! THROWAWAY diagnostic: per-layer DEPTH under the entrance camera.
//!
//! `cam_lock_view` in `spine.rs` derives a depth only for GameObjects PARENTED under the
//! camera, by walking the chain until it reaches the camera transform. Whislash-alter's
//! sprocket bars come out at d=4.36 that way, against the dolly's d0=3.00, and that 1.4533
//! ratio was "exactly the size-and-position error". Every other layer is scaled by the single
//! focal-plane extent instead, so this asks what depth each layer ACTUALLY sits at: world
//! position projected on the camera's forward axis.
//!
//! ⚠️ TWO KINDS OF LAYER, and the distinction is the whole point of the `lock` column. A
//! CAM-LOCKED layer is parented under the camera, so its distance is FIXED at that value for
//! every t however far the shot dollies. A WORLD layer stays put while the camera moves, so
//! its distance at time t is `d_focal(t) + (d_prefab - d_focal_prefab)`. Applying the world
//! arithmetic to a cam-locked row produces a distance that tracks the dolly, which is exactly
//! the geometry a camera-parented quad does NOT have. The prefab `d` printed here is a t=0
//! reading for both kinds; only the WORLD rows may be advanced with the dolly.
//!
//! Usage: cargo run --release --example `probe_layerdepth` -- <bundle.ab> [root-substring]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::mesh::Mat4;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID")
        .and_then(Value::as_i64)
        .filter(|&p| p != 0)
}

fn local_of(all: &HashMap<i64, (i32, Value)>, tf: i64) -> Mat4 {
    all.get(&tf).map_or_else(Mat4::identity, |(_, v)| {
        let g3 = |f: &str, d: f32| {
            let g = |k: &str| {
                v.get(f)
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::from(d)) as f32
            };
            [g("x"), g("y"), g("z")]
        };
        let q = {
            let g = |k: &str, d: f32| {
                v.get("m_LocalRotation")
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::from(d)) as f32
            };
            [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
        };
        Mat4::trs(g3("m_LocalPosition", 0.0), q, g3("m_LocalScale", 1.0))
    })
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle");
    let filter = std::env::args().nth(2).unwrap_or_default();
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
            if matches!(obj.class_id, 28 | 43 | 48 | 49 | 83 | 128 | 213) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        // go -> transform, transform -> father, transform -> go
        let (mut go_tf, mut tf_father, mut tf_go) =
            (HashMap::new(), HashMap::new(), HashMap::new());
        for (p, (cid, v)) in &all {
            if *cid == 4 || *cid == 224 {
                if let Some(g) = v.get("m_GameObject").and_then(pid) {
                    go_tf.insert(g, *p);
                    tf_go.insert(*p, g);
                }
                if let Some(f) = v.get("m_Father").and_then(pid) {
                    tf_father.insert(*p, f);
                }
            }
        }
        let world = |tf: i64| -> Mat4 {
            let mut m = Mat4::identity();
            let mut cur = Some(tf);
            for _ in 0..64 {
                let Some(c) = cur else { break };
                m = local_of(&all, c).mul(&m);
                cur = tf_father.get(&c).copied();
            }
            m
        };
        let name = |go: i64| -> String {
            all.get(&go)
                .and_then(|(_, v)| v.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string()
        };
        // Mirrors `cam_lock_view`'s test: walk the chain and see whether it reaches the
        // camera's own transform. A hit means the layer is parented UNDER the camera and its
        // distance is invariant under the dolly.
        let cam_locked = |go: i64, cam_tf: i64| -> bool {
            let mut cur = go_tf.get(&go).copied();
            for _ in 0..64 {
                let Some(c) = cur else { return false };
                if c == cam_tf {
                    return true;
                }
                cur = tf_father.get(&c).copied();
            }
            false
        };
        let root_of = |go: i64| -> String {
            let mut cur = go_tf.get(&go).copied();
            let mut last = go;
            for _ in 0..64 {
                let Some(c) = cur else { break };
                if let Some(g) = tf_go.get(&c) {
                    last = *g;
                }
                match tf_father.get(&c) {
                    Some(f) => cur = Some(*f),
                    None => break,
                }
            }
            name(last)
        };
        // Entrance camera: the MonoBehaviour `_mainCamera.camera` ref, else the first class 20.
        let cam = all
            .iter()
            .filter(|(_, (c, _))| *c == 114)
            .filter_map(|(_, (_, v))| {
                v.get("_mainCamera")
                    .and_then(|m| m.get("camera"))
                    .and_then(pid)
            })
            .find(|p| matches!(all.get(p), Some((20, _))))
            .or_else(|| all.iter().find(|(_, (c, _))| *c == 20).map(|(p, _)| *p));
        let Some(cam) = cam else { continue };
        let (_, camv) = &all[&cam];
        let fov = camv
            .get("field of view")
            .and_then(Value::as_f64)
            .unwrap_or(60.0);
        let is_ortho = camv
            .get("orthographic")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let Some(cam_tf) = camv
            .get("m_GameObject")
            .and_then(pid)
            .and_then(|g| go_tf.get(&g).copied())
        else {
            continue;
        };
        let cw = world(cam_tf);
        let cpos = cw.point([0.0, 0.0, 0.0]);
        let cfz = cw.point([0.0, 0.0, 1.0]);
        let fwd = [cfz[0] - cpos[0], cfz[1] - cpos[1], cfz[2] - cpos[2]];
        let n = (fwd[0] * fwd[0] + fwd[1] * fwd[1] + fwd[2] * fwd[2]).sqrt();
        let fwd = [fwd[0] / n, fwd[1] / n, fwd[2] / n];
        let mut skel: Vec<(i64, f64)> = all
            .iter()
            .filter(|(_, (_, v))| v.get("skeletonJSON").is_some())
            .filter_map(|(p, (_, v))| Some((*p, v.get("scale").and_then(Value::as_f64)?)))
            .collect();
        skel.sort_unstable_by_key(|(p, _)| *p);
        let inv = 1.0 / skel.first().map_or(0.01, |(_, s)| *s);
        println!(
            "\n=== {} ortho={is_ortho} fov={fov} inv_scale={inv:.1}",
            entry.path
        );
        println!(
            "camera world pos ({:.3},{:.3},{:.3}) fwd ({:.3},{:.3},{:.3})",
            cpos[0], cpos[1], cpos[2], fwd[0], fwd[1], fwd[2]
        );
        // Every GO carrying a MeshRenderer (class 23) is a scene-layer candidate.
        let mut rows: Vec<(f64, String, String, bool)> = Vec::new();
        for (cid, v) in all.values() {
            if *cid != 23 {
                continue;
            }
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            let r = root_of(go);
            if !filter.is_empty() && !r.contains(&filter) {
                continue;
            }
            let Some(tf) = go_tf.get(&go).copied() else {
                continue;
            };
            let p = world(tf).point([0.0, 0.0, 0.0]);
            let d = f64::from(
                (p[0] - cpos[0]) * fwd[0] + (p[1] - cpos[1]) * fwd[1] + (p[2] - cpos[2]) * fwd[2],
            );
            rows.push((d, name(go), r, cam_locked(go, cam_tf)));
        }
        rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        println!(
            "{:>9}  {:>8}  {:<10}  {:<26} root",
            "depth_d", "d/3.00", "lock", "layer"
        );
        for (d, nm, r, locked) in &rows {
            let lk = if *locked { "CAM-LOCKED" } else { "world" };
            println!("{d:9.4}  {:8.4}  {lk:<10}  {nm:<26} {r}", d / 3.0_f64);
        }
        let nlock = rows.iter().filter(|r| r.3).count();
        println!(
            "({} mesh-renderer GOs: {nlock} CAM-LOCKED (distance invariant under the dolly), {} world)",
            rows.len(),
            rows.len() - nlock
        );
    }
}
