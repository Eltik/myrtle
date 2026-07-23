//! Evaluate a dynchar scene's **idle** AnimationClip at its first frame, so the
//! painted mesh quads can be exported at their settled display pose rather than
//! their scattered prefab bind pose.
//!
//! These prefabs (Unity 2021, Mecanim) store animation as OPTIMIZED clips: the
//! legacy `m_*Curves` are empty and the real data lives in
//! `m_MuscleClip.m_Clip.data.{m_StreamedClip,m_DenseClip,m_ConstantClip}`,
//! indexed by `m_ClipBindingConstant.genericBindings`. Many scenes' idle loops
//! drive the local position (and occasionally euler rotation) of a handful of
//! quad GameObjects — e.g. Nearl "Evolved Art"'s sword/petal shards assemble
//! from an exploded bind pose. We only need the pose at t=0 (the loop start), so
//! we read the first real streamed frame plus the constant curves and apply the
//! resulting local position/euler overrides when accumulating world matrices.
//!
//! Scope intentionally narrow (matches what these scenes actually use): Transform
//! position (attribute 1) and euler (attribute 4). Scale, quaternion rotation,
//! and GameObject active-state curves are ignored — the quads render statically
//! regardless of active-state, and none of the observed scenes animate scale.

use serde_json::Value;
use std::collections::HashMap;

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
    /// GameObject `path_id` → animated `m_IsActive` state at the idle pose.
    /// Sampled only from the base idle-loop clip (not soul/interact transitions),
    /// so flash/overlay quads the idle animation switches off are dropped from the
    /// exported scene, and any the idle animation switches on are kept.
    pub active: HashMap<i64, bool>,
}

