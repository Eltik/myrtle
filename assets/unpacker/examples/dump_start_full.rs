//! THROWAWAY diagnostic (cello entrance audit): COMPLETE enumeration of the `_Start`
//! prefab's timeline — every binding of every `*start*` AnimationClip (all typeIDs, all
//! sub-clips, WITH cubic coefficients), the director `_effects[]` `_delayTime` schedule,
//! and a dense cubic-evaluated resample of the camera-chain curves — to adversarially
//! re-test whether the "apple-fall plunge" camera move is in the data and faithfully
//! extracted.
//!
//! Usage: cargo run --release --example dump_start_full -- <bundle.ab>

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

const fn binding_curve_count(tid: i64, attr: i64) -> usize {
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

/// A streamed key with its FULL cubic: value(t) = ((c0*dt + c1)*dt + c2)*dt + c3, dt = t - time.
#[derive(Clone, Copy)]
struct CubicKey {
    time: f32,
    c: [f32; 4],
}

/// All streamed cubic keys for global curve `idx` (boundary frames included; caller filters).
fn streamed_cubics(streamed_raw: &[u32], idx: usize) -> Vec<CubicKey> {
    let mut out = Vec::new();
    let mut i = 0usize;
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
                out.push(CubicKey {
                    time,
                    c: [
                        f32::from_bits(streamed_raw[i + 1]),
                        f32::from_bits(streamed_raw[i + 2]),
                        f32::from_bits(streamed_raw[i + 3]),
                        f32::from_bits(streamed_raw[i + 4]),
                    ],
                });
            }
            i += 5;
        }
    }
    out
}

fn eval_cubics(keys: &[CubicKey], t: f32) -> Option<f32> {
    let real: Vec<&CubicKey> = keys.iter().filter(|k| k.time.is_finite()).collect();
    if real.is_empty() {
        return None;
    }
    if t <= real[0].time {
        return Some(real[0].c[3]);
    }
    for w in real.windows(2) {
        if t < w[1].time {
            let dt = t - w[0].time;
            let c = w[0].c;
            return Some(((c[0] * dt + c[1]) * dt + c[2]) * dt + c[3]);
        }
    }
    Some(real[real.len() - 1].c[3])
}

struct SubClips {
    streamed: Vec<u32>,
    dense: Vec<f32>,
    dense_count: usize,
    dense_begin: f32,
    dense_rate: f32,
    constant: Vec<f32>,
    stream_count: usize,
}

fn sub_clips(clip: &Value, total: usize) -> Option<SubClips> {
    let data = clip
        .get("m_MuscleClip")
        .and_then(|m| m.get("m_Clip"))
        .and_then(|c| c.get("data"))?;
    let streamed = u32_array(data.get("m_StreamedClip").and_then(|s| s.get("data")));
    let dense = f32_array(data.get("m_DenseClip").and_then(|s| s.get("data")));
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
    let dense_rate = data
        .get("m_DenseClip")
        .and_then(|d| d.get("m_SampleRate"))
        .and_then(Value::as_f64)
        .unwrap_or(60.0) as f32;
    let constant = f32_array(data.get("m_ConstantClip").and_then(|s| s.get("data")));
    let stream_count = total.saturating_sub(dense_count + constant.len());
    Some(SubClips {
        streamed,
        dense,
        dense_count,
        dense_begin,
        dense_rate,
        constant,
        stream_count,
    })
}

