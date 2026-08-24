//! Evaluate a dynchar scene's **idle** `AnimationClip` at its first frame, so the
//! painted mesh quads can be exported at their settled display pose rather than
//! their scattered prefab bind pose.
//!
//! These prefabs (Unity 2021, Mecanim) store animation as OPTIMIZED clips: the
//! legacy `m_*Curves` are empty and the real data lives in
//! `m_MuscleClip.m_Clip.data.{m_StreamedClip,m_DenseClip,m_ConstantClip}`,
//! indexed by `m_ClipBindingConstant.genericBindings`. Many scenes' idle loops
//! drive the local position (and occasionally euler rotation) of a handful of
//! quad `GameObjects` — e.g. Nearl "Evolved Art"'s sword/petal shards assemble
//! from an exploded bind pose. We only need the pose at t=0 (the loop start), so
//! we read the first real streamed frame plus the constant curves and apply the
//! resulting local position/euler overrides when accumulating world matrices.
//!
//! Scope intentionally narrow (matches what these scenes actually use): Transform
//! position (attribute 1) and euler (attribute 4). Scale, quaternion rotation,
//! and `GameObject` active-state curves are ignored — the quads render statically
//! regardless of active-state, and none of the observed scenes animate scale.

use serde_json::Value;
use std::collections::{HashMap, HashSet};

use super::spine::get_path_id;

/// Per-transform local-pose overrides evaluated from the idle clip, keyed by
/// Transform `path_id` (what `accumulate_matrix` walks).
#[derive(Default)]
pub struct IdlePose {
    /// Overridden local position (x, y, z) in local units.
    pub pos: HashMap<i64, [f32; 3]>,
    /// Overridden local rotation as euler degrees (x, y, z); takes precedence
    /// over the prefab quaternion when present.
    pub euler: HashMap<i64, [f32; 3]>,
    /// `GameObject` `path_id` → animated `m_IsActive` state at the idle pose.
    /// Sampled only from the base idle-loop clip (not soul/interact transitions),
    /// so flash/overlay quads the idle animation switches off are dropped from the
    /// exported scene, and any the idle animation switches on are kept.
    pub active: HashMap<i64, bool>,
}

impl IdlePose {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pos.is_empty() && self.euler.is_empty() && self.active.is_empty()
    }
}

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

/// `CRC32("m_IsActive")` — the binding attribute Unity uses for a `GameObject`'s
/// active-state curve (verified against the runtime data and `crc32` below).
const M_IS_ACTIVE_CRC: i64 = 2_086_281_974;

/// Number of animation curves a single generic binding expands to (Unity's
/// `FindBinding` accumulation): Transform position/scale/euler = 3, quaternion
/// rotation = 4, everything else = 1.
const fn binding_curve_count(type_id: i64, attribute: i64) -> usize {
    if type_id == 4 {
        match attribute {
            1 | 3 | 4 => 3, // position | scale | euler
            2 => 4,         // quaternion rotation
            _ => 1,
        }
    } else {
        1
    }
}

/// The `(typeID, attribute, path-hash)` triple of one generic binding.
/// `typeID` falls back to the older `classID` field name.
fn binding_fields(b: &Value) -> (i64, i64, u32) {
    let type_id = b
        .get("typeID")
        .or_else(|| b.get("classID"))
        .and_then(Value::as_i64)
        .unwrap_or(-1);
    let attr = b.get("attribute").and_then(Value::as_i64).unwrap_or(-1);
    let path = b.get("path").and_then(Value::as_i64).unwrap_or(0) as u32;
    (type_id, attr, path)
}

/// Total curve count across a clip's bindings. The streamed / dense / constant
/// sub-clips partition the global curve-index space as `[streamed | dense | constant]`.
fn total_curve_count(bindings: &[Value]) -> usize {
    bindings
        .iter()
        .map(|b| {
            let (type_id, attr, _) = binding_fields(b);
            binding_curve_count(type_id, attr)
        })
        .sum()
}

/// A clip's `m_ClipBindingConstant.genericBindings` array.
fn generic_bindings(clip: &Value) -> Option<&Vec<Value>> {
    clip.get("m_ClipBindingConstant")
        .and_then(|b| b.get("genericBindings"))
        .and_then(Value::as_array)
}

/// A clip's optimized-clip payload (`m_MuscleClip.m_Clip.data`).
fn clip_data(clip: &Value) -> Option<&Value> {
    clip.get("m_MuscleClip")
        .and_then(|m| m.get("m_Clip"))
        .and_then(|c| c.get("data"))
}

/// A clip's end time in seconds (`m_MuscleClip.m_StopTime`).
fn clip_stop_time(clip: &Value) -> Option<f32> {
    clip.get("m_MuscleClip")
        .and_then(|m| m.get("m_StopTime"))
        .and_then(Value::as_f64)
        .map(|s| s as f32)
}