impl IdlePose {
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

/// `CRC32("m_IsActive")` — the binding attribute Unity uses for a GameObject's
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

/// A GameObject's serialized (prefab) `m_IsActive` state — what it reverts to when an
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

/// Like {@link read_streamed} but keeps each frame's TIME (seconds) — needed to find
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
/// GameObject must not render until its reveal time. GOs never toggled on (always
/// active) are absent → treated as visible from t=0.
#[must_use]
pub fn active_timelines(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<i64, f32> {
    active_windows(all_objects)
        .into_iter()
        .filter_map(|(go, w)| w.0.map(|from| (go, from)))
        .collect()
}

/// A GameObject's ENTRANCE visibility window: `(activeFrom, activeUntil)` in seconds.
/// `activeFrom = None` → visible from t=0 (starts on); `activeUntil = None` → never hidden
/// (ends on). A scene-mesh layer under such a GameObject renders only while
/// `activeFrom <= t < activeUntil`. This captures BOTH the mirror-world / crystal-throne
/// switch-ON (cathedral shows first, they appear later) AND the cathedral switch-OFF (the
/// bright cathedral deactivates when the mirror world takes over) — the environment SWAP
/// the `_Start` cinematic performs, which a reveal-only timeline misses.
pub type ActiveWindow = (Option<f32>, Option<f32>);

#[must_use]
pub fn active_windows(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<i64, ActiveWindow> {
    let hash_to_gos = build_hash_to_gos(all_objects);
    let is_ancestor = build_ancestor_check(all_objects);
    let mut out: HashMap<i64, ActiveWindow> = HashMap::new();
    for (cid, v) in all_objects.values() {
        if *cid != 74 {
            continue;
        }
        // Only the ENTRANCE clip drives the reveal timeline. The `_Start` bundle also
        // ships the full idle/interact state machine, whose `m_IsActive` toggles are
        // NOT the cinematic sequence — merging them corrupts the windows. Pick the clip
        // whose name marks it as the entrance/start cinematic.
        if !is_entrance_clip(v) {
            continue;
        }
        let stop = clip_stop_time(v);
        for (go, (from, mut until)) in active_timeline(v, &hash_to_gos, &is_ancestor) {
            // A clip's `m_IsActive` override EXPIRES when the entrance state exits at the
            // clip's end (`m_StopTime`): a GO the prefab keeps INACTIVE (an entrance-only
            // overlay, e.g. Mlynar's white transition flash — revealed at 13.0s, clip stop
            // 15.97s) then reverts to inactive. Bound a clip-driven reveal with no authored
            // hide by the clip's own stop, else the overlay holds its final frame (a
            // permanent white-out) into the settle. Prefab-ACTIVE GOs keep their `None`
            // hide — they stay visible after the clip on their own serialized state.
            if until.is_none() && from.is_some() && !prefab_active(all_objects, go) {
                until = stop;
            }
            let e = out.entry(go).or_insert((from, until));
            // Merge across clips: LATEST reveal (entrance's delayed switch-on beats a
            // const-on state clip) and EARLIEST hide (the cinematic's deactivation).
            e.0 = match (e.0, from) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
            e.1 = match (e.1, until) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
        }
    }
    out
}

/// Per-GameObject ENTRANCE emission-rate curves: the `_Start` cinematic can animate a
/// ParticleSystem's `EmissionModule.rateOverTime.scalar` directly (Mlynar "Fields of
/// Ruination"'s sword clips hold the confetti/star emitters at 0 and spike them 8–200/s
/// during the ~7–13s flourish, while the SERIALIZED rates are a constant 8–150/s).
/// Without these curves such systems emit at full rate from t=0. Times are absolute
/// cinematic seconds.
///
/// Identical sibling rigs (the three sword clones) can bind the same RELATIVE transform
/// subpath. Each binding is therefore disambiguated within its own clip by the other
/// hashes that clip binds, and only then are curves for the same resolved GameObject
/// merged into a MAX ENVELOPE.
///
/// Unlike `entrance_transform_curves`' ctrl target (an ANCESTOR of the clip's other
/// bound GOs, so descendant-scoring finds a real winner), a rate-curve binding's path
/// names the ParticleSystem's OWN GameObject — a LEAF, sibling to the clip's other
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
    let want = i64::from(crc32(b"EmissionModule.rateOverTime.scalar"));
    let mut per_go: HashMap<i64, Vec<StoppedCurve>> = HashMap::new();
    for (cid, v) in all_objects.values() {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
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
            let other_hashes = all_hashes.iter().copied().filter(|&h| h != path);
            match disambiguate_owner(candidates, other_hashes, &hash_to_gos, &is_ancestor) {
                Some(go) => per_go.entry(go).or_default().push((curve, stop)),
                // Ambiguous: this clip proves EVERY colliding candidate's system is
                // curve-gated, just not which one it belongs to. Contribute an
                // explicit silent (rate-0) window to all of them instead of leaving
                // the binding out entirely (which would fall back to the far louder
                // serialized constant rate).
                None => {
                    for &go in candidates {
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

    let mut out: HashMap<i64, EntranceTransform> = HashMap::new();
    for (cid, v) in all_objects.values() {
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
                if let Some(c) = decode_curve_any(v, gidx) {
                    pos_x = c;
                }
                if let Some(c) = decode_curve_any(v, gidx + 1) {
                    pos_y = c;
                }
            }
            gidx += binding_curve_count(tid, attr);
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
        let other_hashes = all_hashes.iter().copied().filter(|&h| h != scale_path);
        let Some(ctrl) = disambiguate_owner(candidates, other_hashes, &hash_to_gos, &is_ancestor)
        else {
            continue;
        };
        let e = out.entry(ctrl).or_default();
        if scale.len() > e.scale.len() {
            e.scale = scale;
            e.pos_x = pos_x;
            e.pos_y = pos_y;
        }
    }
    out
}

/// Build a predicate that reports whether its first GameObject is an ancestor of
/// (or equal to) its second, following Transform `m_Father` links.
fn build_ancestor_check(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> impl Fn(i64, i64) -> bool + '_ {
    // GameObject → parent GameObject (via the transform `m_Father` chain).
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

/// GameObjects whose ParticleSystem `EmissionModule.rateOverTime` is ANIMATED by a
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
    let want = i64::from(crc32(b"EmissionModule.rateOverTime.scalar"));
    let mut out = std::collections::HashSet::new();
    for (cid, v) in all_objects.values() {
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
        for b in bindings {
            let (tid, attr, path) = binding_fields(b);
            if tid == 198
                && attr == want
                && let Some(gos) = hash_to_gos.get(&path)
            {
                out.extend(gos.iter().copied());
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

/// The set of GameObject `path_id`s on the CAMERA's ancestor chain (the class-20 Camera's GO and
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
/// SPACE: the chain walk stops *before* any spine-root transform (a GameObject carrying the
/// `skeletonDataAsset` MonoBehaviour), exactly like the scene-mesh `accumulate_matrix`, so the
/// centre curve is expressed in the SAME spine-root-local frame as the exported `[scene]` quads.
/// Accumulating past the root into absolute world space breaks rigs whose entrance prefab root is
/// authored away from the origin (skadi2 "Iteration": root at (8.53, 25.19) → the camera track
/// landed ~2500 px below the scene and the whole entrance framed empty space).
#[must_use]
pub fn entrance_camera_track(
    all_objects: &HashMap<i64, (i32, Value)>,
    inv_scale: f64,
) -> Option<Vec<(f32, f32, f32)>> {
    use super::mesh::Mat4;
    // Ordered chain camera→root (transform pids) + maps.
    let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
    let mut tf_father: HashMap<i64, i64> = HashMap::new();
    let mut tf_go: HashMap<i64, i64> = HashMap::new();
    let mut cam_go: Option<i64> = None;
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
            20 => cam_go = v.get("m_GameObject").and_then(get_path_id).or(cam_go),
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
    let mut cur = cam_go.and_then(|g| go_to_tf.get(&g).copied());
    for _ in 0..64 {
        let Some(tf) = cur else { break };
        if tf_go.get(&tf).is_some_and(|go| spine_gos.contains(go)) {
            break; // reached the spine root; don't include its transform
        }
        chain.push(tf);
        cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
    }
    if chain.is_empty() {
        return None;
    }
    // Static local TRS of each chain transform.
    let local_trs = |tf: i64, pos_override: Option<[f32; 3]>| -> Mat4 {
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
        let q = {
            let g = |k: &str, d: f32| {
                v.get("m_LocalRotation")
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(d.into()) as f32
            };
            [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
        };
        let mut s = vec3("m_LocalScale", 1.0);
        // Some rig transforms ship a 0 scale component (unused axis); treat as 1 so the chain
        // doesn't collapse — the camera view only uses X/Y and forward.
        for c in &mut s {
            if c.abs() < 1e-6 {
                *c = 1.0;
            }
        }
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
    let mut animated: HashMap<i64, [Vec<(f32, f32)>; 3]> = HashMap::new();
    for (cid, v) in all_objects.values() {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            let count = binding_curve_count(type_id, attr);
            // On a subpath-hash collision (same-named twin rigs), pick the candidate
            // that is actually on the camera chain — an arbitrary pick could name the
            // wrong twin and drop the camera move entirely.
            if type_id == 4
                && attr == 1
                && let Some(&go) = hash_to_gos
                    .get(&path)
                    .and_then(|gos| gos.iter().find(|g| chain_gos.contains(g)))
                && let Some(&tf) = go_to_tf.get(&go)
            {
                let cs = [
                    decode_curve_at(v, gidx),
                    decode_curve_at(v, gidx + 1),
                    decode_curve_at(v, gidx + 2),
                ];
                if cs.iter().any(|c| c.as_ref().is_some_and(|c| c.len() > 1)) {
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
    if animated.is_empty() {
        return None;
    }
    // Timeline = union of EVERY animated axis's keyframe times.
    let mut times: Vec<f32> = animated
        .values()
        .flatten()
        .flatten()
        .map(|(t, _)| *t)
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    times.dedup();
    if times.len() < 2 {
        return None;
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
        all_objects
            .get(&tf)
            .map(|(_, v)| {
                let g = |k: &str| {
                    v.get("m_LocalPosition")
                        .and_then(|x| x.get(k))
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0) as f32
                };
                [g("x"), g("y"), g("z")]
            })
            .unwrap_or([0.0; 3])
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
    // Accumulate the world matrix (root→camera) at time t, substituting EVERY animated transform's
    // sampled position.
    let world_at = |t: f32| -> Mat4 {
        let mut m = Mat4::identity();
        for &tf in chain.iter().rev() {
            let local = if animated.contains_key(&tf) {
                local_trs(tf, Some(sample_tf(tf, t)))
            } else {
                local_trs(tf, None)
            };
            m = m.mul(&local);
        }
        m
    };
    // Screen axes from the camera's world orientation (t0). right = col0, up = col1.
    let m0 = world_at(times[0]);
    let axis = |col: usize| {
        let c = [m0.0[0][col], m0.0[1][col], m0.0[2][col]];
        let n = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().max(1e-6);
        [c[0] / n, c[1] / n, c[2] / n]
    };
    let right = axis(0);
    let up = axis(1);
    let inv = inv_scale as f32;
    let mut out = Vec::with_capacity(times.len());
    for &t in &times {
        let p = world_at(t).point([0.0, 0.0, 0.0]);
        // ABSOLUTE frame centre in the spine mesh space (authored px): the camera position projected
        // onto its screen right/up (an ortho camera's centre is its own in-plane position), scaled by
        // invScale. `up` is negated because the spine plane is placed Y-flipped (see note above), so
        // the mesh space is Y-UP with the character to +Y.
        let cx = (p[0] * right[0] + p[1] * right[1] + p[2] * right[2]) * inv;
        let cy = -(p[0] * up[0] + p[1] * up[1] + p[2] * up[2]) * inv;
        out.push((t, cx, cy));
    }
    Some(out)
}

/// The ENTRANCE camera POSITIONAL dolly (pan), extracted from the `_Start` clip that animates a
/// camera-ancestor's Transform position (Virtuosa "Dummy002"). Returned as `(time_s, progress)`
/// where progress ∈ [0,1] normalises the DOMINANT-axis movement (the pan travels almost entirely
/// along one axis — the vertical follow of seated→standing). The frontend replays it as the pan
/// timing between the measured seated & standing framings — the game's exact camera-move timing,
/// no hardcoding. `None` when no camera-ancestor position is animated.
#[must_use]
pub fn entrance_pan_curve(all_objects: &HashMap<i64, (i32, Value)>) -> Option<Vec<(f32, f32)>> {
    let chain = camera_ancestor_gos(all_objects);
    if chain.is_empty() {
        return None;
    }
    let hash_to_gos = build_hash_to_gos(all_objects);
    let mut best: Option<Vec<(f32, f32)>> = None; // the largest-range component curve
    let mut best_range = 0.0f32;
    for (cid, v) in all_objects.values() {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            let count = binding_curve_count(type_id, attr);
            // Collision-robust: pick the candidate GO on the camera chain (see above).
            let go = hash_to_gos
                .get(&path)
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
/// enough for the camera curves). Boundary frames skipped. See {@link decode_streamed_curve}
/// for the key/easing semantics.
fn decode_curve_at(clip: &Value, idx: usize) -> Option<Vec<(f32, f32)>> {
    let data = clip_data(clip)?;
    let streamed_raw = u32_array(data.get("m_StreamedClip").and_then(|s| s.get("data")));
    decode_streamed_curve(&streamed_raw, idx)
}

/// Dense-resample rate (Hz) for eased segments in {@link decode_streamed_curve}: high
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
///   resampled at ~{@link CURVE_RESAMPLE_HZ}, skipped when it doesn't deviate from the
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

/// The ENTRANCE camera dolly, extracted from the `_Start` AnimationClip that animates the
/// Main Camera's orthographic size: `(time_seconds, orthographic_size)` keyframes. This is the
/// game's actual data-driven camera zoom (hold → zoom in on the transform → zoom out to the
/// standing reveal) — there is NO positional camera pan (no Transform curve on the camera).
/// `None` when no clip animates the camera ortho size (most dynchars). The frontend replays it
/// as a relative zoom on the entrance frame, so no world↔authored unit conversion is needed.
#[must_use]
pub fn entrance_ortho_curve(all_objects: &HashMap<i64, (i32, Value)>) -> Option<Vec<(f32, f32)>> {
    let mut best: Option<Vec<(f32, f32)>> = None;
    for (cid, v) in all_objects.values() {
        if *cid != 74 || !is_entrance_clip(v) {
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
/// (streamed / dense / constant) — the per-index core of {@link decode_scalar_curve},
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

/// One animated material-colour channel on a renderer's GameObject, from the `_Start`
/// clip(s). A colour-channel binding (`customType` 22) encodes its target as
/// `attribute = (crc32(propName) & 0x0FFF_FFFF) | ((4 + channel) << 28)`, channel 0..3 =
/// r,g,b,a — verified on Mlynar "Fields of Ruination": the white flash's `_TintColor`
/// alpha binding is `0x7100_F6C6` (crc32("_TintColor") = `0x6100_F6C6`) and particle
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

/// Every ANIMATED material-colour channel in the `_Start` entrance clip(s), keyed by the
/// renderer's GameObject `path_id`. Only genuinely-varying curves are kept (a constant
/// channel adds nothing over the static tint). Drives per-layer colour/alpha replay —
/// e.g. Mlynar's white flash ramps `_TintColor.a` 0→0.671 over 13→15s, which a static
/// tint can only render as a held opaque white-wash.
#[must_use]
pub fn entrance_material_color_channels(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> HashMap<i64, Vec<MaterialColorChannel>> {
    let hash_to_gos = build_hash_to_gos(all_objects);
    let mut out: HashMap<i64, Vec<MaterialColorChannel>> = HashMap::new();
    for (cid, v) in all_objects.values() {
        if *cid != 74 || !is_entrance_clip(v) {
            continue;
        }
        let Some(bindings) = generic_bindings(v) else {
            continue;
        };
        let mut gidx = 0usize;
        for b in bindings {
            let (type_id, attr, path) = binding_fields(b);
            let count = binding_curve_count(type_id, attr);
            let custom = b.get("customType").and_then(Value::as_i64).unwrap_or(0);
            let is_pptr = b.get("isPPtrCurve").and_then(Value::as_i64).unwrap_or(0) != 0;
            // Top nibble 4..=7 = a colour CHANNEL binding (r/g/b/a); a channel-less
            // material float uses the plain crc32 and is not a colour.
            let nibble = ((attr as u64) >> 28) & 0xF;
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
                    // ALL of them — the consumer only uses a channel whose property
                    // matches the layer material's own colour props, so a wrong twin
                    // simply ignores it. (Picking one arbitrary GO dropped the bg
                    // flicker curves from the twin the scene actually exports.)
                    for &go in gos {
                        let entry = out.entry(go).or_default();
                        // Same channel keyed in several entrance clips: keep the richer curve.
                        if let Some(existing) = entry
                            .iter_mut()
                            .find(|c| c.prop_crc28 == prop_crc28 && c.channel == channel)
                        {
                            if curve.len() > existing.curve.len() {
                                existing.curve = curve.clone();
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
/// the renderer's GameObject `path_id`. A `customType == 22` binding whose low-28 attribute
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
    let mut out: HashMap<i64, StChannels> = HashMap::new();
    for (cid, v) in all_objects.values() {
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
            let other_hashes = all_hashes.iter().copied().filter(|&h| h != path);
            let Some(owner) =
                disambiguate_owner(candidates, other_hashes, &hash_to_gos, &is_ancestor)
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
/// `tint` came from ({@link super::spine::material_tint}). Of the animated properties,
/// the most-keyed one is used; its keyed channels REPLACE the static tint channel when
/// it IS the tint property (the tint already holds that property's static value), and
/// MULTIPLY onto the tint otherwise (the shader multiplies its colour properties; an
/// unkeyed channel contributes 1 — consistent with the static export, which ignores
/// non-tint properties). Returns `None` when no animated property matches the material.
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
        let mut rgba = tint;
        for (i, ch) in chans.iter().enumerate() {
            if let Some(c) = ch {
                let v = sample(c, t);
                rgba[i] = if same_prop {
                    (v * tint_scale).clamp(0.0, 1.0)
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
/// matching several same-named GOs is disambiguated from the other paths bound by the
/// same clip; ambiguous bindings are conservatively dropped.
fn active_timeline(
    clip: &Value,
    hash_to_gos: &HashMap<u32, Vec<i64>>,
    is_ancestor: &impl Fn(i64, i64) -> bool,
) -> HashMap<i64, ActiveWindow> {
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
    let mut windows = Vec::new();
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
                    continue;
                }
                if let Some(p) = prev {
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
                windows.push((path, reveal, hide));
            }
        }
        gidx += count;
    }
    for (path, reveal, hide) in windows {
        let Some(candidates) = hash_to_gos.get(&path) else {
            continue;
        };
        let other_hashes = all_hashes.iter().copied().filter(|&h| h != path);
        if let Some(go) = disambiguate_owner(candidates, other_hashes, hash_to_gos, is_ancestor) {
            out.insert(go, (reveal, hide));
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

/// Build a map from CRC32(transform-subpath) → GameObject `path_id`, registering
/// every ancestor-relative subpath (Unity may hash against any root). On a hash
/// collision (identical sibling rigs) the smallest `path_id` wins, deterministically.
fn build_hash_to_go(all_objects: &HashMap<i64, (i32, Value)>) -> HashMap<u32, i64> {
    build_hash_to_gos(all_objects)
        .into_iter()
        .filter_map(|(h, gos)| gos.first().map(|&g| (h, g)))
        .collect()
}

/// Like {@link build_hash_to_go} but keeps EVERY GameObject matching a subpath hash
/// (sorted by `path_id`) — identical sibling rigs (Mlynar's three sword clones) bind
/// the same relative subpath, and per-rig consumers need all candidates.
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
    for gos in hash_to_gos.values_mut() {
        gos.sort_unstable();
        gos.dedup();
    }
    hash_to_gos
}

/// Idle AnimationClips (class 74) among the bundle: any clip whose name contains
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