/// Decode curve `idx` to (t, v) pairs from whichever sub-clip holds it.
fn decode_curve(sc: &SubClips, idx: usize) -> (Vec<(f32, f32)>, &'static str) {
    if idx < sc.stream_count {
        let keys = streamed_cubics(&sc.streamed, idx);
        let pts = keys
            .iter()
            .filter(|k| k.time.is_finite() && k.time >= 0.0)
            .map(|k| (k.time, k.c[3]))
            .collect();
        (pts, "streamed")
    } else if idx < sc.stream_count + sc.dense_count {
        let di = idx - sc.stream_count;
        let frames = if sc.dense_count > 0 {
            sc.dense.len() / sc.dense_count
        } else {
            0
        };
        let pts = (0..frames)
            .filter_map(|f| {
                sc.dense
                    .get(f * sc.dense_count + di)
                    .map(|&v| (sc.dense_begin + f as f32 / sc.dense_rate.max(1.0), v))
            })
            .collect();
        (pts, "dense")
    } else {
        let ci = idx - sc.stream_count - sc.dense_count;
        (
            sc.constant.get(ci).map(|&v| (0.0, v)).into_iter().collect(),
            "constant",
        )
    }
}

fn summarize(pts: &[(f32, f32)]) -> String {
    if pts.is_empty() {
        return "EMPTY".into();
    }
    let mut mn = f32::INFINITY;
    let mut mx = f32::NEG_INFINITY;
    for &(_, v) in pts {
        mn = mn.min(v);
        mx = mx.max(v);
    }
    let varies = (mx - mn).abs() > 1e-5;
    // Times where the value changes vs the previous key (transition times).
    let mut changes: Vec<String> = Vec::new();
    for w in pts.windows(2) {
        if (w[1].1 - w[0].1).abs() > 1e-5 {
            changes.push(format!("{:.2}s:{:.4}->{:.4}", w[1].0, w[0].1, w[1].1));
        }
    }
    if !varies {
        format!("CONST {:.5} ({} keys)", pts[0].1, pts.len())
    } else {
        format!(
            "{} keys [{:.3}..{:.3}] min={:.4} max={:.4} changes: {}",
            pts.len(),
            pts[0].0,
            pts[pts.len() - 1].0,
            mn,
            mx,
            changes.join(" ")
        )
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
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
        // Hierarchy maps.
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
        let go_path = |go: i64| -> String {
            let mut parts: Vec<String> = Vec::new();
            let mut cur = go_tf.get(&go).copied();
            for _ in 0..128 {
                let Some(tf) = cur else { break };
                if let Some(g) = tf_go.get(&tf) {
                    parts.push(go_name.get(g).cloned().unwrap_or_else(|| format!("?{g}")));
                }
                cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
            }
            parts.reverse();
            parts.join("/")
        };
        // hash -> GO for every root-relative subpath.
        let mut hash_to_go: HashMap<u32, i64> = HashMap::new();
        for tf in tf_go.keys() {
            let mut chain: Vec<String> = Vec::new();
            let mut cur = *tf;
            for _ in 0..128 {
                let Some(g) = tf_go.get(&cur) else { break };
                chain.push(go_name.get(g).cloned().unwrap_or_default());
                match tf_father.get(&cur) {
                    Some(f) if *f != 0 && tf_go.contains_key(f) => cur = *f,
                    _ => break,
                }
            }
            chain.reverse();
            for start in 0..chain.len() {
                hash_to_go
                    .entry(crc32(chain[start..].join("/").as_bytes()))
                    .or_insert(tf_go[tf]);
            }
        }
        // Known attribute names.
        let mut attr_names: HashMap<u32, String> = HashMap::new();
        for n in [
            "orthographic size",
            "field of view",
            "m_Enabled",
            "m_IsActive",
            "near clip plane",
            "far clip plane",
            "m_FocalLength",
            "m_Color.r",
            "m_Color.g",
            "m_Color.b",
            "m_Color.a",
            "m_Intensity",
            "m_AnchoredPosition.x",
            "m_AnchoredPosition.y",
        ] {
            attr_names.insert(crc32(n.as_bytes()), n.to_string());
        }
        for base in [
            "material._Color",
            "material._TintColor",
            "material._EmissionColor",
            "material._AddColor",
            "material._MultColor",
        ] {
            for c in ["r", "g", "b", "a"] {
                let n = format!("{base}.{c}");
                attr_names.insert(crc32(n.as_bytes()), n);
            }
        }
        for n in [
            "material._Alpha",
            "material._Intensity",
            "material._Progress",
            "material._Cutoff",
        ] {
            attr_names.insert(crc32(n.as_bytes()), n.to_string());
        }

        // ---- 1) Director `_effects[]` schedule ----
        println!("==== DIRECTOR / EFFECT SCHEDULE ====");
        for (p, (cid, v)) in &all {
            if *cid != 114 {
                continue;
            }
            if let Some(effects) = v.get("_effects").and_then(Value::as_array) {
                let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                println!("DIRECTOR pid={p} on '{}'", go_path(go));
                if let Some(params) = v.get("_params") {
                    println!("  _params = {params}");
                }
                for (i, e) in effects.iter().enumerate() {
                    let ep = pid(e).unwrap_or(0);
                    match all.get(&ep) {
                        Some((ecid, ev)) => {
                            let ego = ev.get("m_GameObject").and_then(pid).unwrap_or(0);
                            // Print the whole effect MB minus bulky arrays.
                            let mut compact = serde_json::Map::new();
                            if let Some(o) = ev.as_object() {
                                for (k, val) in o {
                                    let s = val.to_string();
                                    compact.insert(
                                        k.clone(),
                                        if s.len() > 200 {
                                            Value::String(format!("<{} bytes>", s.len()))
                                        } else {
                                            val.clone()
                                        },
                                    );
                                }
                            }
                            println!(
                                "  _effects[{i}] pid={ep} class={ecid} on '{}' -> {}",
                                go_path(ego),
                                Value::Object(compact)
                            );
                        }
                        None => println!("  _effects[{i}] pid={ep} (NOT READ)"),
                    }
                }
            }
        }
        // Any MonoBehaviour with a _delayTime (the per-effect schedule objects).
        println!("\n==== ALL MonoBehaviours with _delayTime ====");
        let mut delayed: Vec<(f64, String, i64, String)> = Vec::new();
        for (p, (cid, v)) in &all {
            if *cid != 114 {
                continue;
            }
            if let Some(dt) = v.get("_delayTime").and_then(Value::as_f64) {
                let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                let mut keys: Vec<&String> = v
                    .as_object()
                    .map(|o| o.keys().collect())
                    .unwrap_or_default();
                keys.sort();
                let small = v
                    .as_object()
                    .map(|o| {
                        o.iter()
                            .filter(|(k, val)| {
                                k.as_str() != "_delayTime" && val.to_string().len() <= 120
                            })
                            .map(|(k, val)| format!("{k}={val}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default();
                delayed.push((dt, go_path(go), *p, small));
            }
        }
        delayed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for (dt, path, p, small) in &delayed {
            println!("  delay={dt:6.2}s pid={p} '{path}'\n      {small}");
        }

        // ---- 2) Every `*start*` clip: EVERY binding, decoded ----
        println!("\n==== START CLIPS: COMPLETE BINDINGS ====");
        let mut clips: Vec<(&i64, &Value)> = all
            .iter()
            .filter(|(_, (c, v))| {
                *c == 74
                    && v.get("m_Name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_ascii_lowercase()
                        .contains("start")
            })
            .map(|(p, (_, v))| (p, v))
            .collect();
        clips.sort_by_key(|(p, _)| **p);
        for (p, v) in &clips {
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            let stop = v
                .get("m_MuscleClip")
                .and_then(|m| m.get("m_StopTime"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let bindings = v
                .get("m_ClipBindingConstant")
                .and_then(|b| b.get("genericBindings"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let total: usize = bindings
                .iter()
                .map(|b| {
                    let (t, a, _) = binding_fields(b);
                    binding_curve_count(t, a)
                })
                .sum();
            let Some(sc) = sub_clips(v, total) else {
                continue;
            };
            println!(
                "\nCLIP pid={p} '{name}' stop={stop:.3}s bindings={} curves={total} (stream={} dense={} const={})",
                bindings.len(),
                sc.stream_count,
                sc.dense_count,
                sc.constant.len()
            );
            let mut gidx = 0usize;
            for b in &bindings {
                let (tid, attr, ph) = binding_fields(b);
                let n = binding_curve_count(tid, attr);
                let tname = hash_to_go
                    .get(&ph)
                    .map_or_else(|| format!("UNRESOLVED(0x{ph:08x})"), |g| go_path(*g));
                let attr_label = if tid == 4 {
                    match attr {
                        1 => "position".into(),
                        2 => "rotationQ".into(),
                        3 => "scale".into(),
                        4 => "euler".into(),
                        _ => format!("attr{attr}"),
                    }
                } else {
                    attr_names
                        .get(&(attr as u32))
                        .cloned()
                        .unwrap_or_else(|| format!("crc:{}", attr as u32))
                };
                println!("  [tid={tid} {attr_label}] -> {tname}");
                let comp = ["x", "y", "z", "w"];
                for c in 0..n {
                    let (pts, kind) = decode_curve(&sc, gidx + c);
                    println!(
                        "    .{} ({kind}): {}",
                        comp.get(c).unwrap_or(&"?"),
                        summarize(&pts)
                    );
                }
                gidx += n;
            }
        }

        // ---- 3) Camera chain curves: cubic-evaluated dense resample ----
        println!("\n==== CAMERA CURVES (cubic-evaluated, every 0.1s) ====");
        // Find the start_animation_02 clip and the Dummy002 position + ortho indices.
        for (_, v) in &clips {
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            if !name.contains("start_animation") {
                continue;
            }
            let bindings = v
                .get("m_ClipBindingConstant")
                .and_then(|b| b.get("genericBindings"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let total: usize = bindings
                .iter()
                .map(|b| {
                    let (t, a, _) = binding_fields(b);
                    binding_curve_count(t, a)
                })
                .sum();
            let Some(sc) = sub_clips(v, total) else {
                continue;
            };
            let stop = v
                .get("m_MuscleClip")
                .and_then(|m| m.get("m_StopTime"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32;
            let mut gidx = 0usize;
            for b in &bindings {
                let (tid, attr, ph) = binding_fields(b);
                let n = binding_curve_count(tid, attr);
                let tname = hash_to_go
                    .get(&ph)
                    .map_or_else(|| format!("0x{ph:08x}"), |g| go_path(*g));
                let interesting =
                    (tid == 4 && attr == 1 && tname.contains("Dummy002")) || tid == 20;
                if interesting {
                    for c in 0..n {
                        let idx = gidx + c;
                        if idx >= sc.stream_count {
                            continue;
                        }
                        let keys = streamed_cubics(&sc.streamed, idx);
                        // Only dump curves that actually vary.
                        let vals: Vec<f32> = keys
                            .iter()
                            .filter(|k| k.time.is_finite())
                            .map(|k| k.c[3])
                            .collect();
                        let varies = vals
                            .iter()
                            .any(|&x| (x - vals.first().copied().unwrap_or(0.0)).abs() > 1e-4);
                        if !varies {
                            continue;
                        }
                        println!("\n'{name}' [tid={tid}] {tname} component {c}: RAW CUBIC KEYS");
                        for k in keys.iter().filter(|k| k.time.is_finite()) {
                            println!(
                                "  t={:8.3}  c0={:12.5} c1={:12.5} c2={:12.5} value={:12.5}",
                                k.time, k.c[0], k.c[1], k.c[2], k.c[3]
                            );
                        }
                        println!("  dense resample (t, value):");
                        let mut t = 0.0f32;
                        let mut line = String::new();
                        while t <= stop + 1e-3 {
                            if let Some(vv) = eval_cubics(&keys, t) {
                                line.push_str(&format!("({t:.1},{vv:.4}) "));
                            }
                            t += 0.1;
                            if line.len() > 150 {
                                println!("    {line}");
                                line.clear();
                            }
                        }
                        if !line.is_empty() {
                            println!("    {line}");
                        }
                    }
                }
                gidx += n;
            }
        }
    }
}
