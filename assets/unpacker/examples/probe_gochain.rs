//! Diagnostic: the Transform chain of every GameObject whose name contains a substring, composed
//! root to leaf: local position, rotation (as a z angle when it is a pure z rotation) and scale
//! per node, and the composed world position and scale, in Unity units. For placing an emitter
//! or a quad whose export sits somewhere the clip does not show it.
//!
//! Usage: cargo run --release --example `probe_gochain` -- <bundle.ab> <go-substr>
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

/// z angle in degrees of a quaternion, plus whether it is a pure z rotation.
fn z_angle(q: [f64; 4]) -> (f64, bool) {
    let pure = q[0].abs() < 1e-4 && q[1].abs() < 1e-4;
    (2.0 * q[2].atan2(q[3]).to_degrees(), pure)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(path), Some(want)) = (args.next(), args.next()) else {
        eprintln!("usage: probe_gochain <bundle.ab> <go-substr>");
        return;
    };
    let Ok(data) = std::fs::read(&path) else {
        return;
    };
    let Ok(bundle) = BundleFile::parse(data) else {
        return;
    };
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
        for obj in &sf.objects {
            if obj.class_id != 1 && obj.class_id != 4 {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            if obj.class_id == 1 {
                gos.insert(
                    obj.path_id,
                    v.get("m_Name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                );
            } else {
                let go = v.get("m_GameObject").map(pid).unwrap_or(0);
                xform_of_go.insert(go, obj.path_id);
                xforms.insert(obj.path_id, v);
            }
        }
        let mut targets: Vec<(&i64, &String)> =
            gos.iter().filter(|(_, n)| n.contains(&want)).collect();
        targets.sort_by(|a, b| a.1.cmp(b.1));
        for (go, name) in targets {
            let Some(&tid) = xform_of_go.get(go) else {
                continue;
            };
            let mut chain: Vec<i64> = Vec::new();
            let mut cur = tid;
            while cur != 0 && chain.len() < 64 {
                chain.push(cur);
                cur = xforms
                    .get(&cur)
                    .and_then(|t| t.get("m_Father"))
                    .map(pid)
                    .unwrap_or(0);
            }
            // Compose root -> leaf with a pure z rotation per node (the general case is
            // printed as a quaternion and NOT composed, so it cannot pass silently).
            let mut wx = 0.0f64;
            let mut wy = 0.0f64;
            let mut wsx = 1.0f64;
            let mut wsy = 1.0f64;
            let mut wrot = 0.0f64;
            let mut lines: Vec<String> = Vec::new();
            for &t in chain.iter().rev() {
                let tv = &xforms[&t];
                let lp = xyz(tv.get("m_LocalPosition"));
                let ls = xyz(tv.get("m_LocalScale"));
                let q = quat(tv.get("m_LocalRotation"));
                let (ang, pure) = z_angle(q);
                let nm = gos
                    .get(&tv.get("m_GameObject").map(pid).unwrap_or(0))
                    .cloned()
                    .unwrap_or_default();
                // world = parent_world + R(parent_rot) * (parent_scale * local)
                let (px, py) = (wsx * lp[0], wsy * lp[1]);
                let (c, s) = (wrot.to_radians().cos(), wrot.to_radians().sin());
                wx += c * px - s * py;
                wy += s * px + c * py;
                wsx *= ls[0];
                wsy *= ls[1];
                wrot += ang;
                // A rotation that is not a pure z turn is printed as its quaternion and as
                // the tilt it carries: the angle between the node's local z axis and the
                // world z axis (the camera axis), plus the intrinsic x/y/z Euler angles, so
                // a tilted emitter's plane can be read instead of only flagged.
                let tilt = if pure {
                    String::new()
                } else {
                    let [x, y, z, w] = q;
                    // Local z axis rotated by q, third column of the rotation matrix.
                    let zx = 2.0 * (x * z + w * y);
                    let zy = 2.0 * (y * z - w * x);
                    let zz = 1.0 - 2.0 * (x * x + y * y);
                    let tilt_deg = zz.clamp(-1.0, 1.0).acos().to_degrees();
                    let ex = (2.0 * (w * x + y * z))
                        .atan2(1.0 - 2.0 * (x * x + y * y))
                        .to_degrees();
                    let ey = (2.0 * (w * y - z * x)).clamp(-1.0, 1.0).asin().to_degrees();
                    let ez = (2.0 * (w * z + x * y))
                        .atan2(1.0 - 2.0 * (y * y + z * z))
                        .to_degrees();
                    format!(
                        " (NOT pure z: q ({x:+.4}, {y:+.4}, {z:+.4}, {w:+.4}) local z -> ({zx:+.3}, {zy:+.3}, {zz:+.3}), tilt {tilt_deg:.2} deg, euler xyz ({ex:+.2}, {ey:+.2}, {ez:+.2}))"
                    )
                };
                lines.push(format!(
                    "    {nm:<40} local ({:+9.3}, {:+9.3}) scale ({:.4}, {:.4}) rotZ {:+8.2}{}   -> world ({:+9.3}, {:+9.3}) scale ({:.4}, {:.4}) rot {:+8.2}",
                    lp[0], lp[1], ls[0], ls[1], ang, tilt, wx, wy, wsx, wsy, wrot
                ));
            }
            println!("== '{name}'");
            for l in lines {
                println!("{l}");
            }
        }
    }
}
