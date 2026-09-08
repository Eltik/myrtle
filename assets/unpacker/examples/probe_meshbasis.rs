//! THROWAWAY census: for every mesh-mode ParticleSystem, the renderer's alignment and the
//! emitter's COMPOSED world rotation, so the systems whose transform is not a pure z turn can
//! be split into the ones the tilt reaches and the ones it cannot.
//!
//! Motivation: 264 exported mesh-mode systems on 38 skins sit on emitters rotated out of the
//! screen plane. A camera-facing or velocity-aligned renderer ignores that rotation, a Local
//! one takes it, a World one ignores even the z turn; and a 180 degree flip on a mirror-symmetric
//! mesh moves nothing while a 90 degree turn folds the mesh edge-on. The viewer draws every mesh
//! face-on with the flat z angle, so the live count is what this split says, not 264.
//!
//! Prints one line per mesh-mode system: skin, name, alignment, composed quaternion, the world
//! images of the local x, y, z axes (rotation only, scale stripped), the flat z angle the exporter
//! uses, and the angle between local z and the camera axis.
//!
//! Usage: cargo run --release --example `probe_meshbasis` -- <bundle.ab>...
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

fn main() {
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
            let mut systems: Vec<(i64, i64)> = Vec::new();
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
                    198 => systems.push((obj.path_id, go)),
                    _ => {
                        renderers.insert(go, v);
                    }
                }
            }
            systems.sort_by_key(|(p, _)| *p);
            for (_, go) in systems {
                let Some(r) = renderers.get(&go) else {
                    continue;
                };
                let mode = r.get("m_RenderMode").and_then(Value::as_i64).unwrap_or(-1);
                if mode != 4 {
                    continue;
                }
                let align = r
                    .get("m_RenderAlignment")
                    .and_then(Value::as_i64)
                    .unwrap_or(-1);
                let name = gos.get(&go).cloned().unwrap_or_default();
                // Compose the chain leaf -> root: world = q_root * ... * q_leaf.
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
                let tilt = a[2][2].clamp(-1.0, 1.0).acos().to_degrees();
                println!(
                    "{skin}\t{name}\talign={align}\tq=({:+.4},{:+.4},{:+.4},{:+.4})\tx=({:+.3},{:+.3},{:+.3})\ty=({:+.3},{:+.3},{:+.3})\tz=({:+.3},{:+.3},{:+.3})\tflat={flat:+.2}\ttilt={tilt:.2}",
                    q[0],
                    q[1],
                    q[2],
                    q[3],
                    a[0][0],
                    a[0][1],
                    a[0][2],
                    a[1][0],
                    a[1][1],
                    a[1][2],
                    a[2][0],
                    a[2][1],
                    a[2][2]
                );
            }
        }
    }
}
