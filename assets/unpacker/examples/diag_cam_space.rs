//! THROWAWAY diagnostic: for a dynchar `_Start` bundle, dump (1) the camera's
//! transform chain (names + local TRS + which nodes the entrance clip animates),
//! (2) the skeleton (spine-root) chain, (3) the current
//! `entrance_camera_track` output range, and (4) the scene mesh-quad translation
//! span — to pin down the coordinate-space mismatch that blanks skadi2.
//!
//! Usage: cargo run --release --example `diag_cam_space` -- <bundle.ab> [...]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::items_after_statements,
    clippy::too_many_lines
)]

use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::anim;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn vec3(v: &Value, field: &str, d: f64) -> [f64; 3] {
    let g = |k: &str| {
        v.get(field)
            .and_then(|x| x.get(k))
            .and_then(Value::as_f64)
            .unwrap_or(d)
    };
    [g("x"), g("y"), g("z")]
}

fn quat(v: &Value) -> [f64; 4] {
    let g = |k: &str, d: f64| {
        v.get("m_LocalRotation")
            .and_then(|x| x.get(k))
            .and_then(Value::as_f64)
            .unwrap_or(d)
    };
    [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
}

fn main() {
    for arg in std::env::args().skip(1) {
        println!("==== {arg} ====");
        let data = std::fs::read(&arg).expect("read bundle");
        let bundle = BundleFile::parse(data).expect("parse bundle");
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            for obj in &sf.objects {
                if let Ok(v) = read_object(&sf, obj) {
                    all.insert(obj.path_id, (obj.class_id, v));
                }
            }
        }
        // Maps
        let mut go_name: HashMap<i64, String> = HashMap::new();
        let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
        let mut tf_father: HashMap<i64, i64> = HashMap::new();
        let mut tf_go: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            match cid {
                1 => {
                    go_name.insert(
                        *p,
                        v.get("m_Name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                }
                4 | 224 => {
                    if let Some(go) = v.get("m_GameObject").and_then(pid) {
                        go_to_tf.insert(go, *p);
                        tf_go.insert(*p, go);
                    }
                    if let Some(f) = v.get("m_Father").and_then(pid) {
                        tf_father.insert(*p, f);
                    }
                }
                _ => {}
            }
        }
        let chain_of = |go: i64| -> Vec<i64> {
            let mut out = Vec::new();
            let mut cur = go_to_tf.get(&go).copied();
            for _ in 0..64 {
                let Some(tf) = cur else { break };
                out.push(tf);
                cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
            }
            out
        };
        let print_chain = |label: &str, go: i64| {
            println!("-- {label} chain (node -> root):");
            for tf in chain_of(go) {
                let (_, v) = &all[&tf];
                let name = tf_go
                    .get(&tf)
                    .and_then(|g| go_name.get(g))
                    .cloned()
                    .unwrap_or_default();
                let p = vec3(v, "m_LocalPosition", 0.0);
                let s = vec3(v, "m_LocalScale", 1.0);
                let q = quat(v);
                println!(
                    "   {name:40} pos=({:8.3},{:8.3},{:8.3}) rot=({:.3},{:.3},{:.3},{:.3}) scale=({:.3},{:.3},{:.3})",
                    p[0], p[1], p[2], q[0], q[1], q[2], q[3], s[0], s[1], s[2]
                );
            }
        };
        // Camera chain
        let cam_go = all.values().find_map(|(cid, v)| {
            (*cid == 20)
                .then(|| v.get("m_GameObject").and_then(pid))
                .flatten()
        });
        if let Some(cg) = cam_go {
            print_chain("CAMERA", cg);
        } else {
            println!("-- no camera");
        }
        // Skeleton chain(s)
        let skel_gos: Vec<i64> = all
            .values()
            .filter(|(cid, v)| *cid == 114 && v.get("skeletonDataAsset").is_some())
            .filter_map(|(_, v)| v.get("m_GameObject").and_then(pid))
            .collect();
        for sg in &skel_gos {
            print_chain("SKELETON", *sg);
        }
        // Skeleton scale
        let skel_scale = all
            .values()
            .find_map(|(cid, v)| {
                (*cid == 114 && v.get("skeletonJSON").is_some())
                    .then(|| v.get("scale").and_then(Value::as_f64))
                    .flatten()
            })
            .unwrap_or(0.01);
        let inv = 1.0 / skel_scale;
        println!("-- skel_scale={skel_scale} inv={inv}");
        // Current camera track
        let (cam_track, _cam_roll, _cam_aperture) = anim::entrance_camera_track(&all, inv, None);
        match cam_track {
            Some(track) => {
                let (mut ymin, mut ymax, mut xmin, mut xmax) =
                    (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                for &(_, x, y) in &track {
                    ymin = ymin.min(y);
                    ymax = ymax.max(y);
                    xmin = xmin.min(x);
                    xmax = xmax.max(x);
                }
                let first = track.first().unwrap();
                let last = track.last().unwrap();
                println!(
                    "-- cam track: {} keys, x range [{xmin:.1},{xmax:.1}], y range [{ymin:.1},{ymax:.1}], first=({:.1},{:.1}@{:.2}s) last=({:.1},{:.1}@{:.2}s)",
                    track.len(),
                    first.1,
                    first.2,
                    first.0,
                    last.1,
                    last.2,
                    last.0
                );
            }
            None => println!("-- cam track: NONE"),
        }
        // Scene mesh quad translation span: every GO with a MeshRenderer (class 23),
        // accumulated to the ABSOLUTE root (world), and to the spine root frame.
        let mut wy: Vec<f64> = Vec::new();
        let mut wx: Vec<f64> = Vec::new();
        for (cid, v) in all.values() {
            if *cid != 23 {
                continue;
            }
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            // crude world translation: sum of chain local positions rotated is ignored;
            // use full TRS accumulation via Mat4 for correctness.
            use unpacker::export::mesh::Mat4;
            let mut m = Mat4::identity();
            for &tf in chain_of(go).iter().rev() {
                let (_, tv) = &all[&tf];
                let p = vec3(tv, "m_LocalPosition", 0.0);
                let q = quat(tv);
                let s = vec3(tv, "m_LocalScale", 1.0);
                let local = Mat4::trs(
                    [p[0] as f32, p[1] as f32, p[2] as f32],
                    [q[0] as f32, q[1] as f32, q[2] as f32, q[3] as f32],
                    [s[0] as f32, s[1] as f32, s[2] as f32],
                );
                m = m.mul(&local);
            }
            let pt = m.point([0.0, 0.0, 0.0]);
            wx.push(f64::from(pt[0]));
            wy.push(f64::from(pt[1]));
        }
        if !wy.is_empty() {
            let (ymin, ymax) = wy
                .iter()
                .fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
            let (xmin, xmax) = wx
                .iter()
                .fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
            println!(
                "-- mesh renderer WORLD translations: {} quads, x [{xmin:.2},{xmax:.2}] y [{ymin:.2},{ymax:.2}] (world units); in mesh px (x*inv, -y*inv): x [{:.0},{:.0}] y [{:.0},{:.0}]",
                wy.len(),
                xmin * inv,
                xmax * inv,
                -ymax * inv,
                -ymin * inv
            );
        }
        println!();
    }
}
