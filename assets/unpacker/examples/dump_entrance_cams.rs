//! THROWAWAY diagnostic: enumerate every Camera (class 20), `AnimationClip` (class 74),
//! Animator/Animation/PlayableDirector, and `MonoScript` in a dynchar dynillust bundle,
//! and decode every Transform position/euler/scale curve that touches a camera-related
//! `GameObject` — to decide whether a baked "plunge" camera animation exists.
//!
//! Usage: cargo run --release --example `dump_entrance_cams` -- <bundle.ab> [more.ab ...]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::manual_checked_ops,
    clippy::or_fun_call
)]

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

/// Decode global curve index `idx` from a clip's streamed/dense/constant sub-clips.
fn decode_curve(clip: &Value, idx: usize, total: usize) -> Vec<(f32, f32)> {
    let Some(data) = clip
        .get("m_MuscleClip")
        .and_then(|m| m.get("m_Clip"))
        .and_then(|c| c.get("data"))
    else {
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
        // streamed frames: [time_f32, nkeys_u32, {index, c0..c3}*nkeys]
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

fn curve_summary(c: &[(f32, f32)]) -> String {
    if c.is_empty() {
        return "EMPTY".into();
    }
    let first = c[0].1;
    let last = c[c.len() - 1].1;
    let mut mn = f32::INFINITY;
    let mut mx = f32::NEG_INFINITY;
    for &(_, v) in c {
        mn = mn.min(v);
        mx = mx.max(v);
    }
    let inc = c.windows(2).all(|w| w[1].1 >= w[0].1 - 1e-6);
    let dec = c.windows(2).all(|w| w[1].1 <= w[0].1 + 1e-6);
    let mono = if c.len() < 2 || inc || dec {
        "monotonic"
    } else {
        "NON-MONOTONIC"
    };
    let eps = 1e-4_f32.max((mx - mn).abs() * 1e-3);
    let dips = mn < first - eps && last > mn + eps;
    format!(
        "{} keys t=[{:.3}..{:.3}] v: first={:.5} last={:.5} min={:.5} max={:.5} [{}]{}",
        c.len(),
        c[0].0,
        c[c.len() - 1].0,
        first,
        last,
        mn,
        mx,
        mono,
        if dips {
            " <<< DIPS-THEN-RECOVERS (plunge candidate)"
        } else {
            ""
        }
    )
}

fn main() {
    for path in std::env::args().skip(1) {
        analyze(&path);
    }
}

#[allow(clippy::too_many_lines)]
fn analyze(path: &str) {
    println!("\n================================================================");
    println!("BUNDLE {path}");
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            println!("  read error: {e}");
            return;
        }
    };
    let bundle = match BundleFile::parse(data) {
        Ok(b) => b,
        Err(e) => {
            println!("  bundle parse error: {e}");
            return;
        }
    };
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            println!("-- entry {} (resource data, skipped)", entry.path);
            continue;
        }
        let sf = match SerializedFile::parse(entry.data.clone()) {
            Ok(s) => s,
            Err(e) => {
                println!("-- entry {} parse error: {e}", entry.path);
                continue;
            }
        };
        println!(
            "-- entry {} : {} objects, unity {}",
            entry.path,
            sf.objects.len(),
            sf.unity_version
        );
        println!(
            "   externals: {:?}",
            sf.externals
                .iter()
                .map(unpacker::unity::serialized_file::FileIdentifier::cab_name)
                .collect::<Vec<_>>()
        );

        // Histogram + read everything except big binary payload classes.
        let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut hist: HashMap<i32, usize> = HashMap::new();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            *hist.entry(obj.class_id).or_insert(0) += 1;
            if skip.contains(&obj.class_id) {
                continue;
            }
            match read_object(&sf, obj) {
                Ok(v) => {
                    all.insert(obj.path_id, (obj.class_id, v));
                }
                Err(e) => println!(
                    "   ! read err class {} pid {}: {e}",
                    obj.class_id, obj.path_id
                ),
            }
        }
        let mut hv: Vec<_> = hist.into_iter().collect();
        hv.sort_unstable();
        println!("   class histogram: {hv:?}");

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

        // GameObjects with camera-ish names.
        let mut camish: Vec<String> = go_name
            .values()
            .filter(|n| n.to_ascii_lowercase().contains("cam"))
            .cloned()
            .collect();
        camish.sort();
        camish.dedup();
        println!("   GO names containing 'cam': {camish:?}");

        // ---- Cameras (class 20) ----
        let mut cam_gos: Vec<i64> = Vec::new();
        for (p, (cid, v)) in &all {
            if *cid != 20 {
                continue;
            }
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            cam_gos.push(go);
            println!(
                "\n   CAMERA pid={p} on GO '{}' path='{}'",
                go_name.get(&go).cloned().unwrap_or_default(),
                go_path(go)
            );
            // Print camera component fields verbatim (it's small).
            println!(
                "     component: {}",
                serde_json::to_string(v).unwrap_or_default()
            );
            // Static transform chain camera -> root.
            let mut cur = go_tf.get(&go).copied();
            for _ in 0..64 {
                let Some(tf) = cur else { break };
                if let Some((_, tv)) = all.get(&tf) {
                    let g = tf_go.get(&tf).copied().unwrap_or(0);
                    let f = |field: &str, k: &str| {
                        tv.get(field)
                            .and_then(|x| x.get(k))
                            .and_then(Value::as_f64)
                            .unwrap_or(0.0)
                    };
                    println!(
                        "     chain '{}' localPos=({:.4},{:.4},{:.4}) rot=({:.3},{:.3},{:.3},{:.3}) scale=({:.3},{:.3},{:.3})",
                        go_name.get(&g).cloned().unwrap_or_default(),
                        f("m_LocalPosition", "x"),
                        f("m_LocalPosition", "y"),
                        f("m_LocalPosition", "z"),
                        f("m_LocalRotation", "x"),
                        f("m_LocalRotation", "y"),
                        f("m_LocalRotation", "z"),
                        f("m_LocalRotation", "w"),
                        f("m_LocalScale", "x"),
                        f("m_LocalScale", "y"),
                        f("m_LocalScale", "z"),
                    );
                }
                cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
            }
        }

        // Camera-related GO set: each camera GO + all its ancestors.
        let mut cam_related: HashSet<i64> = HashSet::new();
        for &cg in &cam_gos {
            let mut cur = go_tf.get(&cg).copied();
            for _ in 0..64 {
                let Some(tf) = cur else { break };
                if let Some(&g) = tf_go.get(&tf) {
                    cam_related.insert(g);
                }
                cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
            }
        }

        // ---- Animator / Animation / PlayableDirector / MonoBehaviour / MonoScript ----
        for (p, (cid, v)) in &all {
            match cid {
                95 | 111 | 320 => {
                    let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                    let kind = match cid {
                        95 => "Animator",
                        111 => "Animation",
                        _ => "PlayableDirector",
                    };
                    println!("   {kind} pid={p} on '{}'", go_path(go));
                }
                115 => {
                    println!(
                        "   MonoScript pid={p} class='{}' ns='{}' asm='{}'",
                        v.get("m_ClassName").and_then(Value::as_str).unwrap_or(""),
                        v.get("m_Namespace").and_then(Value::as_str).unwrap_or(""),
                        v.get("m_AssemblyName")
                            .and_then(Value::as_str)
                            .unwrap_or(""),
                    );
                }
                _ => {}
            }
        }
        // MonoBehaviours attached to camera-related GOs (director / settings components).
        for (p, (cid, v)) in &all {
            if *cid != 114 {
                continue;
            }
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            let script = v.get("m_Script").and_then(pid).unwrap_or(0);
            let sname = all
                .get(&script)
                .and_then(|(_, sv)| sv.get("m_ClassName"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            if cam_related.contains(&go)
                || sname.to_ascii_lowercase().contains("cam")
                || sname.to_ascii_lowercase().contains("director")
            {
                let js = serde_json::to_string(v).unwrap_or_default();
                let js = if js.len() > 4000 {
                    format!("{}…({} bytes)", &js[..4000], js.len())
                } else {
                    js
                };
                println!(
                    "   MonoBehaviour pid={p} script='{sname}' on '{}': {js}",
                    go_path(go)
                );
            }
        }

        // ---- hash -> GO (all root-relative subpaths) ----
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

        // Known attribute CRCs for non-Transform bindings.
        let mut attr_names: HashMap<u32, &str> = HashMap::new();
        for n in [
            "orthographic size",
            "field of view",
            "m_Enabled",
            "m_IsActive",
            "near clip plane",
            "far clip plane",
            "m_FocalLength",
            "m_BackGroundColor.r",
            "m_BackGroundColor.g",
            "m_BackGroundColor.b",
            "m_BackGroundColor.a",
        ] {
            attr_names.insert(crc32(n.as_bytes()), n);
        }

        // ---- AnimationClips ----
        let mut clips: Vec<(&i64, &Value)> = all
            .iter()
            .filter(|(_, (c, _))| *c == 74)
            .map(|(p, (_, v))| (p, v))
            .collect();
        clips.sort_by_key(|(p, _)| **p);
        for (p, v) in clips {
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
            println!(
                "\n   CLIP pid={p} '{name}' stopTime={stop:.3}s bindings={} curves={total}",
                bindings.len()
            );
            let mut gidx = 0usize;
            let mut unresolved = 0usize;
            for b in &bindings {
                let (tid, attr, ph) = binding_fields(b);
                let n = binding_curve_count(tid, attr);
                let target = hash_to_go.get(&ph).copied();
                let tname = target.map_or_else(|| format!("UNRESOLVED(0x{ph:08x})"), go_path);
                if target.is_none() && ph != 0 {
                    unresolved += 1;
                }
                let is_cam = target.is_some_and(|g| cam_related.contains(&g));
                let attr_label = if tid == 4 {
                    match attr {
                        1 => "position".to_string(),
                        2 => "rotationQ".to_string(),
                        3 => "scale".to_string(),
                        4 => "euler".to_string(),
                        _ => format!("attr{attr}"),
                    }
                } else {
                    attr_names
                        .get(&(attr as u32))
                        .map_or_else(|| format!("crc:{attr}"), |s| (*s).to_string())
                };
                // Decode & summarize Transform pos/euler/scale/quat curves on camera-related
                // GOs, plus ANY class-20 (Camera) property curve, plus any UNRESOLVED transform
                // curve (could target an object outside this prefab).
                let interesting = (tid == 4 && (is_cam || target.is_none())) || tid == 20;
                if interesting {
                    println!(
                        "     [tid={tid} {attr_label}] -> {tname}{}",
                        if is_cam { "  (CAMERA CHAIN)" } else { "" }
                    );
                    let comp = ["x", "y", "z", "w"];
                    for c in 0..n {
                        let curve = decode_curve(v, gidx + c, total);
                        if !curve.is_empty() {
                            println!(
                                "       .{}: {}",
                                comp.get(c).unwrap_or(&"?"),
                                curve_summary(&curve)
                            );
                            // Full dump for non-constant camera-chain / camera-property curves.
                            let varies = curve
                                .iter()
                                .any(|&(_, val)| (val - curve[0].1).abs() > 1e-5);
                            if varies {
                                let pts: Vec<String> = curve
                                    .iter()
                                    .map(|(t, val)| format!("({t:.3},{val:.4})"))
                                    .collect();
                                println!("         {}", pts.join(" "));
                            }
                        }
                    }
                } else if tid == 4 {
                    // Non-camera transform curve: one-line note only if it moves.
                    let moved = (0..n).any(|c| {
                        let cu = decode_curve(v, gidx + c, total);
                        cu.len() > 1 && cu.iter().any(|&(_, val)| (val - cu[0].1).abs() > 1e-4)
                    });
                    if moved {
                        println!("     [tid=4 {attr_label}] -> {tname} (non-camera, animated)");
                    }
                }
                gidx += n;
            }
            if unresolved > 0 {
                println!("     ({unresolved} bindings had UNRESOLVED path hashes)");
            }
        }
    }
}
