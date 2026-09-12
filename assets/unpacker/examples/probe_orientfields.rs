//! THROWAWAY census: the five Unity particle ORIENTATION fields the exporter never reads, for
//! every mesh-mode ParticleSystem, beside the composed emitter tilt `probe_meshbasis` prints.
//!
//! Motivation: 363 Local-aligned mesh systems sit on emitters turned out of the screen plane
//! (the foreshortened class, register 2026-09-08, twenty-ninth run), held as not buildable
//! because the game draws a 180 degree in-plane turn as the un-turned picture. The exporter's
//! vocabulary has `startRotation` (z only), `rotOverLifeDegPerSec` and `meshBasis`, and none of
//! `InitialModule.rotation3D`, `startRotationX` / `startRotationY` (radians), `RotationModule
//! .separateAxes`, `ShapeModule.alignToDirection`; `m_RenderAlignment` is read only for the
//! basis. If the affected emitters carry `rotation3D 0`, the X and Y particle rotations never
//! reach the runtime at any angle. The turn the register measured lives in the EMITTER's
//! transform chain, a different mechanism, so this prints both side by side: which field
//! carries the turn is the first thing the sweep has to establish.
//!
//! Prints one TSV row per mesh-mode system: skin, name, alignment, composed tilt (degrees
//! between local z and the camera axis), cos(tilt), flat z angle, then the five fields as the
//! bundle serializes them (`-` when absent), with `startRotationX` / `Y` / `startRotation` as
//! `minMaxState:scalar:constantMin`.
//!
//! Usage: cargo run --release --example `probe_orientfields` -- <bundle.ab>...
#![allow(clippy::too_many_lines)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> i64 {
    v.get("m_PathID").and_then(Value::as_i64).unwrap_or(0)
}

fn quat(v: Option<&Value>) -> [f64; 4] {
    let g = |k: &str| {
        v.and_then(|o| o.get(k))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    [g("x"), g("y"), g("z"), g("w")]
}

/// Hamilton product a * b (apply b first, then a), Unity's parent * child order.
fn qmul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// The rotation matrix columns: where the local x, y, z axes land.
fn axes(q: [f64; 4]) -> [[f64; 3]; 3] {
    let [x, y, z, w] = q;
    [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + w * z),
            2.0 * (x * z - w * y),
        ],
        [
            2.0 * (x * y - w * z),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + w * x),
        ],
        [
            2.0 * (x * z + w * y),
            2.0 * (y * z - w * x),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ]
}

/// A serialized field as text, `-` when the path is absent.
fn field(v: &Value, path: &[&str]) -> String {
    let mut cur = v;
    for p in path {
        match cur.get(p) {
            Some(n) => cur = n,
            None => return "-".to_string(),
        }
    }
    match cur {
        Value::Bool(b) => i32::from(*b).to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// A MinMaxCurve as `minMaxState:scalar:constantMin`, `-` when absent.
fn mmcurve(v: &Value, path: &[&str]) -> String {
    let mut cur = v;
    for p in path {
        match cur.get(p) {
            Some(n) => cur = n,
            None => return "-".to_string(),
        }
    }
    format!(
        "{}:{}:{}",
        field(cur, &["minMaxState"]),
        field(cur, &["scalar"]),
        field(cur, &["minScalar"])
    )
}

fn main() {
    println!(
        "skin\tname\talign\ttilt\tcos\tflat\trotation3D\tstartRotationX\tstartRotationY\tstartRotation\trot.enabled\trot.separateAxes\tshape.enabled\tshape.alignToDirection"
    );
    let mut printed_keys = false;
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        let skin = std::path::Path::new(&path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let mut gos: HashMap<i64, String> = HashMap::new();
            let mut xforms: HashMap<i64, Value> = HashMap::new();
            let mut xform_of_go: HashMap<i64, i64> = HashMap::new();
            let mut renderers: HashMap<i64, Value> = HashMap::new();
            let mut systems: Vec<(i64, i64, Value)> = Vec::new();
            for obj in &sf.objects {
                if !matches!(obj.class_id, 1 | 4 | 198 | 199) {
                    continue;
                }
                let Ok(v) = read_object(&sf, obj) else {
                    continue;
                };
                let go = v.get("m_GameObject").map(pid).unwrap_or(0);
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
                        xform_of_go.insert(go, obj.path_id);
                        xforms.insert(obj.path_id, v);
                    }
                    198 => systems.push((obj.path_id, go, v)),
                    _ => {
                        renderers.insert(go, v);
                    }
                }
            }
            systems.sort_by_key(|(p, _, _)| *p);
            for (_, go, ps) in systems {
                let Some(r) = renderers.get(&go) else {
                    continue;
                };
                let mode = r.get("m_RenderMode").and_then(Value::as_i64).unwrap_or(-1);
                if mode != 4 {
                    continue;
                }
                if !printed_keys {
                    printed_keys = true;
                    for m in ["InitialModule", "RotationModule", "ShapeModule"] {
                        let keys: Vec<&str> = ps
                            .get(m)
                            .and_then(Value::as_object)
                            .map(|o| o.keys().map(String::as_str).collect())
                            .unwrap_or_default();
                        eprintln!("[keys] {m}: {}", keys.join(","));
                    }
                }
                let align = r
                    .get("m_RenderAlignment")
                    .and_then(Value::as_i64)
                    .unwrap_or(-1);
                let name = gos.get(&go).cloned().unwrap_or_default();
                let mut q = [0.0, 0.0, 0.0, 1.0];
                let mut cur = xform_of_go.get(&go).copied().unwrap_or(0);
                let mut depth = 0;
                while cur != 0 && depth < 64 {
                    let Some(t) = xforms.get(&cur) else { break };
                    q = qmul(quat(t.get("m_LocalRotation")), q);
                    cur = t.get("m_Father").map(pid).unwrap_or(0);
                    depth += 1;
                }
                let a = axes(q);
                let flat = a[0][1].atan2(a[0][0]).to_degrees();
                let cz = a[2][2].clamp(-1.0, 1.0);
                let tilt = cz.acos().to_degrees();
                println!(
                    "{skin}\t{name}\t{align}\t{tilt:.2}\t{cz:+.4}\t{flat:+.2}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    field(&ps, &["InitialModule", "rotation3D"]),
                    mmcurve(&ps, &["InitialModule", "startRotationX"]),
                    mmcurve(&ps, &["InitialModule", "startRotationY"]),
                    mmcurve(&ps, &["InitialModule", "startRotation"]),
                    field(&ps, &["RotationModule", "enabled"]),
                    field(&ps, &["RotationModule", "separateAxes"]),
                    field(&ps, &["ShapeModule", "enabled"]),
                    field(&ps, &["ShapeModule", "alignToDirection"]),
                );
            }
        }
    }
}