/// A `GameObject`'s serialized (prefab) `m_IsActive` state — what it reverts to when an
/// animated active-state override expires at its clip's end.
fn prefab_active(all_objects: &HashMap<i64, (i32, Value)>, go: i64) -> bool {
    all_objects
        .get(&go)
        .and_then(|(_, v)| v.get("m_IsActive"))
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

/// Whether a clip's name marks it as part of the `_Start` entrance cinematic.
fn is_entrance_clip(clip: &Value) -> bool {
    let name = clip
        .get("m_Name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    name.contains("start") || name.contains("entrance") || name.contains("enter")
}

/// Clips that DRIVE THE ENTRANCE CAMERA, identified structurally rather than by name.
///
/// `is_entrance_clip` name-matches "start" | "entrance" | "enter", and three entrance skins name
/// their camera clip something else entirely — Eyjafjalla the Hvit Aska's is
/// `char_1016_agoat2#_camera_01`, Ch'en the Holungday's is `Take 002` (Unity's default clip name),
/// Whislash-alter's is just `03`. For those the ortho + centre tracks came back `None` and the
/// entrance was framed on the IDLE's tight bounds instead.
///
/// A clip drives the camera if any binding's path hashes to a subpath of the camera's own
/// ancestor chain AND the binding is camera MOTION: the Camera component (`typeID 20`, its
/// orthographic size) or a Transform position/rotation/scale (`typeID 4`, attributes 1/2/3/4).
/// `m_IsActive` toggles (`typeID 1`) are deliberately excluded — idle and interact clips switch
/// chain objects on and off without moving the camera at all.
///
/// **Name gate FIRST, structure as fallback.** Where any camera-motion clip is already
/// name-admitted this returns exactly that set, so every skin the old rule handled is untouched by
/// construction; the structural set is consulted only when the name gate finds nothing. Verified:
/// each of the 13 entrance bundles holds exactly ONE clip binding camera motion.
fn camera_motion_clips(all_objects: &HashMap<i64, (i32, Value)>) -> HashSet<i64> {
    let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
    let mut tf_go: HashMap<i64, i64> = HashMap::new();
    let mut tf_father: HashMap<i64, i64> = HashMap::new();
    let mut cam_go: Option<i64> = None;
    for (pid, (cid, v)) in all_objects {
        match cid {
            4 | 224 => {
                if let Some(g) = v.get("m_GameObject").and_then(get_path_id) {
                    go_to_tf.insert(g, *pid);
                    tf_go.insert(*pid, g);
                }
                if let Some(f) = v.get("m_Father").and_then(get_path_id) {
                    tf_father.insert(*pid, f);
                }
            }
            20 => {
                if let Some(g) = v.get("m_GameObject").and_then(get_path_id) {
                    cam_go = Some(cam_go.map_or(g, |g0| if *pid < g0 { g } else { g0 }));
                }
            }
            _ => {}
        }
    }
    let Some(cg) = cam_go else {
        return HashSet::new();
    };
    // Names camera -> root, then every CONTIGUOUS subpath, because a binding path is relative to
    // whichever Animator plays the clip and we do not know which that is yet.
    let mut names: Vec<String> = Vec::new();
    let mut cur = go_to_tf.get(&cg).copied();
    for _ in 0..64 {
        let Some(tf) = cur else { break };
        let go = tf_go.get(&tf).copied().unwrap_or(0);
        names.push(
            all_objects
                .get(&go)
                .and_then(|(_, v)| v.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        );
        cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
    }
    names.reverse();
    let mut hashes: HashSet<u32> = HashSet::new();
    for i in 0..names.len() {
        for j in i..names.len() {
            hashes.insert(crc32(names[i..=j].join("/").as_bytes()));
        }
    }
    let mut named: HashSet<i64> = HashSet::new();
    let mut structural: HashSet<i64> = HashSet::new();
    for (pid, (cid, v)) in all_objects {
        if *cid != 74 {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let drives = bindings.iter().any(|b| {
            let (type_id, attr, path) = binding_fields(b);
            hashes.contains(&path) && (type_id == 20 || (type_id == 4 && (1..=4).contains(&attr)))
        });
        if !drives {
            continue;
        }
        structural.insert(*pid);
        if is_entrance_clip(v) {
            named.insert(*pid);
        }
    }
    if named.is_empty() { structural } else { named }
}

/// Clips that run on the ENTRANCE's clock but whose NAME does not say so: the clips played
/// by an Animator living inside a `<start> only …` group.
///
/// Those groups ship `m_IsActive = 0` and the game switches them on exactly while the start
/// state plays — the rule `go_effectively_active` already relies on to keep them in the
/// entrance scene at all. Their contents carry their OWN Animator, and the effect prefab
/// inside is named for the effect rather than the state (Mlynar "Fields of Ruination" holds
/// `char_4064_Mlynar_epoque#28_weisheng(Clone)` in `Start Only Effects`, driven by clip
/// `char_4064_Mlynar_##_weisheng`). Name-matching the clip therefore misses it, and the
/// layer exports as an unmodulated always-on fog sheet across the whole 16.5 s cinematic
/// instead of the ~10 s fade its clip actually paints.
///
/// Scoped to material-COLOUR reading (`entrance_material_color_channels`): every channel it
/// yields is matched against the owning layer's own material properties and restricted to
/// the playing Animator's subtree, so an admitted clip can only ever modulate art inside
/// its own group. The camera/pan/ortho tracks stay name-gated to the main entrance clip.
fn start_only_effect_clips(all_objects: &HashMap<i64, (i32, Value)>) -> HashSet<i64> {
    let go_parent = build_go_parent(all_objects);
    let name_of = |go: i64| -> String {
        all_objects
            .get(&go)
            .and_then(|(_, v)| v.get("m_Name"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase()
    };
    let in_start_only_group = |go: i64| -> bool {
        root_path(go, &go_parent).iter().any(|&a| {
            let n = name_of(a);
            n.contains("only") && n.contains("start")
        })
    };
    build_clip_animator_gos(all_objects)
        .into_iter()
        .filter(|(_, animators)| animators.iter().copied().any(in_start_only_group))
        .map(|(clip, _)| clip)
        .collect()
}

/// One streamed frame: a time plus a set of `(global curve index, value)` keys.
struct StreamedKey {
    index: usize,
    /// Cubic segment coefficients `c0..c2` (the dt³, dt², dt terms) describing the
    /// eased curve from THIS key toward the next: `v(dt) = c0·dt³ + c1·dt² + c2·dt + value`
    /// with `dt` = seconds since this key.
    coeff: [f32; 3],
    value: f32,
}

/// Decode `m_StreamedClip.data` (a `vector<uint32>`) into frames. Each frame is
/// `[time_f32, keyCount_u32, {index_i32, coeff0..3_f32} * keyCount]`; the value
/// at the frame is the cubic's constant term `coeff[3]`, and `coeff[0..3]` are the
/// segment's higher-order terms (retained for cubic interpolation between keys).
fn read_streamed(data: &[u32]) -> Vec<Vec<StreamedKey>> {
    read_streamed_timed(data)
        .into_iter()
        .map(|(_, keys)| keys)
        .collect()
}

/// Like {@link `read_streamed`} but keeps each frame's TIME (seconds) — needed to find
/// WHEN an `m_IsActive` curve toggles on, for the entrance's per-layer reveal timeline.
fn read_streamed_timed(data: &[u32]) -> Vec<(f32, Vec<StreamedKey>)> {
    let mut frames = Vec::new();
    let mut i = 0usize;
    while i + 2 <= data.len() {
        let time = f32::from_bits(data[i]);
        let num_keys = data[i + 1] as usize;
        i += 2;
        let mut keys = Vec::with_capacity(num_keys);
        for _ in 0..num_keys {
            if i + 5 > data.len() {
                frames.push((time, keys));
                return frames;
            }
            let index = data[i] as i32 as usize;
            let coeff = [
                f32::from_bits(data[i + 1]),
                f32::from_bits(data[i + 2]),
                f32::from_bits(data[i + 3]),
            ];
            let value = f32::from_bits(data[i + 4]); // coeff[3]
            keys.push(StreamedKey {
                index,
                coeff,
                value,
            });
            i += 5;
        }
        frames.push((time, keys));
    }
    frames
}

/// Per-GameObject ENTRANCE reveal time (seconds): for every `m_IsActive` curve in the
/// prefab's clips that starts OFF and toggles ON, the time it turns on. The `_Start`
/// cinematic sequences its scene elements this way (the cathedral shows first, the
/// mirror-world + crystal throne switch on later), so a scene-mesh layer under such a
/// `GameObject` must not render until its reveal time. GOs never toggled on (always
/// active) are absent → treated as visible from t=0.
#[must_use]
pub fn active_timelines(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<i64, f32> {
    active_windows(all_objects)
        .into_iter()
        .filter_map(|(go, w)| w.first().and_then(|iv| iv.0).map(|from| (go, from)))
        .collect()
}

/// A `GameObject`'s ENTRANCE visibility window: `(activeFrom, activeUntil)` in seconds.
/// `activeFrom = None` → visible from t=0 (starts on); `activeUntil = None` → never hidden
/// (ends on). A scene-mesh layer under such a `GameObject` renders only while
/// `activeFrom <= t < activeUntil`. This captures BOTH the mirror-world / crystal-throne
/// switch-ON (cathedral shows first, they appear later) AND the cathedral switch-OFF (the
/// bright cathedral deactivates when the mirror world takes over) — the environment SWAP
/// the `_Start` cinematic performs, which a reveal-only timeline misses.
pub type ActiveWindow = (Option<f32>, Option<f32>);

/// A `GameObject`'s FULL visibility schedule: the ordered, disjoint `[from, until)` windows the
/// clip switches it on for. Almost every object has exactly one, but a cinematic is free to
/// flash something twice, and a single `(from, until)` pair silently truncates that to the
/// first window — hiding everything after the first switch-off for the rest of the entrance.
pub type ActiveWindowList = Vec<ActiveWindow>;

/// Fold a clip's `m_IsActive` on/off transitions into the intervals the object is VISIBLE for.
/// `start_on` is the clip's opening state; `None` bounds mean "from t=0" / "never hidden".
/// An always-on object yields a single `(None, None)`, which callers treat as "no window".
fn windows_from_transitions(start_on: bool, transitions: &[(f32, bool)]) -> ActiveWindowList {
    let mut out: ActiveWindowList = Vec::new();
    let mut on = start_on;
    let mut from: Option<f32> = None;
    for &(t, next) in transitions {
        if next && !on {
            from = Some(t);
            on = true;
        } else if !next && on {
            out.push((from, Some(t)));
            from = None;
            on = false;
        }
    }
    if on {
        out.push((from, None));
    }
    out
}

/// Merge two schedules by INTERSECTION — the object is visible only where BOTH agree. For the
/// single-window case this is exactly the previous "latest reveal, earliest hide" rule, so the
/// generalisation cannot move any object that has one window per clip.
fn intersect_windows(a: &[ActiveWindow], b: &[ActiveWindow]) -> ActiveWindowList {
    let mut out: ActiveWindowList = Vec::new();
    for &(af, au) in a {
        for &(bf, bu) in b {
            let from = match (af, bf) {
                (Some(x), Some(y)) => Some(x.max(y)),
                (x, y) => x.or(y),
            };
            let until = match (au, bu) {
                (Some(x), Some(y)) => Some(x.min(y)),
                (x, y) => x.or(y),
            };
            // Drop windows the intersection collapsed to nothing.
            if let (Some(f), Some(u)) = (from, until)
                && u <= f
            {
                continue;
            }
            out.push((from, until));
        }
    }
    out
}

/// `m_StopTime` of the clip that actually drives the entrance CAMERA — the authored end of the
/// cinematic's content, which can fall BEFORE the director's nominal `_params.duration`
/// (Executor's camera clip stops at 6.100 against a duration of 6.500; Mlynar's at 15.967
/// against 16.500). The director's end-of-entrance fade is anchored to this, not to `duration`.
///
/// Identified STRUCTURALLY via {@link `camera_motion_clips`}, not by name: three skins do not use
/// the `start|entrance|enter` vocabulary, and a name-gated scan picks up short prop clips
/// instead (Lappland's longest name-admitted clip stops at 0.833 on a 14.5 s entrance, which
/// would truncate her whole cinematic).
#[must_use]
pub fn entrance_clip_stop(all_objects: &HashMap<i64, (i32, Value)>) -> Option<f32> {
    let cam_clips = camera_motion_clips(all_objects);
    all_objects
        .iter()
        .filter(|(pid, (cid, _))| *cid == 74 && cam_clips.contains(pid))
        .filter_map(|(_, (_, v))| clip_stop_time(v))
        .fold(None, |acc: Option<f32>, s| {
            Some(acc.map_or(s, |a| a.max(s)))
        })
}

#[must_use]
pub fn active_windows(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<i64, ActiveWindowList> {
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let go_parent = build_go_parent(all_objects);
    let go_name: HashMap<i64, String> = all_objects
        .iter()
        .filter(|(_, (cid, _))| *cid == 1)
        .filter_map(|(p, (_, v))| {
            v.get("m_Name")
                .and_then(Value::as_str)
                .map(|n| (*p, n.to_string()))
        })
        .collect();
    // The entrance DIRECTOR's authored duration (`_params.duration` on the behaviour that
    // owns `_mainCamera`). A clip can STOP before the state it belongs to exits — Mlynar
    // "Fields of Ruination"'s `_Start` clip stops at 15.97 while the director runs to 16.5 —
    // and the game holds the clip's final `m_IsActive` state for that remainder.
    let director_duration: Option<f32> = all_objects
        .values()
        .find_map(|(cid, v)| {
            (*cid == 114 && v.get("_mainCamera").is_some())
                .then(|| {
                    v.get("_params")
                        .and_then(|p| p.get("duration"))
                        .and_then(Value::as_f64)
                })
                .flatten()
        })
        .map(|d| d as f32);
    // ENTRANCE CLIP ADMISSION. `is_entrance_clip` is a NAME match, and three entrance bundles
    // name their cinematic something else entirely — Eyjafjalla the Hvit Aska
    // `char_1016_agoat2#_camera_01`, Ch'en the Holungday `Take 002`, Whislash-alter just `03`.
    // That is the exact trap `camera_motion_clips` already documents and solves for the CAMERA,
    // and it applied here too: the clip that drives the entrance camera IS the cinematic, so its
    // `m_IsActive` toggles ARE the reveal timeline.
    //
    // Whislash-alter is the case that exposed it. Her clip `03` binds `m_IsActive` on
    // `Dummy002/Main Camera/char_01/static_offset/fixed`, the parent of all eight camera-locked
    // film-strip quads — so the strip is authored to switch OFF, and we drew it for the whole
    // entrance because the name gate skipped `03`. Every one of her 26 entrance layers came back
    // with zero gating.
    //
    // UNION, so every skin the name gate already handled is untouched by construction: where a
    // camera-motion clip is already name-admitted, `camera_motion_clips` returns exactly that
    // clip, and the union adds nothing. `DYNCHAR_WINCAMCLIP=0` reverts to the name gate alone.
    let cam_clips = if std::env::var("DYNCHAR_WINCAMCLIP").as_deref() == Ok("0") {
        HashSet::new()
    } else {
        camera_motion_clips(all_objects)
    };
    let mut out: HashMap<i64, ActiveWindowList> = HashMap::new();
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 {
            continue;
        }
        // Only the ENTRANCE clip drives the reveal timeline. The `_Start` bundle also
        // ships the full idle/interact state machine, whose `m_IsActive` toggles are
        // NOT the cinematic sequence — merging them corrupts the windows.
        if !is_entrance_clip(v) && !cam_clips.contains(clip_pid) {
            continue;
        }
        let stop = clip_stop_time(v);
        if std::env::var("SCENE_DEBUG").is_ok() {
            eprintln!(
                "  [clip] {:?} stop={stop:?} director_duration={director_duration:?}",
                v.get("m_Name").and_then(Value::as_str).unwrap_or("?")
            );
        }
        for (go, mut ivs) in active_timeline(
            v,
            &hash_to_gos,
            &is_ancestor,
            clip_animators.get(clip_pid).map(Vec::as_slice),
            &go_parent,
            &go_name,
        ) {
            // A clip's `m_IsActive` override EXPIRES when the entrance state exits at the
            // clip's end (`m_StopTime`): a GO the prefab keeps INACTIVE (an entrance-only
            // overlay, e.g. Mlynar's white transition flash — revealed at 13.0s, clip stop
            // 15.97s) then reverts to inactive. Bound a clip-driven reveal with no authored
            // hide by the clip's own stop, else the overlay holds its final frame (a
            // permanent white-out) into the settle. Prefab-ACTIVE GOs keep their `None`
            // hide — they stay visible after the clip on their own serialized state.
            let Some(&(last_from, last_until)) = ivs.last() else {
                continue;
            };
            if last_until.is_none() && last_from.is_some() && !prefab_active(all_objects, go) {
                // Expire at the STATE's exit, not the CLIP's stop. Measured against the
                // recording: Mlynar's white transition flash is held full-white by the game
                // until ~16.5s (the director duration) while its clip stops at 15.97 — cutting
                // at the clip stop exposes the bare scene for the last ~0.55s of the entrance
                // (luminance 248 -> 97 where the game stays 248). Falls back to the clip stop
                // when no director duration is authored, and never SHORTENS a window.
                let bound = match (stop, director_duration) {
                    (Some(s), Some(d)) if d > s => Some(d),
                    (s, _) => s,
                };
                if let Some(last) = ivs.last_mut() {
                    last.1 = bound;
                }
            }
            // Merge across clips by intersecting the schedules (see `intersect_windows`).
            match out.entry(go) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    let merged = intersect_windows(e.get(), &ivs);
                    e.insert(merged);
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(ivs);
                }
            }
        }
    }
    out
}

/// Per-GameObject ENTRANCE emission-rate curves: the `_Start` cinematic can animate a
/// `ParticleSystem`'s `EmissionModule.rateOverTime.scalar` directly (Mlynar "Fields of
/// Ruination"'s sword clips hold the confetti/star emitters at 0 and spike them 8–200/s
/// during the ~7–13s flourish, while the SERIALIZED rates are a constant 8–150/s).
/// Without these curves such systems emit at full rate from t=0. Times are absolute
/// cinematic seconds.
///
/// Identical sibling rigs (the three sword clones) can bind the same RELATIVE transform
/// subpath. Each binding is therefore disambiguated within its own clip by the other
/// hashes that clip binds, and only then are curves for the same resolved `GameObject`
/// merged into a MAX ENVELOPE.
///
/// Unlike `entrance_transform_curves`' ctrl target (an ANCESTOR of the clip's other
/// bound GOs, so descendant-scoring finds a real winner), a rate-curve binding's path
/// names the `ParticleSystem`'s OWN `GameObject` — a LEAF, sibling to the clip's other
/// bound systems under the same generic `ctrl`, not their ancestor. Descendant-scoring
/// therefore often can't find a unique winner here (every sibling scores 0 against every
/// other sibling). Dropping such a binding outright would be WORSE than the bug this
/// function exists to fix: silently omitting it from the output map falls back (in
/// `particles.rs`) to the system's SERIALIZED constant rate — i.e. the very "full rate
/// from t=0" behavior this function was built to replace. A clip that animates a
/// system's `rateOverTime` at all is proof the system is meant to be GATED, not ambient,
/// so an unresolved binding still contributes an explicit rate-0 (silent) curve to every
/// colliding candidate — never a bare drop. Correctly-disambiguated candidates still
/// receive their real curve from other clips/bindings and win the max-envelope merge
/// wherever nonzero; an all-candidates-ambiguous system ends up correctly silent.
#[must_use]
pub fn entrance_ps_rate_curves(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> HashMap<i64, Vec<(f32, f32)>> {
    /// A decoded rate curve plus its clip's stop time (the override's expiry).
    type StoppedCurve = (Vec<(f32, f32)>, f32);
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let want = i64::from(crc32(b"EmissionModule.rateOverTime.scalar"));
    let mut per_go: HashMap<i64, Vec<StoppedCurve>> = HashMap::new();
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
        let animator_gos = clip_animators.get(clip_pid).map(Vec::as_slice);
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let stop = clip_stop_time(v).unwrap_or(f32::MAX);
        let mut all_hashes = std::collections::HashSet::new();
        let mut curves = Vec::new();
        let mut gidx = 0usize;
        for b in bindings {
            let (tid, attr, path) = binding_fields(b);
            all_hashes.insert(path);
            if tid == 198
                && attr == want
                && let Some(curve) = decode_curve_any(v, gidx)
                && curve.len() > 1
            {
                curves.push((path, curve));
            }
            gidx += binding_curve_count(tid, attr);
        }
        for (path, curve) in curves {
            let Some(candidates) = hash_to_gos.get(&path) else {
                continue;
            };
            let candidates = scope_to_animator(candidates, animator_gos, &is_ancestor);
            let other_hashes = all_hashes.iter().copied().filter(|&h| h != path);
            match disambiguate_owner(&candidates, other_hashes, &hash_to_gos, &is_ancestor) {
                Some(go) => per_go.entry(go).or_default().push((curve, stop)),
                // Ambiguous: this clip proves EVERY colliding candidate's system is
                // curve-gated, just not which one it belongs to. Contribute an
                // explicit silent (rate-0) window to all of them instead of leaving
                // the binding out entirely (which would fall back to the far louder
                // serialized constant rate).
                None => {
                    for &go in candidates.iter() {
                        per_go.entry(go).or_default().push((vec![(0.0, 0.0)], stop));
                    }
                }
            }
        }
    }
    let mut out: HashMap<i64, Vec<(f32, f32)>> = HashMap::new();
    for (go, curves) in per_go {
        let merged = merge_curves_max(&curves);
        if merged.len() < 2 {
            continue;
        }
        out.insert(go, merged);
    }
    out
}

/// One effect-host transform DRIVEN by the `_Start` cinematic: a uniform SCALE-factor
/// curve (and optional local X/Y position curves), keyed in absolute cinematic seconds.
/// Virtuosa "Diversity in Oneness": the `start_glow_01` clip scales the crown host
/// `…interact_weapon_01(Clone)/…/ctrl` from 1.0→0.28 over 9.4–12.43s (a big golden halo
/// SHRINKING into the small crown) and slides it (−1.13,−0.56)→(0,0) over 9.9–10.37s. The
/// baked prefab pose captures only the resting (final) transform, so without these curves
/// the shrink-in never plays. Values are the RAW local transform (parent frame: Unity
/// units for position, a plain factor for scale); the particle exporter converts them to
/// the emitter's pivot + px offset.
#[derive(Default)]
pub struct EntranceTransform {
    /// Uniform scale factor (the x component) keyframes, absolute cinematic seconds.
    pub scale: Vec<(f32, f32)>,
    /// Local position x/y keyframes (parent frame, Unity units); empty when unanimated.
    pub pos_x: Vec<(f32, f32)>,
    pub pos_y: Vec<(f32, f32)>,
}

/// A transform's POSITION keyframes: the x and y curves decoded from one binding.
type PosXY = (Vec<(f32, f32)>, Vec<(f32, f32)>);

/// Per-GameObject ENTRANCE transform curves: `_Start` clips that animate a Transform's
/// SCALE (attribute 3) — and, on the same transform, its POSITION (attribute 1). Only
/// SCALE-animated transforms are returned (a varying scale is the driver of an effect
/// host's scale-in; a pure position curve is the camera dolly, which owns no particles);
/// the position is attached alongside when the same transform carries it.
///
/// The binding `path` is a subpath hash relative to the clip's ANIMATOR ROOT (the effect
/// prefab clone), and effect rigs share generic tails (`static_offset/fixed/ctrl`), so one
/// hash collides across dozens of rigs. Blindly applying to all matched GOs would scale
/// (and cross-root-admit) every rig. Disambiguate per clip: the CORRECT target ctrl is the
/// one whose subtree actually contains the particle hosts the SAME clip binds — its
/// scored by how many of the clip's OTHER binding hashes resolve to a descendant. The best
/// (unique) candidate wins; ambiguous clips are dropped (conservative — no scale-in beats a
/// wrong one).
#[must_use]
pub fn entrance_transform_curves(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> HashMap<i64, EntranceTransform> {
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);

    let mut out: HashMap<i64, EntranceTransform> = HashMap::new();
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        // Pass 1: this clip's scale/pos curves per path + the set of ALL its binding hashes.
        let mut scale_path: Option<u32> = None;
        let mut scale = Vec::new();
        let mut pos_x = Vec::new();
        let mut pos_y = Vec::new();
        let mut pos_path: Option<u32> = None;
        // POSITION curves keyed by their OWN binding path. A clip binds several transforms'
        // positions (Executor's clip carries both the scope rim and the camera dummy), and
        // the single `pos_x`/`pos_y` pair below keeps only the LAST one it walks past. Pairing
        // that with the scale's ctrl attaches one transform's motion to another — measured:
        // her rim exported the CAMERA's pan (correlation 1.0, constant −1037.6 px offset).
        let mut pos_by_path: HashMap<u32, PosXY> = HashMap::new();
        let mut all_hashes: std::collections::HashSet<u32> = std::collections::HashSet::new();
        let mut gidx = 0usize;
        for b in bindings {
            let (tid, attr, path) = binding_fields(b);
            all_hashes.insert(path);
            if tid == 4 && attr == 3 {
                if let Some(c) = decode_curve_any(v, gidx)
                    && c.len() > 1
                    && c.len() > scale.len()
                {
                    scale = c;
                    scale_path = Some(path);
                }
            } else if tid == 4 && attr == 1 {
                pos_path = Some(path);
                if let Some(c) = decode_curve_any(v, gidx) {
                    pos_x = c;
                }
                if let Some(c) = decode_curve_any(v, gidx + 1) {
                    pos_y = c;
                }
                pos_by_path.insert(path, (pos_x.clone(), pos_y.clone()));
            }
            gidx += binding_curve_count(tid, attr);
        }
        // DIAGNOSTIC (`DYNCHAR_POSCURVE_DEBUG=1`): report every POSITION curve in the clip
        // BEFORE the scale filter below discards the pure-position ones. Scene mesh quads
        // consume no transform curve at all, so a pure-position host is invisible to them —
        // which is the shape of Mlynar's left-half displacement.
        if std::env::var("DYNCHAR_POSCURVE_DEBUG").is_ok()
            && (!pos_x.is_empty() || !pos_y.is_empty())
        {
            let rng = |c: &Vec<(f32, f32)>| {
                let mn = c.iter().map(|&(_, v)| v).fold(f32::MAX, f32::min);
                let mx = c.iter().map(|&(_, v)| v).fold(f32::MIN, f32::max);
                (mn, mx, mx - mn)
            };
            let (xn, xx, xs) = rng(&pos_x);
            let (yn, yx, ys) = rng(&pos_y);
            let names: Vec<String> = hash_to_gos
                .get(&pos_path.unwrap_or(0))
                .map(|gos| {
                    gos.iter()
                        .take(4)
                        .map(|&g| {
                            all_objects
                                .get(&g)
                                .and_then(|(_, o)| o.get("m_Name"))
                                .and_then(Value::as_str)
                                .unwrap_or("?")
                                .to_string()
                        })
                        .collect()
                })
                .unwrap_or_default();
            eprintln!(
                "  [poscurve] scaleAnimated={} keys={}/{} x[{xn:.2}..{xx:.2}] span {xs:.2}  y[{yn:.2}..{yx:.2}] span {ys:.2}  gos={names:?}",
                scale.len() >= 2,
                pos_x.len(),
                pos_y.len(),
            );
        }
        let Some(scale_path) = scale_path.filter(|_| scale.len() >= 2) else {
            continue;
        };
        // The pos curves belong to the same ctrl only when they share the scale's path.
        // (Re-scan is unnecessary — pos bindings on the same transform share the hash.)

        // Pass 2: pick the target ctrl among the colliding candidates — the one whose
        // subtree contains the most of the clip's OTHER bound GOs.
        let Some(candidates) = hash_to_gos.get(&scale_path) else {
            continue;
        };
        let candidates = scope_to_animator(
            candidates,
            clip_animators.get(clip_pid).map(Vec::as_slice),
            &is_ancestor,
        );
        let other_hashes = all_hashes.iter().copied().filter(|&h| h != scale_path);
        let Some(ctrl) = disambiguate_owner(&candidates, other_hashes, &hash_to_gos, &is_ancestor)
        else {
            continue;
        };
        // Take the position curve bound to the SCALE's own path — never whichever one the
        // binding walk happened to end on (see `pos_by_path`).
        let (own_pos_x, own_pos_y) = pos_by_path.remove(&scale_path).unwrap_or_default();
        let e = out.entry(ctrl).or_default();
        if scale.len() > e.scale.len() {
            e.scale = scale;
            e.pos_x = own_pos_x;
            e.pos_y = own_pos_y;
        }
    }
    out
}

/// Build a predicate that reports whether its first `GameObject` is an ancestor of
/// (or equal to) its second, following Transform `m_Father` links.
/// `GameObject` → parent `GameObject`, via the transform `m_Father` chain.
fn build_go_parent(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<i64, i64> {
    let mut go_parent: HashMap<i64, i64> = HashMap::new();
    let mut tr_go: HashMap<i64, i64> = HashMap::new();
    for (pid, (cid, v)) in all_objects {
        if (*cid == 4 || *cid == 224)
            && let Some(go) = v.get("m_GameObject").and_then(get_path_id)
        {
            tr_go.insert(*pid, go);
        }
    }
    for (cid, v) in all_objects.values() {
        if (*cid == 4 || *cid == 224)
            && let Some(go) = v.get("m_GameObject").and_then(get_path_id)
            && let Some(father_tr) = v.get("m_Father").and_then(get_path_id).filter(|&p| p != 0)
            && let Some(&father_go) = tr_go.get(&father_tr)
        {
            go_parent.insert(go, father_go);
        }
    }
    go_parent
}

/// A `GameObject`'s ancestor chain, root first, ending at the object itself.
fn root_path(go: i64, go_parent: &HashMap<i64, i64>) -> Vec<i64> {
    let mut chain = vec![go];
    let mut cur = go;
    for _ in 0..256 {
        let Some(&p) = go_parent.get(&cur) else { break };
        chain.push(p);
        cur = p;
    }
    chain.reverse();
    chain
}

fn build_ancestor_check(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> impl Fn(i64, i64) -> bool + '_ {
    let go_parent = build_go_parent(all_objects);
    move |g: i64, d: i64| -> bool {
        let mut cur = Some(d);
        for _ in 0..256 {
            let Some(c) = cur else { return false };
            if c == g {
                return true;
            }
            cur = go_parent.get(&c).copied();
        }
        false
    }
}

/// Map every `AnimationClip` (`path_id`) to the GameObject(s) whose Animator plays it.
///
/// A clip's binding path hashes are relative to the `GameObject` carrying the **Animator**
/// that plays the clip — so that `GameObject`'s subtree is the only scope in which a hash
/// may be resolved. The linkage is serialized in full: Animator (class 95) `m_Controller`
/// → `AnimatorController` (class 91) `m_AnimationClips`, an array of `{m_FileID, m_PathID}`
/// references to clips (class 74).
///
/// Mlynar "Fields of Ruination" is the case this exists for: the bundle ships twelve
/// Animators, one per sword/idle rig clone, each with its own controller listing only that
/// rig's clips. Six identically-pathed `static_offset/fixed/scale_01/scale02/glow_01`
/// blade quads collide on one hash, and the animator that plays the clip is what tells
/// them apart — no name matching involved.
///
/// A clip may legitimately be listed by SEVERAL Animators (a controller reused across rig
/// clones), hence a `Vec` per clip. `m_PathID == 0` and references that don't resolve to a
/// clip in this file (external `m_FileID` targets) are skipped.
fn build_clip_animator_gos(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<i64, Vec<i64>> {
    let mut out: HashMap<i64, Vec<i64>> = HashMap::new();
    for (cid, v) in all_objects.values() {
        if *cid != 95 {
            continue;
        }
        let Some(go) = v
            .get("m_GameObject")
            .and_then(get_path_id)
            .filter(|&p| p != 0)
        else {
            continue;
        };
        let Some(ctrl) = v
            .get("m_Controller")
            .and_then(get_path_id)
            .filter(|&p| p != 0)
        else {
            continue;
        };
        let Some((91, controller)) = all_objects.get(&ctrl) else {
            continue;
        };
        let clips = controller
            .get("m_AnimationClips")
            .and_then(Value::as_array)
            .into_iter()
            .flatten();
        for r in clips {
            if let Some(clip) = get_path_id(r).filter(|&p| p != 0)
                && all_objects.get(&clip).is_some_and(|(c, _)| *c == 74)
            {
                let owners = out.entry(clip).or_default();
                if !owners.contains(&go) {
                    owners.push(go);
                }
            }
        }
    }
    out
}

/// Narrow a binding hash's colliding candidates to the Animator subtree(s) that play the
/// clip: keep only candidates that ARE an animator `GameObject` or a descendant of one.
///
/// This can only ever shrink an ambiguous set. It falls back to the untouched candidate
/// list when the clip is listed by no Animator (an externally-referenced controller, or a
/// bundle that ships clips without their state machine) or when the restriction would
/// leave nothing — a binding that resolves today must keep resolving.
fn scope_to_animator<'a>(
    candidates: &'a [i64],
    animator_gos: Option<&[i64]>,
    is_ancestor: &impl Fn(i64, i64) -> bool,
) -> std::borrow::Cow<'a, [i64]> {
    let Some(roots) = animator_gos.filter(|r| !r.is_empty()) else {
        return std::borrow::Cow::Borrowed(candidates);
    };
    let kept: Vec<i64> = candidates
        .iter()
        .copied()
        .filter(|&c| roots.iter().any(|&root| is_ancestor(root, c)))
        .collect();
    if kept.is_empty() {
        std::borrow::Cow::Borrowed(candidates)
    } else {
        std::borrow::Cow::Owned(kept)
    }
}

/// Resolve a binding's owner among colliding path-hash candidates. A lone candidate
/// is always valid; otherwise the unique non-zero descendant-score winner is required.
fn disambiguate_owner(
    candidates: &[i64],
    other_hashes: impl Iterator<Item = u32>,
    hash_to_gos: &HashMap<u32, Vec<i64>>,
    is_ancestor: &impl Fn(i64, i64) -> bool,
) -> Option<i64> {
    if candidates.len() == 1 {
        return Some(candidates[0]);
    }
    let other_hashes: Vec<u32> = other_hashes.collect();
    let mut best: Option<(i64, usize)> = None;
    let mut tie = false;
    for &candidate in candidates {
        let score = other_hashes
            .iter()
            .filter(|&&hash| {
                hash_to_gos.get(&hash).is_some_and(|gos| {
                    gos.iter()
                        .any(|&go| go != candidate && is_ancestor(candidate, go))
                })
            })
            .count();
        match best {
            Some((_, best_score)) if score > best_score => {
                best = Some((candidate, score));
                tie = false;
            }
            Some((_, best_score)) if score == best_score => tie = true,
            None => best = Some((candidate, score)),
            _ => {}
        }
    }
    best.filter(|&(_, score)| score > 0 && !tie)
        .map(|(candidate, _)| candidate)
}

/// [`disambiguate_owner`], with fallbacks for LEAF targets.
///
/// The descendant score only resolves a candidate that CONTAINS the clip's other targets —
/// the shape a `ctrl` node with animated children has. A leaf resolves nothing: Mlynar
/// "Fields of Ruination" ships six identically-pathed `.../scale_01/scale02/glow_01` blade
/// quads, one per sword rig, so each sword clip's `m_IsActive` binding collides six ways,
/// scores zero everywhere, and the window is dropped — leaving every rig's glow ungated
/// instead of only the one the clip drives.
///
/// Two fallbacks, cheapest first:
/// 1. **Hierarchy proximity** — the candidate sharing the DEEPEST common ancestor with any
///    other target. Resolves a leaf whose siblings live under different parents.
/// 2. **Clip-name ↔ rig-root name** — identical rig instances defeat proximity too (their
///    subtrees are byte-identical, so every candidate ties). Unity names the instance after
///    the clip that drives it (`<clip>(Clone)`), which is the one signal that separates them.
///
/// Both require a STRICT winner, so a genuinely ambiguous binding stays dropped as before.
fn disambiguate_owner_near(
    candidates: &[i64],
    other_hashes: impl Iterator<Item = u32> + Clone,
    hash_to_gos: &HashMap<u32, Vec<i64>>,
    is_ancestor: &impl Fn(i64, i64) -> bool,
    go_parent: &HashMap<i64, i64>,
    clip_name: &str,
    go_name: &HashMap<i64, String>,
) -> Option<i64> {
    if let Some(go) = disambiguate_owner(candidates, other_hashes.clone(), hash_to_gos, is_ancestor)
    {
        return Some(go);
    }
    if candidates.len() < 2 {
        return None;
    }
    if !clip_name.is_empty() {
        let owned: Vec<i64> = candidates
            .iter()
            .copied()
            .filter(|&c| {
                root_path(c, go_parent).iter().any(|g| {
                    go_name.get(g).is_some_and(|n| {
                        n == clip_name
                            || n.strip_suffix("(Clone)")
                                .is_some_and(|b| b.trim_end() == clip_name)
                    })
                })
            })
            .collect();
        if owned.len() == 1 {
            return Some(owned[0]);
        }
    }
    let others: Vec<i64> = other_hashes
        .flat_map(|h| hash_to_gos.get(&h).into_iter().flatten().copied())
        .collect();
    if others.is_empty() {
        return None;
    }
    let mut best: Option<(i64, usize)> = None;
    let mut runner_up = 0usize;
    for &candidate in candidates {
        let cpath = root_path(candidate, go_parent);
        let score = others
            .iter()
            .filter(|&&o| o != candidate && !candidates.contains(&o))
            .map(|&o| {
                let opath = root_path(o, go_parent);
                cpath
                    .iter()
                    .zip(opath.iter())
                    .take_while(|(a, b)| a == b)
                    .count()
            })
            .max()
            .unwrap_or(0);
        match best {
            Some((_, b)) if score > b => {
                runner_up = b;
                best = Some((candidate, score));
            }
            Some((_, _)) if score > runner_up => runner_up = score,
            None => best = Some((candidate, score)),
            _ => {}
        }
    }
    // A win only counts when strictly deeper than every rival: an equal-depth tie means the
    // clip's targets sit above the fork and cannot distinguish the candidates.
    best.filter(|&(_, score)| score > runner_up && score > 1)
        .map(|(candidate, _)| candidate)
}

/// `GameObjects` whose `ParticleSystem` `EmissionModule.rateOverTime` is ANIMATED by a
/// state/transition clip other than the steady idle loop. In the MAIN (idle) scene
/// such emitters are QUIET at the steady state: the binding clips are one-shot
/// transitions/flourishes whose curves gate the big SERIALIZED constant (0 outside
/// their windows — the same relay convention as the entrance sword clips), so the
/// constant is a flourish peak, not an ambient rate. Verified against the game
/// recording: Mlynar's idle sword-rig confetti (`caidai` 60/s, `star_01_1` 150/s,
/// bound only by `IdleTo*`/special clips) never shows in the settled idle. A rate
/// animated by the steady idle-loop clip itself is left alone (that IS ambient).
#[must_use]
pub fn event_driven_rate_gos(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> std::collections::HashSet<i64> {
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let want = i64::from(crc32(b"EmissionModule.rateOverTime.scalar"));
    let mut out = std::collections::HashSet::new();
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 {
            continue;
        }
        let name = v
            .get("m_Name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();
        // The steady idle-loop clip's own animated rate (if any) IS the ambient rate.
        if name.contains("idle_toidle")
            || name.contains("idle_to_idle")
            || name.contains("idle_loop")
            || name.contains("idleloop")
        {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let animator_gos = clip_animators.get(clip_pid).map(Vec::as_slice);
        for b in bindings {
            let (tid, attr, path) = binding_fields(b);
            if tid == 198
                && attr == want
                && let Some(gos) = hash_to_gos.get(&path)
            {
                // Only the rig this clip's Animator drives is proven event-gated; the
                // identically-pathed emitters in sibling rigs keep their own verdict.
                out.extend(
                    scope_to_animator(gos, animator_gos, &is_ancestor)
                        .iter()
                        .copied(),
                );
            }
        }
    }
    out
}

/// Max envelope of several `(t, v)` curves over the union of their sample times. Each
/// curve contributes only inside `[first_key, clip_stop]` and 0 outside: Mlynar's three
/// sword clips form a RELAY (each ends on a 0→200 spike the next clip resets, and one
/// first-keys `150` mid-clip) — held-before/-after values would emit stars from t=0 and
/// forever after the flourish, which the game recording disproves. At `clip_stop` the
/// animator state exits and the override expires (same rule as `active_windows`). Runs
/// of equal values are thinned to their endpoints.
fn merge_curves_max(curves: &[(Vec<(f32, f32)>, f32)]) -> Vec<(f32, f32)> {
    // One frame — the edge sharpness of the on/off transitions at window bounds.
    const EDGE: f32 = 1.0 / 30.0;
    let mut times: Vec<f32> = curves
        .iter()
        .flat_map(|(c, stop)| {
            let first = c[0].0;
            c.iter()
                .map(|&(t, _)| t)
                // Explicit samples just OUTSIDE each curve's window, so the exported
                // envelope carries real 0-points at both edges — the frontend clamps
                // to the curve's endpoint values, and linear interpolation between
                // sparse points would otherwise ramp across the on/off boundary.
                .chain([(first - EDGE).max(0.0), *stop, stop - EDGE])
        })
        .filter(|t| t.is_finite() && *t >= 0.0)
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    times.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    let sample = |c: &[(f32, f32)], stop: f32, t: f32| -> f32 {
        if t < c[0].0 || t >= stop {
            return 0.0;
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
        c[c.len() - 1].1 // between the last key and the clip stop: held
    };
    let dense: Vec<(f32, f32)> = times
        .iter()
        .map(|&t| {
            let v = curves
                .iter()
                .map(|(c, stop)| sample(c, *stop, t))
                .fold(0.0f32, f32::max);
            (t, v.max(0.0))
        })
        .collect();
    // Thin interior points of constant runs (dense 30Hz resamples are mostly holds).
    let mut out = Vec::with_capacity(dense.len());
    for (i, &p) in dense.iter().enumerate() {
        let keep = i == 0
            || i == dense.len() - 1
            || (dense[i - 1].1 - p.1).abs() > 1e-4
            || (dense[i + 1].1 - p.1).abs() > 1e-4;
        if keep {
            out.push(p);
        }
    }
    out
}

/// CRC32("orthographic size") — the animated Camera (class 20) property that drives the
/// entrance dolly ZOOM (verified: the cello `start_animation_02` clip keys it 1.87→1.50→1.91).
const ORTHO_SIZE_CRC: i64 = 2_389_637_943;

/// The set of `GameObject` `path_id`s on the CAMERA's ancestor chain (the class-20 Camera's GO and
/// every `m_Father` above it). A camera dolly is authored as a Transform-position curve on one of
/// these (Virtuosa: the parent "Dummy002"), NOT on the Camera GO itself — which is why a
/// camera-GO-only scan misses it.
fn camera_ancestor_gos(all_objects: &HashMap<i64, (i32, Value)>) -> std::collections::HashSet<i64> {
    let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
    let mut tf_father: HashMap<i64, i64> = HashMap::new();
    let mut tf_go: HashMap<i64, i64> = HashMap::new();
    let mut cam_gos: Vec<i64> = Vec::new();
    for (pid, (cid, v)) in all_objects {
        match cid {
            4 | 224 => {
                if let Some(go) = v.get("m_GameObject").and_then(get_path_id) {
                    go_to_tf.insert(go, *pid);
                    tf_go.insert(*pid, go);
                }
                if let Some(f) = v.get("m_Father").and_then(get_path_id) {
                    tf_father.insert(*pid, f);
                }
            }
            20 => {
                if let Some(go) = v.get("m_GameObject").and_then(get_path_id) {
                    cam_gos.push(go);
                }
            }
            _ => {}
        }
    }
    let mut chain = std::collections::HashSet::new();
    for &cam_go in &cam_gos {
        let mut cur_tf = go_to_tf.get(&cam_go).copied();
        for _ in 0..64 {
            let Some(tf) = cur_tf else { break };
            if let Some(&go) = tf_go.get(&tf) {
                chain.insert(go);
            }
            cur_tf = tf_father.get(&tf).copied().filter(|&f| f != 0);
        }
    }
    chain
}

/// The ENTRANCE camera FRAME-CENTRE trajectory in authored px, computed PURELY from the gamedata
/// camera rig: accumulate the full camera ancestor chain (camera→root) as TRS matrices, with the
/// animated ancestor's position substituted per keyframe, to get the camera's WORLD position over
/// time; project the per-keyframe DELTA (from t0) onto the camera's screen right/up axes and scale
/// by `inv_scale` (1/skeletonScale) into authored px. Returned as `[(t_s, dxPx, dyPx)]` offsets
/// from the opening frame (dy is authored Y-UP, matching the mesh). `None` when no camera-ancestor
/// position is animated. Fully generic — no per-skin constants; the rig's rotations/scales are
/// honoured, so a differently-oriented camera projects correctly.
///
/// SPACE: the chain walk stops *before* any spine-root transform (a `GameObject` carrying the
/// `skeletonDataAsset` `MonoBehaviour`), exactly like the scene-mesh `accumulate_matrix`, so the
/// centre curve is expressed in the SAME spine-root-local frame as the exported `[scene]` quads.
/// Accumulating past the root into absolute world space breaks rigs whose entrance prefab root is
/// authored away from the origin (skadi2 "Iteration": root at (8.53, 25.19) → the camera track
/// landed ~2500 px below the scene and the whole entrance framed empty space).
/// Centre trajectory `[(t, cx, cy)]`, roll `[(t, degrees)]`, and aperture `([l,r,t,b], mode)` for
/// an entrance camera track — see `entrance_camera_track`.
pub type CameraTrack = (
    Option<Vec<(f32, f32, f32)>>,
    Option<Vec<(f32, f32)>>,
    Option<([f32; 4], i32)>,
);

#[must_use]
/// The ENTRANCE camera's frame-CENTRE trajectory and, when the rig is rolled, its ROLL.
///
/// Returns `(centre[(t, cx, cy)], roll[(t, degrees)])`. The roll is `None` unless the rig actually
/// rotates the camera about the view axis — 12 of the 13 entrance skins have an exactly
/// axis-aligned camera basis (right = +X, up = +Y, roll 0.000°), so they keep a `None` and their
/// exported centre is bit-identical.
/// How many keyframes make a camera-chain curve "animated"?
///
/// A SINGLE-key curve is still an OVERRIDE: Unity's Animator writes that value every frame,
/// replacing whatever `m_LocalPosition`/`m_LocalRotation` the transform serialised. Requiring
/// `> 1` therefore silently falls back to the STATIC pose for any node the clip merely PINS.
///
/// Chongyue is the case that exposed it. Her `_Start` pins `Dummy002`'s position at (0,0,0) with
/// one key, while its serialized `localPos` is (-3.2063, 0, -5.6353) — so we compose a camera
/// 3.21 units to the side and 5.64 up. Feature-matching her render against the capture measures
/// exactly that: a CONSTANT (-328.4 ± 13.5) authored px in X and (-593 ± 96) in Y, against a
/// chain-matrix prediction of (-320.6, -563.5). See `camfit.py`.
///
/// `DYNCHAR_CAMKEY1=0` restores the old `> 1` behaviour.
fn min_keys() -> usize {
    if std::env::var("DYNCHAR_CAMKEY1").as_deref() == Ok("0") {
        2
    } else {
        1
    }
}

pub fn entrance_camera_track(
    all_objects: &HashMap<i64, (i32, Value)>,
    inv_scale: f64,
    view_half: Option<f64>,
) -> CameraTrack {
    use super::mesh::Mat4;
    // Ordered chain camera→root (transform pids) + maps.
    let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
    let mut tf_father: HashMap<i64, i64> = HashMap::new();
    let mut tf_go: HashMap<i64, i64> = HashMap::new();
    let mut cam_go: Option<(i64, i64)> = None;
    for (pid, (cid, v)) in all_objects {
        match cid {
            4 | 224 => {
                if let Some(go) = v.get("m_GameObject").and_then(get_path_id) {
                    go_to_tf.insert(go, *pid);
                    tf_go.insert(*pid, go);
                }
                if let Some(f) = v.get("m_Father").and_then(get_path_id) {
                    tf_father.insert(*pid, f);
                }
            }
            // Deterministic camera pick: LOWEST path_id, not "whichever this HashMap happened to
            // yield last". A bundle shipping several cameras would otherwise flip between runs —
            // the exact bug already fixed for the `_cameraSize` display controllers in spine.rs.
            // (Every entrance bundle checked so far ships exactly one, so this is a guard.)
            20 => {
                if let Some(g) = v.get("m_GameObject").and_then(get_path_id) {
                    cam_go = Some(match cam_go {
                        Some((p0, g0)) if p0 <= *pid => (p0, g0),
                        _ => (*pid, g),
                    });
                }
            }
            _ => {}
        }
    }
    // Spine-root GameObjects: the walk must not include their transforms (or anything above),
    // matching the quad exporter's stop set so camera + scene share one frame.
    let spine_gos: std::collections::HashSet<i64> = all_objects
        .values()
        .filter(|(cid, v)| *cid == 114 && v.get("skeletonDataAsset").is_some())
        .filter_map(|(_, v)| v.get("m_GameObject").and_then(get_path_id))
        .collect();
    let mut chain: Vec<i64> = Vec::new(); // transform pids, camera→(just below the spine root)
    let mut cur = cam_go.and_then(|(_, g)| go_to_tf.get(&g).copied());
    for _ in 0..64 {
        let Some(tf) = cur else { break };
        if tf_go.get(&tf).is_some_and(|go| spine_gos.contains(go)) {
            break; // reached the spine root; don't include its transform
        }
        chain.push(tf);
        cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
    }
    if chain.is_empty() {
        return (None, None, None);
    }
    // Static local TRS of each chain transform. `unit_scale` builds the same node with scale 1
    // on every axis — used ONLY to derive the screen basis (see `axis` below), never to place the
    // camera.
    let local_trs = |tf: i64,
                     pos_override: Option<[f32; 3]>,
                     rot_override: Option<[f32; 4]>,
                     scale_override: Option<[f32; 3]>,
                     unit_scale: bool|
     -> Mat4 {
        let Some((_, v)) = all_objects.get(&tf) else {
            return Mat4::identity();
        };
        let vec3 = |field: &str, d: f32| {
            let g = |k: &str| {
                v.get(field)
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(d.into()) as f32
            };
            [g("x"), g("y"), g("z")]
        };
        let pos = pos_override.unwrap_or_else(|| vec3("m_LocalPosition", 0.0));
        let q = rot_override.unwrap_or_else(|| {
            let g = |k: &str, d: f32| {
                v.get("m_LocalRotation")
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(d.into()) as f32
            };
            [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
        });
        // A ZERO scale component is REAL, not a defect to be patched out. Unity flattens that
        // axis, so a child's local offset along it contributes NOTHING to the world position —
        // rigs use this deliberately to neutralise an axis of a camera's own local placement.
        // Muelsyse: `Dummy002` ships scale (1, 0, 1), which cancels the Main Camera's local
        // +2.9947 Y. Substituting 1 let that through as a 299.47 authored-px vertical error and
        // left X (scale 1, so unaffected) exactly right — the signature measured in the render:
        // MADC 87.168 -> 23.699 at a +300 px Y shift, with 0 px wanted in X.
        //
        // The substitution existed because the screen basis below is read off this matrix's
        // columns, and a collapsed column normalises to garbage. That is fixed properly by taking
        // the basis from a UNIT-SCALE build of the same chain: an orthonormal basis belongs to the
        // rotation, not to the scale.
        // ...and the collapse can be ANIMATED rather than serialized. Mlynar's `_Start` pins
        // `Dummy002`'s scale to (1, 0, 1) with single-key curves while the transform itself
        // ships (1, 1, 1), so reading only `m_LocalScale` let his Main Camera's local +2.9947 Y
        // through — 0.912 (the `static_offset` scale) x 2.9947 = 2.731 units = 273 authored px of
        // constant vertical error, measured at +271.6 +/- 2.0 px against his capture.
        let s = if unit_scale {
            [1.0, 1.0, 1.0]
        } else {
            scale_override.unwrap_or_else(|| vec3("m_LocalScale", 1.0))
        };
        Mat4::trs(pos, q, s)
    };
    // Find EVERY animated chain transform + its 3-component position curve. A camera dolly may split
    // its motion across SEVERAL ancestor transforms (e.g. one that dips to follow a falling prop, one
    // that lifts for the reform), so we must accumulate all of them — capturing only the first would
    // drop part of the move. Keyed by transform pid → per-component position curves.
    let chain_gos: std::collections::HashSet<i64> = chain
        .iter()
        .filter_map(|&tf| {
            all_objects
                .get(&tf)
                .and_then(|(_, v)| v.get("m_GameObject"))
                .and_then(get_path_id)
        })
        .collect();
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let mut animated: HashMap<i64, [Vec<(f32, f32)>; 3]> = HashMap::new();
    // SCALE curves on the chain. A clip can COLLAPSE an axis (scale 0) to neutralise a child's
    // local offset along it — see `local_trs`. Never sampled before, so such a pin was ignored.
    let mut animated_scale: HashMap<i64, [Vec<(f32, f32)>; 3]> = HashMap::new();
    // ROTATION curves on the same chain, keyed transform pid -> (is_euler, per-component curves).
    // Wiš'adel's `_Start` animates her camera parent's EULER Z from 11.338° to 29.556° over the
    // first 2.4 s; reading only `attr == 1` froze the shot at its opening roll.
    // One-off internal accumulator (transform pid -> (is_euler, per-component curves)); a named
    // alias for this single use site would add indirection rather than clarity.
    #[allow(clippy::type_complexity)]
    let mut animated_rot: HashMap<i64, (bool, Vec<Vec<(f32, f32)>>)> = HashMap::new();
    let cam_clips = camera_motion_clips(all_objects);
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 || !cam_clips.contains(clip_pid) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let animator_gos = clip_animators.get(clip_pid).map(Vec::as_slice);
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            let count = binding_curve_count(type_id, attr);
            // On a subpath-hash collision (same-named twin rigs), scope to the Animator
            // that plays this clip and then pick the candidate that is actually on the
            // camera chain — an arbitrary pick could name the wrong twin and drop the
            // camera move entirely.
            let is_pos = type_id == 4 && attr == 1;
            let is_rot = type_id == 4 && (attr == 2 || attr == 4);
            let is_scale = type_id == 4 && attr == 3;
            let scoped = (is_pos || is_rot || is_scale)
                .then(|| hash_to_gos.get(&path))
                .flatten()
                .map(|gos| scope_to_animator(gos, animator_gos, &is_ancestor));
            if is_rot
                && let Some(&go) = scoped
                    .as_deref()
                    .and_then(|gos| gos.iter().find(|g| chain_gos.contains(g)))
                && let Some(&tf) = go_to_tf.get(&go)
            {
                // `decode_curve_any`, NOT `decode_curve_at`: the latter reads only the STREAMED
                // sub-clip, and a curve that merely PINS a value lives in the CONSTANT one.
                let cs: Vec<Option<Vec<(f32, f32)>>> =
                    (0..count).map(|i| decode_curve_any(v, gidx + i)).collect();
                if cs
                    .iter()
                    .any(|c| c.as_ref().is_some_and(|c| c.len() >= min_keys()))
                {
                    let entry = animated_rot
                        .entry(tf)
                        .or_insert_with(|| (attr == 4, vec![Vec::new(); count]));
                    for (i, c) in cs.into_iter().enumerate() {
                        if let Some(c) = c
                            && i < entry.1.len()
                            && c.len() > entry.1[i].len()
                        {
                            entry.1[i] = c;
                        }
                    }
                }
            }
            if is_scale
                && let Some(&go) = scoped
                    .as_deref()
                    .and_then(|gos| gos.iter().find(|g| chain_gos.contains(g)))
                && let Some(&tf) = go_to_tf.get(&go)
            {
                let cs = [
                    decode_curve_any(v, gidx),
                    decode_curve_any(v, gidx + 1),
                    decode_curve_any(v, gidx + 2),
                ];
                if cs
                    .iter()
                    .any(|c| c.as_ref().is_some_and(|c| c.len() >= min_keys()))
                {
                    let entry = animated_scale
                        .entry(tf)
                        .or_insert_with(|| [Vec::new(), Vec::new(), Vec::new()]);
                    for (i, c) in cs.into_iter().enumerate() {
                        if let Some(c) = c
                            && c.len() > entry[i].len()
                        {
                            entry[i] = c;
                        }
                    }
                }
            }
            if is_pos
                && let Some(&go) = scoped
                    .as_deref()
                    .and_then(|gos| gos.iter().find(|g| chain_gos.contains(g)))
                && let Some(&tf) = go_to_tf.get(&go)
            {
                let cs = [
                    decode_curve_any(v, gidx),
                    decode_curve_any(v, gidx + 1),
                    decode_curve_any(v, gidx + 2),
                ];
                if cs
                    .iter()
                    .any(|c| c.as_ref().is_some_and(|c| c.len() >= min_keys()))
                {
                    let entry = animated
                        .entry(tf)
                        .or_insert_with(|| [Vec::new(), Vec::new(), Vec::new()]);
                    // Same transform animated in multiple clips: keep the richer curve per axis.
                    for (i, c) in cs.into_iter().enumerate() {
                        if let Some(c) = c
                            && c.len() > entry[i].len()
                        {
                            entry[i] = c;
                        }
                    }
                }
            }
            gidx += count;
        }
    }
    // A camera that never MOVES still has a position, and dropping it costs real behaviour: the
    // renderer only engages its entrance follow (and therefore the authored ORTHO zoom) when a
    // centre curve exists. Kal'tsit ships 302 ortho keys with a static chain, so her whole
    // entrance zoom was being discarded; Civilight Eterna is fully static and was framed on the
    // IDLE's tight bounds instead of her own entrance camera. Emit a constant two-key curve for
    // them — `world_at` reads the static TRS when nothing is animated, so the value is exact.
    let is_static = animated.is_empty() && animated_rot.is_empty() && animated_scale.is_empty();
    // Timeline = union of EVERY animated axis's keyframe times.
    let mut times: Vec<f32> = animated
        .values()
        .flatten()
        .flatten()
        .map(|(t, _)| *t)
        .collect();
    times.extend(
        animated_rot
            .values()
            .flat_map(|(_, cs)| cs.iter().flatten())
            .map(|(t, _)| *t),
    );
    // SCALE times too. `is_static` counts a scale-only animated chain as NON-static, so without
    // this a rig whose clip animates ONLY scale would reach the `times.len() < 2` guard below and
    // lose its camera track entirely — worse than the static fallback it would have had. No skin
    // in the corpus is scale-only today; this keeps that from becoming a silent trap.
    times.extend(animated_scale.values().flatten().flatten().map(|(t, _)| *t));
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    times.dedup();
    if is_static {
        times = vec![0.0, 1.0];
    }
    if times.len() < 2 {
        return (None, None, None);
    }
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
    // A transform's static local position (fallback for its un-animated axes).
    let static_pos = |tf: i64| -> [f32; 3] {
        all_objects.get(&tf).map_or([0.0; 3], |(_, v)| {
            let g = |k: &str| {
                v.get("m_LocalPosition")
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0) as f32
            };
            [g("x"), g("y"), g("z")]
        })
    };
    // An animated transform's local position at time t (animated axes sampled, others static).
    let sample_tf = |tf: i64, t: f32| -> [f32; 3] {
        let sp = static_pos(tf);
        match animated.get(&tf) {
            Some(curves) => [
                sample(&curves[0], t, sp[0]),
                sample(&curves[1], t, sp[1]),
                sample(&curves[2], t, sp[2]),
            ],
            None => sp,
        }
    };
    // An animated transform's local ROTATION at time t, as a quaternion. Euler curves (attr 4)
    // are Unity degrees in ZXY order; quaternion curves (attr 2) are used as-is.
    // An animated transform's local SCALE at time t (animated axes sampled, others static).
    let sample_scale_tf = |tf: i64, t: f32| -> [f32; 3] {
        let st = all_objects.get(&tf).map_or([1.0f32; 3], |(_, v)| {
            let g = |k: &str| {
                v.get("m_LocalScale")
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(1.0) as f32
            };
            [g("x"), g("y"), g("z")]
        });
        match animated_scale.get(&tf) {
            Some(cs) => [
                sample(&cs[0], t, st[0]),
                sample(&cs[1], t, st[1]),
                sample(&cs[2], t, st[2]),
            ],
            None => st,
        }
    };
    let sample_rot_tf = |tf: i64, t: f32| -> Option<[f32; 4]> {
        let (is_euler, cs) = animated_rot.get(&tf)?;
        if *is_euler {
            let e: Vec<f32> = (0..3)
                .map(|i| cs.get(i).map_or(0.0, |c| sample(c, t, 0.0)))
                .collect();
            let (rx, ry, rz) = (e[0].to_radians(), e[1].to_radians(), e[2].to_radians());
            let (cx, sx) = ((rx * 0.5).cos(), (rx * 0.5).sin());
            let (cy, sy) = ((ry * 0.5).cos(), (ry * 0.5).sin());
            let (cz, sz) = ((rz * 0.5).cos(), (rz * 0.5).sin());
            let qmul = |a: [f32; 4], b: [f32; 4]| {
                [
                    a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
                    a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
                    a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
                    a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
                ]
            };
            // Unity's Quaternion.Euler(x, y, z) composes as Ry * Rx * Rz.
            Some(qmul(
                qmul([0.0, sy, 0.0, cy], [sx, 0.0, 0.0, cx]),
                [0.0, 0.0, sz, cz],
            ))
        } else {
            let q: Vec<f32> = (0..4)
                .map(|i| {
                    let d = if i == 3 { 1.0 } else { 0.0 };
                    cs.get(i).map_or(d, |c| sample(c, t, d))
                })
                .collect();
            Some([q[0], q[1], q[2], q[3]])
        }
    };
    // Accumulate the world matrix (root→camera) at time t, substituting EVERY animated transform's
    // sampled position AND rotation.
    let world_at = |t: f32, unit_scale: bool| -> Mat4 {
        let mut m = Mat4::identity();
        for &tf in chain.iter().rev() {
            let pos = animated.contains_key(&tf).then(|| sample_tf(tf, t));
            let rot = sample_rot_tf(tf, t);
            let scl = animated_scale
                .contains_key(&tf)
                .then(|| sample_scale_tf(tf, t));
            let local = local_trs(tf, pos, rot, scl, unit_scale);
            m = m.mul(&local);
        }
        m
    };
    // The camera's ROLL about its view axis, per frame, in the spine-root plane.
    let roll_at = |t: f32| -> f32 {
        let m = world_at(t, true);
        let c = [m.0[0][0], m.0[1][0], m.0[2][0]];
        let n = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().max(1e-6);
        (c[1] / n).atan2(c[0] / n).to_degrees()
    };
    let inv = inv_scale as f32;
    let mut out = Vec::with_capacity(times.len());
    let mut rolls = Vec::with_capacity(times.len());
    for &t in &times {
        let p = world_at(t, false).point([0.0, 0.0, 0.0]);
        // ABSOLUTE frame centre in the spine mesh space (authored px). An orthographic camera
        // aimed at the spine PLANE centres the frame on its own position expressed in THAT
        // plane's axes — so read the world X/Y directly. The previous form projected onto the
        // CAMERA's own right/up, which is identical while the rig is unrolled (12 of the 13
        // entrance skins have an exactly axis-aligned basis: right = +X, up = +Y, roll 0.000°)
        // but folds a rolled rig's rotation INTO the centre, which is wrong twice over — the
        // centre moves when it should not, and the roll never reaches the renderer.
        // Wiš'adel is the one rolled rig; her measured correction (+182 px, +25..32 px against
        // the old form) matches this model's (+182.5, +33.4).
        // `y` is negated because the spine plane is placed Y-flipped, so mesh space is Y-UP.
        out.push((t, p[0] * inv, -p[1] * inv));
        rolls.push((t, roll_at(t)));
    }
    // Emit the roll only when the rig is actually rolled, so every axis-aligned skin keeps a
    // `None` and its exported scene JSON stays byte-identical.
    let rolled = rolls.iter().any(|(_, r)| r.abs() > 0.01);
    let cam0 = world_at(times[0], false).point([0.0, 0.0, 0.0]);
    let aperture =
        view_half.and_then(|vh| find_letterbox(all_objects, [cam0[0], cam0[1]], inv, vh as f32));
    (Some(out), rolled.then_some(rolls), aperture)
}

/// The entrance LETTERBOX, when the prefab paints one.
///
/// Civilight Eterna's cinematic renders into a hard 16:9 window inside the 2340x1080 screen, and
/// that has no per-skin rule behind it: her prefab simply ships FOUR opaque planes whose inner
/// edges bound the window. `BG_black_02` stops at x −18.83, `BG_black_01` starts at x +9.26,
/// `BG_black_04` stops at y +7.09, `BG_black_03` starts at y +22.90 — a 28.09 x 15.81 window,
/// aspect **1.777**, whose height equals her `entranceViewPx` (15.80) exactly. She is the only
/// entrance skin that ships them; the other twelve have none, which is why every universal rule
/// ever tried (director, `_maxSize`, camera count, `skin_table`, release date) came back identical.
///
/// Detected GEOMETRICALLY, never by name: a quad is a bar when it spans the camera centre on one
/// axis and lies wholly to one side on the other. The aperture is the intersection of the inner
/// edges. Returned in the same authored-px space as the frame centre (so `y` is negated the same
/// way), as `[x0, y0, x1, y1]`, and only when it is a strict sub-rectangle of the camera's view —
/// so a skin without a letterbox emits `None` and its scene JSON is unchanged.
///
/// ⚠️ These planes are dropped by the `_meshExtResolved` + no-window gate in `collect_dynchar_bg_quads`
/// and this deliberately does NOT relax that gate: splitting it by scene was tried before and cost
/// Skadi a real regression. The geometry is read directly instead.
///
/// Returned WITH the bars' own `m_SortingOrder`, because a letterbox is not automatically the
/// topmost thing on screen: Civilight Eterna's bars sort at 100, exactly TIED with the two
/// full-screen `Transition_*` planes the director flashes, and the game paints her end-of-cinematic
/// white fade OVER the bars (its pillarbox ramps to 0.99 alpha while ours stayed at 0.00 for the
/// whole fade). Drawing the bars unconditionally last cost ~20 luma on that beat.
fn find_letterbox(
    all_objects: &HashMap<i64, (i32, Value)>,
    cam_xy: [f32; 2],
    inv: f32,
    view_half: f32,
) -> Option<([f32; 4], i32)> {
    use super::mesh::Mat4;
    let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
    let mut tf_father: HashMap<i64, i64> = HashMap::new();
    for (pid, (cid, v)) in all_objects {
        if matches!(cid, 4 | 224) {
            if let Some(g) = v.get("m_GameObject").and_then(get_path_id) {
                go_to_tf.insert(g, *pid);
            }
            if let Some(f) = v.get("m_Father").and_then(get_path_id) {
                tf_father.insert(*pid, f);
            }
        }
    }
    let local_of = |tf: i64| -> Mat4 {
        let Some((_, v)) = all_objects.get(&tf) else {
            return Mat4::identity();
        };
        let vec3 = |field: &str, d: f32| {
            let g = |k: &str| {
                v.get(field)
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(d.into()) as f32
            };
            [g("x"), g("y"), g("z")]
        };
        let q = {
            let g = |k: &str, d: f32| {
                v.get("m_LocalRotation")
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(d.into()) as f32
            };
            [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
        };
        Mat4::trs(vec3("m_LocalPosition", 0.0), q, vec3("m_LocalScale", 1.0))
    };
    // GameObject -> `m_SortingOrder` of its MeshRenderer, the same scale every scene quad is
    // ordered on. Read here rather than assumed, so the bars take their authored place in the
    // draw order instead of being pinned to the top.
    let mut go_sort: HashMap<i64, i32> = HashMap::new();
    for (cid, v) in all_objects.values() {
        if *cid != 23 {
            continue; // MeshRenderer
        }
        if let Some(g) = v.get("m_GameObject").and_then(get_path_id) {
            go_sort.insert(
                g,
                v.get("m_SortingOrder").and_then(Value::as_i64).unwrap_or(0) as i32,
            );
        }
    }
    // (left inner, right inner, bottom inner, top inner) in WORLD units
    let (mut li, mut ri, mut bi, mut ti) = (
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
    );
    let mut bars = 0usize;
    // The bars share one sorting order in practice; take the HIGHEST so the exported value is the
    // one that must be cleared for a layer to draw over the whole frame.
    let mut bar_sort = i32::MIN;
    for (cid, v) in all_objects.values() {
        if *cid != 33 {
            continue; // MeshFilter
        }
        let Some(go) = v.get("m_GameObject").and_then(get_path_id) else {
            continue;
        };
        // the owning GameObject must be active, else it paints nothing
        if !all_objects
            .get(&go)
            .and_then(|(_, gv)| gv.get("m_IsActive"))
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            continue;
        }
        let Some((_, mv)) = v
            .get("m_Mesh")
            .and_then(get_path_id)
            .and_then(|m| all_objects.get(&m))
        else {
            continue;
        };
        let Some(ab) = mv.get("m_LocalAABB") else {
            continue;
        };
        let g = |f: &str, k: &str| {
            ab.get(f)
                .and_then(|x| x.get(k))
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32
        };
        let (cx, cy) = (g("m_Center", "x"), g("m_Center", "y"));
        let (ex, ey) = (g("m_Extent", "x"), g("m_Extent", "y"));
        if ex <= 0.0 || ey <= 0.0 {
            continue;
        }
        let _ = ey;
        // world matrix of this GameObject
        let mut m = Mat4::identity();
        let mut cur = go_to_tf.get(&go).copied();
        let mut stack: Vec<i64> = Vec::new();
        for _ in 0..64 {
            let Some(tf) = cur else { break };
            stack.push(tf);
            cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
        }
        for &tf in stack.iter().rev() {
            m = m.mul(&local_of(tf));
        }
        // world AABB from the four transformed corners (rotation matters — the rigs are rotated)
        let (mut x0, mut x1, mut y0, mut y1) = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
        );
        for sx in [-1.0f32, 1.0] {
            for sy in [-1.0f32, 1.0] {
                let p = m.point([cx + sx * ex, cy + sy * ey, 0.0]);
                x0 = x0.min(p[0]);
                x1 = x1.max(p[0]);
                y0 = y0.min(p[1]);
                y1 = y1.max(p[1]);
            }
        }
        // A LETTERBOX bar reaches beyond the frame on both axes — that is what makes it a mask
        // rather than a prop. Without this every small quad sitting off to one side qualifies and
        // five skins pick up a bogus aperture.
        let view = 2.0 * view_half;
        if (x1 - x0) < view || (y1 - y0) < view {
            continue;
        }
        let (kx, ky) = (cam_xy[0], cam_xy[1]);
        let spans_x = x0 <= kx && kx <= x1;
        let spans_y = y0 <= ky && ky <= y1;
        let is_bar = if spans_y && x1 < kx {
            li = li.max(x1);
            true
        } else if spans_y && x0 > kx {
            ri = ri.min(x0);
            true
        } else if spans_x && y1 < ky {
            bi = bi.max(y1);
            true
        } else if spans_x && y0 > ky {
            ti = ti.min(y0);
            true
        } else {
            false
        };
        if is_bar {
            bars += 1;
            bar_sort = bar_sort.max(go_sort.get(&go).copied().unwrap_or(0));
        }
    }
    // All FOUR bars, or it is not a frame.
    if bars < 4
        || !li.is_finite()
        || !ri.is_finite()
        || !bi.is_finite()
        || !ti.is_finite()
        || ri <= li
        || ti <= bi
    {
        return None;
    }
    // A LETTERBOX crops the WIDTH and preserves the camera's view HEIGHT — that is what makes it
    // a letterbox rather than an arrangement of props that happens to surround the centre. Both
    // conditions are needed: Kal'tsit and Executor each produce a four-sided window from ordinary
    // scenery, but theirs are PORTRAIT (aspect 0.84 and 0.39) and Kal'tsit's is 42% off the view
    // height. Civilight Eterna's is aspect 1.7767 with a height of 1581 against her `entranceViewPx`
    // of 1580 — 0.06% out.
    let (w, h) = ((ri - li) * inv, (ti - bi) * inv);
    let view_px = 2.0 * view_half * inv;
    if w < h || (h / view_px - 1.0).abs() > 0.02 {
        return None;
    }
    // Same authored-px space as the frame centre: y negated.
    Some((
        [li * inv, -ti * inv, ri * inv, -bi * inv],
        if bar_sort == i32::MIN { 0 } else { bar_sort },
    ))
}

/// The ENTRANCE camera POSITIONAL dolly (pan), extracted from the `_Start` clip that animates a
/// camera-ancestor's Transform position (Virtuosa "Dummy002"). Returned as `(time_s, progress)`
/// where progress ∈ [0,1] normalises the DOMINANT-axis movement (the pan travels almost entirely
/// along one axis — the vertical follow of seated→standing). The frontend replays it as the pan
/// timing between the measured seated & standing framings — the game's exact camera-move timing,
/// no hardcoding. `None` when no camera-ancestor position is animated.
/// CRC32("weight") — the animated `PostProcessVolume.weight` the entrance clips drive.
const PP_WEIGHT_CRC: u32 = 0x07cd_5541;

/// The entrance POST-PROCESS chain: which effect the `_Start` cinematic runs, how strong it is,
/// and the animated volume weight that fades it in and out.
///
/// Every dyn-illust entrance ships a `GameObject` named `pp` carrying a Unity `PostProcessVolume`
/// (`isGlobal: 1`, `weight: 0.0` at rest) pointing at a per-skin `sharedProfile`, plus the
/// `Hidden/PostProcessing/*` shaders. The `_Start` clip then animates the volume's **weight**, so
/// the effect ramps in over the cinematic. We were reproducing none of it.
///
/// Whislash the Decadenza's profile is **`HGGreyScale` at intensity 1.0**, and her capture is
/// objectively monochrome early on — mean saturation **0.000 at beat 2 and 0.006 at beat 4** —
/// while we render full colour. Desaturating our render at a constant full weight already takes
/// her 77.51 -> 71.52.
///
/// Returns `(effect_name, intensity, weight_curve)`. `None` when the skin ships no volume, no
/// profile, or no clip animating the weight — so a skin whose volume never opens (Civilight
/// Eterna: an `HGMobileBlur` profile that no clip ever drives) exports nothing and is untouched.
///
/// ⚠️ Iterates objects in `path_id` order, never a `HashMap`: the probe that found this printed a
/// different matching clip on each run because it walked hash order.
/// `(effect_name, intensity, weight_curve[(t, weight)])` — see `entrance_post_fx`. Also used by
/// `SpineAssets::bg_entrance_post_fx`.
/// `(effect, intensity, weight_curve, params)` — `params` are the settings object's own
/// overridden scalar values (`blurDegree`, `blurSpread`, `quality`, `resMode`, ...), so an effect
/// that needs a magnitude gets it from the profile rather than a fitted constant.
pub type EntrancePostFx = (String, f32, Vec<(f32, f32)>, Vec<(String, f32)>);

#[must_use]
pub fn entrance_post_fx(all_objects: &HashMap<i64, (i32, Value)>) -> Option<EntrancePostFx> {
    let ordered = super::spine::objects_by_path_id_pub(all_objects);
    // The `pp` volume: a MonoBehaviour carrying `sharedProfile` + `isGlobal`.
    let (vol_go, profile_pid) = ordered.iter().find_map(|(_, (cid, v))| {
        if *cid != 114 {
            return None;
        }
        let prof = v
            .get("sharedProfile")
            .and_then(get_path_id)
            .filter(|&p| p != 0)?;
        let go = v.get("m_GameObject").and_then(get_path_id)?;
        Some((go, prof))
    })?;
    // Its profile's first settings entry names the effect and carries its intensity.
    let (effect, intensity, params) = ordered.iter().find_map(|(pid, (_, v))| {
        if **pid != profile_pid {
            return None;
        }
        let first = v.get("settings").and_then(Value::as_array)?.first()?;
        let sp = first.get("m_PathID").and_then(Value::as_i64)?;
        ordered.iter().find_map(|(p2, (_, sv))| {
            if **p2 != sp {
                return None;
            }
            let name = sv.get("m_Name").and_then(Value::as_str)?.to_string();
            let inten = sv
                .get("intensity")
                .and_then(|i| i.get("value"))
                .and_then(Value::as_f64)
                .unwrap_or(1.0) as f32;
            // Every OVERRIDDEN scalar the settings object carries. Post-process settings are
            // `{overrideState, value}` pairs; an entry with `overrideState == 0` is an inert
            // default the volume does not apply, so it is skipped rather than exported as if
            // it were authored. `HGMobileBlur` supplies `blurDegree` / `blurSpread` / `quality`
            // / `resMode` this way, which is the magnitude a blur needs.
            let mut params: Vec<(String, f32)> = sv
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(k, o)| {
                    if k == "intensity" {
                        return None;
                    }
                    let on = o.get("overrideState").and_then(Value::as_i64).unwrap_or(0);
                    let v = o.get("value").and_then(Value::as_f64)?;
                    (on != 0).then(|| (k.clone(), v as f32))
                })
                .collect();
            params.sort_by(|a, b| a.0.cmp(&b.0));
            Some((name, inten, params))
        })
    })?;
    // The weight curve: whichever clip binds crc32("weight") on the volume's GameObject.
    let hash_to_gos = build_hash_to_gos(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    for (clip_pid, (cid, v)) in &ordered {
        if *cid != 74 {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let animator_gos = clip_animators.get(clip_pid).map(Vec::as_slice);
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            let count = binding_curve_count(type_id, attr);
            if type_id == 114 && (attr as u32) == PP_WEIGHT_CRC {
                let hits = hash_to_gos
                    .get(&path)
                    .map(|gos| scope_to_animator(gos, animator_gos, &is_ancestor));
                if hits.as_deref().is_some_and(|g| g.contains(&vol_go))
                    && let Some(curve) = decode_curve_at(v, gidx)
                    && curve.len() > 1
                {
                    return Some((effect, intensity, curve, params));
                }
            }
            gidx += count;
        }
    }
    None
}

pub fn entrance_pan_curve(all_objects: &HashMap<i64, (i32, Value)>) -> Option<Vec<(f32, f32)>> {
    let chain = camera_ancestor_gos(all_objects);
    if chain.is_empty() {
        return None;
    }
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let mut best: Option<Vec<(f32, f32)>> = None; // the largest-range component curve
    let mut best_range = 0.0f32;
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let animator_gos = clip_animators.get(clip_pid).map(Vec::as_slice);
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            let count = binding_curve_count(type_id, attr);
            // Collision-robust: scope to the Animator playing this clip, then pick the
            // candidate GO on the camera chain (see above).
            let go = hash_to_gos
                .get(&path)
                .map(|gos| scope_to_animator(gos, animator_gos, &is_ancestor))
                .as_deref()
                .and_then(|gos| gos.iter().find(|g| chain.contains(g)))
                .copied()
                .unwrap_or(0);
            if type_id == 4 && attr == 1 && chain.contains(&go) {
                for c in 0..count {
                    if let Some(curve) = decode_curve_at(v, gidx + c) {
                        let (mn, mx) = curve
                            .iter()
                            .fold((f32::MAX, f32::MIN), |(a, b), &(_, val)| {
                                (a.min(val), b.max(val))
                            });
                        let range = mx - mn;
                        if range > best_range {
                            best_range = range;
                            best = Some(curve);
                        }
                    }
                }
            }
            gidx += count;
        }
    }
    // Normalise the dominant curve to progress 0..1 (start value → 0). Skip degenerate pans.
    let curve = best?;
    if best_range < 1e-3 {
        return None;
    }
    let v0 = curve.first()?.1;
    let (mn, mx) = curve
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &(_, val)| {
            (a.min(val), b.max(val))
        });
    // Orient so progress rises from the start pose regardless of the axis sign.
    let (lo, hi) = if (v0 - mn).abs() <= (v0 - mx).abs() {
        (mn, mx)
    } else {
        (mx, mn)
    };
    let span = hi - lo;
    Some(
        curve
            .into_iter()
            .map(|(t, val)| (t, ((val - lo) / span).clamp(0.0, 1.0)))
            .collect(),
    )
}

/// Decode the streamed curve at global index `idx` into `(time, value)` (streamed only —
/// enough for the camera curves). Boundary frames skipped. See {@link `decode_streamed_curve`}
/// for the key/easing semantics.
fn decode_curve_at(clip: &Value, idx: usize) -> Option<Vec<(f32, f32)>> {
    let data = clip_data(clip)?;
    let streamed_raw = u32_array(data.get("m_StreamedClip").and_then(|s| s.get("data")));
    decode_streamed_curve(&streamed_raw, idx)
}

/// Dense-resample rate (Hz) for eased segments in {@link `decode_streamed_curve}`: high
/// enough that the frontend's linear interpolation between samples reproduces the
/// authored cubic easing, low enough to keep the exported curves small.
const CURVE_RESAMPLE_HZ: f32 = 30.0;

/// Decode ONE streamed curve (global index `idx`) into `(time, value)` samples that honour
/// the authored keyframe TIMES and cubic EASING:
/// - a sample is emitted only where THIS curve has a real key. Streamed frames exist for
///   every keyed curve in the clip, so pushing `(frame_time, last_seen_value)` at frames
///   where only a SIBLING curve is keyed (the previous behaviour) fabricated flat holds
///   that shoved a key's onset to the next unrelated frame — e.g. the cello entrance
///   camera plunge (keys at 6.07s → 10.07s) became a hold-until-8.57s-then-linear jerk;
/// - between consecutive keys the segment cubic `v(dt) = c0·dt³ + c1·dt² + c2·dt + v0`
///   (`dt` = seconds since the segment's start key, coefficients from that key) is densely
///   resampled at ~{@link `CURVE_RESAMPLE_HZ`}, skipped when it doesn't deviate from the
///   straight chord (constant / linear segments need only their endpoints).
///
/// The boundary padding frame (index 0) and the ±inf sentinel frames are skipped, as before.
fn decode_streamed_curve(streamed_raw: &[u32], idx: usize) -> Option<Vec<(f32, f32)>> {
    // The curve's real keys: (time, segment coeffs c0..c2, value).
    let mut curve_keys: Vec<(f32, [f32; 3], f32)> = Vec::new();
    for (fi, (time, keys)) in read_streamed_timed(streamed_raw).into_iter().enumerate() {
        if fi == 0 || !time.is_finite() {
            continue;
        }
        for k in &keys {
            if k.index == idx {
                curve_keys.push((time.max(0.0), k.coeff, k.value));
            }
        }
    }
    let mut out = Vec::new();
    for w in curve_keys.windows(2) {
        let (t0, c, v0) = w[0];
        let t1 = w[1].0;
        out.push((t0, v0));
        let dur = t1 - t0;
        if dur <= 0.0 {
            continue;
        }
        // Upper bound on the cubic's deviation from the straight chord between the two
        // keys; when negligible, the segment endpoints alone reproduce it.
        let dev = c[0].abs() * dur * dur * dur + c[1].abs() * dur * dur;
        if dev <= 1e-4 {
            continue;
        }
        let steps = (dur * CURVE_RESAMPLE_HZ).ceil().max(1.0) as usize;
        for s in 1..steps {
            let dt = dur * s as f32 / steps as f32;
            let v = ((c[0] * dt + c[1]) * dt + c[2]) * dt + v0;
            out.push((t0 + dt, v));
        }
    }
    if let Some(&(tl, _, vl)) = curve_keys.last() {
        out.push((tl, vl));
    }
    (!out.is_empty()).then_some(out)
}

/// The ENTRANCE camera dolly, extracted from the `_Start` `AnimationClip` that animates the
/// Main Camera's orthographic size: `(time_seconds, orthographic_size)` keyframes. This is the
/// game's actual data-driven camera zoom (hold → zoom in on the transform → zoom out to the
/// standing reveal) — there is NO positional camera pan (no Transform curve on the camera).
/// `None` when no clip animates the camera ortho size (most dynchars). The frontend replays it
/// as a relative zoom on the entrance frame, so no world↔authored unit conversion is needed.
#[must_use]
/// The entrance camera's DOLLY curve for a PERSPECTIVE camera: `(time_s, distance)` from the
/// camera's own animated local Z.
///
/// Almost every dyn-illust entrance camera is orthographic, where a Z move is pure depth and
/// cannot reframe anything — which is why the exporter only ever read `orthographic size`.
/// Whislash the Decadenza (`char_1038_whitw2_sale#15`) is the exception: her Main Camera is
/// `orthographic = 0`, fov 60, and her `_Start` clip animates the camera's position on **Z only**
/// (3.000 -> 1.224 at t=2.8, then back out), leaving X/Y at zero. Read as an orthographic rig that
/// is exactly a camera which never moves, so her whole 14.5 s cinematic framed on one static box.
///
/// Under perspective the visible extent at the subject plane is `2·d·tan(fov/2)`, i.e. LINEAR in
/// the camera distance — so returning `(t, d)` here lets the existing consumer treat it exactly
/// like an ortho-size curve: the client's `orthoZoomRatio` divides by the first keyframe, and
/// `d(t)/d(0)` IS the perspective scale ratio. No client change, and no new units.
///
/// Verified numerically before it was written: at t=2 the model puts her extent at 293.1 px
/// against the 806.0 the ortho path assumed, i.e. a 0.3637 factor, and a `?camscale=` sweep of the
/// live renderer bottoms out at **0.36** (0.28 -> 69.4, 0.32 -> 63.7, 0.36 -> 62.0, 0.40 -> 66.0,
/// 0.45 -> 72.1). Predicted and measured optimum agree to two digits.
///
/// Returns `None` for an orthographic camera, so every existing skin is untouched by construction.
pub fn entrance_dolly_curve(all_objects: &HashMap<i64, (i32, Value)>) -> Option<Vec<(f32, f32)>> {
    let cam_pid = super::spine::entrance_camera_pid(all_objects)?;
    // Orthographic cameras keep the ortho path; only a perspective rig dollies.
    let is_ortho = all_objects
        .get(&cam_pid)
        .and_then(|(_, v)| v.get("orthographic"))
        .and_then(|o| o.as_i64().or_else(|| o.as_bool().map(i64::from)))
        .unwrap_or(1);
    if is_ortho != 0 {
        return None;
    }
    let cam_go = all_objects
        .get(&cam_pid)
        .and_then(|(_, v)| v.get("m_GameObject"))
        .and_then(get_path_id)?;
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let cam_clips = camera_motion_clips(all_objects);
    let mut best: Option<Vec<(f32, f32)>> = None;
    let mut best_range = 0.0f32;
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 || !cam_clips.contains(clip_pid) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let animator_gos = clip_animators.get(clip_pid).map(Vec::as_slice);
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            let count = binding_curve_count(type_id, attr);
            let go = hash_to_gos
                .get(&path)
                .map(|gos| scope_to_animator(gos, animator_gos, &is_ancestor))
                .as_deref()
                .and_then(|gos| gos.iter().find(|&&g| g == cam_go))
                .copied()
                .unwrap_or(0);
            // Position on the CAMERA's own GameObject; component 2 is Z.
            if type_id == 4
                && attr == 1
                && go == cam_go
                && count > 2
                && let Some(curve) = decode_curve_at(v, gidx + 2)
            {
                let (mn, mx) = curve
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(a, b), &(_, val)| {
                        (a.min(val), b.max(val))
                    });
                // A camera whose Z never changes carries no dolly; prefer the moving one.
                let range = mx - mn;
                if range > best_range && curve.iter().all(|&(_, d)| d > 0.0) {
                    best_range = range;
                    best = Some(curve);
                }
            }
            gidx += count;
        }
    }
    best
}

