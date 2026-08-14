//! Diagnostic: dump every `ParticleSystem`'s RAW `ShapeModule` (unmapped Unity type int,
//! radii, arc) plus its `VelocityOverLifetimeModule` orbital/radial terms.
//!
//! Motivation: `shape_type_name` folds Unity's Donut (17) / Rectangle (18) / Sprite (19)
//! into "none", and orbital velocity is what turns a scattered emission into a moving ring.
//! Either would silently flatten a uniform halo into a static scatter.
//!
//! Usage: cargo run --release --example `probe_shape` -- <bundle.ab> [name-filter]
#![allow(
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

fn f(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN)
}

/// A Unity `MinMaxCurve` serializes as `{minMaxState, scalar, minScalar, ...}`.
fn mmc(v: Option<&Value>) -> String {
    let Some(v) = v else { return "-".into() };
    if let Some(x) = v.as_f64() {
        return format!("{x:.3}");
    }
    let s = f(v, "scalar");
    let mn = f(v, "minScalar");
    let st = v.get("minMaxState").and_then(Value::as_i64).unwrap_or(0);
    format!("scalar={s:.3} min={mn:.3} state={st}")
}

const UNITY_SHAPES: &[(i64, &str)] = &[
    (0, "Sphere"),
    (1, "SphereShell"),
    (2, "Hemisphere"),
    (3, "HemisphereShell"),
    (4, "Cone"),
    (5, "Box"),
    (6, "Mesh"),
    (7, "ConeShell"),
    (8, "ConeVolume"),
    (9, "ConeVolumeShell"),
    (10, "Circle"),
    (11, "CircleEdge"),
    (12, "SingleSidedEdge"),
    (13, "MeshRenderer"),
    (14, "SkinnedMeshRenderer"),
    (15, "BoxShell"),
    (16, "BoxEdge"),
    (17, "Donut"),
    (18, "Rectangle"),
    (19, "Sprite"),
    (20, "SpriteRenderer"),
];

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
        let mut go_name: HashMap<i64, String> = HashMap::new();
        let mut tf_go: HashMap<i64, i64> = HashMap::new();
        let mut go_tf: HashMap<i64, i64> = HashMap::new();
        let mut tf_father: HashMap<i64, i64> = HashMap::new();
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
                    if let Some(g) = v.get("m_GameObject").and_then(pid) {
                        tf_go.insert(*p, g);
                        go_tf.insert(g, *p);
                    }
                    if let Some(fa) = v.get("m_Father").and_then(pid)
                        && fa != 0
                    {
                        tf_father.insert(*p, fa);
                    }
                }
                _ => {}
            }
        }
        let full_path = |go: i64| -> String {
            let mut parts = Vec::new();
            let mut cur = go_tf.get(&go).copied();
            while let Some(tf) = cur {
                if let Some(g) = tf_go.get(&tf) {
                    parts.push(go_name.get(g).cloned().unwrap_or_default());
                }
                cur = tf_father.get(&tf).copied();
            }
            parts.reverse();
            parts.join("/")
        };

        println!("\n########## {} ##########", entry.path);
        let mut counts: HashMap<i64, usize> = HashMap::new();
        let mut rows: Vec<String> = Vec::new();
        for (cid, v) in all.values() {
            if *cid != 198 {
                continue;
            }
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            let name = go_name.get(&go).cloned().unwrap_or_default();
            let fp = full_path(go);
            let t = v
                .get("ShapeModule")
                .and_then(|s| s.get("type"))
                .and_then(Value::as_i64)
                .unwrap_or(-1);
            *counts.entry(t).or_insert(0) += 1;
            if !filter.is_empty()
                && !name.to_lowercase().contains(&filter)
                && !fp.to_lowercase().contains(&filter)
            {
                continue;
            }
            let tname = UNITY_SHAPES
                .iter()
                .find(|(i, _)| *i == t)
                .map_or("?", |(_, n)| *n);
            let sh = v.get("ShapeModule");
            let shape_desc = sh.map_or_else(
                || "<none>".to_string(),
                |s| {
                    format!(
                        "enabled={} radius={:.2} donutRadius={:.2} radiusThickness={:.2} arc={:.1} arcMode={}",
                        s.get("enabled").and_then(Value::as_i64).unwrap_or(-1),
                        f(s, "radius"),
                        f(s, "donutRadius"),
                        f(s, "radiusThickness"),
                        f(s, "arc"),
                        s.get("arcMode").and_then(Value::as_i64).unwrap_or(-1),
                    )
                },
            );
            let vol = v.get("VelocityModule");
            let vol_desc = vol.map_or_else(
                || "<none>".to_string(),
                |m| {
                    format!(
                        "enabled={} orbitalX={} orbitalY={} orbitalZ={} radial={} inWorld={}",
                        m.get("enabled").and_then(Value::as_i64).unwrap_or(-1),
                        mmc(m.get("orbitalX")),
                        mmc(m.get("orbitalY")),
                        mmc(m.get("orbitalZ")),
                        mmc(m.get("radial")),
                        m.get("inWorldSpace").and_then(Value::as_i64).unwrap_or(-1),
                    )
                },
            );
            rows.push(format!(
                "  '{name}'  shapeType={t} ({tname})\n     path={fp}\n     shape: {shape_desc}\n     velocityOverLifetime: {vol_desc}"
            ));
        }
        let mut cs: Vec<_> = counts.into_iter().collect();
        cs.sort_unstable();
        println!(
            "shape-type census: {}",
            cs.iter()
                .map(|(t, n)| {
                    let tn = UNITY_SHAPES
                        .iter()
                        .find(|(i, _)| i == t)
                        .map_or("?", |(_, x)| *x);
                    format!("{t}({tn})={n}")
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
        rows.sort();
        for r in rows {
            println!("{r}");
        }
    }
}
