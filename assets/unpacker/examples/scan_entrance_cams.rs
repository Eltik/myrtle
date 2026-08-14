//! Scan ALL dynchar skin bundles for a baked ENTRANCE (`_Start`) camera movement and
//! classify each skin's camera curve: does the accumulated camera-centre POSITION move
//! monotonically (a plain dolly — the plunge, if any, is added by encrypted runtime code),
//! or does some axis reverse direction (a real BAKED plunge we could extract)?
//!
//! Per bundle this mirrors `export::anim::entrance_camera_track`: find the class-20 Camera,
//! walk its Transform ancestor chain, decode every `_Start`-clip position curve targeting a
//! chain node, then accumulate the full TRS chain per keyframe to get the camera's WORLD
//! centre over time. The Camera's animated `orthographic size` curve is decoded too (zoom is
//! expected to be non-monotonic — e.g. cello 1.87→1.50→1.91 — and is NOT a plunge signal).
//!
//! Usage:
//!   cargo run --release --example `scan_entrance_cams`                     # default roots
//!   cargo run --release --example `scan_entrance_cams` -- <dir-or-.ab>...  # explicit inputs
//!
//! Default roots: `assets/ArkAssets/{en,cn}/arts/dynchars/char_*.ab`, deduped by bundle
//! basename (EN preferred). Writes `assets/il2cpp-work/entrance_camera_catalog.json` and
//! prints a per-skin table + flagged-plunge summary. Read-only over the bundles.
#![allow(
    // mul_add/hypot change float rounding, not just spelling; never worth it for byte-exact
    // parity output, even in a throwaway diagnostic.
    clippy::suboptimal_flops,
    clippy::imprecise_flops,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::index_refutable_slice,
    clippy::items_after_statements,
    clippy::manual_checked_ops,
    clippy::many_single_char_names,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::similar_names,
    clippy::struct_excessive_bools,
    clippy::too_many_lines
)]

use rayon::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

// ---------------------------------------------------------------------------
// Small helpers shared with dump_entrance_cams / export::anim (self-contained
// copies so the library's behavior is untouched).
// ---------------------------------------------------------------------------

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