#[must_use]
pub fn entrance_ortho_curve(all_objects: &HashMap<i64, (i32, Value)>) -> Option<Vec<(f32, f32)>> {
    let cam_clips = camera_motion_clips(all_objects);
    let mut best: Option<Vec<(f32, f32)>> = None;
    for (pid, (cid, v)) in all_objects {
        if *cid != 74 || !cam_clips.contains(pid) {
            continue;
        }
        if let Some(curve) = decode_scalar_curve(v, 20, ORTHO_SIZE_CRC) {
            // Prefer the richest (most-keyframed) curve if several clips carry one.
            if best.as_ref().is_none_or(|b| curve.len() > b.len()) {
                best = Some(curve);
            }
        }
    }
    best
}

/// Decode a single scalar animation curve `(type_id, attribute)` from a clip into
/// `(time_seconds, value)` samples, reading whichever of the streamed / dense / constant
/// sub-clips holds it. Returns `None` when the clip has no such binding. The boundary
/// padding frame (streamed) is skipped. Used for the entrance camera's orthographic-size
/// dolly curve (`start_animation_02`).
#[must_use]
pub fn decode_scalar_curve(
    clip: &Value,
    type_id_want: i64,
    attr_want: i64,
) -> Option<Vec<(f32, f32)>> {
    let bindings = generic_bindings(clip)?;
    // Global curve index of the wanted binding.
    let mut gidx = 0usize;
    let mut found: Option<usize> = None;
    for b in bindings {
        let (tid, attr, _) = binding_fields(b);
        if tid == type_id_want && attr == attr_want {
            found = Some(gidx);
            break;
        }
        gidx += binding_curve_count(tid, attr);
    }
    decode_curve_any(clip, found?)
}

