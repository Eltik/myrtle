//! Decode a GENERIC `AnimationClip`'s compressed muscle clip and sample its curves.
//!
//! Executor's scope rim (`heip_01 (1)`) is animated by Transform POSITION and SCALE curves that
//! the exporter drops, because scene layers carry a single static `pos`
//! ([[dynchar-scene-layer-transform-curves]]). The values live in
//! `m_MuscleClip.m_Clip.data` split three ways, and `m_ClipBindingConstant.genericBindings`
//! indexes them:
//!
//!   * `StreamedClip` — a packed `uint32` stream of per-frame cubic segments; curve indices
//!     `[0, streamedCurveCount)`.
//!   * `DenseClip`    — flat float samples, `frameCount x curveCount`, at `m_SampleRate`.
//!   * `ConstantClip` — one value per curve, for curves that never change.
//!
//! ⚠️ A BINDING IS NOT A CURVE. A Transform position/scale binding covers THREE float curves and
//! a rotation FOUR, so the curve index of binding *i* is the running sum of the dimensions before
//! it. Executor's clip has 24 bindings but 30 curves (24 streamed + 6 constant), which is the
//! check that the accounting is right.
//!
//! Usage: cargo run --release --example `probe_curves` -- <bundle.ab> <clip-substr> [path-substr]
#![allow(
    // mul_add/hypot change float rounding, not just spelling; never worth it for byte-exact
    // parity output, even in a throwaway diagnostic.
    clippy::suboptimal_flops,
    clippy::imprecise_flops,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::while_float
)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn crc32(s: &str) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for b in s.as_bytes() {
        crc ^= u32::from(*b);
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

fn go_path(all: &HashMap<i64, (i32, Value)>, go_pid: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = Some(go_pid);
    for _ in 0..64 {
        let Some(pid) = cur else { break };
        let Some((_, gv)) = all.get(&pid) else { break };
        parts.push(
            gv.get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string(),
        );
        let tf = gv
            .get("m_Component")
            .and_then(Value::as_array)
            .and_then(|c| {
                c.iter().find_map(|x| {
                    let p = x.get("component")?.get("m_PathID")?.as_i64()?;
                    if all.get(&p)?.0 == 4 { Some(p) } else { None }
                })
            });
        cur = tf
            .and_then(|t| all.get(&t))
            .and_then(|(_, tv)| tv.get("m_Father"))
            .and_then(|f| f.get("m_PathID"))
            .and_then(Value::as_i64)
            .filter(|p| *p != 0)
            .and_then(|p| all.get(&p))
            .and_then(|(_, pv)| pv.get("m_GameObject"))
            .and_then(|g| g.get("m_PathID"))
            .and_then(Value::as_i64);
    }
    parts.reverse();
    parts.join("/")
}

fn u32s(v: Option<&Value>) -> Vec<u32> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_u64().map(|n| n as u32))
                .collect()
        })
        .unwrap_or_default()
}
fn f32s(v: Option<&Value>) -> Vec<f32> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_f64().map(|n| n as f32))
                .collect()
        })
        .unwrap_or_default()
}
const fn bits(x: u32) -> f32 {
    f32::from_bits(x)
}

/// One cubic segment of a streamed curve: value(t) = ((c0*d + c1)*d + c2)*d + c3, d = t - time.
struct Seg {
    time: f32,
    c: [f32; 4],
}

/// Decode the packed stream into per-curve segment lists.
fn decode_streamed(data: &[u32]) -> HashMap<u32, Vec<Seg>> {
    let mut out: HashMap<u32, Vec<Seg>> = HashMap::new();
    let mut i = 0usize;
    while i + 1 < data.len() {
        let time = bits(data[i]);
        let n = data[i + 1] as usize;
        i += 2;
        for _ in 0..n {
            if i + 4 >= data.len() {
                return out;
            }
            let idx = data[i];
            let c = [
                bits(data[i + 1]),
                bits(data[i + 2]),
                bits(data[i + 3]),
                bits(data[i + 4]),
            ];
            i += 5;
            out.entry(idx).or_default().push(Seg { time, c });
        }
    }
    out
}