/// Whether a clip's name marks it as part of the `_Start` entrance cinematic
/// (same predicate as `export::anim::is_entrance_clip`).
fn is_entrance_clip(clip: &Value) -> bool {
    let name = clip
        .get("m_Name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    name.contains("start") || name.contains("entrance") || name.contains("enter")
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

// ---------------------------------------------------------------------------
// Minimal column-major-ish 4x4 (row-of-rows) TRS math, mirroring export::mesh::Mat4
// usage inside entrance_camera_track.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Mat4([[f32; 4]; 4]);

impl Mat4 {
    const fn identity() -> Self {
        let mut m = [[0.0f32; 4]; 4];
        m[0][0] = 1.0;
        m[1][1] = 1.0;
        m[2][2] = 1.0;
        m[3][3] = 1.0;
        Self(m)
    }

    fn trs(p: [f32; 3], q: [f32; 4], s: [f32; 3]) -> Self {
        let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
        let r = [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y - z * w),
                2.0 * (x * z + y * w),
            ],
            [
                2.0 * (x * y + z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z - x * w),
            ],
            [
                2.0 * (x * z - y * w),
                2.0 * (y * z + x * w),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ];
        let mut m = [[0.0f32; 4]; 4];
        for (i, row) in r.iter().enumerate() {
            for (j, &v) in row.iter().enumerate() {
                m[i][j] = v * s[j];
            }
            m[i][3] = p[i];
        }
        m[3][3] = 1.0;
        Self(m)
    }

    fn mul(&self, o: &Self) -> Self {
        let mut m = [[0.0f32; 4]; 4];
        for (i, row) in m.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..4).map(|k| self.0[i][k] * o.0[k][j]).sum();
            }
        }
        Self(m)
    }

    fn point(&self, p: [f32; 3]) -> [f32; 3] {
        let mut out = [0.0f32; 3];
        for (i, o) in out.iter_mut().enumerate() {
            *o = self.0[i][0] * p[0] + self.0[i][1] * p[1] + self.0[i][2] * p[2] + self.0[i][3];
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Catalog records.
// ---------------------------------------------------------------------------

/// Per-axis curve summary + monotonicity/plunge classification.
#[derive(Serialize, Clone)]
struct AxisSummary {
    axis: String,
    keys: usize,
    start: f32,
    end: f32,
    min: f32,
    max: f32,
    /// Time (s) at which the minimum / maximum are reached.
    t_min: f32,
    t_max: f32,
    /// "constant" | "increasing" | "decreasing" | "non-monotonic"
    monotonic: String,
    /// DOWN-then-UP: a local min below the start value that the curve recovers from
    /// — the baked-plunge signature.
    dips_then_recovers: bool,
    /// UP-then-DOWN mirror (overshoot): rises above start, then falls back.
    rises_then_falls: bool,
}

fn summarize_axis(axis: &str, c: &[(f32, f32)]) -> AxisSummary {
    let (mut mn, mut mx, mut t_mn, mut t_mx) = (f32::INFINITY, f32::NEG_INFINITY, 0.0f32, 0.0f32);
    for &(t, v) in c {
        if v < mn {
            mn = v;
            t_mn = t;
        }
        if v > mx {
            mx = v;
            t_mx = t;
        }
    }
    let (start, end) = (
        c.first().map_or(0.0, |k| k.1),
        c.last().map_or(0.0, |k| k.1),
    );
    let range = mx - mn;
    // Tolerance: ignore numeric wobble below 2% of the axis's own travel (and an absolute floor).
    let tol = 1e-4f32.max(range * 0.02);
    let monotonic = if range < 1e-5 {
        "constant"
    } else {
        // Max reverse excursion against each direction (drawdown / drawup).
        let (mut run_max, mut run_min) = (f32::NEG_INFINITY, f32::INFINITY);
        let (mut drawdown, mut drawup) = (0.0f32, 0.0f32);
        for &(_, v) in c {
            run_max = run_max.max(v);
            run_min = run_min.min(v);
            drawdown = drawdown.max(run_max - v); // violates "increasing"
            drawup = drawup.max(v - run_min); // violates "decreasing"
        }
        if drawdown <= tol {
            "increasing"
        } else if drawup <= tol {
            "decreasing"
        } else {
            "non-monotonic"
        }
    };
    AxisSummary {
        axis: axis.to_string(),
        keys: c.len(),
        start,
        end,
        min: mn,
        max: mx,
        t_min: t_mn,
        t_max: t_mx,
        monotonic: monotonic.to_string(),
        dips_then_recovers: range >= 1e-5 && (start - mn) > tol && (end - mn) > tol,
        rises_then_falls: range >= 1e-5 && (mx - start) > tol && (mx - end) > tol,
    }
}

/// One animated Transform node on the camera's ancestor chain.
#[derive(Serialize)]
struct NodeCurves {
    node_path: String,
    axes: Vec<AxisSummary>,
}

#[derive(Serialize)]
struct SkinRecord {
    skin: String,
    region: String,
    bundle: String,
    has_entrance_clip: bool,
    entrance_clips: Vec<String>,
    camera_count: usize,
    camera_path: Option<String>,
    /// Any `_Start` clip animates a position on the camera's ancestor chain.
    has_camera_movement: bool,
    /// Rotation/scale curves on the chain (informational; never seen so far).
    has_camera_rotation_anim: bool,
    /// Raw per-node position curves (world accumulation inputs).
    animated_nodes: Vec<NodeCurves>,
    /// Accumulated WORLD camera-centre curve summaries (x/y/z), from the full TRS chain
    /// with every animated ancestor's position substituted per keyframe.
    centre: Option<Vec<AxisSummary>>,
    /// Camera `orthographic size` curve from the `_Start` clip (zoom; non-monotonic is
    /// NORMAL here and not a plunge).
    ortho: Option<AxisSummary>,
    /// "no-entrance-clip" | "no-camera" | "static-camera" | "monotonic-dolly" | "NON-MONOTONIC"
    classification: String,
    /// True baked-plunge signature: some world-centre axis dips below its start value
    /// and recovers.
    plunge: bool,
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// Per-bundle analysis.
// ---------------------------------------------------------------------------

struct EntryAnalysis {
    entrance_clips: Vec<String>,
    camera_count: usize,
    camera_path: Option<String>,
    has_rotation_anim: bool,
    animated_nodes: Vec<NodeCurves>,
    centre: Option<Vec<AxisSummary>>,
    ortho: Option<AxisSummary>,
    has_camera_movement: bool,
}

#[allow(clippy::too_many_lines)]
fn analyze_entry(sf: &SerializedFile) -> EntryAnalysis {
    // Read every object except big binary payload classes (textures/meshes/audio/...).
    let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
    let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
    for obj in &sf.objects {
        if skip.contains(&obj.class_id) {
            continue;
        }
        if let Ok(v) = read_object(sf, obj) {
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

    // Entrance clips.
    let entrance_clips: Vec<String> = {
        let mut v: Vec<String> = all
            .values()
            .filter(|(c, val)| *c == 74 && is_entrance_clip(val))
            .map(|(_, val)| {
                val.get("m_Name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        v.sort();
        v
    };

    // First camera (class 20) — matches entrance_camera_track, which uses one camera.
    let cameras: Vec<i64> = all
        .iter()
        .filter(|(_, (c, _))| *c == 20)
        .filter_map(|(_, (_, v))| v.get("m_GameObject").and_then(pid))
        .collect();
    let camera_count = cameras.len();
    let cam_go = cameras.first().copied();
    let camera_path = cam_go.map(go_path);

    // Camera ancestor chain (transform pids, camera→root) + GO set.
    let mut chain: Vec<i64> = Vec::new();
    let mut cur = cam_go.and_then(|g| go_tf.get(&g).copied());
    for _ in 0..64 {
        let Some(tf) = cur else { break };
        chain.push(tf);
        cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
    }
    let chain_gos: HashSet<i64> = chain
        .iter()
        .filter_map(|tf| tf_go.get(tf).copied())
        .collect();

    // hash -> GO over all root-relative subpaths.
    let mut hash_to_go: HashMap<u32, i64> = HashMap::new();
    for tf in tf_go.keys() {
        let mut names: Vec<String> = Vec::new();
        let mut cur = *tf;
        for _ in 0..128 {
            let Some(g) = tf_go.get(&cur) else { break };
            names.push(go_name.get(g).cloned().unwrap_or_default());
            match tf_father.get(&cur) {
                Some(f) if *f != 0 && tf_go.contains_key(f) => cur = *f,
                _ => break,
            }
        }
        names.reverse();
        for start in 0..names.len() {
            hash_to_go
                .entry(crc32(names[start..].join("/").as_bytes()))
                .or_insert(tf_go[tf]);
        }
    }

    const ORTHO_SIZE_CRC: i64 = 2_389_637_943; // CRC32("orthographic size")

    // Scan every entrance clip's bindings: chain position curves, chain rot/scale
    // presence, and the Camera ortho-size curve.
    let mut animated: HashMap<i64, [Vec<(f32, f32)>; 3]> = HashMap::new(); // tf pid -> xyz curves
    let mut has_rotation_anim = false;
    let mut ortho_curve: Option<Vec<(f32, f32)>> = None;
    for (cid, v) in all.values() {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
        let Some(bindings) = v
            .get("m_ClipBindingConstant")
            .and_then(|b| b.get("genericBindings"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        let total: usize = bindings
            .iter()
            .map(|b| {
                let (t, a, _) = binding_fields(b);
                binding_curve_count(t, a)
            })
            .sum();
        let mut gidx = 0usize;
        for b in bindings {
            let (tid, attr, ph) = binding_fields(b);
            let n = binding_curve_count(tid, attr);
            let target = hash_to_go.get(&ph).copied();
            let on_chain = target.is_some_and(|g| chain_gos.contains(&g));
            if tid == 4 && attr == 1 && on_chain {
                let tf = target.and_then(|g| go_tf.get(&g).copied());
                if let Some(tf) = tf {
                    let cs = [
                        decode_curve(v, gidx, total),
                        decode_curve(v, gidx + 1, total),
                        decode_curve(v, gidx + 2, total),
                    ];
                    if cs.iter().any(|c| c.len() > 1) {
                        let entry = animated
                            .entry(tf)
                            .or_insert_with(|| [Vec::new(), Vec::new(), Vec::new()]);
                        // Same transform in multiple clips: keep the richer curve per axis.
                        for (i, c) in cs.into_iter().enumerate() {
                            if c.len() > entry[i].len() {
                                entry[i] = c;
                            }
                        }
                    }
                }
            } else if tid == 4 && matches!(attr, 2..=4) && on_chain {
                // Rotation/scale on the camera chain: note if it actually varies.
                let varies = (0..n).any(|c| {
                    let cu = decode_curve(v, gidx + c, total);
                    cu.len() > 1 && cu.iter().any(|&(_, val)| (val - cu[0].1).abs() > 1e-4)
                });
                has_rotation_anim |= varies;
            } else if tid == 20 && attr == ORTHO_SIZE_CRC {
                let c = decode_curve(v, gidx, total);
                if !c.is_empty() && ortho_curve.as_ref().is_none_or(|b| c.len() > b.len()) {
                    ortho_curve = Some(c);
                }
            }
            gidx += n;
        }
    }

    // Accumulate the WORLD camera centre over the union keyframe timeline.
    let centre = (!animated.is_empty() && !chain.is_empty()).then(|| {
        let mut times: Vec<f32> = animated
            .values()
            .flatten()
            .flatten()
            .map(|(t, _)| *t)
            .collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        times.dedup();
        let sample = |c: &Vec<(f32, f32)>, t: f32, fallback: f32| -> f32 {
            if c.is_empty() {
                return fallback;
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
        let static_pos = |tf: i64| -> [f32; 3] {
            all.get(&tf).map_or([0.0; 3], |(_, v)| {
                let g = |k: &str| {
                    v.get("m_LocalPosition")
                        .and_then(|x| x.get(k))
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0) as f32
                };
                [g("x"), g("y"), g("z")]
            })
        };
        let local_trs = |tf: i64, pos_override: Option<[f32; 3]>| -> Mat4 {
            let Some((_, v)) = all.get(&tf) else {
                return Mat4::identity();
            };
            let pos = pos_override.unwrap_or_else(|| static_pos(tf));
            let q = {
                let g = |k: &str, d: f32| {
                    v.get("m_LocalRotation")
                        .and_then(|x| x.get(k))
                        .and_then(Value::as_f64)
                        .unwrap_or(d.into()) as f32
                };
                [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
            };
            let mut s = {
                let g = |k: &str| {
                    v.get("m_LocalScale")
                        .and_then(|x| x.get(k))
                        .and_then(Value::as_f64)
                        .unwrap_or(1.0) as f32
                };
                [g("x"), g("y"), g("z")]
            };
            for c in &mut s {
                if c.abs() < 1e-6 {
                    *c = 1.0; // zero scale on an unused axis: don't collapse the chain
                }
            }
            Mat4::trs(pos, q, s)
        };
        let world_at = |t: f32| -> [f32; 3] {
            let mut m = Mat4::identity();
            for &tf in chain.iter().rev() {
                let local = if let Some(curves) = animated.get(&tf) {
                    let sp = static_pos(tf);
                    local_trs(
                        tf,
                        Some([
                            sample(&curves[0], t, sp[0]),
                            sample(&curves[1], t, sp[1]),
                            sample(&curves[2], t, sp[2]),
                        ]),
                    )
                } else {
                    local_trs(tf, None)
                };
                m = m.mul(&local);
            }
            m.point([0.0, 0.0, 0.0])
        };
        let mut xyz: [Vec<(f32, f32)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for &t in &times {
            let p = world_at(t);
            for (i, c) in xyz.iter_mut().enumerate() {
                c.push((t, p[i]));
            }
        }
        ["x", "y", "z"]
            .iter()
            .zip(xyz.iter())
            .map(|(a, c)| summarize_axis(a, c))
            .collect::<Vec<_>>()
    });

    // Per-node raw curve summaries (world accumulation inputs, for the catalog).
    let mut animated_nodes: Vec<NodeCurves> = animated
        .iter()
        .map(|(tf, curves)| {
            let go = tf_go.get(tf).copied().unwrap_or(0);
            NodeCurves {
                node_path: go_path(go),
                axes: ["x", "y", "z"]
                    .iter()
                    .zip(curves.iter())
                    .filter(|(_, c)| !c.is_empty())
                    .map(|(a, c)| summarize_axis(a, c))
                    .collect(),
            }
        })
        .collect();
    animated_nodes.sort_by(|a, b| a.node_path.cmp(&b.node_path));

    EntryAnalysis {
        entrance_clips,
        camera_count,
        camera_path,
        has_rotation_anim,
        has_camera_movement: !animated.is_empty(),
        animated_nodes,
        centre,
        ortho: ortho_curve.map(|c| summarize_axis("orthoSize", &c)),
    }
}

fn analyze_bundle(path: &Path, region: &str) -> SkinRecord {
    let skin = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string();
    let mut rec = SkinRecord {
        skin,
        region: region.to_string(),
        bundle: path.display().to_string(),
        has_entrance_clip: false,
        entrance_clips: Vec::new(),
        camera_count: 0,
        camera_path: None,
        has_camera_movement: false,
        has_camera_rotation_anim: false,
        animated_nodes: Vec::new(),
        centre: None,
        ortho: None,
        classification: String::new(),
        plunge: false,
        error: None,
    };
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            rec.error = Some(format!("read: {e}"));
            rec.classification = "error".into();
            return rec;
        }
    };
    let bundle = match BundleFile::parse(data) {
        Ok(b) => b,
        Err(e) => {
            rec.error = Some(format!("bundle: {e}"));
            rec.classification = "error".into();
            return rec;
        }
    };
    // Analyze each serialized entry; keep the best (entrance clip + camera > entrance > any).
    let mut best: Option<EntryAnalysis> = None;
    let score = |a: &EntryAnalysis| {
        u32::from(!a.entrance_clips.is_empty()) * 2 + u32::from(a.camera_count > 0)
    };
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let a = analyze_entry(&sf);
        if best.as_ref().is_none_or(|b| score(&a) > score(b)) {
            best = Some(a);
        }
    }
    let Some(a) = best else {
        rec.error = Some("no parsable serialized file".into());
        rec.classification = "error".into();
        return rec;
    };
    rec.has_entrance_clip = !a.entrance_clips.is_empty();
    rec.entrance_clips = a.entrance_clips;
    rec.camera_count = a.camera_count;
    rec.camera_path = a.camera_path;
    rec.has_camera_movement = a.has_camera_movement;
    rec.has_camera_rotation_anim = a.has_rotation_anim;
    rec.animated_nodes = a.animated_nodes;
    rec.centre = a.centre;
    rec.ortho = a.ortho;
    rec.plunge = rec
        .centre
        .as_ref()
        .is_some_and(|axes| axes.iter().any(|ax| ax.dips_then_recovers));
    let non_mono = rec
        .centre
        .as_ref()
        .is_some_and(|axes| axes.iter().any(|ax| ax.monotonic == "non-monotonic"));
    rec.classification = if !rec.has_entrance_clip {
        "no-entrance-clip"
    } else if rec.camera_count == 0 {
        "no-camera"
    } else if !rec.has_camera_movement {
        "static-camera"
    } else if non_mono {
        "NON-MONOTONIC"
    } else {
        "monotonic-dolly"
    }
    .to_string();
    rec
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn axis_brief(ax: &AxisSummary) -> String {
    format!(
        "{}: {:.3}→{:.3} [min {:.3} @{:.2}s, max {:.3} @{:.2}s] {}",
        ax.axis, ax.start, ax.end, ax.min, ax.t_min, ax.max, ax.t_max, ax.monotonic
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let assets_root = manifest
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

    // Collect bundles: explicit args (dirs or .ab files) or the default en+cn roots,
    // deduped by basename with EN preferred.
    let mut by_name: BTreeMap<String, (PathBuf, String)> = BTreeMap::new();
    fn add_dir(by_name: &mut BTreeMap<String, (PathBuf, String)>, dir: &Path, region: &str) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if name.starts_with("char_") && name.ends_with(".ab") {
                by_name.entry(name).or_insert((p, region.to_string()));
            }
        }
    }
    if args.is_empty() {
        for region in ["en", "cn"] {
            add_dir(
                &mut by_name,
                &assets_root
                    .join("ArkAssets")
                    .join(region)
                    .join("arts/dynchars"),
                region,
            );
        }
    } else {
        for a in &args {
            let p = PathBuf::from(a);
            if p.is_dir() {
                add_dir(&mut by_name, &p, "arg");
            } else if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                by_name
                    .entry(name.to_string())
                    .or_insert((p.clone(), "arg".to_string()));
            }
        }
    }
    if by_name.is_empty() {
        eprintln!(
            "no dynchar bundles found (looked under assets/ArkAssets/{{en,cn}}/arts/dynchars)"
        );
        std::process::exit(1);
    }
    println!("scanning {} dynchar bundles...", by_name.len());

    let mut records: Vec<SkinRecord> = by_name
        .par_iter()
        .map(|(_, (path, region))| analyze_bundle(path, region))
        .collect();
    records.sort_by(|a, b| a.skin.cmp(&b.skin));

    // ---- table ----
    println!(
        "\n{:<44} {:<3} {:<9} {:<4} {:<5} {:<16} centre-summary",
        "skin", "reg", "entrance", "cams", "moves", "classification"
    );
    for r in &records {
        let centre = r.centre.as_ref().map_or_else(String::new, |axes| {
            axes.iter()
                .filter(|ax| ax.monotonic != "constant")
                .map(axis_brief)
                .collect::<Vec<_>>()
                .join(" | ")
        });
        let ortho = r.ortho.as_ref().map_or_else(String::new, |o| {
            format!(
                "  ortho {:.3}→{:.3} (min {:.3}, max {:.3}, {})",
                o.start, o.end, o.min, o.max, o.monotonic
            )
        });
        println!(
            "{:<44} {:<3} {:<9} {:<4} {:<5} {:<16} {}{}{}",
            r.skin,
            r.region,
            if r.has_entrance_clip { "yes" } else { "no" },
            r.camera_count,
            if r.has_camera_movement { "yes" } else { "no" },
            r.classification,
            centre,
            ortho,
            if r.plunge { "  <<< PLUNGE" } else { "" },
        );
    }

    // ---- summary ----
    let total = records.len();
    let with_entrance = records.iter().filter(|r| r.has_entrance_clip).count();
    let with_cam = records.iter().filter(|r| r.camera_count > 0).count();
    let with_move = records.iter().filter(|r| r.has_camera_movement).count();
    let with_ortho = records.iter().filter(|r| r.ortho.is_some()).count();
    let flagged: Vec<&SkinRecord> = records
        .iter()
        .filter(|r| r.plunge || r.classification == "NON-MONOTONIC")
        .collect();
    let errors = records.iter().filter(|r| r.error.is_some()).count();
    println!("\n==== SUMMARY ====");
    println!("bundles scanned:            {total}");
    println!("with _Start entrance clip:  {with_entrance}");
    println!("with camera (class 20):     {with_cam}");
    println!("with camera POSITION anim:  {with_move}");
    println!("with camera ortho-size anim:{with_ortho}");
    println!("errors:                     {errors}");
    if flagged.is_empty() {
        println!(
            "\nNON-MONOTONIC / plunge flags: NONE — every baked camera-centre curve is monotonic;"
        );
        println!("the entrance plunge (where seen in game) is runtime/encrypted, not baked.");
    } else {
        println!("\nNON-MONOTONIC / PLUNGE flags ({}):", flagged.len());
        for r in &flagged {
            println!("  {} [{}] plunge={}", r.skin, r.classification, r.plunge);
            if let Some(axes) = &r.centre {
                for ax in axes {
                    if ax.monotonic != "constant" {
                        println!("    centre {}", axis_brief(ax));
                    }
                }
            }
            for n in &r.animated_nodes {
                for ax in &n.axes {
                    if ax.monotonic == "non-monotonic" {
                        println!("    node {} {}", n.node_path, axis_brief(ax));
                    }
                }
            }
        }
    }

    // ---- catalog ----
    let out_dir = assets_root.join("il2cpp-work");
    let _ = std::fs::create_dir_all(&out_dir);
    let out_path = out_dir.join("entrance_camera_catalog.json");
    let catalog = serde_json::json!({
        "generated_by": "cargo run --release --example scan_entrance_cams",
        "bundles_scanned": total,
        "with_entrance_clip": with_entrance,
        "with_camera": with_cam,
        "with_camera_position_anim": with_move,
        "with_ortho_anim": with_ortho,
        "plunge_or_non_monotonic": flagged.iter().map(|r| r.skin.clone()).collect::<Vec<_>>(),
        "skins": records,
    });
    match std::fs::write(
        &out_path,
        serde_json::to_string_pretty(&catalog).unwrap_or_default(),
    ) {
        Ok(()) => println!("\ncatalog written: {}", out_path.display()),
        Err(e) => eprintln!("\ncatalog write FAILED ({}): {e}", out_path.display()),
    }
}