/// Decode the animation curve at GLOBAL index `idx` from whichever sub-clip holds it
/// (streamed / dense / constant) — the per-index core of {@link `decode_scalar_curve`},
/// for bindings located by path (per-renderer material-colour channels) rather than by
/// `(type, attribute)`.
fn decode_curve_any(clip: &Value, idx: usize) -> Option<Vec<(f32, f32)>> {
    let bindings = generic_bindings(clip)?;
    let data = clip_data(clip)?;
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
    let total = total_curve_count(bindings);
    let const_count = const_data.len();
    let stream_count = total.saturating_sub(dense_count + const_count);
    let mut out = Vec::new();
    if idx < stream_count {
        // Streamed: this curve's real keys, cubic-eased between them.
        if let Some(pts) = decode_streamed_curve(&streamed_raw, idx) {
            out = pts;
        }
    } else if idx < stream_count + dense_count {
        // Dense: a flat [frame][curve] grid at m_SampleRate; pull this curve's column.
        let di = idx - stream_count;
        let frames = dense_data.len().checked_div(dense_count).unwrap_or(0);
        for f in 0..frames {
            let v = dense_data.get(f * dense_count + di).copied()?;
            let t = dense_begin + f as f32 / dense_sample.max(1.0);
            out.push((t, v));
        }
    } else {
        // Constant: single value across the clip.
        let ci = idx - stream_count - dense_count;
        if let Some(&v) = const_data.get(ci) {
            out.push((0.0, v));
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Unity's `customType` for a renderer MATERIAL-property binding.
const MATERIAL_CUSTOM_TYPE: i64 = 22;

/// One animated material-colour channel on a renderer's `GameObject`, from the `_Start`
/// clip(s). A colour-channel binding (`customType` 22) encodes its target as
/// `attribute = (crc32(propName) & 0x0FFF_FFFF) | ((4 + channel) << 28)`, channel 0..3 =
/// r,g,b,a — verified on Mlynar "Fields of Ruination": the white flash's `_TintColor`
/// alpha binding is `0x7100_F6C6` (crc32("_`TintColor`") = `0x6100_F6C6`) and particle
/// `_MainColor` bindings ladder `0x456C_062A..0x756C_062A` across r/g/b/a.
pub struct MaterialColorChannel {
    /// `crc32(property_name) & 0x0FFF_FFFF` — matched against the layer material's own
    /// saved colour-property names, so the capture stays data-derived.
    pub prop_crc28: u32,
    /// RGBA channel index 0..3.
    pub channel: usize,
    /// Decoded `(t_seconds, value)` samples (dense-resampled cubic easing).
    pub curve: Vec<(f32, f32)>,
}

/// Whether the entrance clip animates the named colour property on this layer — i.e.
/// whether the clip itself NAMES the property the shader modulates by.
#[must_use]
pub fn animates_prop(channels: &[MaterialColorChannel], prop: &str) -> bool {
    let crc28 = crc32(prop.as_bytes()) & 0x0FFF_FFFF;
    channels.iter().any(|c| c.prop_crc28 == crc28)
}

/// Every ANIMATED material-colour channel in the `_Start` entrance clip(s), keyed by the
/// renderer's `GameObject` `path_id`. Only genuinely-varying curves are kept (a constant
/// channel adds nothing over the static tint). Drives per-layer colour/alpha replay —
/// e.g. Mlynar's white flash ramps `_TintColor.a` 0→0.671 over 13→15s, which a static
/// tint can only render as a held opaque white-wash.
#[must_use]
pub fn entrance_material_color_channels(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> HashMap<i64, Vec<MaterialColorChannel>> {
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let start_only = start_only_effect_clips(all_objects);
    let mut out: HashMap<i64, Vec<MaterialColorChannel>> = HashMap::new();
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 || !(is_entrance_clip(v) || start_only.contains(clip_pid)) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let animator_gos = clip_animators.get(clip_pid).map(Vec::as_slice);
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            let count = binding_curve_count(type_id, attr);
            let custom = b.get("customType").and_then(Value::as_i64).unwrap_or(0);
            let is_pptr = b.get("isPPtrCurve").and_then(Value::as_i64).unwrap_or(0) != 0;
            // Top nibble 4..=7 = a colour CHANNEL binding (r/g/b/a); a channel-less
            // material float uses the plain crc32 and is not a colour.
            let nibble = ((attr as u64) >> 28) & 0xF;
            // DIAGNOSTIC (`DYNCHAR_COLORDBG=1`): which of the filter's conditions rejects a
            // material-colour binding. Printed per binding of an admitted clip.
            if std::env::var("DYNCHAR_COLORDBG").is_ok() && (4..=7).contains(&nibble) {
                eprintln!(
                    "    [chan] clip='{}' attr=0x{:08x} custom={custom} (want {MATERIAL_CUSTOM_TYPE}) pptr={is_pptr} nibble={nibble} pathresolved={} curvelen={:?}",
                    v.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                    attr,
                    hash_to_gos.contains_key(&path),
                    decode_curve_any(v, gidx).map(|c| c.len()),
                );
                // The re-validation in `spine.rs` (`admitted_by_reveal`) requires a reveal curve
                // to START HIDDEN, so the first sample decides whether a colour-reveal-only layer
                // survives. Print the endpoints: a fade-OUT plane starts at 1.0 and is dropped.
                if let Some(c) = decode_curve_any(v, gidx)
                    && !c.is_empty()
                {
                    eprintln!(
                        "           first=(t {:.3}, v {:.3})  last=(t {:.3}, v {:.3})",
                        c[0].0,
                        c[0].1,
                        c[c.len() - 1].0,
                        c[c.len() - 1].1
                    );
                }
            }
            if custom == MATERIAL_CUSTOM_TYPE
                && !is_pptr
                && (4..=7).contains(&nibble)
                && let Some(gos) = hash_to_gos.get(&path)
                && let Some(curve) = decode_curve_any(v, gidx)
                && curve.len() > 1
            {
                let (mn, mx) = curve
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(a, b), &(_, val)| {
                        (a.min(val), b.max(val))
                    });
                if mx - mn > 1e-4 {
                    let prop_crc28 = (attr as u32) & 0x0FFF_FFFF;
                    let channel = (nibble - 4) as usize;
                    // A subpath hash matching several same-named GOs (twin rigs across
                    // the entrance/idle prefabs, e.g. Mlynar's `bg01_Idle`) applies to
                    // ALL of them WITHIN the playing Animator's subtree — the consumer
                    // only uses a channel whose property matches the layer material's own
                    // colour props, so a wrong twin simply ignores it. (Picking one
                    // arbitrary GO dropped the bg flicker curves from the twin the scene
                    // actually exports.) Outside that subtree it is a different rig
                    // entirely: without the animator scope Mlynar's six blade glows each
                    // inherited every sword clip's colour curve.
                    for &go in scope_to_animator(gos, animator_gos, &is_ancestor).iter() {
                        let entry = out.entry(go).or_default();
                        // Same channel keyed in several entrance clips: keep the richer curve.
                        if let Some(existing) = entry
                            .iter_mut()
                            .find(|c| c.prop_crc28 == prop_crc28 && c.channel == channel)
                        {
                            if curve.len() > existing.curve.len() {
                                existing.curve.clone_from(&curve);
                            }
                        } else {
                            entry.push(MaterialColorChannel {
                                prop_crc28,
                                channel,
                                curve: curve.clone(),
                            });
                        }
                    }
                }
            }
            gidx += count;
        }
    }
    out
}

/// `crc32("_MainTex_ST") & 0x0FFF_FFFF` — the low-28 property hash shared by all four
/// `_MainTex_ST` component bindings (customType 22, top nibble = component 0..3).
const MAINTEX_ST_CRC28: u32 = 0x0686_C589;

type StChannels = [Option<Vec<(f32, f32)>>; 4];

/// Every ANIMATED `_MainTex_ST` component curve in the `_Start` entrance clip(s), keyed by
/// the renderer's `GameObject` `path_id`. A `customType == 22` binding whose low-28 attribute
/// bits equal `MAINTEX_ST_CRC28` and whose top nibble is 0..3 (0=scaleX,1=scaleY,2=offsetX,
/// 3=offsetY) is an ST-component binding. The owning GO is disambiguated among colliding
/// subpath hashes exactly like `entrance_transform_curves` (lone candidate accepted; else the
/// unique descendant-scoring winner). Drives the entrance ST sweep replay (e.g. Skadi2's
/// `01 (4)`/`01 (5)` offset-Y 0.69→±1.4). Constant/undecodable channels are still recorded so
/// the assembler has the static components; a layer with no VARYING channel is dropped by the
/// assembler.
#[must_use]
pub fn entrance_st_curves(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<i64, StChannels> {
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let clip_animators = build_clip_animator_gos(all_objects);
    let mut out: HashMap<i64, StChannels> = HashMap::new();
    for (clip_pid, (cid, v)) in all_objects {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        // Pass 1: collect this clip's ST-component (path, component, gidx) and ALL its hashes.
        let mut all_hashes: std::collections::HashSet<u32> = std::collections::HashSet::new();
        let mut hits: Vec<(u32, usize, usize)> = Vec::new(); // (path, component, gidx)
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            all_hashes.insert(path);
            let count = binding_curve_count(type_id, attr);
            let custom = b.get("customType").and_then(Value::as_i64).unwrap_or(0);
            let is_pptr = b.get("isPPtrCurve").and_then(Value::as_i64).unwrap_or(0) != 0;
            let nibble = ((attr as u64) >> 28) & 0xF;
            if custom == MATERIAL_CUSTOM_TYPE
                && !is_pptr
                && nibble <= 3
                && (attr as u32) & 0x0FFF_FFFF == MAINTEX_ST_CRC28
            {
                hits.push((path, nibble as usize, gidx));
            }
            gidx += count;
        }
        if hits.is_empty() {
            continue;
        }
        // Pass 2: for each ST binding, resolve its owning GO among colliding hashes exactly
        // like entrance_transform_curves, then record the decoded component curve.
        for (path, component, g) in hits {
            let Some(candidates) = hash_to_gos.get(&path) else {
                continue;
            };
            let candidates = scope_to_animator(
                candidates,
                clip_animators.get(clip_pid).map(Vec::as_slice),
                &is_ancestor,
            );
            let other_hashes = all_hashes.iter().copied().filter(|&h| h != path);
            let Some(owner) =
                disambiguate_owner(&candidates, other_hashes, &hash_to_gos, &is_ancestor)
            else {
                continue;
            };
            let Some(curve) = decode_curve_any(v, g) else {
                continue;
            };
            let entry = out.entry(owner).or_default();
            // Keep the richer (more-keyed) curve if the same component is bound in several
            // entrance clips.
            match &entry[component] {
                Some(existing) if existing.len() >= curve.len() => {}
                _ => entry[component] = Some(curve),
            }
        }
    }
    out
}

/// Assemble one scene layer's absolute `_MainTex_ST` curve `(t, [sx, sy, ox, oy])` from its
/// animated ST component curves and the material's STATIC ST (`static_st` = `[scaleX, scaleY,
/// offsetX, offsetY]` from `mat_texenv`). The time base is the union of the VARYING components'
/// sample times; a component with no curve (or a constant one) contributes its static value at
/// every sample. Returns `None` when NO component actually varies (range ≤ 1e-4) — the static
/// ST bake already covers that layer, so no curve is emitted.
#[must_use]
pub fn layer_st_curve(channels: &StChannels, static_st: [f32; 4]) -> Option<Vec<(f32, [f32; 4])>> {
    let varies = |c: &Vec<(f32, f32)>| {
        let (mn, mx) = c.iter().fold((f32::MAX, f32::MIN), |(a, b), &(_, val)| {
            (a.min(val), b.max(val))
        });
        mx - mn > 1e-4
    };
    // Union time base from the VARYING channels only.
    let mut times: Vec<f32> = channels
        .iter()
        .flatten()
        .filter(|c| varies(c))
        .flat_map(|c| c.iter().map(|&(t, _)| t))
        .collect();
    if times.is_empty() {
        return None;
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    times.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    let sample = |c: &Vec<(f32, f32)>, t: f32| -> f32 {
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
    let mut out = Vec::with_capacity(times.len());
    for &t in &times {
        let mut st = static_st;
        for i in 0..4 {
            if let Some(c) = &channels[i] {
                // A bound component (even a constant one) overrides the static value.
                st[i] = sample(c, t);
            }
        }
        out.push((t, st));
    }
    Some(out)
}

/// Assemble one scene layer's full RGBA colour curve `(t, [r, g, b, a])` from its
/// renderer's animated material-colour channels. `props` are the material's saved
/// colour properties `(name, rgba)` and `tint_prop` names the one the exported static
/// `tint` came from ({@link `super::spine::material_tint`}). Of the animated properties,
/// the most-keyed one is used; its keyed channels REPLACE the static tint channel when
/// it IS the tint property (the tint already holds that property's static value), and
/// MULTIPLY onto the tint otherwise (the shader multiplies its colour properties). For an
/// ADDITIVE non-tint property the tint carries none of its colour, so its RGB channels are
/// seeded from the property's OWN serialized value: a keyed channel takes the animated
/// value, an unkeyed one the static value (both × tint) — an additive glow's colour is the
/// colour of the light it adds, so an unkeyed warm-amber RGB behind an animated alpha must
/// still tint the added light (`fold_static`, gated on `additive`). The ALPHA channel is
/// exempt (it is opacity/intensity, not colour). Returns `None` when nothing matches.
///
/// `tint_scale` is the shader family's tint multiplier (2.0 for the legacy `×2
/// _TintColor` particle shaders, else 1.0 — see `legacy_tint_scale` in spine.rs): a
/// keyed tint channel carries the RAW authored value, so the replacement branch must
/// re-apply the scale the static `tint` already folded in (Mlynar's flash plane keys
/// `_TintColor.a` to 0.671 = effective 1.0 — the game's FULL white-out). Channels are
/// clamped to [0, 1] after scaling (the blend stage clamps the same way).
#[must_use]
pub fn layer_color_curve(
    channels: &[MaterialColorChannel],
    props: &[(String, [f32; 4])],
    tint_prop: Option<&str>,
    tint: [f32; 4],
    tint_scale: f32,
    // The layer's scaled colour is HDR (see `ram_tint_scale`): keep the scaled RGB
    // UNCLAMPED so an over-bright ramp survives to the frontend's half-float target,
    // instead of collapsing baseline and peak onto the same ceiling. Alpha is always
    // clamped — it is a coverage weight, and a premultiplied source alpha above 1
    // makes the destination factor `1 - a` negative and corrupts the composite.
    hdr_color: bool,
    additive: bool,
) -> Option<Vec<(f32, [f32; 4])>> {
    let prop_of = |crc28: u32| {
        props
            .iter()
            .find(|(n, _)| crc32(n.as_bytes()) & 0x0FFF_FFFF == crc28)
            .map(|(n, _)| n.as_str())
    };
    // Pick the animated property with the most keys (clips rarely animate two); the crc
    // tie-break keeps the choice deterministic.
    let mut by_prop: HashMap<u32, usize> = HashMap::new();
    for c in channels {
        if prop_of(c.prop_crc28).is_some() {
            *by_prop.entry(c.prop_crc28).or_insert(0) += c.curve.len();
        }
    }
    let best = by_prop
        .iter()
        .max_by_key(|&(crc, n)| (*n, *crc))
        .map(|(&crc, _)| crc)?;
    let same_prop = tint_prop == prop_of(best);
    // Static serialized RGB of the animated property, folded onto unkeyed colour channels
    // for ADDITIVE layers only (`fold_static`). An additive glow layer with a WHITE glow
    // texture derives ALL its colour from the material property, so the shader's added
    // light is `_MainColor.rgb × alpha`; when the clip keys only the alpha (Mlynar's `huan`
    // warm-ring: `_MainColor` = (1.0,0.48,0.30) with an animated α 0→0.43→0), seeding the
    // unkeyed RGB from 1 (the old behaviour) added pure WHITE light instead of the authored
    // warm AMBER — the reported "too white/cool" ring. Restricted to additive layers on
    // purpose: an additive layer's colour is unambiguously the colour of the light it adds,
    // whereas a normal-blend layer composites OVER the scene, where folding a secondary
    // property's colour (e.g. a blue `_MainColor` smoke sheet) tints regions the frontend
    // can't counter-balance (an under-rendered warm campfire) and would over-cool them.
    // Property-derived, no per-skin constant — a no-op when the property's static RGB is
    // white or when `same_prop` (the tint already carries it).
    let best_static: [f32; 4] = prop_of(best)
        .and_then(|name| props.iter().find(|(n, _)| n == name).map(|(_, c)| *c))
        .unwrap_or([1.0; 4]);
    let fold_static = additive && !same_prop;
    let mut chans: [Option<&Vec<(f32, f32)>>; 4] = [None; 4];
    for c in channels {
        if c.prop_crc28 == best && c.channel < 4 {
            chans[c.channel] = Some(&c.curve);
        }
    }
    // Time base: the union of the keyed channels' sample times.
    let mut times: Vec<f32> = chans
        .iter()
        .flatten()
        .flat_map(|c| c.iter().map(|&(t, _)| t))
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    times.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    if times.len() < 2 {
        return None;
    }
    let sample = |c: &Vec<(f32, f32)>, t: f32| -> f32 {
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
    let mut out = Vec::with_capacity(times.len());
    for &t in &times {
        // Seed from the static tint; for an additive non-tint animated property, fold in
        // its static RGB (the added light's colour) so unkeyed colour channels carry the
        // property's serialized value rather than 1. Alpha is left on the base tint — it is
        // the layer's opacity/intensity, not a colour to fold (see `fold_static`).
        let mut rgba = tint;
        if fold_static {
            for i in 0..3 {
                rgba[i] = tint[i] * best_static[i];
            }
        }
        for (i, ch) in chans.iter().enumerate() {
            if let Some(c) = ch {
                let v = sample(c, t);
                rgba[i] = if same_prop {
                    let scaled = v * tint_scale;
                    if hdr_color && i < 3 {
                        scaled
                    } else {
                        scaled.clamp(0.0, 1.0)
                    }
                } else {
                    tint[i] * v
                };
            }
        }
        out.push((t, rgba));
    }
    Some(out)
}

/// One clip's `m_IsActive` transitions per GO: `(first off→on time, first on→off time)`.
/// Only records genuinely-toggled objects (skips always-on/always-off). A subpath hash
/// matching several same-named GOs is first restricted to the subtree of the Animator
/// playing this clip, then disambiguated from the other paths bound by the same clip;
/// ambiguous bindings are conservatively dropped.
fn active_timeline(
    clip: &Value,
    hash_to_gos: &HashMap<u32, Vec<i64>>,
    is_ancestor: &impl Fn(i64, i64) -> bool,
    animator_gos: Option<&[i64]>,
    go_parent: &HashMap<i64, i64>,
    go_name: &HashMap<i64, String>,
) -> HashMap<i64, ActiveWindowList> {
    let clip_name = clip.get("m_Name").and_then(Value::as_str).unwrap_or("");
    let mut out = HashMap::new();
    let Some(bindings) = generic_bindings(clip) else {
        return out;
    };
    let Some(data) = clip_data(clip) else {
        return out;
    };
    let streamed_raw = u32_array(data.get("m_StreamedClip").and_then(|s| s.get("data")));
    let dense_count = data
        .get("m_DenseClip")
        .and_then(|d| d.get("m_CurveCount"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0) as usize;
    let const_count = f32_array(data.get("m_ConstantClip").and_then(|s| s.get("data"))).len();
    let stream_count = total_curve_count(bindings).saturating_sub(dense_count + const_count);
    let timed = read_streamed_timed(&streamed_raw);
    let mut all_hashes = std::collections::HashSet::new();
    let mut windows: Vec<(u32, ActiveWindowList)> = Vec::new();
    let mut gidx = 0usize;
    for b in bindings {
        let (type_id, attr, path) = binding_fields(b);
        all_hashes.insert(path);
        let count = binding_curve_count(type_id, attr);
        if type_id == 1 && attr == M_IS_ACTIVE_CRC && gidx < stream_count {
            let mut val = f32::NAN;
            let mut prev: Option<bool> = None;
            let mut reveal: Option<f32> = None;
            let mut hide: Option<f32> = None;
            // EVERY on/off transition, not just the first of each kind: a GameObject the clip
            // toggles off and back ON collapses to a PERMANENT hide if only the first of each is
            // kept. Civilight Eterna's overlay runs on 5.17 / off 6.33 / on 11.50 / off 12.67 —
            // two separate windows, the second of which is the cut the capture shows at t=12.
            let mut transitions: Vec<(f32, bool)> = Vec::new();
            // The state the clip OPENS in, taken from the boundary padding frame that the
            // `fi == 0` skip below deliberately does not treat as a transition.
            let mut start_on = false;
            for (fi, (time, keys)) in timed.iter().enumerate() {
                for k in keys {
                    if k.index == gidx {
                        val = k.value;
                    }
                }
                if val.is_nan() {
                    continue;
                }
                let on = val >= 0.5;
                // Skip the boundary padding frame (index 0) so its clamped value
                // (time = f32::MIN sentinel) doesn't register as a spurious state.
                if fi == 0 {
                    prev = Some(on);
                    start_on = on;
                    continue;
                }
                if let Some(p) = prev {
                    if p != on {
                        transitions.push((time.max(0.0), on));
                    }
                    if !p && on && reveal.is_none() {
                        reveal = Some(time.max(0.0));
                    }
                    if p && !on && hide.is_none() {
                        hide = Some(time.max(0.0));
                    }
                }
                prev = Some(on);
            }
            if reveal.is_some() || hide.is_some() {
                windows.push((path, windows_from_transitions(start_on, &transitions)));
            }
        }
        gidx += count;
    }
    for (path, ivs) in windows {
        let Some(candidates) = hash_to_gos.get(&path) else {
            continue;
        };
        let candidates = scope_to_animator(candidates, animator_gos, is_ancestor);
        let other_hashes = all_hashes.iter().copied().filter(|&h| h != path);
        if let Some(go) = disambiguate_owner_near(
            &candidates,
            other_hashes,
            hash_to_gos,
            is_ancestor,
            go_parent,
            clip_name,
            go_name,
        ) {
            // `SCENE_DEBUG=1` prints the resolved visibility SCHEDULE per object. This is what
            // turned "Civilight Eterna's backdrop just stops at t=12" into a one-line diagnosis:
            // `Transition_black_01` reads `[(5.17, 6.33), (11.5, 12.67)]`, i.e. the cinematic
            // blacks the frame out TWICE and only the first window was ever exported.
            if std::env::var("SCENE_DEBUG").is_ok() {
                eprintln!(
                    "  [active] {:?} (clip {clip_name}) {ivs:?}",
                    go_name.get(&go).map_or("?", String::as_str)
                );
            }
            out.insert(go, ivs);
        }
    }
    out
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

/// Build a map from CRC32(transform-subpath) → `GameObject` `path_id`, registering
/// every ancestor-relative subpath (Unity may hash against any root). On a hash
/// collision (identical sibling rigs) the smallest `path_id` wins, deterministically.
fn build_hash_to_go(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<u32, i64> {
    build_hash_to_gos(all_objects)
        .into_iter()
        .filter_map(|(h, gos)| gos.first().map(|&g| (h, g)))
        .collect()
}

/// Like {@link `build_hash_to_go`} but keeps EVERY `GameObject` matching a subpath hash
/// (sorted by `path_id`) — identical sibling rigs (Mlynar's three sword clones) bind
/// the same relative subpath, and per-rig consumers need all candidates.
/// DIAGNOSTIC (`DYNCHAR_BINDDBG=1`): every `AnimationClip` binding whose path hash does NOT
/// resolve to a `GameObject`, grouped by clip.
///
/// An unresolved hash means a curve we decode and then throw away, which is exactly how the
/// camera dolly was lost for a whole skin (see the empty-path note in `build_hash_to_gos`).
/// The count is worth watching: `dump_entrance_cams` reported "9 bindings had UNRESOLVED path
/// hashes" on Kal'tsit and only ONE of them was the animator root.
pub fn report_unresolved_bindings(all_objects: &HashMap<i64, (i32, Value)>) {
    if std::env::var("DYNCHAR_BINDDBG").is_err() {
        return;
    }
    let hash_to_gos = build_hash_to_gos(all_objects);
    for (cid, v) in all_objects.values() {
        if *cid != 74 {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
        let mut miss: Vec<(i64, i64, u32)> = Vec::new();
        for b in bindings {
            let (tid, attr, path) = binding_fields(b);
            if !hash_to_gos.contains_key(&path) {
                miss.push((tid, attr, path));
            }
        }
        if !miss.is_empty() {
            eprintln!(
                "  [bind] clip {name:<44} {}/{} UNRESOLVED",
                miss.len(),
                bindings.len()
            );
            for (tid, attr, path) in miss.iter().take(12) {
                eprintln!("           tid={tid:<4} attr={attr:<12} path=0x{path:08x}");
            }
        }
    }
}

fn build_hash_to_gos(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<u32, Vec<i64>> {
    let mut go_name: HashMap<i64, String> = HashMap::new();
    let mut tr_go: HashMap<i64, i64> = HashMap::new();
    let mut tr_father: HashMap<i64, i64> = HashMap::new();
    for (pid, (cid, v)) in all_objects {
        match cid {
            1 => {
                go_name.insert(
                    *pid,
                    v.get("m_Name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                );
            }
            4 | 224 => {
                if let Some(go) = v.get("m_GameObject").and_then(get_path_id) {
                    tr_go.insert(*pid, go);
                }
                if let Some(f) = v.get("m_Father").and_then(get_path_id) {
                    tr_father.insert(*pid, f);
                }
            }
            _ => {}
        }
    }

    let mut hash_to_gos: HashMap<u32, Vec<i64>> = HashMap::new();
    for tr in tr_go.keys() {
        // Full GameObject-name chain from root down to this transform.
        let mut chain: Vec<String> = Vec::new();
        let mut cur = *tr;
        let mut guard = 0;
        loop {
            guard += 1;
            if guard > 128 {
                break;
            }
            let Some(go) = tr_go.get(&cur) else { break };
            chain.push(go_name.get(go).cloned().unwrap_or_default());
            match tr_father.get(&cur) {
                Some(f) if *f != 0 && tr_go.contains_key(f) => cur = *f,
                _ => break,
            }
        }
        chain.reverse();
        for start in 0..chain.len() {
            let p = chain[start..].join("/");
            hash_to_gos
                .entry(crc32(p.as_bytes()))
                .or_default()
                .push(tr_go[tr]);
        }
    }
    // 🔑 THE EMPTY PATH. Unity binds a curve on the Animator's OWN GameObject with an empty
    // path, whose `crc32("")` is **0** — and the suffix loop above can never produce it, so
    // every such curve resolved to nothing and was silently dropped.
    //
    // Kal'tsit is the case that exposed it: her entrance clip binds the camera dolly as
    // `[tid=4 position] -> path 0` with 20 keys of real motion (x 0.47 -> -2.46,
    // y 7.45 -> 1.37 -> 5.39), because her Animator sits on `.../static_offset/fixed/03`,
    // an ANCESTOR of `Dummy002/Main Camera`. We exported her `entranceCamCenterCurve` as a
    // 2-point CONSTANT while the game pans, which a normalised cross-correlation against the
    // capture shows directly: her best alignment needs dx 165 -> 8 px across t=2..8 while
    // wisdel (our best skin) sits at 0.99 NCC with zero offset.
    //
    // Mapping 0 to every Animator-owning GameObject is deliberate: several Animators exist per
    // bundle, and each caller already disambiguates by intersecting the candidate list with the
    // subtree it cares about (the camera chain here), so the ambiguity resolves itself exactly
    // as it does for a colliding subpath hash between twin rigs.
    //
    // `DYNCHAR_ANIMROOT=0` reverts.
    if std::env::var("DYNCHAR_ANIMROOT").as_deref() != Ok("0") {
        let roots: Vec<i64> = all_objects
            .iter()
            .filter(|(_, (cid, _))| *cid == 95)
            .filter_map(|(_, (_, v))| v.get("m_GameObject").and_then(get_path_id))
            .collect();
        if !roots.is_empty() {
            hash_to_gos.entry(0).or_default().extend(roots);
        }
    }
    for gos in hash_to_gos.values_mut() {
        gos.sort_unstable();
        gos.dedup();
    }
    hash_to_gos
}

/// Idle `AnimationClips` (class 74) among the bundle: any clip whose name contains
/// "idle" and none of the non-idle-state markers. A scene may split into several
/// sub-state-machines each with its own idle loop (e.g. Ines "Melodic Flutter":
/// a `bg` idle and a `web` idle), so **all** matching clips are returned and
/// their transform overrides merged. Markers are chosen not to collide with
/// operator names (`ines`, `skadi`, …), so no fragile `in`/`out` fragments.
fn find_idle_clips(all_objects: &HashMap<i64, (i32, Value)>) -> Vec<&Value> {
    const EXCLUDED: &[&str] = &["interact", "special", "skill", "start", "attack", "die"];
    let mut idle: Vec<&Value> = all_objects
        .values()
        .filter(|(cid, _)| *cid == 74)
        .filter(|(_, v)| {
            let name = v
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            name.contains("idle") && !EXCLUDED.iter().any(|e| name.contains(e))
        })
        .map(|(_, v)| v)
        .collect();
    // Deterministic order.
    idle.sort_by_key(|v| {
        v.get("m_Name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    });
    idle
}

/// Evaluate the idle clip at t=0 and return the mesh-quad transform overrides.
/// Returns an empty pose (no overrides) when there is no idle clip or it drives
/// no transforms — callers then fall back to the prefab bind pose.
#[must_use]
pub fn evaluate_idle_pose(
    all_objects: &HashMap<i64, (i32, Value)>,
    go_to_transform: &HashMap<i64, i64>,
) -> IdlePose {
    let dbg = std::env::var("IDLE_DEBUG").is_ok();
    let mut pose = IdlePose::default();
    let clips = find_idle_clips(all_objects);
    if clips.is_empty() {
        if dbg {
            let names: Vec<String> = all_objects
                .values()
                .filter(|(c, _)| *c == 74)
                .filter_map(|(_, v)| v.get("m_Name").and_then(Value::as_str).map(String::from))
                .collect();
            eprintln!("[idle] NO idle clip; class-74 clips present: {names:?}");
        }
        return pose;
    }

    let hash_to_go = build_hash_to_go(all_objects);
    // Animated `m_IsActive` is only meaningful from the steady idle-loop clip: a
    // `<state>_toIdle` transition sampled at its first frame still shows the source
    // state's active-set, which would wrongly resurrect flash quads. A loop clip is
    // one whose source state is idle itself (`idle_toIdle` / `idle_loop`); skins
    // without one record no active override (no-op).
    let is_loop = |c: &&Value| {
        let n = c
            .get("m_Name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();
        n.contains("idle_toidle")
            || n.contains("idle_to_idle")
            || n.contains("idle_loop")
            || n.contains("idleloop")
    };
    for clip in &clips {
        if dbg {
            eprintln!(
                "[idle] clip: {:?}",
                clip.get("m_Name").and_then(Value::as_str)
            );
        }
        apply_clip(
            clip,
            all_objects,
            go_to_transform,
            &hash_to_go,
            &mut pose,
            is_loop(clip),
            dbg,
        );
    }

    if dbg {
        eprintln!(
            "[idle] TOTAL pos_overrides={} euler_overrides={}",
            pose.pos.len(),
            pose.euler.len()
        );
    }
    pose
}

/// Decode one clip at t=0 and merge its Transform position/euler overrides into
/// `pose`.
fn apply_clip(
    clip: &Value,
    all_objects: &HashMap<i64, (i32, Value)>,
    go_to_transform: &HashMap<i64, i64>,
    hash_to_go: &HashMap<u32, i64>,
    pose: &mut IdlePose,
    record_active: bool,
    dbg: bool,
) {
    let Some(bindings) = generic_bindings(clip) else {
        return;
    };
    let Some(data) = clip_data(clip) else { return };

    let streamed_raw = u32_array(data.get("m_StreamedClip").and_then(|s| s.get("data")));
    let dense_data = f32_array(data.get("m_DenseClip").and_then(|s| s.get("data")));
    let dense_count = data
        .get("m_DenseClip")
        .and_then(|d| d.get("m_CurveCount"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0) as usize;
    let const_data = f32_array(data.get("m_ConstantClip").and_then(|s| s.get("data")));

    let total = total_curve_count(bindings);
    let const_count = const_data.len();
    let stream_count = total.saturating_sub(dense_count + const_count);

    // Assemble the value array at t=0: streamed from the first real frame (frame 0
    // is a boundary padding frame), dense from frame 0, constants verbatim.
    let mut values: Vec<Option<f32>> = vec![None; total];
    let frames = read_streamed(&streamed_raw);
    // Frame index 1 is the first real keyframe (0 is the pre-boundary); fall back
    // to whatever exists for very short clips.
    let real = frames.get(1).or_else(|| frames.first());
    if let Some(keys) = real {
        for k in keys {
            if k.index < values.len() {
                values[k.index] = Some(k.value);
            }
        }
    }
    for (k, &dv) in dense_data.iter().enumerate().take(dense_count) {
        let gi = stream_count + k;
        if gi < values.len() {
            values[gi] = Some(dv);
        }
    }
    for (k, &cv) in const_data.iter().enumerate() {
        let gi = stream_count + dense_count + k;
        if gi < values.len() {
            values[gi] = Some(cv);
        }
    }

    // Walk bindings, tracking the running global curve index, and pull position /
    // euler triples for Transform bindings that resolve to a known GameObject.
    let mut gidx = 0usize;
    for b in bindings {
        let (type_id, attr, path) = binding_fields(b);
        let count = binding_curve_count(type_id, attr);

        // GameObject (class 1) bindings animate `m_IsActive` (the only animatable
        // GameObject property). A value < 0.5 at the idle pose means the object is
        // switched off in the loop; >= 0.5 means switched on. Recorded only for the
        // base idle-loop clip (`record_active`).
        if record_active
            && type_id == 1
            && attr == M_IS_ACTIVE_CRC
            && let (Some(&go), Some(v)) =
                (hash_to_go.get(&path), values.get(gidx).copied().flatten())
        {
            pose.active.insert(go, v >= 0.5);
        }

        if type_id == 4
            && (attr == 1 || attr == 4)
            && let Some(&go) = hash_to_go.get(&path)
            && let Some(&tf) = go_to_transform.get(&go)
        {
            let comp = |o: usize| values.get(gidx + o).copied().flatten();
            // Only emit an override when at least one component resolved;
            // fill any missing component from the prefab local pose.
            if comp(0).is_some() || comp(1).is_some() || comp(2).is_some() {
                let base = if attr == 1 {
                    local_vec3(all_objects, tf, "m_LocalPosition", 0.0)
                } else {
                    [0.0, 0.0, 0.0]
                };
                let out = [
                    comp(0).unwrap_or(base[0]),
                    comp(1).unwrap_or(base[1]),
                    comp(2).unwrap_or(base[2]),
                ];
                if attr == 1 {
                    pose.pos.insert(tf, out);
                } else {
                    pose.euler.insert(tf, out);
                }
            }
        }
        gidx += count;
    }

    if dbg {
        eprintln!(
            "  total_curves={total} stream={stream_count} dense={dense_count} const={const_count} frames={} -> cumulative pos={} euler={}",
            frames.len(),
            pose.pos.len(),
            pose.euler.len()
        );
    }
}

/// Read a transform's local `field` vector (defaulting each component to `d`).
fn local_vec3(
    all_objects: &HashMap<i64, (i32, Value)>,
    tf_pid: i64,
    field: &str,
    d: f32,
) -> [f32; 3] {
    let Some((4, tf)) = all_objects.get(&tf_pid) else {
        return [d, d, d];
    };
    let g = |k: &str| {
        tf.get(field)
            .and_then(|v| v.get(k))
            .and_then(Value::as_f64)
            .unwrap_or(d.into()) as f32
    };
    [g("x"), g("y"), g("z")]
}

/// Convert Unity euler degrees (applied Z, then X, then Y) to a quaternion
/// `[x, y, z, w]`.
#[must_use]
pub fn euler_deg_to_quat(e: [f32; 3]) -> [f32; 4] {
    let (hx, hy, hz) = (
        e[0].to_radians() * 0.5,
        e[1].to_radians() * 0.5,
        e[2].to_radians() * 0.5,
    );
    let qx = [hx.sin(), 0.0, 0.0, hx.cos()];
    let qy = [0.0, hy.sin(), 0.0, hy.cos()];
    let qz = [0.0, 0.0, hz.sin(), hz.cos()];
    quat_mul(quat_mul(qy, qx), qz)
}

fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}
