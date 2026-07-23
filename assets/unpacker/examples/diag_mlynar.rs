//! THROWAWAY: Mlynar entrance parity diagnosis (Gap A camera descent, Gap B white flash).
//! Usage: cargo run --release --example diag_mlynar -- <bundle.ab>

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}
fn u32_array(v: Option<&Value>) -> Vec<u32> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_u64().map(|n| n as u32))
                .collect()
        })
        .unwrap_or_default()
}
fn f32_array(v: Option<&Value>) -> Vec<f32> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_f64().map(|n| n as f32))
                .collect()
        })
        .unwrap_or_default()
}
fn binding_fields(b: &Value) -> (i64, i64, u32) {
    let tid = b
        .get("typeID")
        .or_else(|| b.get("classID"))
        .and_then(Value::as_i64)
        .unwrap_or(-1);
    let attr = b.get("attribute").and_then(Value::as_i64).unwrap_or(-1);
    let path = b.get("path").and_then(Value::as_i64).unwrap_or(0) as u32;
    (tid, attr, path)
}
const fn bcount(tid: i64, attr: i64) -> usize {
    if tid == 4 {
        match attr {
            1 | 3 | 4 => 3,
            2 => 4,
            _ => 1,
        }
    } else {
        1
    }
}
fn clip_data(clip: &Value) -> Option<&Value> {
    clip.get("m_MuscleClip")
        .and_then(|m| m.get("m_Clip"))
        .and_then(|c| c.get("data"))
}
fn subclip_counts(clip: &Value, total: usize) -> (usize, usize, usize) {
    let data = clip_data(clip).unwrap();
    let dense_count = data
        .get("m_DenseClip")
        .and_then(|d| d.get("m_CurveCount"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0) as usize;
    let const_count = f32_array(data.get("m_ConstantClip").and_then(|s| s.get("data"))).len();
    let stream_count = total.saturating_sub(dense_count + const_count);
    (stream_count, dense_count, const_count)
}
fn decode_curve(clip: &Value, idx: usize, total: usize) -> Vec<(f32, f32)> {
    let Some(data) = clip_data(clip) else {
        return Vec::new();
    };
    let streamed_raw = u32_array(data.get("m_StreamedClip").and_then(|s| s.get("data")));
    let dense_data = f32_array(data.get("m_DenseClip").and_then(|s| s.get("data")));
    let dense_count = data
        .get("m_DenseClip")
        .and_then(|d| d.get("m_CurveCount"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0) as usize;
    let dense_begin = data
        .get("m_DenseClip")
        .and_then(|d| d.get("m_BeginTime"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0) as f32;
    let dense_sample = data
        .get("m_DenseClip")
        .and_then(|d| d.get("m_SampleRate"))
        .and_then(Value::as_f64)
        .unwrap_or(60.0) as f32;
    let const_data = f32_array(data.get("m_ConstantClip").and_then(|s| s.get("data")));
    let const_count = const_data.len();
    let stream_count = total.saturating_sub(dense_count + const_count);
    let mut out = Vec::new();
    if idx < stream_count {
        let mut val = f32::NAN;
        let mut i = 0usize;
        let mut fi = 0usize;
        while i + 2 <= streamed_raw.len() {
            let time = f32::from_bits(streamed_raw[i]);
            let nk = streamed_raw[i + 1] as usize;
            i += 2;
            for _ in 0..nk {
                if i + 5 > streamed_raw.len() {
                    return out;
                }
                let ci = streamed_raw[i] as i32 as usize;
                if ci == idx {
                    val = f32::from_bits(streamed_raw[i + 4]);
                }
                i += 5;
            }
            if fi != 0 && time.is_finite() && !val.is_nan() {
                out.push((time.max(0.0), val));
            }
            fi += 1;
        }
    } else if idx < stream_count + dense_count {
        let di = idx - stream_count;
        let frames = if dense_count > 0 {
            dense_data.len() / dense_count
        } else {
            0
        };
        for f in 0..frames {
            if let Some(&v) = dense_data.get(f * dense_count + di) {
                out.push((dense_begin + f as f32 / dense_sample.max(1.0), v));
            }
        }
    } else {
        let ci = idx - stream_count - dense_count;
        if let Some(&v) = const_data.get(ci) {
            out.push((0.0, v));
        }
    }
    out
}
fn tf_total(clip: &Value) -> usize {
    clip.get("m_ClipBindingConstant")
        .and_then(|b| b.get("genericBindings"))
        .and_then(Value::as_array)
        .map(|bs| {
            bs.iter()
                .map(|b| {
                    let (t, a, _) = binding_fields(b);
                    bcount(t, a)
                })
                .sum()
        })
        .unwrap_or(0)
}

#[derive(Clone)]
struct M(pub [[f32; 4]; 4]);
impl M {
    fn id() -> M {
        let mut m = [[0.0; 4]; 4];
        for i in 0..4 {
            m[i][i] = 1.0;
        }
        M(m)
    }
    fn trs(pos: [f32; 3], q: [f32; 4], s: [f32; 3]) -> M {
        let [x, y, z, w] = q;
        let (xx, yy, zz) = (x * x, y * y, z * z);
        let (xy, xz, yz) = (x * y, x * z, y * z);
        let (wx, wy, wz) = (w * x, w * y, w * z);
        let r = [
            [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy)],
            [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx)],
            [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy)],
        ];
        let mut m = [[0.0; 4]; 4];
        for i in 0..3 {
            m[i][0] = r[i][0] * s[0];
            m[i][1] = r[i][1] * s[1];
            m[i][2] = r[i][2] * s[2];
            m[i][3] = pos[i];
        }
        m[3][3] = 1.0;
        M(m)
    }
    fn mul(&self, o: &M) -> M {
        let mut m = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                let mut s = 0.0;
                for k in 0..4 {
                    s += self.0[i][k] * o.0[k][j];
                }
                m[i][j] = s;
            }
        }
        M(m)
    }
    fn point(&self, p: [f32; 3]) -> [f32; 3] {
        let m = &self.0;
        [
            m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3],
            m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3],
            m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3],
        ]
    }
}
fn quat_to_euler(q: [f32; 4]) -> [f32; 3] {
    let [x, y, z, w] = q;
    let sinr = 2.0 * (w * x + y * z);
    let cosr = 1.0 - 2.0 * (x * x + y * y);
    let roll = sinr.atan2(cosr).to_degrees();
    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        90.0f32.copysign(sinp)
    } else {
        sinp.asin().to_degrees()
    };
    let siny = 2.0 * (w * z + x * y);
    let cosy = 1.0 - 2.0 * (y * y + z * z);
    let yaw = siny.atan2(cosy).to_degrees();
    [roll, pitch, yaw]
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
    let data = std::fs::read(&path).unwrap();
    let bundle = BundleFile::parse(data).unwrap();
    let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
        for obj in &sf.objects {
            if skip.contains(&obj.class_id) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
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
                if let Some(f) = v.get("m_Father").and_then(pid) {
                    tf_father.insert(*p, f);
                }
            }
            _ => {}
        }
    }
    let name_of = |go: i64| {
        go_name
            .get(&go)
            .cloned()
            .unwrap_or_else(|| format!("?{go}"))
    };
    let go_path = |go: i64| -> String {
        let mut parts = Vec::new();
        let mut cur = go_tf.get(&go).copied();
        for _ in 0..128 {
            let Some(tf) = cur else { break };
            if let Some(g) = tf_go.get(&tf) {
                parts.push(name_of(*g));
            }
            cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
        }
        parts.reverse();
        parts.join("/")
    };
    let mut hash_to_go: HashMap<u32, i64> = HashMap::new();
    for (tf, go) in &tf_go {
        let mut chain: Vec<String> = Vec::new();
        let mut cur = *tf;
        let mut guard = 0;
        loop {
            guard += 1;
            if guard > 128 {
                break;
            }
            let Some(g) = tf_go.get(&cur) else { break };
            chain.push(name_of(*g));
            match tf_father.get(&cur) {
                Some(f) if *f != 0 && tf_go.contains_key(f) => cur = *f,
                _ => break,
            }
        }
        chain.reverse();
        for start in 0..chain.len() {
            let p = chain[start..].join("/");
            hash_to_go.entry(crc32(p.as_bytes())).or_insert(*go);
        }
    }
    let local = |tf: i64| -> ([f32; 3], [f32; 4], [f32; 3]) {
        let (_, v) = all.get(&tf).unwrap();
        let g3 = |field: &str, d: f32| {
            let g = |k: &str| {
                v.get(field)
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(d as f64) as f32
            };
            [g("x"), g("y"), g("z")]
        };
        let pos = g3("m_LocalPosition", 0.0);
        let gr = |k: &str, d: f32| {
            v.get("m_LocalRotation")
                .and_then(|x| x.get(k))
                .and_then(Value::as_f64)
                .unwrap_or(d as f64) as f32
        };
        let q = [gr("x", 0.0), gr("y", 0.0), gr("z", 0.0), gr("w", 1.0)];
        let mut s = g3("m_LocalScale", 1.0);
        for c in &mut s {
            if c.abs() < 1e-6 {
                *c = 1.0;
            }
        }
        (pos, q, s)
    };

    let mut cams: Vec<i64> = Vec::new();
    for (_p, (cid, v)) in &all {
        if *cid == 20 {
            if let Some(go) = v.get("m_GameObject").and_then(pid) {
                cams.push(go);
            }
        }
    }
    println!("=== CAMERAS ({}) ===", cams.len());
    for c in &cams {
        println!("  cam GO '{}' path={}", name_of(*c), go_path(*c));
    }
    let spine_gos: HashSet<i64> = all
        .values()
        .filter(|(c, v)| *c == 114 && v.get("skeletonDataAsset").is_some())
        .filter_map(|(_, v)| v.get("m_GameObject").and_then(pid))
        .collect();
    println!(
        "spine root GOs: {:?}",
        spine_gos.iter().map(|g| name_of(*g)).collect::<Vec<_>>()
    );

    let cam_go = *cams.first().expect("no camera");
    let mut chain: Vec<i64> = Vec::new();
    let mut cur = go_tf.get(&cam_go).copied();
    for _ in 0..64 {
        let Some(tf) = cur else { break };
        if tf_go.get(&tf).is_some_and(|g| spine_gos.contains(g)) {
            break;
        }
        chain.push(tf);
        cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
    }
    println!(
        "\n=== CAMERA CHAIN (camera -> below-root), {} nodes ===",
        chain.len()
    );
    for &tf in &chain {
        let go = tf_go[&tf];
        let (pos, q, s) = local(tf);
        let e = quat_to_euler(q);
        println!(
            "  GO '{}' tf={} pos=[{:.4},{:.4},{:.4}] quat=[{:.4},{:.4},{:.4},{:.4}] euler~=[{:.1},{:.1},{:.1}] scale=[{:.3},{:.3},{:.3}]",
            name_of(go),
            tf,
            pos[0],
            pos[1],
            pos[2],
            q[0],
            q[1],
            q[2],
            q[3],
            e[0],
            e[1],
            e[2],
            s[0],
            s[1],
            s[2]
        );
    }
    let chain_gos: HashSet<i64> = chain.iter().map(|tf| tf_go[tf]).collect();

    let entrance: Vec<(&i64, &Value)> = all
        .iter()
        .filter(|(_, (c, v))| {
            *c == 74 && {
                let n = v
                    .get("m_Name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_ascii_lowercase();
                n.contains("start") || n.contains("entrance") || n.contains("enter")
            }
        })
        .map(|(p, (_, v))| (p, v))
        .collect();
    println!("\n=== ENTRANCE CLIPS ({}) ===", entrance.len());
    for (_p, v) in &entrance {
        println!(
            "  '{}'",
            v.get("m_Name").and_then(Value::as_str).unwrap_or("?")
        );
    }

    println!("\n=== CAMERA-CHAIN POSITION BINDINGS IN ENTRANCE CLIPS ===");
    let mut animated: HashMap<i64, [Vec<(f32, f32)>; 3]> = HashMap::new();
    for (_p, v) in &entrance {
        let cname = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
        let Some(bindings) = v
            .get("m_ClipBindingConstant")
            .and_then(|b| b.get("genericBindings"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        let total = tf_total(v);
        let (sc, dc, _cc) = subclip_counts(v, total);
        let mut gidx = 0usize;
        for b in bindings {
            let (tid, attr, path) = binding_fields(b);
            let count = bcount(tid, attr);
            if tid == 4 && attr == 1 {
                if let Some(&go) = hash_to_go.get(&path) {
                    if chain_gos.contains(&go) {
                        for axis in 0..3 {
                            let gi = gidx + axis;
                            let loc = if gi < sc {
                                "STREAM"
                            } else if gi < sc + dc {
                                "DENSE"
                            } else {
                                "CONST"
                            };
                            let c = decode_curve(v, gi, total);
                            let (mn, mx) = c
                                .iter()
                                .fold((f32::MAX, f32::MIN), |(a, b), &(_, x)| (a.min(x), b.max(x)));
                            println!(
                                "  clip '{}' GO '{}' axis{} gidx={} [{}] keys={} t=[{:.2}..{:.2}] v0={:.4} vN={:.4} min={:.4} max={:.4} range={:.4}",
                                cname,
                                name_of(go),
                                axis,
                                gi,
                                loc,
                                c.len(),
                                c.first().map(|x| x.0).unwrap_or(0.0),
                                c.last().map(|x| x.0).unwrap_or(0.0),
                                c.first().map(|x| x.1).unwrap_or(0.0),
                                c.last().map(|x| x.1).unwrap_or(0.0),
                                mn,
                                mx,
                                mx - mn
                            );
                            if c.len() > 1 {
                                let e = animated
                                    .entry(go_tf[&go])
                                    .or_insert_with(|| [Vec::new(), Vec::new(), Vec::new()]);
                                if c.len() > e[axis].len() {
                                    e[axis] = c;
                                }
                            }
                        }
                    }
                }
            }
            gidx += count;
        }
    }

    println!("\n=== RECONSTRUCTED WORLD CAMERA CENTER (inv_scale=100, Y-up mesh px) ===");
    let inv = 100.0f32;
    let static_pos = |tf: i64| local(tf).0;
    let sample = |c: &Vec<(f32, f32)>, t: f32, fb: f32| -> f32 {
        if c.is_empty() {
            return fb;
        }
        if t <= c[0].0 {
            return c[0].1;
        }
        for w in c.windows(2) {
            if t <= w[1].0 {
                let (t0, v0) = w[0];
                let (t1, v1) = w[1];
                return if t1 > t0 {
                    v0 + (v1 - v0) * (t - t0) / (t1 - t0)
                } else {
                    v0
                };
            }
        }
        c[c.len() - 1].1
    };
    let world_at = |t: f32| -> M {
        let mut m = M::id();
        for &tf in chain.iter().rev() {
            let (pos, q, s) = local(tf);
            let p = match animated.get(&tf) {
                Some(cs) => [
                    sample(&cs[0], t, static_pos(tf)[0]),
                    sample(&cs[1], t, static_pos(tf)[1]),
                    sample(&cs[2], t, static_pos(tf)[2]),
                ],
                None => pos,
            };
            m = m.mul(&M::trs(p, q, s));
        }
        m
    };
    let m0 = world_at(0.0);
    let axis = |col: usize, m: &M| {
        let c = [m.0[0][col], m.0[1][col], m.0[2][col]];
        let n = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().max(1e-6);
        [c[0] / n, c[1] / n, c[2] / n]
    };
    let right = axis(0, &m0);
    let up = axis(1, &m0);
    println!(
        "cam right axis (t0): [{:.3},{:.3},{:.3}]  up axis: [{:.3},{:.3},{:.3}]",
        right[0], right[1], right[2], up[0], up[1], up[2]
    );
    println!("  t     worldPos(x,y,z)                cx_px      cy_px");
    let mut t = 0.0f32;
    while t <= 16.01 {
        let p = world_at(t).point([0.0, 0.0, 0.0]);
        let cx = (p[0] * right[0] + p[1] * right[1] + p[2] * right[2]) * inv;
        let cy = -(p[0] * up[0] + p[1] * up[1] + p[2] * up[2]) * inv;
        println!(
            "  {:5.2}  [{:8.3},{:8.3},{:8.3}]   {:9.1}  {:9.1}",
            t, p[0], p[1], p[2], cx, cy
        );
        t += 1.0;
    }

    println!("\n=== GAP B: NON-POSITION VARYING BINDINGS IN ENTRANCE CLIPS ===");
    for (_p, v) in &entrance {
        let cname = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
        let Some(bindings) = v
            .get("m_ClipBindingConstant")
            .and_then(|b| b.get("genericBindings"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        let total = tf_total(v);
        let (sc, dc, _cc) = subclip_counts(v, total);
        let mut gidx = 0usize;
        for b in bindings {
            let (tid, attr, path) = binding_fields(b);
            let count = bcount(tid, attr);
            let is_transform_geo = tid == 4 && (1..=4).contains(&attr);
            if !is_transform_geo {
                let goname = hash_to_go
                    .get(&path)
                    .map(|g| go_path(*g))
                    .unwrap_or_else(|| format!("hash{path}"));
                for c_i in 0..count {
                    let gi = gidx + c_i;
                    let loc = if gi < sc {
                        "STREAM"
                    } else if gi < sc + dc {
                        "DENSE"
                    } else {
                        "CONST"
                    };
                    let c = decode_curve(v, gi, total);
                    if c.len() > 1 {
                        let (mn, mx) = c
                            .iter()
                            .fold((f32::MAX, f32::MIN), |(a, b), &(_, x)| (a.min(x), b.max(x)));
                        if (mx - mn).abs() > 1e-4 {
                            let peak =
                                c.iter()
                                    .cloned()
                                    .fold((0.0f32, f32::MIN), |(pt, pv), (t, x)| {
                                        if x > pv { (t, x) } else { (pt, pv) }
                                    });
                            println!(
                                "  clip '{}' tid={} attr={} GO '{}' comp{} [{}] keys={} t=[{:.2}..{:.2}] v0={:.4} vN={:.4} min={:.4} max={:.4} peak@t={:.2}",
                                cname,
                                tid,
                                attr,
                                goname,
                                c_i,
                                loc,
                                c.len(),
                                c.first().map(|x| x.0).unwrap_or(0.0),
                                c.last().map(|x| x.0).unwrap_or(0.0),
                                c.first().map(|x| x.1).unwrap_or(0.0),
                                c.last().map(|x| x.1).unwrap_or(0.0),
                                mn,
                                mx,
                                peak.0
                            );
                        }
                    }
                }
            }
            gidx += count;
        }
    }

    println!("\n=== GO NAMES (flash/white/light/glow/screen/mask/fade/over/bloom/shine/cover) ===");
    for (go, nm) in &go_name {
        let l = nm.to_ascii_lowercase();
        if [
            "flash", "white", "light", "glow", "screen", "mask", "fade", "over", "bloom", "shine",
            "cover",
        ]
        .iter()
        .any(|k| l.contains(k))
        {
            println!("  GO '{}' path={}", nm, go_path(*go));
        }
    }

    // Probe: baizhuanchang_02 GO active-chain (prefab m_IsActive) + reveal window
    println!("\n=== GAP B PROBE: baizhuanchang / white-transition GO active chain ===");
    let m_isactive = |go: i64| -> String {
        all.get(&go)
            .and_then(|(c, v)| {
                if *c == 1 {
                    v.get("m_IsActive").map(|x| format!("{x:?}"))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "MISSING".into())
    };
    for (go, nm) in &go_name {
        let l = nm.to_ascii_lowercase();
        if l.contains("baizhuanchang") || l.contains("zhuanchang") {
            println!(
                "  GO '{}' m_IsActive(prefab)={}  path={}",
                nm,
                m_isactive(*go),
                go_path(*go)
            );
            // walk ancestors
            let mut cur = go_tf.get(go).copied();
            let mut depth = 0;
            while let Some(tf) = cur {
                depth += 1;
                if depth > 20 {
                    break;
                }
                if let Some(g) = tf_go.get(&tf) {
                    println!(
                        "      anc GO '{}' m_IsActive={}",
                        name_of(*g),
                        m_isactive(*g)
                    );
                }
                cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
            }
        }
    }
    // reveal windows from active_windows on all GOs whose name is baizhuanchang
    println!("  -- active_windows() reveal for _Start-clip toggled GOs (baizhuanchang) --");
    // reproduce active_timeline for the _Start clip on baizhuanchang path
    for (_p, v) in &entrance {
        let cname = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
        if cname != "char_4064_Mlynar_epoque#28_Start" {
            continue;
        }
        let Some(bindings) = v
            .get("m_ClipBindingConstant")
            .and_then(|b| b.get("genericBindings"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        let total = tf_total(v);
        let (sc, _dc, _cc) = subclip_counts(v, total);
        let mut gidx = 0usize;
        for b in bindings {
            let (tid, attr, path) = binding_fields(b);
            let count = bcount(tid, attr);
            if tid == 1 && attr == 2086281974 {
                if let Some(&go) = hash_to_go.get(&path) {
                    let nm = name_of(go);
                    if nm.to_ascii_lowercase().contains("zhuanchang") {
                        let c = decode_curve(v, gidx.min(sc.saturating_sub(1)), total);
                        // print first on->off transition
                        let mut rev = None;
                        let mut prev: Option<bool> = None;
                        for (fi, (t, val)) in c.iter().enumerate() {
                            let on = *val >= 0.5;
                            if fi == 0 {
                                prev = Some(on);
                                continue;
                            }
                            if let Some(p) = prev {
                                if !p && on && rev.is_none() {
                                    rev = Some(*t);
                                }
                            }
                            prev = Some(on);
                        }
                        println!(
                            "    clip m_IsActive '{}' keys={} v0={:.2} reveal@={:?}",
                            nm,
                            c.len(),
                            c.first().map(|x| x.1).unwrap_or(-1.0),
                            rev
                        );
                    }
                }
            }
            gidx += count;
        }
    }

    // Replicate go_effectively_active (prefab m_IsActive + state-only-name), per class-23 renderer.
    println!("\n=== GAP B: class-23 renderers PASS/DROP (go_effectively_active) ===");
    let is_active_bool = |go: i64| -> bool {
        all.get(&go)
            .and_then(|(c, v)| {
                if *c == 1 {
                    v.get("m_IsActive").and_then(Value::as_bool)
                } else {
                    None
                }
            })
            .unwrap_or(true)
    };
    let eff_active = |go_pid: i64| -> (bool, String) {
        let state_only = [
            "start", "interact", "special", "skill", "attack", "die", "assist",
        ];
        let mut cur = match go_tf.get(&go_pid) {
            Some(&t) => t,
            None => return (true, "no-tf".into()),
        };
        for _ in 0..256 {
            let Some(g) = tf_go.get(&cur).copied() else {
                return (true, "walk-end".into());
            };
            if !is_active_bool(g) {
                return (false, format!("inactive:{}", name_of(g)));
            }
            let nm = name_of(g).to_ascii_lowercase();
            if nm.contains("only") && state_only.iter().any(|s| nm.contains(s)) {
                return (false, format!("state-only:{}", name_of(g)));
            }
            match tf_father.get(&cur).copied().filter(|&f| f != 0) {
                Some(f) => cur = f,
                None => return (true, "root".into()),
            }
        }
        (true, "maxdepth".into())
    };
    for (_p, (cid, v)) in &all {
        if *cid != 23 {
            continue;
        }
        if let Some(go) = v.get("m_GameObject").and_then(pid) {
            if spine_gos.contains(&go) {
                continue;
            }
            let (pass, reason) = eff_active(go);
            let top = go_path(go).split('/').next().unwrap_or("").to_string();
            println!(
                "  {} '{}' root={} reason={}",
                if pass { "PASS" } else { "DROP" },
                name_of(go),
                top,
                reason
            );
        }
    }

    println!("\n=== RENDERERS (23 MeshRenderer / 212 SpriteRenderer / 137 Skinned) ===");
    for (_p, (cid, v)) in &all {
        if [23i32, 212, 137].contains(cid) {
            if let Some(go) = v.get("m_GameObject").and_then(pid) {
                println!("  class{} GO '{}' path={}", cid, name_of(go), go_path(go));
            }
        }
    }
}
