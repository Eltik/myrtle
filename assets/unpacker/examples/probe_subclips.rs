//! THROWAWAY census: which entrance clips are SUB-CLIPS, played by an Animator on a GameObject
//! the root clip activates later than t=0, and how many shipped colour bindings they carry.
//!
//! Motivation: kalts `Start_05` binds `_TintColor.a` and `_Amount` on `wenli` at clip-local
//! 2.9667 to 4.5 s while the layer is active 9.1667 to 12.1 s. The colour reader keys every
//! entrance clip on the root timeline, so if a sub-clip's local zero is the activation of its
//! Animator's GameObject, every curve it carries is placed early by that activation. This prints,
//! per entrance-admitted clip: its stop time, the Animator GameObject(s) that play it, that
//! object's first active window from `active_windows` (the root clip's `m_IsActive` toggles),
//! and the number of varying material colour bindings the clip carries.
//!
//! Usage: cargo run --release --example `probe_subclips` -- <bundle.ab>...
#![allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::anim::{active_windows, decode_curve_any};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID")
        .and_then(Value::as_i64)
        .filter(|p| *p != 0)
}

fn go_name(all: &HashMap<i64, (i32, Value)>, go: i64) -> String {
    all.get(&go)
        .and_then(|(_, v)| v.get("m_Name"))
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string()
}

/// Walk up the transform chain to the nearest ancestor (or self) that has an active window.
fn window_up(
    all: &HashMap<i64, (i32, Value)>,
    windows: &HashMap<i64, Vec<unpacker::export::anim::ActiveWindow>>,
    mut go: i64,
) -> Option<(i64, Option<f32>, Option<f32>)> {
    for _ in 0..64 {
        if let Some(w) = windows.get(&go)
            && let Some(first) = w.first()
        {
            return Some((go, first.0, first.1));
        }
        // parent via the GO's Transform
        let tf = all
            .get(&go)?
            .1
            .get("m_Component")?
            .as_array()?
            .iter()
            .find_map(|c| {
                let p = c.get("component").and_then(pid)?;
                (all.get(&p)?.0 == 4).then_some(p)
            })?;
        let father = all.get(&tf)?.1.get("m_Father").and_then(pid)?;
        go = all.get(&father)?.1.get("m_GameObject").and_then(pid)?;
    }
    None
}

fn main() {
    println!("skin\tclip\tstop\tanimatorGO\twindowGO\tactiveFrom\tactiveUntil\tvaryingColour");
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
            // clip -> animator GOs (Animator 95 -> controller 91 -> m_AnimationClips)
            let mut clip_gos: HashMap<i64, Vec<i64>> = HashMap::new();
            for (cid, v) in all.values() {
                if *cid != 95 {
                    continue;
                }
                let (Some(go), Some(ctrl)) = (
                    v.get("m_GameObject").and_then(pid),
                    v.get("m_Controller").and_then(pid),
                ) else {
                    continue;
                };
                let Some((91, c)) = all.get(&ctrl) else {
                    continue;
                };
                for r in c
                    .get("m_AnimationClips")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if let Some(clip) = pid(r) {
                        clip_gos.entry(clip).or_default().push(go);
                    }
                }
            }
            let windows = active_windows(&all);
            let mut clips: Vec<(i64, &Value)> = all
                .iter()
                .filter(|(_, (cid, _))| *cid == 74)
                .map(|(p, (_, v))| (*p, v))
                .collect();
            clips.sort_by_key(|(_, v)| {
                v.get("m_Name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string()
            });
            for (cpid, v) in clips {
                let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                let lower = name.to_ascii_lowercase();
                if !(lower.contains("start")
                    || lower.contains("entrance")
                    || lower.contains("enter"))
                {
                    continue;
                }
                let stop = v
                    .get("m_MuscleClip")
                    .and_then(|m| m.get("m_StopTime"))
                    .and_then(Value::as_f64)
                    .unwrap_or(-1.0);
                // varying colour bindings
                let mut varying = 0usize;
                if let Some(binds) = v
                    .get("m_ClipBindingConstant")
                    .and_then(|c| c.get("genericBindings"))
                    .and_then(Value::as_array)
                {
                    let mut gidx = 0usize;
                    for b in binds {
                        let ty = b
                            .get("typeID")
                            .or_else(|| b.get("classID"))
                            .and_then(Value::as_i64)
                            .unwrap_or(0);
                        let a = b.get("attribute").and_then(Value::as_i64).unwrap_or(0);
                        let my = gidx;
                        gidx += if ty == 4 {
                            match a {
                                1 | 3 | 4 => 3,
                                2 => 4,
                                _ => 1,
                            }
                        } else {
                            1
                        };
                        if b.get("customType").and_then(Value::as_i64) != Some(22) {
                            continue;
                        }
                        let nib = ((a as u64) >> 28) & 0xF;
                        if !(4..=7).contains(&nib) {
                            continue;
                        }
                        if let Some(c) = decode_curve_any(v, my)
                            && c.len() > 1
                        {
                            let lo = c.iter().map(|s| s.1).fold(f32::INFINITY, f32::min);
                            let hi = c.iter().map(|s| s.1).fold(f32::NEG_INFINITY, f32::max);
                            if hi - lo > 1e-4 {
                                varying += 1;
                            }
                        }
                    }
                }
                let gos = clip_gos.get(&cpid).cloned().unwrap_or_default();
                if gos.is_empty() {
                    println!("{skin}\t{name}\t{stop:.4}\t-\t-\t-\t-\t{varying}");
                }
                for go in gos {
                    let w = window_up(&all, &windows, go);
                    let (wgo, from, until) = w.map_or(
                        ("-".to_string(), "-".to_string(), "-".to_string()),
                        |(g, f, u)| {
                            (
                                go_name(&all, g),
                                f.map_or("-".to_string(), |x| format!("{x:.4}")),
                                u.map_or("-".to_string(), |x| format!("{x:.4}")),
                            )
                        },
                    );
                    println!(
                        "{skin}\t{name}\t{stop:.4}\t{}\t{wgo}\t{from}\t{until}\t{varying}",
                        go_name(&all, go)
                    );
                }
            }
        }
    }
}