fn sample(segs: &[Seg], t: f32) -> Option<f32> {
    if segs.is_empty() {
        return None;
    }
    // last segment starting at or before t
    let mut k = 0usize;
    for (j, s) in segs.iter().enumerate() {
        if s.time <= t {
            k = j;
        } else {
            break;
        }
    }
    let s = &segs[k];
    let d = t - s.time;
    Some(((s.c[0] * d + s.c[1]) * d + s.c[2]) * d + s.c[3])
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe_curves <bundle.ab> <clip> [pathfilter]");
    let clipwant = std::env::args().nth(2).unwrap_or_default();
    let pathwant = std::env::args()
        .nth(3)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let l = entry.path.to_ascii_lowercase();
        if l.ends_with(".ress") || l.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for o in &sf.objects {
            if let Ok(v) = read_object(&sf, o) {
                all.insert(o.path_id, (o.class_id, v));
            }
        }
        let mut hash_to_path: HashMap<u32, String> = HashMap::new();
        for (pid, (cid, _)) in &all {
            if *cid != 1 {
                continue;
            }
            let full = go_path(&all, *pid);
            let segs: Vec<&str> = full.split('/').collect();
            for i in 0..segs.len() {
                let rel = segs[i..].join("/");
                hash_to_path.entry(crc32(&rel)).or_insert(rel);
            }
        }

        for (cid, v) in all.values() {
            if *cid != 74 {
                continue;
            }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            // A leading '=' means EXACT match. Substring matching alone picks whichever of
            // `_Start`, `_Start_Idle`, `_Start_Mlynar_L_Sword2`, ... the HashMap happens to
            // yield first, which is not reproducible between runs.
            let hit = if let Some(exact) = clipwant.strip_prefix('=') {
                name == exact
            } else {
                clipwant.is_empty() || name.contains(&clipwant)
            };
            if !hit {
                continue;
            }
            let Some(binds) = v
                .get("m_ClipBindingConstant")
                .and_then(|c| c.get("genericBindings"))
                .and_then(Value::as_array)
            else {
                continue;
            };
            let clip = v
                .get("m_MuscleClip")
                .and_then(|m| m.get("m_Clip"))
                .and_then(|c| c.get("data"));
            let streamed = u32s(
                clip.and_then(|c| c.get("m_StreamedClip"))
                    .and_then(|s| s.get("data")),
            );
            let sc = clip
                .and_then(|c| c.get("m_StreamedClip"))
                .and_then(|s| s.get("curveCount"))
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32;
            let dense = f32s(
                clip.and_then(|c| c.get("m_DenseClip"))
                    .and_then(|d| d.get("m_SampleArray")),
            );
            let dc = clip
                .and_then(|c| c.get("m_DenseClip"))
                .and_then(|d| d.get("m_CurveCount"))
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32;
            let drate = clip
                .and_then(|c| c.get("m_DenseClip"))
                .and_then(|d| d.get("m_SampleRate"))
                .and_then(Value::as_f64)
                .unwrap_or(30.0) as f32;
            let dbegin = clip
                .and_then(|c| c.get("m_DenseClip"))
                .and_then(|d| d.get("m_BeginTime"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32;
            let constant = f32s(
                clip.and_then(|c| c.get("m_ConstantClip"))
                    .and_then(|d| d.get("data")),
            );

            let segs = decode_streamed(&streamed);
            println!(
                "== [{name}]  bindings {}  streamed {sc}  dense {dc}  constant {}",
                binds.len(),
                constant.len()
            );

            // Expand bindings -> curve index ranges. Position/scale = 3, rotation = 4, else 1.
            let mut off: u32 = 0;
            let mut total: u32 = 0;
            let mut rows: Vec<(String, String, u32, u32)> = Vec::new();
            for b in binds {
                let p = b.get("path").and_then(Value::as_u64).unwrap_or(0) as u32;
                let a = b.get("attribute").and_then(Value::as_u64).unwrap_or(0) as u32;
                let ty = b.get("typeID").and_then(Value::as_u64).unwrap_or(0);
                let dim = if ty == 4 {
                    match a {
                        1 | 3 | 4 => 3,
                        2 => 4,
                        _ => 1,
                    }
                } else {
                    1
                };
                let pname = hash_to_path
                    .get(&p)
                    .cloned()
                    .unwrap_or_else(|| format!("<{p:#x}>"));
                let aname = if ty == 4 {
                    match a {
                        1 => "position",
                        2 => "rotation",
                        3 => "scale",
                        4 => "euler",
                        _ => "?",
                    }
                    .to_string()
                } else {
                    format!("{a:#x}")
                };
                rows.push((pname, aname, off, dim));
                off += dim;
                total += dim;
            }
            println!(
                "   expanded to {total} float curve(s)  [streamed {sc} + dense {dc} + constant {}]",
                constant.len()
            );

            let val = |ci: u32, t: f32| -> Option<f32> {
                if ci < sc {
                    return segs.get(&ci).and_then(|s| sample(s, t));
                }
                if ci < sc + dc {
                    let di = (ci - sc) as usize;
                    let fr = (((t - dbegin) * drate).max(0.0) as usize).min(if dc > 0 {
                        dense.len() / dc as usize - 1
                    } else {
                        0
                    });
                    return dense.get(fr * dc as usize + di).copied();
                }
                constant.get((ci - sc - dc) as usize).copied()
            };

            for (pname, aname, o, dim) in &rows {
                if !pathwant.is_empty() && !pname.to_ascii_lowercase().contains(&pathwant) {
                    continue;
                }
                if *dim != 3 {
                    continue; // only position/scale are interesting here
                }
                println!("\n   -- {pname}  {aname}  (curves {}..{})", o, o + dim - 1);
                println!("      {:>6} {:>12} {:>12} {:>12}", "t", "x", "y", "z");
                let mut t = 0.0f32;
                while t <= 6.6 {
                    let x = val(*o, t).unwrap_or(f32::NAN);
                    let y = val(o + 1, t).unwrap_or(f32::NAN);
                    let z = val(o + 2, t).unwrap_or(f32::NAN);
                    println!("      {t:>6.2} {x:>12.4} {y:>12.4} {z:>12.4}");
                    t += 0.4;
                }
            }
            return;
        }
    }
}
