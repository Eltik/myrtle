//! Unity `ParticleSystem` (198) + `ParticleSystemRenderer` (199) → JSON export.
//!
//! Emitted alongside each dynchar's `[scene].json` as `<name>[particles].json`
//! plus deduped textures in `<name>[particles]/<i>.png`. The schema is a reduced
//! form of the Unity module graph — the frontend runs a lightweight CPU
//! simulator against it, so only the high-impact modules are decoded and curves
//! are linearly sampled (Bezier tangents dropped).
//!
//! Coordinate convention is identical to `[scene].json`: spine-authored pixels,
//! Y-up, origin at the skeleton root. Unity world units are converted to pixels
//! by dividing by `skeletonScale` (i.e. ×100 for the usual `0.01`). Speed / size
//! / position-derived scalars carry that ×(1/scale) factor; lifetime (seconds),
//! rotation (degrees) and emission rate (per-second) do not.

use std::collections::HashMap;
use std::f64::consts::PI;
use std::path::Path;

use serde_json::{Value, json};

use super::spine::{BgParticleHost, get_path_id, is_additive};
use super::texture::decode_texture_object;

const RAD_TO_DEG: f64 = 180.0 / PI;

/// Resample resolution (intervals over the particle lifetime) for a
/// `velocityOverLifetime` module whose authored curves can't be flattened to one
/// constant — 20 intervals resolves an orbit closely enough for the frontend's linear
/// read while keeping the exported array small.
const VELOCITY_CURVE_SAMPLES: u32 = 20;

/// How far a `velocityOverLifetime` sample must OPPOSE the representative vector
/// (as a fraction of its magnitude) before the drift counts as reversing — i.e. as
/// motion no single constant can stand in for. See the reversal gate in
/// [`collect_dynchar_particles`].
const VELOCITY_REVERSAL_FRAC: f64 = 0.05;

/// How far a `velocityOverLifetime` sample's DIRECTION must swing from the representative
/// vector before the drift stops being expressible as one constant. A stretched billboard is
/// drawn along its velocity, so an angle error is a visible slope error for the whole
/// lifetime — and it smears the streak's light across more rows than the game's, which reads
/// as under-brightness per row rather than as a rotation. 20 deg is well outside the few
/// degrees of numerical wobble a magnitude-only curve produces (those samples are exactly
/// parallel and deviate 0), and well inside the 56 deg swing that motivated the gate.
const VELOCITY_TURN_DEG: f64 = 20.0;

/// Samples slower than this fraction of the representative speed are ignored by the turn
/// test: near a standstill the direction is numerically unstable and displaces nothing.
const VELOCITY_TURN_MIN_FRAC: f64 = 0.1;

/// Identity `_MainTex_ST` tuple: `[scaleX, scaleY, offsetX, offsetY]`.
pub(super) const ST_IDENTITY: [f64; 4] = [1.0, 1.0, 0.0, 0.0];

/// One parsed particle system, ready for texture dedup + JSON emit. The system
/// JSON is fully built except for the `tex` index, which is resolved in
/// [`export_particles`] once textures are deduped across all systems of the asset.
pub struct ParticleData {
    /// System object minus the resolved `tex` (and `trail.tex`) indices.
    pub json: Value,
    /// Main texture object (`Texture2D`, class 28) for decode, if any.
    pub tex_val: Option<Value>,
    /// Optional `_AlphaTex` companion (merged on decode).
    pub alpha_val: Option<Value>,
    /// Source texture `path_id`, the dedup key across systems.
    pub tex_pid: Option<i64>,
    /// The material's `_MainTex_ST` — WHICH sub-rect of `tex_pid` this system samples.
    /// Consumed by the scene exporter's frozen-burst heuristic (a baked copy of a burst
    /// reads the same rect; a different rect of a shared atlas is different artwork).
    pub tex_st: [f64; 4],
    /// Trail material texture (renderer's 2nd material), deduped into the same
    /// `[particles]/<i>.png` set; index written to `trail.tex`.
    pub trail_tex_val: Option<Value>,
    pub trail_alpha_val: Option<Value>,
    pub trail_tex_pid: Option<i64>,
    /// The trail material's `_MainTex_ST` (see [`ParticleData::tex_st`]).
    pub trail_tex_st: [f64; 4],
    /// Ram-shader-family material data (params + the extra texture slots), when
    /// the material's `_shaderName` contains `"Ram/"`. Its texture slots dedup
    /// into the same `[particles]/<i>.png` set as `tex`.
    pub ram: Option<RamData>,
}

/// One Ram-family (`Ram/Disturb` / `Ram/VertexDisturb`) material, ready for the
/// `"ram"` JSON block. The scalar/color/vector params are pre-baked into `json`;
/// the texture slots are carried as `(json_key, path_id, texture_object)` and
/// resolved to shared `[particles]/<i>.png` indices in [`export_particles`].
pub struct RamData {
    /// The `"ram"` object with every texture-index field left `null`.
    pub json: Value,
    /// `(json field name, source path_id, Texture2D value)` per sampled slot.
    pub texs: Vec<(&'static str, Option<i64>, Option<Value>)>,
}

// ---------------------------------------------------------------------------
// Small typed readers over the serde_json module tree.
// ---------------------------------------------------------------------------

fn f(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64)
}
fn fd(v: &Value, k: &str, d: f64) -> f64 {
    f(v, k).unwrap_or(d)
}
fn i(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(Value::as_i64)
}
fn b(v: &Value, k: &str, d: bool) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(d)
}

/// Interior samples emitted across a curve segment that carries real tangents. A cubic
/// resolved at eighths is within a fraction of a percent of itself under the linear
/// interpolation the frontend applies between samples — an accuracy figure, not a look.
const CURVE_SUBDIV: usize = 8;

/// Sample a Unity `AnimationCurve` (`m_Curve` keyframe array) into `[{t,v}]`,
/// applying `scale` to the value.
///
/// The frontend interpolates these samples LINEARLY, so a segment carrying real Bezier
/// tangents has to be RESOLVED here or its authored shape is silently replaced by the
/// chord. Virtuosa "Diversity in Oneness"'s `scene_02/window/quad_p` shows what that
/// costs: its size-over-lifetime is two keys, (0, 0) and (1, 1), with out/in slopes
/// -0.02 and 3.24 — a hard ease-IN, so the expanding diamond rings it emits stay tiny
/// for most of their life and only open up at the end. Read as the straight line those
/// two keys describe, the rings spread EVENLY instead and ten of them fill the frame as
/// a solid wash where the game shows three or four with wide gaps.
///
/// Segments Unity authored as linear (both tangents equal the chord slope) or as a
/// constant hold are emitted exactly as before, so every already-correct curve in the
/// corpus stays byte-identical.
fn sample_curve(curve: &Value, mul: f64, scale: f64) -> Vec<Value> {
    let Some(kfs) = curve.get("m_Curve").and_then(Value::as_array) else {
        return Vec::new();
    };
    let key = |kf: &Value| {
        (
            fd(kf, "time", 0.0),
            fd(kf, "value", 0.0),
            fd(kf, "inSlope", 0.0),
            fd(kf, "outSlope", 0.0),
        )
    };
    let mut out: Vec<Value> = Vec::with_capacity(kfs.len());
    for (n, kf) in kfs.iter().enumerate() {
        let (t0, v0, _, out0) = key(kf);
        out.push(json!({ "t": t0, "v": v0 * mul * scale }));
        let Some(next) = kfs.get(n + 1) else { continue };
        let (t1, v1, in1, _) = key(next);
        let span = t1 - t0;
        if span <= 0.0 {
            continue;
        }
        let chord = (v1 - v0) / span;
        let tol = 1e-6 * chord.abs().max(1.0);
        if (out0 - chord).abs() <= tol && (in1 - chord).abs() <= tol {
            continue; // straight line: the frontend's own lerp already draws it
        }
        for k in 1..CURVE_SUBDIV {
            let u = k as f64 / CURVE_SUBDIV as f64;
            let (u2, u3) = (u * u, u * u * u);
            let v = (2.0 * u3 - 3.0 * u2 + 1.0) * v0
                + (u3 - 2.0 * u2 + u) * span * out0
                + (-2.0 * u3 + 3.0 * u2) * v1
                + (u3 - u2) * span * in1;
            out.push(json!({ "t": t0 + u * span, "v": v * mul * scale }));
        }
    }
    out
}

/// Reduce a Unity `MinMaxCurve` to the schema's `MMScalar`.
/// `scale` is the unit multiplier (×1/skeletonScale for px-space quantities,
/// `1.0` for seconds / degrees / per-second rates, `RAD_TO_DEG` for radians).
fn mmscalar(v: &Value, scale: f64) -> Value {
    let scalar = fd(v, "scalar", 0.0);
    let min_scalar = fd(v, "minScalar", 0.0);
    match i(v, "minMaxState").unwrap_or(0) {
        // Const
        0 => json!({ "mode": "const", "v": scalar * scale }),
        // Single curve: value * scalar (the curve is normalized, scalar is the range top)
        1 => {
            let curve = v.get("maxCurve").cloned().unwrap_or(Value::Null);
            json!({ "mode": "curve", "curve": sample_curve(&curve, scalar, scale) })
        }
        // Two curves (random between)
        2 => {
            let maxc = v.get("maxCurve").cloned().unwrap_or(Value::Null);
            let minc = v.get("minCurve").cloned().unwrap_or(Value::Null);
            json!({
                "mode": "rangeCurve",
                "min": sample_curve(&minc, min_scalar, scale),
                "max": sample_curve(&maxc, scalar, scale),
            })
        }
        // Random between two constants
        _ => json!({ "mode": "range", "min": min_scalar * scale, "max": scalar * scale }),
    }
}

/// Whether an `MMScalar` field is effectively zero everywhere (const 0 / empty).
fn mmscalar_is_zero(v: &Value) -> bool {
    matches!(i(v, "minMaxState").unwrap_or(0), 0 | 3)
        && fd(v, "scalar", 0.0).abs() < 1e-9
        && fd(v, "minScalar", 0.0).abs() < 1e-9
}

/// Representative single value of an `MMScalar` (max/scalar branch), scaled.
/// Used where the schema flattens a curve/range to one number (velocity axes,
/// angular velocity, burst counts).
fn mmscalar_repr(v: &Value, scale: f64) -> f64 {
    fd(v, "scalar", 0.0) * scale
}

/// Evaluate a Unity `AnimationCurve` (`m_Curve` keyframes) at `t` with cubic HERMITE
/// interpolation, honouring the keys' in/out tangents. Unlike [`sample_curve`] (which
/// exports the raw key points for the frontend to lerp), this is used where the
/// exporter must resample a curve onto its own grid — a linear read of 3–4 sparse keys
/// visibly flattens the authored shape (Skadi2 iteration's orbiting crown ring).
fn curve_eval(curve: &Value, t: f64) -> f64 {
    let Some(keys) = curve.get("m_Curve").and_then(Value::as_array) else {
        return 0.0;
    };
    let read = |k: &Value| {
        (
            fd(k, "time", 0.0),
            fd(k, "value", 0.0),
            fd(k, "inSlope", 0.0),
            fd(k, "outSlope", 0.0),
        )
    };
    let Some(first) = keys.first().map(&read) else {
        return 0.0;
    };
    if t <= first.0 {
        return first.1;
    }
    let last = keys.last().map_or(first, &read);
    if t >= last.0 {
        return last.1;
    }
    for w in keys.windows(2) {
        let (t0, v0, _, out0) = read(&w[0]);
        let (t1, v1, in1, _) = read(&w[1]);
        if t <= t1 {
            let span = t1 - t0;
            if span <= 0.0 {
                return v1;
            }
            let u = (t - t0) / span;
            let u2 = u * u;
            let u3 = u2 * u;
            return (2.0 * u3 - 3.0 * u2 + 1.0) * v0
                + (u3 - 2.0 * u2 + u) * span * out0
                + (-2.0 * u3 + 3.0 * u2) * v1
                + (u3 - u2) * span * in1;
        }
    }
    last.1
}

/// Evaluate a `MinMaxCurve` at normalized particle age `t`, ×`scale`. The curve states
/// are the authored shape over the lifetime; the constant states are flat. Two-sided
/// states take the "max" side, matching [`mmscalar_repr`].
fn mmscalar_eval(v: &Value, t: f64, scale: f64) -> f64 {
    let scalar = fd(v, "scalar", 0.0);
    match i(v, "minMaxState").unwrap_or(0) {
        1 | 2 => curve_eval(v.get("maxCurve").unwrap_or(&Value::Null), t) * scalar * scale,
        _ => scalar * scale,
    }
}

/// Whether a `MinMaxCurve` holds the same value at every `t` — i.e. the schema's
/// single-number flattening loses nothing. Constant states are flat by definition; a
/// curve state is flat only when every keyframe carries the same value.
fn mmscalar_is_flat(v: &Value) -> bool {
    match i(v, "minMaxState").unwrap_or(0) {
        1 | 2 => v
            .get("maxCurve")
            .and_then(|c| c.get("m_Curve"))
            .and_then(Value::as_array)
            .is_none_or(|keys| {
                let first = keys.first().map_or(0.0, |k| fd(k, "value", 0.0));
                keys.iter()
                    .all(|k| (fd(k, "value", 0.0) - first).abs() < 1e-9)
            }),
        _ => true,
    }
}

/// Read an rgba color object → `[r,g,b,a]` (0..1).
fn read_color(v: &Value) -> [f64; 4] {
    [
        fd(v, "r", 1.0),
        fd(v, "g", 1.0),
        fd(v, "b", 1.0),
        fd(v, "a", 1.0),
    ]
}

/// Decode a Unity serialized `Gradient` (the `maxGradient`/`minGradient` object)
/// into merged rgba stops sorted by `t`. Color keys (`ctime0..7` + `key{i}.rgb`)
/// and alpha keys (`atime0..7` + `key{i}.a`) are sampled onto the union of their
/// times, each channel linearly interpolated within its own key list.
fn decode_gradient(g: &Value) -> Vec<Value> {
    let n_col = i(g, "m_NumColorKeys").unwrap_or(2).clamp(0, 8) as usize;
    let n_alpha = i(g, "m_NumAlphaKeys").unwrap_or(2).clamp(0, 8) as usize;

    let mut col_keys: Vec<(f64, [f64; 3])> = Vec::new();
    for idx in 0..n_col {
        let t = fd(g, &format!("ctime{idx}"), 0.0) / 65535.0;
        let c = g.get(format!("key{idx}")).map_or([1.0; 4], read_color);
        col_keys.push((t, [c[0], c[1], c[2]]));
    }
    let mut alpha_keys: Vec<(f64, f64)> = Vec::new();
    for idx in 0..n_alpha {
        let t = fd(g, &format!("atime{idx}"), 0.0) / 65535.0;
        let a = g
            .get(format!("key{idx}"))
            .and_then(|k| f(k, "a"))
            .unwrap_or(1.0);
        alpha_keys.push((t, a));
    }
    if col_keys.is_empty() {
        col_keys.push((0.0, [1.0, 1.0, 1.0]));
    }
    if alpha_keys.is_empty() {
        alpha_keys.push((0.0, 1.0));
    }
    col_keys.sort_by(|a, b| a.0.total_cmp(&b.0));
    alpha_keys.sort_by(|a, b| a.0.total_cmp(&b.0));

    // Linear-interp helper over a sorted (t, val) list.
    let sample3 = |t: f64| -> [f64; 3] {
        if t <= col_keys[0].0 {
            return col_keys[0].1;
        }
        for w in col_keys.windows(2) {
            let (t0, c0) = w[0];
            let (t1, c1) = w[1];
            if t <= t1 {
                let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
                return [
                    c0[0] + (c1[0] - c0[0]) * f,
                    c0[1] + (c1[1] - c0[1]) * f,
                    c0[2] + (c1[2] - c0[2]) * f,
                ];
            }
        }
        col_keys[col_keys.len() - 1].1
    };
    let sample_a = |t: f64| -> f64 {
        if t <= alpha_keys[0].0 {
            return alpha_keys[0].1;
        }
        for w in alpha_keys.windows(2) {
            let (t0, a0) = w[0];
            let (t1, a1) = w[1];
            if t <= t1 {
                let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
                return a0 + (a1 - a0) * f;
            }
        }
        alpha_keys[alpha_keys.len() - 1].1
    };

    // Union of all stop times.
    let mut times: Vec<f64> = col_keys
        .iter()
        .map(|k| k.0)
        .chain(alpha_keys.iter().map(|k| k.0))
        .collect();
    times.sort_by(f64::total_cmp);
    times.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

    times
        .into_iter()
        .map(|t| {
            let c = sample3(t);
            json!({ "t": t, "r": c[0], "g": c[1], "b": c[2], "a": sample_a(t) })
        })
        .collect()
}

/// Reduce a Unity `MinMaxGradient` (e.g. `startColor`, `ColorModule.gradient`)
/// to `{mode:"color",...}` (constant) or `{mode:"gradient",stops:[...]}`.
fn mmgradient(v: &Value) -> Value {
    // MinMaxGradient states: 0 Color, 1 Gradient, 2 TwoColors, 3 TwoGradients,
    // 4 RandomColor. Constant-color states use maxColor; gradient states use
    // maxGradient. (For the two-GRADIENT state we still take the "max" side.)
    match i(v, "minMaxState").unwrap_or(0) {
        0 => {
            let c = v.get("maxColor").map_or([1.0; 4], read_color);
            json!({ "mode": "color", "r": c[0], "g": c[1], "b": c[2], "a": c[3] })
        }
        // RandomBetweenTwoColors: Unity draws ONE colour per particle uniformly between
        // `minColor` and `maxColor`. Collapsing to `maxColor` throws the other endpoint
        // away — Skadi2 iteration's four `xian` threads author min = pure RED and
        // max = cyan, so every thread we drew was locked cyan. Export BOTH endpoints and
        // let the frontend pick per particle.
        2 => {
            let mx = v.get("maxColor").map_or([1.0; 4], read_color);
            let mn = v.get("minColor").map_or(mx, read_color);
            json!({
                "mode": "twoColors",
                "min": { "r": mn[0], "g": mn[1], "b": mn[2], "a": mn[3] },
                "max": { "r": mx[0], "g": mx[1], "b": mx[2], "a": mx[3] },
            })
        }
        _ => {
            let stops = v
                .get("maxGradient")
                .map(decode_gradient)
                .unwrap_or_default();
            json!({ "mode": "gradient", "stops": stops })
        }
    }
}

/// Map a Unity `ShapeModule.type` int to the schema's shape family.
const fn shape_type_name(t: i64) -> &'static str {
    match t {
        0 | 1 => "sphere",
        2 | 3 => "hemisphere",
        4 | 7 | 8 | 9 => "cone",
        5 | 15 | 16 => "box",
        10 | 11 => "circle",
        12 => "edge",
        _ => "none", // mesh / donut / rectangle / sprite → approximated as none
    }
}

/// Map a Unity `ParticleSystemRenderMode` int to the schema's render mode.
const fn render_mode_name(m: i64) -> &'static str {
    match m {
        1 => "stretch",
        4 => "mesh",
        // 5 = None: the particle draws NO head sprite at all — the system is a pure
        // TRAIL emitter (Skadi2 iteration's `xian` threads). Billboarding it stamps a
        // bright sprite at every ribbon tip the game never draws, which is what made
        // our threads read thick and glowy.
        5 => "none",
        _ => "billboard", // 0 Billboard, 2/3 axis-billboards
    }
}

/// Emitter world Z-rotation (degrees) from an accumulated world matrix.
fn matrix_z_deg(m: &super::mesh::Mat4) -> f64 {
    let a = &m.0;
    (f64::from(a[1][0]).atan2(f64::from(a[0][0]))) * RAD_TO_DEG
}

/// Prefab-root membership scoping for a particle collection pass: the exporting
/// skeleton's OWN root and the set of ALL skeleton roots in the bundle (see the
/// scoping skip in [`collect_dynchar_particles`]).
pub(crate) struct RootScope<'a> {
    pub own: Option<i64>,
    pub skeleton_roots: &'a std::collections::HashSet<i64>,
}

/// The `_Start`-cinematic animation context for a particle collection pass: the
/// per-GameObject reveal windows, animated emission-rate curves, event-driven-rate
/// `GameObjects` (idle path), and effect-host transform (scale/position) curves. All
/// empty for the idle/main scene (which plays none of these clips).
pub(crate) struct EntranceCtx<'a> {
    pub windows: &'a HashMap<i64, super::anim::ActiveWindowList>,
    pub rate_curves: &'a HashMap<i64, Vec<(f32, f32)>>,
    pub event_rate_gos: &'a std::collections::HashSet<i64>,
    pub transform_curves: &'a HashMap<i64, super::anim::EntranceTransform>,
    /// GO → the `_Start` clip's animated material-colour channels. The scene-quad
    /// exporter has always replayed these (`layer_color_curve`); the particle path
    /// never did, so a Ram emitter whose `_MainColor` the cinematic drives (Mlynar's
    /// `fangkuai_*` city slabs brightening through the transformation beat) rendered
    /// at its static serialized colour for the whole shot. Empty on the idle path.
    pub color_channels: &'a HashMap<i64, Vec<super::anim::MaterialColorChannel>>,
    /// Whether this pass is the `_Start` cinematic (vs. the idle/main scene).
    /// Gates [`apply_followbone_reveal_inheritance`] — a bone-follower rig
    /// "sibling reveal" is a cinematic reveal-window (`m_IsActive`) concept;
    /// the idle scene's `delay` field means something different (a per-system
    /// serialized `startDelay`/`_delayTime`, not a group reveal), so it must
    /// not borrow a sibling's individually-authored startup delay.
    pub is_entrance: bool,
}

/// Propagate a shared reveal through a bone-follower rig. Some falling-apple/
/// comet-rig leaves are not individually gated by `m_IsActive`/`_delayTime`,
/// but share a `followBone` with siblings that are and should not render from
/// t=0.
fn apply_followbone_reveal_inheritance(out: &mut [ParticleData]) {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, particle) in out.iter().enumerate() {
        if let Some(follow_bone) = particle.json.get("followBone").and_then(Value::as_str) {
            groups.entry(follow_bone.to_owned()).or_default().push(idx);
        }
    }

    for indices in groups.into_values() {
        let mut gated = Vec::new();
        let mut ungated = Vec::new();
        for idx in indices {
            match out[idx].json.get("delay").and_then(Value::as_f64) {
                Some(delay) => gated.push(delay),
                None if out[idx].json.get("delay").is_none() => ungated.push(idx),
                None => {}
            }
        }

        if gated.is_empty() || ungated.is_empty() {
            continue;
        }

        gated.sort_by(f64::total_cmp);
        let mut reveal_value = gated[0];
        let mut largest_count = 0;
        for &candidate in &gated {
            let count = gated
                .iter()
                .filter(|&&delay| (delay - candidate).abs() < 1e-4)
                .count();
            if count > largest_count {
                largest_count = count;
                reveal_value = candidate;
            }
        }

        for idx in ungated {
            out[idx].json["delay"] = json!(reveal_value);
        }
    }
}

/// Parse every enabled, emitting `ParticleSystem` in a dynchar prefab into
/// [`ParticleData`]. `inv_scale` is `1.0 / skeletonScale` (Unity units → px).
///
/// Emits are skipped (and counted by the caller) when: the `InitialModule` is
/// disabled, the emitter `GameObject` is inactive, the renderer is disabled, or the
/// `EmissionModule` is disabled with no bursts (nothing is emitted).
#[must_use]
/// Per-reason tally of `ParticleSystem`s dropped by [`collect_dynchar_particles`].
///
/// The collector used to report one opaque total, which is useless when an effect is missing
/// from a render: two thirds of a skin's systems being "skipped" is normal (state-gated
/// Start/Interact/Special-Only groups, sibling-root effects), so the total cannot distinguish
/// a correct drop from a lost one. Breaking it out by gate makes a missing element
/// attributable to the exact rule that removed it.
#[derive(Default, Clone, Copy)]
pub(crate) struct ParticleSkips {
    /// System component with no owning `m_GameObject`.
    pub no_gameobject: usize,
    /// Under a state-gated inactive group ("Start/Interact/Special Only Effects").
    pub inactive_group: usize,
    /// Belongs to a sibling skeleton's prefab root.
    pub cross_root: usize,
    /// `InitialModule.enabled == false`.
    pub initial_disabled: usize,
    /// Emission disabled AND no bursts — nothing would ever spawn.
    pub never_emits: usize,
    /// Event-clip-driven rate with no bursts — quiet at the steady state.
    pub event_rate_no_burst: usize,
    /// `ParticleSystemRenderer.m_Enabled == false`.
    pub renderer_disabled: usize,
}

impl ParticleSkips {
    pub(crate) const fn total(&self) -> usize {
        self.no_gameobject
            + self.inactive_group
            + self.cross_root
            + self.initial_disabled
            + self.never_emits
            + self.event_rate_no_burst
            + self.renderer_disabled
    }
}

impl std::fmt::Display for ParticleSkips {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (label, n) in [
            ("no-gameobject", self.no_gameobject),
            ("inactive-group", self.inactive_group),
            ("cross-root", self.cross_root),
            ("initial-disabled", self.initial_disabled),
            ("never-emits", self.never_emits),
            ("event-rate-no-burst", self.event_rate_no_burst),
            ("renderer-disabled", self.renderer_disabled),
        ] {
            if n == 0 {
                continue;
            }
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{label} {n}")?;
            first = false;
        }
        Ok(())
    }
}

pub(crate) fn collect_dynchar_particles(
    all_objects: &HashMap<i64, (i32, Value)>,
    inv_scale: f64,
    resources: &HashMap<String, Vec<u8>>,
    host: &BgParticleHost,
    entrance: &EntranceCtx<'_>,
    scope: &RootScope<'_>,
) -> (Vec<ParticleData>, ParticleSkips) {
    let entrance_windows = entrance.windows;
    let entrance_rate_curves = entrance.rate_curves;
    let event_rate_gos = entrance.event_rate_gos;
    let entrance_transform_curves = entrance.transform_curves;
    // GameObject path_id → Renderer(199) value.
    let mut go_to_renderer: HashMap<i64, &Value> = HashMap::new();
    for (cid, v) in all_objects.values() {
        if *cid == 199
            && let Some(go) = v.get("m_GameObject").and_then(get_path_id)
        {
            go_to_renderer.insert(go, v);
        }
    }

    // Deterministic iteration order.
    let mut systems: Vec<(i64, &Value)> = all_objects
        .iter()
        .filter(|(_, (cid, _))| *cid == 198)
        .map(|(pid, (_, v))| (*pid, v))
        .collect();
    systems.sort_unstable_by_key(|(pid, _)| *pid);

    let mut out = Vec::new();
    let mut skipped = ParticleSkips::default();
    // Env-gated attribution — see the twin in `collect_dynchar_bg_quads`. Output-neutral.
    let attrib_dbg = std::env::var("SCENE_ATTRIB").is_ok();

    // World origins of every SAME-ROOT particle system, so a cross-root candidate can be tested
    // for a positional twin. See `admit_positional_unique` below. **Default ON since
    // 2026-08-08** — `DYNCHAR_CROSSROOT_UNIQUE=0` restores the old drop-everything behaviour.
    let crossroot_unique = std::env::var("DYNCHAR_CROSSROOT_UNIQUE").as_deref() != Ok("0");
    // A positional twin must ALSO share the GameObject's NAME (`DYNCHAR_CROSSROOT_NAMEKEY=1`).
    // The same authored object reachable from two prefab roots keeps its name; two genuinely
    // different systems that merely both sit at the WORLD ORIGIN do not. Origin-only matching
    // is near-meaningless at (0, 0), where rig nodes and full-frame backdrops pile up — on
    // Civilight Eterna it silently drops `BG_lizi`, `BG_Idle` and `Special_01`, none of which
    // has an exported counterpart of that name.
    let crossroot_namekey = std::env::var("DYNCHAR_CROSSROOT_NAMEKEY").as_deref() == Ok("1");
    let own_positions: Vec<(f32, f32, String)> = if crossroot_unique {
        systems
            .iter()
            .filter_map(|(_, ps)| {
                let go = ps.get("m_GameObject").and_then(get_path_id)?;
                let same = matches!(
                    (scope.own, host.prefab_root_of_go(all_objects, go)),
                    (Some(own), Some(root)) if root == own
                );
                if !same {
                    return None;
                }
                let o = host.world_of_go(all_objects, go).point([0.0, 0.0, 0.0]);
                Some((o[0], o[1], host.go_name(all_objects, go)))
            })
            .collect()
    } else {
        Vec::new()
    };

    for (_, ps) in systems {
        let Some(go_pid) = ps.get("m_GameObject").and_then(get_path_id) else {
            skipped.no_gameobject += 1;
            continue;
        };

        // The nearest ancestor the `_Start` clip drives with a Transform-scale curve
        // (Virtuosa's crown `ctrl` above the glow hosts). Empty map on the idle path.
        let transform_host =
            host.entrance_transform_host_of_go(all_objects, go_pid, entrance_transform_curves);

        // Is this system under a DIFFERENT skeleton's prefab root (the sibling
        // `dyn_illust_*`/`dyn_entrance_*` instance)? Such systems are normally that
        // scene's effect, dropped here — UNLESS the cinematic reaches across roots to
        // drive this exact host (a clip-scaled effect host), in which case it belongs
        // in the entrance scene and bypasses both the active-state and scoping gates.
        let cross_root = matches!(
            (scope.own, host.prefab_root_of_go(all_objects, go_pid)),
            (Some(own), Some(root)) if root != own && scope.skeleton_roots.contains(&root)
        );
        // EXPERIMENT (`DYNCHAR_ADMIT_ENTRANCE_ROOT=1`, default OFF): admit cross-root systems
        // that live on the sibling `dyn_entrance_*` prefab specifically.
        //
        // Narrower than either variant measured below: those admitted the IDLE rig wholesale.
        // The motivation is Skadi the Corrupting Heart's t=7 red transient — the game spikes
        // red-excess to +39.1 there while we reach +4.4, and no `_Start` system starts emitting
        // in 6.0-7.6s. The entrance rig holds unexported candidates that fit, notably
        // `xiaoyu (2)` (pure red 1,0,0 at full alpha, 1000 particles, 2s).
        let admit_entrance_root = std::env::var("DYNCHAR_ADMIT_ENTRANCE_ROOT").is_ok()
            && cross_root
            && host
                .root_name_of_go(all_objects, go_pid)
                .starts_with("dyn_entrance");
        // A `Start Only Effects` system reaches the ENTRANCE scene even across roots.
        //
        // Those groups are authored on the IDLE prefab, so a start-only emitter is dropped by the
        // idle export (`inactive-group`, correctly — it must not play in the idle loop) AND by the
        // `_Start` export (`cross-root`, since its prefab root is the idle one). It therefore
        // lands in NO export, even though the entrance is the one state it exists for. This is
        // the cross-root partner of the already-shipped rule that keeps start-only groups in the
        // entrance scene, and it is deliberately narrow: only the entrance export, only groups the
        // game reserves for the START state. Blanket cross-root admission is a different change
        // and is measured-worse twice over (see the note below).
        //
        // Found via Skadi the Corrupting Heart's pure-red `xiaoyu` (start colour 1.00,0.05,0.05 at
        // full alpha): at t=7 every red line in the frame matched the game except one at row 234,
        // three times stronger than any other, which nothing we exported could draw.
        // MEASURED AND REJECTED (2026-08-01) — kept behind `DYNCHAR_START_ONLY_CROSS_ROOT=1`,
        // default OFF.
        //
        // The reasoning still looks right (a start-only system authored on the IDLE prefab root
        // reaches NEITHER export), and on Skadi it helps slightly: 13.795 -> 13.789. But it adds
        // 10 systems to Mlynar's entrance (50 -> 60) and costs him **30.816 -> 31.306**, so it is
        // net-negative across the corpus.
        //
        // I originally scored it having installed only SKADI's re-export, saw mly/cel unchanged,
        // and reported it as a win — the other two simply had stale assets. ALWAYS re-export and
        // install EVERY reference skin before attributing a delta.
        let admit_start_only_cross_root = std::env::var("DYNCHAR_START_ONLY_CROSS_ROOT").is_ok()
            && cross_root
            && entrance.is_entrance
            && host.has_start_only_ancestor(all_objects, go_pid);
        // Admit a cross-root system only when NO same-root system sits at the same world
        // origin? **MEASURED AND REJECTED — the FOURTH refutation of cross-root admission.**
        //
        // The premise was the best one yet, and the observation behind it is real: matching
        // every dropped cross-root system's world origin against the systems the entrance
        // DOES export, 24 of Mlynar's 46 are positional twins. So blanket admission
        // double-draws half the set — for additive emitters a straight doubling of light,
        // matching the sign and rough size of the +1.84 blanket cost on Virtuosa. That
        // explains the earlier failures, and de-duplicating geometrically looked like the
        // missing discriminator. It is not:
        //
        //     baseline                mly 17.721   cel 19.422   ska 10.505
        //     positional de-dup       mly 17.721   cel 29.881   ska 10.530
        //
        // Virtuosa loses TEN POINTS. The gate adds 18 systems to her entrance (58 -> 76) and
        // 4 to Skadi's; Mlynar gains ZERO and is bit-identical, so it could never have
        // addressed the t=4 warm deficit that motivated the search.
        //
        // Note the twin set here is every SAME-ROOT system, not only the exported ones —
        // strictly wider than the analysis that suggested 22 unique candidates for Mlynar,
        // which is why he gains none: the entrance owns systems at those positions that are
        // themselves dropped for other reasons. A narrower twin set would admit MORE, not
        // less, so relaxing it only moves further in the direction that already fails.
        //
        // **⚠️ RE-MEASURED 2026-08-08 — the ten-point loss was the PLACEMENT BUG, not the
        // admission.** Every one of the four refutations ran while cross-root systems were
        // emitted in their OWN root's frame (see the re-basing below), so each admitted a set of
        // emitters that then drew in the wrong place. With the re-basing in, the same positional
        // de-dup gate measures:
        //
        //     shipped baseline            mly 17.864  cel 17.646  ska 10.403   cet 58.925
        //     positional de-dup + rebase  mly 17.864  cel 17.630  ska 10.446   cet 57.949
        //
        // Virtuosa goes from −10.5 to +0.016. Across the three references it is neutral (±0.05,
        // and Mlynar is bit-identical because he gains no systems), and Civilight Eterna — whose
        // entrance embers are ALL cross-root — gains ~1.0. So the conclusion "the idle rig's
        // emitters are not what the entrance is missing" no longer follows from the evidence.
        //
        // **Corpus re-export + QA sweep run 2026-08-08 — now DEFAULT ON.** Only 10 of 82 skins
        // change at all (every one an entrance set); the other 72 are byte-identical. Seven of
        // the ten have game captures and were scored before/after:
        //
        //     mly 17.864→17.864   cel 17.646→17.630   ska 10.403→10.446   exc 24.074→24.073
        //     cet 58.925→57.949   mue 86.446→86.412   wis 88.117→88.135
        //
        // Worst regression +0.043 (Skadi), best −0.976 (Civilight Eterna), net −0.966. The three
        // changed skins with no capture (Kalt'sits, Chongyue, Passenger) were QA-rendered at 5
        // beats before and after: mean|diff| ≤ 1.51, no BLANK/DEGENERATE flags.
        // `DYNCHAR_CROSSROOT_UNIQUE=0` restores the old behaviour.
        let admit_positional_unique = crossroot_unique && cross_root && entrance.is_entrance && {
            let o = host.world_of_go(all_objects, go_pid).point([0.0, 0.0, 0.0]);
            let my_name = host.go_name(all_objects, go_pid);
            !own_positions.iter().any(|(x, y, n)| {
                (x - o[0]).abs() <= 3.0f32
                    && (y - o[1]).abs() <= 3.0f32
                    && (!crossroot_namekey || *n == my_name)
            })
        };
        let admit_cross_root = (cross_root && transform_host.is_some())
            || admit_entrance_root
            || admit_start_only_cross_root
            || admit_positional_unique;
        // A cross-root system is NOT admitted by the entrance's cross-root REVEAL beat,
        // even though the scene-quad path admits cross-root MESH layers exactly that way
        // (`cross_root_reveal` in `spine.rs` → a layer's `rootRevealFrom`). Both readings
        // of that beat were built and measured against the three reference captures, and
        // both are worse than dropping these systems:
        //
        //   drop (this behaviour)           Mlynar 21.482  Virtuosa 30.105  Skadi 18.608
        //   admit the idle rig at the beat         21.482           31.946         18.634
        //   …and retire the entrance rig too       21.482           31.714         18.696
        //
        // The first says the two rigs do not both run — density doubles after the beat and
        // Virtuosa's whole loss is in her post-beat frames. The second says the entrance
        // rig is not retired either — removing it costs Skadi at t=13 and t=19. So the game
        // composites the two prefabs' EMITTERS by some rule the mesh-layer beat does not
        // describe, and guessing it renders worse than not guessing. The known casualty is
        // Skadi the Corrupting Heart's `hongxian_01` coral ribbon, which lives only in the
        // idle root and so is still absent from her `_Start` render at t=13.

        // Active up the whole hierarchy — excludes emitters under a state-gated
        // inactive group ("Start/Interact/Special Only Effects"), which otherwise
        // all play at once in the idle scene (noise).
        // Emitter position (spine-root px, Y-up) and start colour on the drop lines — without
        // them a drop list cannot answer "which dropped system paints the red line at y~400?",
        // which is exactly the question a missing effect poses.
        let drop_where = |all_objects: &HashMap<i64, (i32, Value)>| -> String {
            let w = host.world_of_go(all_objects, go_pid);
            let o = w.point([0.0, 0.0, 0.0]);
            let c = ps
                .get("InitialModule")
                .and_then(|m| m.get("startColor"))
                .and_then(|c| c.get("maxColor").or_else(|| c.get("minColor")).or(Some(c)))
                .map_or_else(
                    || "-".to_string(),
                    |c| {
                        let g = |k: &str| c.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN);
                        format!("({:.2},{:.2},{:.2},{:.2})", g("r"), g("g"), g("b"), g("a"))
                    },
                );
            format!(
                "pos=({:.0},{:.0}) col={c}",
                f64::from(o[0]) * inv_scale,
                f64::from(o[1]) * inv_scale
            )
        };
        // 🚨 THE ENTRANCE MUST NOT PLAY THE IDLE STATES' EFFECTS.
        //
        // A dynchar prefab instantiates its effect rig once PER STATE, as `<skin>_<State>(Clone)`
        // subtrees — `_Start`, `_Start_Idle`, `_Idle`, `_Interact`, `_Special`. The game runs the
        // clone for the state it is in; the `_Start` cinematic must not draw the idle ones. The
        // existing gate only blocks `<State> Only Effects` GROUPS, so an idle clone parked under
        // an ordinary group (`General Effects`, or nothing) sails straight through.
        //
        // Ch'en the Holungday exposed it. Her two `glow 01` emitters are the SAME authored effect
        // at the same local position (0.05, 9.85, 0), instantiated under
        // `..._Start_Idle_01(Clone)` and `..._Idle_01(Clone)/General Effects` — and both were
        // drawn through her whole cinematic: a 1250px full-frame additive glow, twice over. They
        // cost 6.76 MADC of her 28.530, and removing them takes her frame-mean luma to
        // -0.8 / +2.4 / +13.3 against the game where it had been +13.2 / +16.8 / +21.6.
        //
        // ⚠️ Match on the STATE, not on the substring: `_Start_Idle` contains "start" and is an
        // IDLE clone. That is the same name trap that hid the reveal-timeline bug (`03`), so the
        // test is "an ancestor `(Clone)` whose state names idle", never "the name has start in it".
        // `DYNCHAR_ENT_IDLE_CLONE=1` reverts.
        if entrance.is_entrance
            && std::env::var("DYNCHAR_ENT_IDLE_CLONE").as_deref() != Ok("1")
            && let Some(clone) = host
                .ancestor_go_names(all_objects, go_pid)
                .into_iter()
                .find(|n| super::spine::non_cinematic_state_clone(n))
        {
            skipped.inactive_group += 1;
            if attrib_dbg {
                eprintln!(
                    "    [ptcl] DROP idle-state-clone {:<22} {} host='{clone}'",
                    host.go_name(all_objects, go_pid),
                    drop_where(all_objects),
                );
            }
            continue;
        }
        if !admit_cross_root && !host.effectively_active(all_objects, go_pid, entrance.is_entrance)
        {
            skipped.inactive_group += 1;
            if attrib_dbg {
                // Name the ANCESTOR that blocked it, not just the bundle root: a legitimate
                // "<State> Only Effects" gate and a wrongly-blocked ordinary group look
                // identical otherwise, and they need opposite fixes.
                let (by, why) = host
                    .blocking_ancestor(all_objects, go_pid, entrance.is_entrance)
                    .unwrap_or_else(|| ("?".to_string(), "?"));
                eprintln!(
                    "    [ptcl] DROP inactive-group  {:<26} {} blocked_by='{by}' ({why})",
                    host.go_name(all_objects, go_pid),
                    drop_where(all_objects),
                );
            }
            continue;
        }
        if cross_root && !admit_cross_root {
            skipped.cross_root += 1;
            if attrib_dbg {
                eprintln!(
                    "    [ptcl] DROP cross-root      {:<26} {} root={}",
                    host.go_name(all_objects, go_pid),
                    drop_where(all_objects),
                    host.root_name_of_go(all_objects, go_pid)
                );
            }
            continue;
        }

        let initial = ps.get("InitialModule").cloned().unwrap_or(Value::Null);
        if !b(&initial, "enabled", false) {
            skipped.initial_disabled += 1;
            continue;
        }

        let emission = ps.get("EmissionModule").cloned().unwrap_or(Value::Null);
        let bursts_raw = emission
            .get("m_Bursts")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let emission_enabled = b(&emission, "enabled", false);
        if !emission_enabled && bursts_raw.is_empty() {
            skipped.never_emits += 1; // nothing is ever emitted
            if attrib_dbg {
                eprintln!(
                    "    [ptcl] DROP never-emits     {:<26} {} root={}",
                    host.go_name(all_objects, go_pid),
                    drop_where(all_objects),
                    host.root_name_of_go(all_objects, go_pid)
                );
            }
            continue;
        }
        // Event-clip-driven rate with no bursts: quiet at the steady state (the rate
        // is exported as 0 below) — nothing would ever spawn, so drop the system.
        if event_rate_gos.contains(&go_pid) && bursts_raw.is_empty() {
            skipped.event_rate_no_burst += 1;
            continue;
        }

        // Renderer (blend / sort / render mode / material texture).
        let renderer = go_to_renderer.get(&go_pid).copied();
        if let Some(r) = renderer
            && !b(r, "m_Enabled", true)
        {
            skipped.renderer_disabled += 1;
            continue;
        }

        let sort = renderer.and_then(|r| i(r, "m_SortingOrder")).unwrap_or(0);
        let render_mode =
            render_mode_name(renderer.and_then(|r| i(r, "m_RenderMode")).unwrap_or(0));

        // Stretched-billboard geometry from the renderer (Unity `ParticleSystemRenderer`): the
        // quad is elongated along the particle's screen velocity by `lengthScale·size +
        // velocityScale·speed` (world units); `cameraVelocityScale` adds camera-relative velocity.
        // The frontend previously invented a single speed factor (0.05) with no size term, which
        // over-stretched fast, tiny specks (Virtuosa's `rain_01`, Mlynar's `fire_sdx`) into long
        // foreground streaks the game never shows. Export the authored scales so the stretch length
        // is faithful per system; absent → Unity's defaults (lengthScale 2, velocity scales 0).
        let (stretch_len_scale, stretch_vel_scale, stretch_cam_vel_scale) =
            if render_mode == "stretch" {
                (
                    renderer.and_then(|r| f(r, "m_LengthScale")).unwrap_or(2.0),
                    renderer
                        .and_then(|r| f(r, "m_VelocityScale"))
                        .unwrap_or(0.0),
                    renderer
                        .and_then(|r| f(r, "m_CameraVelocityScale"))
                        .unwrap_or(0.0),
                )
            } else {
                (0.0, 0.0, 0.0)
            };

        // Resolve the first material's _MainTex (+ optional _AlphaTex), blend, tiling.
        let (tex_val, alpha_val, tex_pid, blend, main_st, tint, ab_tint) =
            resolve_renderer_texture(all_objects, renderer);
        // DIAGNOSTIC (`DYNCHAR_PSMAT`): the system's side of the material join, by the main
        // texture's path id (the material line prints the same id as `mainpid=`).
        if std::env::var("DYNCHAR_PSMAT").is_ok() {
            eprintln!(
                "PSSYS '{}' mainpid={:?} additive={blend} mode={render_mode} looping={:?}",
                host.go_name(all_objects, go_pid),
                tex_pid,
                ps.get("looping").and_then(Value::as_bool)
            );
        }

        // Ram shader family (`Ram/Disturb` / `Ram/VertexDisturb`): a ramp-tint +
        // dissolve + UV-disturb compositor sampling 4–6 textures. Its params +
        // extra texture slots are exported in a `"ram"` block; the textures share
        // the same `[particles]/<i>.png` dedup set as `tex`.
        let ram = resolve_ram(
            all_objects,
            renderer,
            entrance.color_channels.get(&go_pid).map(Vec::as_slice),
            blend,
            go_pid,
        );
        // Same clip data, for the emitters the Ram path does not cover (see
        // `particle_color_curve`). Gated for measurement: `DYNCHAR_PTCL_COLORCURVE=1`.
        let color_curve = if std::env::var("DYNCHAR_PTCL_COLORCURVE").is_ok() {
            particle_color_curve(
                all_objects,
                renderer,
                entrance.color_channels.get(&go_pid).map(Vec::as_slice),
                blend,
            )
        } else {
            None
        };

        // Emitter world transform (spine-root frame) → position (px) + Z rot.
        //
        // CROSS-ROOT RE-BASING. `world_of_go` accumulates up to whichever skeleton root it
        // reaches, so a system living under the IDLE root comes back in the IDLE root's frame —
        // but the entrance composite draws in the `_Start` root's frame. Emitting that position
        // unchanged places the effect wherever the two roots happen to differ.
        //
        // Measured on Civilight Eterna (`char_4134_cetsyr_epoque#50`), whose entrance embers are
        // all cross-root: with admission on, `emitprobe` showed every `fire_*` emitter alive and
        // painting ~50k px² with ONE particle of 11-16 on screen, boxes at y ≈ −100..−250 against
        // a 416 px frame — roughly 300 px too high, where the game puts them mid-frame. Their
        // chain roots at `..._Idle_01(Clone)` through idle-rig bones (`fixed`, `static_offset`)
        // the `_Start` composite does not have.
        //
        // Fix: express both in the common prefab frame (`world_full_of_go`, which does not stop
        // at a root and so includes the root's own transform) and map into the entrance root's
        // local frame — `own_root⁻¹ · system_world`. Same-root systems are untouched, so this is
        // inert wherever cross-root admission is off, which is everywhere by default.
        let world = host.world_of_go(all_objects, go_pid);
        let world = if cross_root && admit_cross_root {
            match scope.own {
                Some(own) => {
                    let own_root = host.world_full_of_go(all_objects, own);
                    let sys_full = host.world_full_of_go(all_objects, go_pid);
                    own_root.inverse_affine().mul(&sys_full)
                }
                None => world,
            }
        } else {
            world
        };
        let origin = world.point([0.0, 0.0, 0.0]);
        let pos = [
            f64::from(origin[0]) * inv_scale,
            f64::from(origin[1]) * inv_scale,
        ];
        let rot = matrix_z_deg(&world);

        // Ancestor GameObject-name chain (nearest→root). The frontend matches
        // these against spine bone names to make bone-parented emitters drift
        // with the character (e.g. SilverAsh the Reignfrost's magic circles).
        let bone_chain = host.ancestor_go_names(all_objects, go_pid);

        // Start delay (seconds) before this emitter fires — the `_Start` cinematic
        // sequences its effect groups (the apple, the glow, the gala transform BURST at
        // ~12s), so an emitter must not fire until its group is enabled. Two mechanisms
        // gate an emitter, both absolute from cinematic start; take the LATER (the emitter
        // is live only once BOTH conditions hold):
        //   1. `_delayTime` activator MonoBehaviours in the ancestry (`delay_of_go`).
        //   2. the `_Start` clip's `m_IsActive` switch-ON of the emitter's group
        //      (`entrance_reveal_of_go`) — the apple/glow/wing sparks toggle on at
        //      ~4.8–9.8s; without this they emit from t=0 (seated intro) and are spent
        //      before the apple falls, so the gold sparkle is missing at the transform.
        // Empty `entrance_windows` (main/idle scenes) → mechanism 2 is a no-op.
        //   3. the ParticleSystem's own serialized `startDelay` (seconds) — Unity waits
        //      this long after Play() before the system's clock starts. Mlynar "Fields
        //      of Ruination" sequences its ENTIRE entrance with it (weapon flash 8.72s,
        //      the transform explosion `bao_01` 12.4s, wind gusts 2.3–11.6s, the bird
        //      burst 4.4s) — no `_delayTime` activators exist in that prefab at all.
        //      It runs AFTER the activation gates, so it ADDS to the later of (1)/(2).
        let ent_reveal = host.entrance_reveal_of_go(all_objects, go_pid, entrance_windows);
        // DIAGNOSTIC (SCENE_ATTRIB=1): particles currently take only the START of the clip's
        // `m_IsActive` window and DISCARD its end, so a system the cinematic switches OFF keeps
        // emitting forever. Print the full window to see whether the data is even there.
        if attrib_dbg {
            if let Some(w) = host.entrance_window_of_go(all_objects, go_pid, entrance_windows) {
                eprintln!(
                    "    [ptcl] WINDOW {:<26} from={:?} until={:?}",
                    host.go_name(all_objects, go_pid),
                    w.0,
                    w.1
                );
            }
            // Does the `_Start` clip animate this emitter's MATERIAL COLOUR? Scene layers get
            // that curve (`colorCurve`); particles do not. A clip that fades an emitter's
            // material would make the game's effect die away while ours keeps going.
            if let Some(chs) = entrance.color_channels.get(&go_pid) {
                let span: Vec<String> = chs
                    .iter()
                    .map(|c| {
                        let f = c.curve.first().map_or(f32::NAN, |&(_, v)| v);
                        let l = c.curve.last().map_or(f32::NAN, |&(_, v)| v);
                        let t0 = c.curve.first().map_or(f32::NAN, |&(t, _)| t);
                        let t1 = c.curve.last().map_or(f32::NAN, |&(t, _)| t);
                        format!(
                            "ch{} {:.2}->{:.2} over {:.1}-{:.1}s",
                            c.channel, f, l, t0, t1
                        )
                    })
                    .collect();
                eprintln!(
                    "    [ptcl] MATCOLOR {:<24} {} {}",
                    host.go_name(all_objects, go_pid),
                    drop_where(all_objects),
                    span.join(" | ")
                );
            }
        }
        let start_delay = ps
            .get("startDelay")
            .and_then(|sd| sd.get("scalar"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .max(0.0);
        let delay = host
            .delay_of_go(all_objects, go_pid)
            .max(f64::from(ent_reveal.unwrap_or(0.0)))
            + start_delay;

        // The emitter's world scale. Particle SIZE, SPEED, SHAPE offsets and
        // per-particle VELOCITY are authored in the emitter's LOCAL frame; Unity
        // renders them scaled by the emitter transform. These dynchar prefabs nest
        // emitters at wildly different scales (Logos' glow systems sit at ~0.0025,
        // others at 1.0). The exporter otherwise applies only the global skeleton
        // `inv_scale`, so a 0.0025-scale emitter blows its particles up ~400x —
        // the "oppressive" storm. Bake the emitter's own scale in: local-frame
        // quantities use `em_inv`, world-frame ones (position above, gravity below)
        // keep `inv_scale`. Emitters already at world-scale 1.0 (the common case,
        // and every currently-correct skin) are unchanged (× 1.0).
        let ex = world.point([1.0, 0.0, 0.0]);
        let ey = world.point([0.0, 1.0, 0.0]);
        // Axis lengths measured in the SCREEN PLANE.
        //
        // REFUTED, do not "fix" this to 3D lengths. An emitter axis rotated to point along
        // the view direction projects to ~0 length and drags the mean down, so `em_inv`
        // comes out at half the emitter's true uniform scale — Skadi the Corrupting Heart's
        // entrance emitters are exactly that (3D |X| = |Y| = 100 px, screen |X| = 0.00). The
        // billboard argument says a camera-facing particle's size should not foreshorten, so
        // substituting the 3D length looks obviously right. It measures WORSE, every way it
        // has been tried:
        //
        //     3D length unconditionally        mly 21.481 → 21.659   ska 18.669 → 18.964
        //     3D for SIZE quantities only      mly 21.481 → 21.659   ska 18.669 → 18.897
        //     3D only when the projection has
        //       collapsed (< 0.5 of 3D)        mly 21.481 → 21.638   ska 18.669 → 18.964
        //     …same rule at < 0.05             mly 21.481 → 21.638   ska 18.669 → 18.964
        //
        // Narrowing the rule cannot rescue it: these axes project to ~0.00, so every
        // threshold catches them and Skadi lands on the identical 18.964. Cello is untouched
        // throughout (0 systems changed).
        //
        // 2026-07-31 — the ANSWER is now known: the halving IS wrong, but it cannot be fixed
        // on its own. Re-measured at today's baseline, SIZE-only 3D reproduces the historic
        // number (mly 19.278 → 19.447, cel 29.904 → 29.908, ska 18.671 → 18.896), and the
        // 3D size is nonetheless demonstrably CORRECT: Skadi's crown shoal (`_Start` sys23,
        // `followBone Skadi_Head_Fh`) draws 5x7 px fish where the game draws 12-18 px, and
        // texture 7's bright core is 25.8% x 42.2% of its page — so the exported startSize
        // predicts exactly the 5x7 we render. Doubling it makes the fish MATCH the game's
        // size and ring positions on screen.
        //
        // It scores worse because a SECOND fault is still in place: additive BILLBOARD
        // particles never receive the ×2 Unity additive factor that the MESH path applies as
        // `ADDITIVE_MESH_BOOST`, so the fish stay warm-orange where the game's are white
        // (their authored `startColor` really is warm — both `minMaxState 2` endpoints — and
        // texture 7 is white, so only intensity can neutralise them). Doubling the size while
        // the colour is still wrong just paints 4x the area in the wrong colour.
        //
        // With BOTH corrections applied the halo matches the game almost exactly. So: do not
        // re-attempt this alone — land it together with a ×2 additive path for sprites.
        // See memory `dynchar-skadi-crown-fish`.
        //
        // ...but WHICH scale Unity applies is authored, and we were ignoring it.
        // `ParticleSystemScalingMode` (`scalingMode`): 0 Hierarchy — particles scale with the
        // whole transform chain, which is what the world basis below gives; 1 Local — the system
        // scales particles by its OWN transform only and IGNORES ancestors; 2 Shape — only the
        // shape module is scaled and particle SIZE is not.
        //
        // Reading the world basis unconditionally therefore shrinks (or inflates) every Local
        // system by whatever its ancestors contribute. Skadi the Corrupting Heart's crown shoal
        // is exactly that: `scalingMode = 1` with an ancestor at 0.5, so her fish exported at
        // half size and drew ~5x7px where the game draws 12-18px, leaving her halo effectively
        // invisible. Across the three reference skins 96 of 276 kept systems are Local WITH a
        // non-unit world basis, so this is a corpus-wide correction, not a one-skin patch.
        // MEASURED AND REJECTED as written (2026-08-01) — kept behind
        // `DYNCHAR_SCALING_MODE=1`, default OFF. Honouring the mode corpus-wide regresses every
        // reference skin: with the sprite additive boost also on, mly 30.816 -> 32.536,
        // cel 23.367 -> 24.139, ska 13.789 -> 13.994; size-only, mly -> 31.305, ska -> 13.945.
        //
        // The mechanism is real (Skadi's fish DO double to the game's size) but the application
        // is too broad: it changed 178 of 266 systems while only 96 are Local WITH a non-unit
        // world basis, so it also rewrites systems whose accumulated basis was already 1.0 and
        // which were therefore already correct. Narrow it to that population before retrying.
        // THE PER-SYSTEM DISCRIMINATOR, found and then shelved (2026-08-03). A BONE-FOLLOWING
        // emitter is the one case where the accumulated world basis is provably the wrong basis
        // for particle SIZE: its POSITION comes from the live spine bone at runtime (see the
        // `followBone` export below) and that bone already carries the skeleton's own scale, so
        // baking the same ancestor scale into the size applies it TWICE. A baked-position
        // emitter has no such split — `pos` and size share one basis and stay self-consistent —
        // which is why honouring Local semantics for them measured worse (see below).
        //
        // It is a GOOD discriminator: on the benchmarks it touches 3 of Skadi's 90 systems
        // (vs `narrow`'s 74), 0 of Mlynar's, and 23 of Virtuosa's — all wing rigs. It captures
        // 97% of `narrow`'s benefit for 10% of its cost: Skadi's crown fish go 1067 -> 2149 px
        // and 5.0x4.8 -> 6.7x6.7, for ska 10.505 -> 10.530 with mly/cel bit-identical.
        //
        // ENABLED on an apples-to-apples measurement. Scoring both sides with the IDENTICAL
        // absolute criterion (composite luma > 170 over a dark backdrop, crown box, blobs
        // >= 25 px) at Skadi's t=19 shows the shipped render produces **ZERO** visible birds
        // (43 scattered pixels) where the game has **12** (1727 px). This gate alone lifts that
        // to 5 birds / 301 px. Earlier comparisons that suggested the shoal was merely "small"
        // were measuring our faint halo through a DELTA threshold against the game's ABSOLUTE
        // one — not comparable; the shoal was effectively absent.
        //
        // Costs Skadi 10.505 -> 10.530 MADC and leaves Mlynar and Virtuosa bit-identical. That
        // pixel metric cannot reward this element anyway: the crown region's own noise floor is
        // 15.230 (game vs its own adjacent frame) and per-particle positions of sparse
        // randomly-emitted sprites are unmatchable by construction.
        let follows_bone = host.follower_of_go(all_objects, go_pid).is_some();
        let scaling_mode = if std::env::var("DYNCHAR_SCALING_MODE").is_ok() || follows_bone {
            i(ps, "scalingMode").unwrap_or(0)
        } else {
            0
        };
        // ANISOTROPIC emitter scale. `world_axis_mean` below collapses the emitter's world basis
        // to ONE scalar, which draws a mesh particle SQUARE even where the rig deliberately
        // stretches it. Civilight Eterna's seven background planes all carry a basis of exactly
        // |X| = 2·|Y|, so her 2:1 backdrop rendered as a square that stopped ~50 px short of the
        // aperture on each side — the empirical best-fit width correction was 1.333×, which is
        // precisely `|X| / mean` for a 2:1 basis.
        //
        // Emitted as a RATIO against the mean so `startSize` keeps its meaning and every
        // isotropic system is bit-identical. Mesh render mode only: a mesh particle is the
        // transform's own geometry, so the non-uniformity is unambiguous, whereas a billboard
        // faces the camera and its size semantics are not the transform's.
        //
        // ⚠️ Degenerate bases are COMMON (a collapsed axis is real — see the camera zero-scale
        // note) and must fall back to the mean, or the mesh would vanish entirely.
        let axis_x = f64::from(ex[0] - origin[0]).hypot(f64::from(ex[1] - origin[1]));
        let axis_y = f64::from(ey[0] - origin[0]).hypot(f64::from(ey[1] - origin[1]));
        let world_axis_mean = f64::midpoint(
            f64::from(ex[0] - origin[0]).hypot(f64::from(ex[1] - origin[1])),
            f64::from(ey[0] - origin[0]).hypot(f64::from(ey[1] - origin[1])),
        );
        // NARROWED variant (`DYNCHAR_SCALING_MODE=narrow`): only strip ANCESTOR contamination,
        // i.e. apply Local semantics solely where the system's OWN scale is already 1.0. That is
        // the conservative subset — it can only ever REMOVE a parent's contribution, never
        // introduce the system's own scale — and it is exactly the shape of Skadi's fish case
        // (own 1.0 under a 0.5 ancestor).
        // The `narrow` bound also guards the bone-follower path: it can only ever REMOVE an
        // ancestor's contribution, never introduce the system's own scale.
        let narrow =
            follows_bone || std::env::var("DYNCHAR_SCALING_MODE").is_ok_and(|v| v == "narrow");
        let wsx = match scaling_mode {
            // Local: the system's OWN local scale, ancestors excluded.
            1 => host.local_scale_pos_of_go(all_objects, go_pid).map_or(
                world_axis_mean,
                |(sc, _)| {
                    let own = f64::midpoint(f64::from(sc[0].abs()), f64::from(sc[1].abs()));
                    if narrow && (own - 1.0).abs() > 0.02 {
                        world_axis_mean
                    } else {
                        own
                    }
                },
            ),
            // Shape: particle size is not scaled by the transform at all.
            2 => 1.0,
            // Hierarchy (and anything unrecognised): the accumulated world basis.
            _ => world_axis_mean,
        };
        let em_inv = wsx * inv_scale;

        // Emission direction for CONE-family emitters whose local +X axis points along the
        // camera (out of the screen plane). `rot` (= `matrix_z_deg`, the screen angle of that
        // +X axis) is then numerically unstable noise, so the frontend's "local +Y rotated by
        // rot" cone direction fires these streaks in an arbitrary direction (e.g. Skadi2's
        // `xiaoyu`/`xiaoyu_r_lan` red streaks — authored to sweep INTO the entrance frame — get
        // aimed off-screen). Unity's cone emits along the emitter's local +Z; project that
        // through the FULL world matrix into the screen (Y-up px) frame and export it as
        // `emitDir` so the frontend can aim the cone faithfully. Gated to genuinely tilted
        // cones with a well-defined in-screen +Z: in-plane emitters (every currently-correct
        // skin's cones, where `rot` is meaningful) emit NO field and keep the byte-identical
        // `rot`-based path. Data-derived, no per-skin constant.
        let mut sys_emit_dir: Option<[f64; 2]> = None;
        let shape_type_int = ps
            .get("ShapeModule")
            .and_then(|s| i(s, "type"))
            .unwrap_or(-1);
        if matches!(shape_type_int, 4 | 7 | 8 | 9) {
            let ez = world.point([0.0, 0.0, 1.0]);
            let axis = |p: [f32; 3]| {
                [
                    f64::from(p[0] - origin[0]),
                    f64::from(p[1] - origin[1]),
                    f64::from(p[2] - origin[2]),
                ]
            };
            let xv = axis(ex);
            let zv = axis(ez);
            let x_len = xv[0].hypot(xv[1]).hypot(xv[2]);
            let x_scr = xv[0].hypot(xv[1]);
            let z_len = zv[0].hypot(zv[1]).hypot(zv[2]);
            let z_scr = zv[0].hypot(zv[1]);
            if x_len > 1e-6 && z_len > 1e-6 && x_scr / x_len < 0.5 && z_scr / z_len > 0.7 {
                sys_emit_dir = Some([zv[0] / z_scr, zv[1] / z_scr]);
            }
        }

        // ---- Build the reduced system JSON ------------------------------
        let mut sys = json!({
            // The OWNING GameObject's name. `ps` is the ParticleSystem COMPONENT, and Unity
            // components carry no `m_Name` — reading it there yielded "" for every system in
            // the corpus, which is why particle diagnostics could only ever refer to a system
            // by its index and no probe output could be tied back to an authored object.
            "name": host.go_name(all_objects, go_pid),
            "sort": sort,
            "blend": if blend { "additive" } else { "normal" },
            // The material's ×2 `_TintColor`, or null at the neutral — see `particle_tint`.
            "tint": tint.map_or(Value::Null, |t| json!(t)),
            "abTint": ab_tint.map_or(Value::Null, |t| json!(t)),

            "renderMode": render_mode,
            "pos": pos,
            "rot": rot,
            // DEPTH RELATIVE TO THE SKELETON ROOT (2026-09-06), in Unity units, positive
            // when the emitter sits FARTHER from the camera than the character (the L2D
            // camera looks down +z, so a larger z is farther). Unity orders transparent
            // renderers that share a sorting order back to front by that distance, and 271
            // systems in the corpus share the character's sorting order exactly (Pozemka 71,
            // Surtr summer#9 34, Texas epoque 28, Ch'en 22, Chongyue cfa#1's `vein` grain over
            // his face); the viewer had put every tie in front. Null when the system has no
            // root scope to measure against.
            "zRel": scope.own.map_or(Value::Null, |own| {
                let own_z = host.world_of_go(all_objects, own).point([0.0, 0.0, 0.0])[2];
                json!(f64::from(origin[2]) - f64::from(own_z))
            }),
            "duration": fd(ps, "lengthInSec", 1.0),
            "looping": b(ps, "looping", false),
            // `moveWithTransform` stores the `ParticleSystemSimulationSpace` enum, whose
            // members are Local = 0 and World = 1 — NOT a "does it move with the transform"
            // boolean, despite the name. Reading it as one inverted every system's
            // simulation space; it stayed invisible because almost every emitter is
            // static, where the two are indistinguishable.
            "simulationSpace": if i(ps, "moveWithTransform").unwrap_or(0) == 1 { "world" } else { "local" },
            "maxParticles": i(&initial, "maxNumParticles").unwrap_or(1000),
            "boneChain": bone_chain,
        });
        // ENTRANCE material-colour animation, `[t, r, g, b, a]` — see `particle_color_curve`.
        // Inserted only when present so the default export stays byte-identical.
        if let Some(cc) = &color_curve {
            sys["colorCurve"] = json!(
                cc.iter()
                    .map(|&(t, c)| [t, c[0], c[1], c[2], c[3]])
                    .collect::<Vec<_>>()
            );
        }
        // Cinematic start delay (omitted when 0 to keep always-on scenes lean).
        if delay > 0.0 {
            sys["delay"] = json!(delay);
        }
        // MAIN-MODULE CLOCK. Unity's `simulationSpeed` scales the system's ENTIRE clock —
        // emission interval, particle age, every over-lifetime curve, rotation and velocity all
        // advance at this multiple of real time — and `prewarm` pre-simulates one full
        // `duration` at t=0 so a looping system opens in steady state instead of building up
        // from empty. Neither was ever read, so a system authored at 0.3x ran 3.3x fast and a
        // prewarmed one opened empty. Corpus: 1700 systems below 1x, 403 above, 3057 prewarmed
        // (963 both) of 13069, across 80 of 83 bundles.
        //
        // Both are omitted at their Unity defaults so an unaffected system's JSON is unchanged.
        let sim_speed = fd(ps, "simulationSpeed", 1.0);
        if (sim_speed - 1.0).abs() > 1e-3 && sim_speed > 0.0 {
            sys["simSpeed"] = json!(sim_speed);
        }
        if b(ps, "prewarm", false) {
            sys["prewarm"] = json!(true);
        }
        // Faithful stretched-billboard elongation scales (see above) — only for stretch systems.
        if render_mode == "stretch" {
            sys["stretch"] = json!({
                "lengthScale": stretch_len_scale,
                "velocityScale": stretch_vel_scale,
                "cameraVelocityScale": stretch_cam_vel_scale,
            });
        }
        // `ParticleSystemRenderer.pivot` — the quad's pivot point, as a MULTIPLIER of the
        // particle size. Unity keeps the pivot at the particle's position and rotates the
        // quad about it, so a non-zero pivot both OFFSETS the sprite and moves its rotation
        // centre. 1202 of 13069 renderers in this corpus set one (values up to 1.0), which
        // at a 64 px particle is a 64 px displacement — so ignoring it silently misplaced
        // ~9% of the corpus's systems. Z is dropped: it only separates depth, which a 2D
        // renderer has no use for. Omitted when zero so every unaffected system stays
        // byte-identical.
        if let Some(pv) = renderer.and_then(|r| r.get("m_Pivot")) {
            let px = fd(pv, "x", 0.0);
            let py = fd(pv, "y", 0.0);
            if px.abs() > 1e-6 || py.abs() > 1e-6 {
                sys["pivot"] = json!([px, py]);
            }
        }
        // Faithful cone emission direction for camera-facing (tilted) emitters (see above).
        if let Some(ed) = sys_emit_dir {
            sys["emitDir"] = json!(ed);
        }

        // spine-unity `BoneFollower` in the ancestry: the followed rig's serialized
        // transform is only an editor pose — at runtime the follower SNAPS that GO
        // onto the named SPINE BONE, so the emitter's true position is
        // `bone(t) + followOffset` (the emitter's offset within the rig). Virtuosa's
        // entrance apple/comet rig (`start_apple_01(Clone)` → `L_C_Apple_F`) rides a
        // bone the `Start` animation plunges down the shaft — without this the whole
        // rig exports at its editor pose and the falling apple never appears. The
        // baked `pos` is kept as-is; the frontend rebases when the bone exists.
        if let Some((bone, follow_rot, follower_go)) = host.follower_of_go(all_objects, go_pid) {
            let fw = host.world_of_go(all_objects, follower_go);
            let fo = fw.point([0.0, 0.0, 0.0]);
            sys["followBone"] = json!(bone);
            sys["followBoneRot"] = json!(follow_rot);
            sys["followOffset"] = json!([
                (f64::from(origin[0]) - f64::from(fo[0])) * inv_scale,
                (f64::from(origin[1]) - f64::from(fo[1])) * inv_scale,
            ]);
        }

        // Entrance-clip Transform SCALE/position on an effect-host ancestor (Virtuosa's
        // crown `ctrl`): a big golden halo shrinks (scale 1.0→0.28) into the small resting
        // crown over 9.4–12.43s. The baked pose captures only the resting transform, so the
        // shrink-in is emitted as replayable curves the frontend applies to the emitter
        // container. Fully generic — any skin whose `_Start` clip scales an effect host
        // gains its scale-in, keyed on data alone.
        if let Some(ctrl) = transform_host
            && let Some(et) = entrance_transform_curves.get(&ctrl)
            && let Some((scale0, pos0)) = host.local_scale_pos_of_go(all_objects, ctrl)
        {
            // The ctrl's world ORIGIN — the fixed point the scale pivots about (px, Y-up).
            let ctrl_world = host.world_of_go(all_objects, ctrl);
            let pivot_w = ctrl_world.point([0.0, 0.0, 0.0]);
            let pivot_px = [
                f64::from(pivot_w[0]) * inv_scale,
                f64::from(pivot_w[1]) * inv_scale,
            ];
            // scaleCurve: the animated factor / the baked (resting) factor → a multiplier
            // of the emitter's already-baked size, 1.0 at the resting pose.
            let s0 = f64::from(scale0[0]);
            if s0.abs() > 1e-4 {
                let sc: Vec<Value> = et
                    .scale
                    .iter()
                    .map(|&(t, v)| json!({ "t": t, "v": f64::from(v) / s0 }))
                    .collect();
                sys["scaleCurve"] = json!(sc);
                sys["scalePivot"] = json!(pivot_px);
            }
            // posCurve: the ctrl's animated LOCAL position, projected through its PARENT
            // world matrix into a px OFFSET of the pivot from its resting position (Y-up).
            if !et.pos_x.is_empty()
                && let Some(parent_w) = host.parent_world_of_go(all_objects, ctrl)
            {
                let base = parent_w.point(pos0);
                let mut times: Vec<f32> = et
                    .pos_x
                    .iter()
                    .map(|&(t, _)| t)
                    .chain(et.pos_y.iter().map(|&(t, _)| t))
                    .collect();
                times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                times.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
                let sample = |c: &[(f32, f32)], t: f32, fb: f32| -> f32 {
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
                let pc: Vec<Value> = times
                    .iter()
                    .map(|&t| {
                        let lp = [
                            sample(&et.pos_x, t, pos0[0]),
                            sample(&et.pos_y, t, pos0[1]),
                            pos0[2],
                        ];
                        let w = parent_w.point(lp);
                        json!({
                            "t": t,
                            "x": f64::from(w[0] - base[0]) * inv_scale,
                            "y": f64::from(w[1] - base[1]) * inv_scale,
                        })
                    })
                    .collect();
                sys["posCurve"] = json!(pc);
            }
        }

        // _MainTex UV tiling/offset [scaleX,scaleY,offsetX,offsetY]. Emitted only
        // when non-identity so the frontend crops the billboard to the atlas cell
        // the material selects (else it draws the whole flipbook atlas as one sprite).
        if (main_st[0] - 1.0).abs() > 1e-4
            || (main_st[1] - 1.0).abs() > 1e-4
            || main_st[2].abs() > 1e-4
            || main_st[3].abs() > 1e-4
        {
            sys["mainST"] = json!(main_st);
        }

        // gravityModifier: dimensionless multiplier of Unity gravity. Emitted as
        // an MMScalar ×inv_scale (px), so the frontend applies accel =
        // value * 9.81 (px/s²) downward. See report notes on units.
        if let Some(g) = initial.get("gravityModifier") {
            sys["gravity"] = mmscalar(g, inv_scale);
        }
        if let Some(l) = initial.get("startLifetime") {
            sys["lifetime"] = mmscalar(l, 1.0);
        }
        if let Some(s) = initial.get("startSpeed") {
            sys["startSpeed"] = mmscalar(s, em_inv);
        }
        // ANISOTROPIC emitter basis → per-axis particle size, folded into the SAME `startSizeY`
        // channel Unity's `size3D` uses (see below). `world_axis_mean` collapsed the basis to one
        // scalar, drawing a mesh particle square wherever the rig deliberately stretched it.
        // Mesh render mode only, and only when BOTH axes are non-degenerate — a collapsed axis is
        // real and must keep the mean, or the mesh would vanish.
        let aniso = std::env::var("DYNCHAR_PS_ANISO").as_deref() != Ok("0")
            && render_mode == "mesh"
            && axis_x > 1e-6
            && axis_y > 1e-6
            && (axis_x - axis_y).abs() > 5e-3 * axis_x.max(axis_y);
        let (fx, fy) = if aniso {
            let mean = f64::midpoint(axis_x, axis_y);
            ((axis_x / mean) as f32, (axis_y / mean) as f32)
        } else {
            (1.0f32, 1.0f32)
        };
        if let Some(s) = initial.get("startSize") {
            sys["startSize"] = mmscalar(s, em_inv * f64::from(fx));
            // Unity `size3D`: the particle quad is a RECTANGLE, its height authored
            // separately in `startSizeY`. Exporting only `startSize` renders such a
            // system SQUARE — Mlynar's 14 `fangkuai_*` city slabs are 0.5 × 2.0 units,
            // a 1:4 tall bar that comes out 4× too short. Emitted only when the two
            // axes genuinely differ, so every uniform system stays byte-identical.
            if b(&initial, "size3D", false)
                && let Some(sy) = initial.get("startSizeY")
            {
                let size_y = mmscalar(sy, em_inv * f64::from(fy));
                if size_y != sys["startSize"] {
                    sys["startSizeY"] = size_y;
                }
            } else if aniso {
                // No authored `size3D`: the Y size is the X size re-scaled by the basis ratio.
                sys["startSizeY"] = mmscalar(s, em_inv * f64::from(fy));
            }
        }
        // startRotation is authored in radians → degrees.
        if let Some(r) = initial.get("startRotation") {
            sys["startRotation"] = mmscalar(r, RAD_TO_DEG);
        }
        if let Some(c) = initial.get("startColor") {
            sys["startColor"] = mmgradient(c);
        }

        // Mesh-render-mode systems emit a shared MESH per particle (ice-crystal
        // shards, ribbons, blades) instead of a camera-facing billboard. Export the
        // mesh's LOCAL-space triangle geometry (Z dropped — dynchar fx meshes are
        // flat cards) so the frontend can instance it per particle, scaled by the
        // particle's size: `finalVertPx = localVert * exportedStartSize` (startSize
        // is already emitter-scaled to px, and Unity scales the mesh by particle
        // size, so the raw local geometry needs no extra scale here). `resources`
        // carries the bundle's .resS entries, so a mesh whose vertex buffer is
        // EXTERNAL (`m_StreamData`) resolves here too; a compressed mesh still
        // yields None → the system stays skipped by the frontend, exactly as
        // before. Bounded to guard against a pathological mesh.
        // DIAGNOSTIC (`DYNCHAR_PSMAT`): why a mesh-mode system carries no geometry. Prints the
        // mesh reference, whether the object is in this bundle, its class, whether it is
        // compressed, its vertex count and its stream-data path, and what the parser returned.
        if render_mode == "mesh" && std::env::var("DYNCHAR_PSMAT").is_ok() {
            let mref = renderer.and_then(|r| r.get("m_Mesh"));
            let pid = mref.and_then(get_path_id);
            let file_id = mref.and_then(|m| m.get("m_FileID")).and_then(Value::as_i64);
            let obj = pid.and_then(|p| all_objects.get(&p));
            let (cls, val) = obj
                .map(|(c, v)| (Some(*c), Some(v)))
                .unwrap_or((None, None));
            let compressed = val
                .and_then(|v| v.get("m_CompressedMesh"))
                .and_then(|c| c.get("m_Vertices"))
                .and_then(|x| x.get("m_NumItems"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let vcount = val
                .and_then(|v| v.get("m_VertexData"))
                .and_then(|d| d.get("m_VertexCount"))
                .and_then(Value::as_i64)
                .unwrap_or(-1);
            let stream = val
                .and_then(|v| v.get("m_StreamData"))
                .and_then(|s| s.get("path"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let parsed = val.and_then(|v| super::mesh::parse_mesh(v, resources));
            eprintln!(
                "PSMESH '{}' pid={pid:?} fileId={file_id:?} class={cls:?} compressedVerts={compressed} vertexCount={vcount} stream='{stream}' parsed={}",
                host.go_name(all_objects, go_pid),
                parsed
                    .as_ref()
                    .map(|m| format!("{}v/{}i", m.positions.len(), m.indices.len()))
                    .unwrap_or_else(|| "NONE".into())
            );
        }
        // UNITY'S BUILT-IN QUAD (2026-09-05). A mesh-render system whose `m_Mesh` points OUT of
        // the bundle (`m_FileID` != 0) at path id 10210 references Unity's built-in `Quad`
        // primitive from `unity default resources`, the 1x1 square the engine ships. Nothing in
        // the bundle answers that id, so the geometry came back `None` and the viewer dropped
        // the system: 37 of Kal'tsit sale#14's 40 mesh-mode systems (stones, birds, glass, light
        // flows, background rings), 4 more across chen2_2 / svash2_2 / bgsnow, per the
        // `DYNCHAR_PSMAT` mesh census. The quad IS the geometry, so emit it.
        // `DYNCHAR_BUILTIN_QUAD=0` restores the old export (those systems carry no `mesh`).
        //
        // ONLY FOR A MATERIAL THE EXPORT CAN CARRY (2026-09-06). The quad turned 176 systems
        // from "not drawn" into "drawn", and for 53 of them the plain sprite path draws the
        // WRONG thing: Kal'tsit's 17 `stone_flow` / `glass01` and Vina Victoria's 24 are
        // `Particles-L2D/Dissolve/Dissolve Add UVTween`, whose caustic `_MainTex` is shaped by a
        // dissolve mask the plain export has no field for, so the card showed the raw 128 px
        // caustic as a white blob at 300..1300 px over her lower body (the register, "the
        // particles feel a bit oppressive"). The same holds for `RGBsplit` (Nian's 10), the
        // single-map `Dissolve(CustomData)` (1) and a material with no main texture (1). So the
        // quad is emitted only when the material is one the export represents: the plain
        // `Additive` / `AlphaBlend` programs, or a compositor that resolved to a `ram` block
        // (`Ram/`, plain `Disturb/`, the live two-map `Dissolve/`). The rest stay as they were,
        // undrawn, until their program is ported. Derived from the material's shader name in
        // the bundle; `DYNCHAR_BUILTIN_QUAD=all` emits the quad for every material.
        const UNITY_BUILTIN_QUAD_PATH_ID: i64 = 10210;
        let builtin_quad_env = std::env::var("DYNCHAR_BUILTIN_QUAD").unwrap_or_default();
        let builtin_quad_on = builtin_quad_env != "0";
        let plain_program = renderer
            .and_then(|r| r.get("m_Materials"))
            .and_then(Value::as_array)
            .and_then(|mats| {
                mats.iter().find_map(|mat_ref| {
                    let pid = get_path_id(mat_ref).filter(|&p| p != 0)?;
                    let (21, m) = all_objects.get(&pid)? else {
                        return None;
                    };
                    m.get("_shaderName").and_then(Value::as_str)
                })
            })
            .is_some_and(|s| {
                s.ends_with("/Particles-L2D/Additive")
                    || s.ends_with("/Particles-L2D/AlphaBlend")
                    || s.ends_with("/Particles/Additive")
                    || s.ends_with("/Particles/AlphaBlend")
                    // `Particles/RGBsplit`, from its decompiled program (pathID
                    // -5816859062873505013): `_MainTex` sampled three times at per-channel UV
                    // offsets (`_OffsetR/G/B` scaled by a time-jittered `_SplitIntensity`),
                    // `rgb = main * 2 * (vertexColour * _TintColor)`, alpha the tint's. With the
                    // split at zero it IS the plain sprite, and the split is a few px of
                    // chromatic jitter on a poster, so the plain path carries the image and
                    // omits the jitter. Nian cfa#1's twelve `02` stage screens (269..845 px)
                    // are this program and were her whole painted backdrop.
                    || s.ends_with("/Particles/RGBsplit")
            });
        let quad_material_ok = builtin_quad_env == "all" || plain_program || ram.is_some();
        // DISPLAYED TEXTURE (2026-09-06). On a plain program (`Additive`, `AlphaBlend`,
        // `RGBsplit`) `_MainTex` is the image the system shows; it is never a flow or
        // distortion input, which only the compositor families (`Ram/`, `Disturb/`,
        // `Dissolve/`) consume in that role. The viewer's flow-map and desaturated-panel
        // rules exist to keep such inputs from being stamped as sprites, and on a plain
        // program they can only ever discard a picture: Nian cfa#1's ten `02` stage screens
        // (posters, low saturation by nature), Wang's `Left BG 01`, Vina Victoria's
        // `bg_building`, Rosmon's `baoqi`. The flag lets the viewer scope those rules to the
        // compositor programs. Emitted only when true, so every other system's JSON is
        // byte-identical.
        if plain_program {
            sys["displayTex"] = json!(true);
        }
        let mesh_ref = renderer.and_then(|r| r.get("m_Mesh"));
        let mesh_is_builtin_quad = builtin_quad_on
            && quad_material_ok
            && mesh_ref
                .and_then(|m| m.get("m_FileID"))
                .and_then(Value::as_i64)
                .is_some_and(|f| f != 0)
            && mesh_ref.and_then(get_path_id) == Some(UNITY_BUILTIN_QUAD_PATH_ID);
        let mesh_data: Option<super::mesh::MeshData> = if render_mode != "mesh" {
            None
        } else if mesh_is_builtin_quad {
            Some(super::mesh::unit_quad())
        } else if let Some(mesh_pid) = mesh_ref.and_then(get_path_id)
            && mesh_pid != 0
            && let Some((43, mesh_val)) = all_objects.get(&mesh_pid).map(|(c, v)| (*c, v))
        {
            super::mesh::parse_mesh(mesh_val, resources)
        } else {
            None
        };
        if let Some(m) = mesh_data
            && !m.indices.is_empty()
            && m.positions.len() <= 8192
        {
            let mut mpos = Vec::with_capacity(m.positions.len() * 2);
            for p in &m.positions {
                mpos.push(f64::from(p[0]));
                mpos.push(f64::from(p[1]));
            }
            let mut muv = Vec::with_capacity(m.uvs.len() * 2);
            for uv in &m.uvs {
                muv.push(f64::from(uv[0]));
                muv.push(f64::from(uv[1]));
            }
            let mut mesh_json = json!({ "pos": mpos, "uv": muv, "idx": m.indices });
            // Per-vertex colour only when it carries real (non-white) modulation
            // (fx meshes bake edge falloff / opacity here). Omitted when all white.
            if m.colors
                .iter()
                .any(|c| c[0] < 0.999 || c[1] < 0.999 || c[2] < 0.999 || c[3] < 0.999)
            {
                let mut mcol = Vec::with_capacity(m.colors.len() * 4);
                for c in &m.colors {
                    mcol.extend([
                        f64::from(c[0]),
                        f64::from(c[1]),
                        f64::from(c[2]),
                        f64::from(c[3]),
                    ]);
                }
                mesh_json["col"] = json!(mcol);
            }
            sys["mesh"] = mesh_json;
        }

        // Emission (rate + bursts). A MAIN-scene emitter whose rate is driven by
        // transition/one-shot state clips (`event_rate_gos`) is quiet at the steady
        // idle — export rate 0 so the flourish-peak constant doesn't rain forever.
        let event_quiet = event_rate_gos.contains(&go_pid);
        let rate = if event_quiet {
            json!({ "mode": "const", "v": 0.0 })
        } else {
            emission.get("rateOverTime").map_or_else(
                || json!({ "mode": "const", "v": 0.0 }),
                |r| mmscalar(r, 1.0),
            )
        };
        let bursts: Vec<Value> = bursts_raw
            .iter()
            .map(|bu| {
                let count = bu
                    .get("countCurve")
                    .map_or(0.0, |c| mmscalar_repr(c, 1.0))
                    .round() as i64;
                json!({ "t": fd(bu, "time", 0.0), "count": count })
            })
            .collect();
        sys["emission"] = json!({
            "rate": rate,
            "bursts": bursts,
            "enabled": emission_enabled,
        });
        // `rateOverDistance` (particles per world-unit of EMITTER travel): the trail
        // mechanic of rigs that ride a moving anchor — Virtuosa's falling-apple comet
        // (`spark_small`/`sand_ab_01`, rate-over-TIME 0) sheds gold dust only while
        // its `L_C_Apple_F` bone plunges. Exported per-px (÷ inv_scale, the frontend
        // measures emitter movement in export px). Omitted when zero everywhere.
        if let Some(rod) = emission.get("rateOverDistance")
            && !mmscalar_is_zero(rod)
        {
            sys["rateOverDistance"] = mmscalar(rod, 1.0 / inv_scale);
        }
        // Per-particle DISSOLVE-amount curve over normalized lifetime, from the
        // `CustomDataModule`'s first Vector-mode stream (component 0 — the value the
        // Ram shader reads as vs_TEXCOORD2.x and adds to `_Amount`). Virtuosa's
        // entrance apples crumble away with it (−0.12 → 1.0 by ~30% of their 10s
        // life); without it they'd ride the falling rig forever. Consumed by the
        // frontend's RamEmitter only (plain billboards have no dissolve stage).
        //
        // ⚠️ Read the stream the RENDERER ACTUALLY SENDS, not "the first Vector-mode stream".
        // Unity only uploads the custom streams listed in `ParticleSystemRenderer.m_VertexStreams`
        // (`ParticleSystemVertexStream`: 30..=33 are Custom1X..XYZW, 34..=37 Custom2X..XYZW), and a
        // CustomData stream configured in the module but absent from that list never reaches the
        // shader at all — the threshold it would have supplied is simply 0.
        //
        // Civilight Eterna is the case that found this. All 32 of her CustomData systems send
        // Custom2 and nothing else; 29 of them put their Vector data in stream 1, which matches.
        // But `guangyun01` (her end-of-cinematic lens flare) and the two `down_vase_lizi_*` author
        // theirs in stream 0, which is NEVER sent. Taking the first Vector-mode stream therefore
        // exported a 0.392 -> 0.465 dissolve threshold for a flare the game dissolves by 0, and
        // `dAlpha = saturate(dis - thr)` rendered it at ~20% of its intended brightness.
        // Default ON since the overnight A/B of 2026-08-31: it rewrites 15 systems in 3 JSONs
        // (Wis'adel's `disturb_p_*` and `name_rgb`, whose Custom1.z moves from a dissolve-UV
        // scroll to the `_Amount` offset) and wis, whitw2, chyue and cel score bit-identical, so
        // the corrected table costs nothing measurable and the data-derived one ships.
        let vstream_uv2 = std::env::var("DYNCHAR_VSTREAM_UV2").as_deref() != Ok("0");
        // ⚠️ This table (Custom1 30..=33, Custom2 34..=37) is OFF BY ONE against the payload
        // table below and against the data; it lands on the right stream for `[34,38]` and
        // `[34,36]` only because 34 falls in its Custom2 range. Under `DYNCHAR_VSTREAM_UV2=1`
        // the `_Amount` source is taken from payload slot 5 instead and this is unused.
        let sent_stream = renderer
            .and_then(|r| r.get("m_VertexStreams"))
            .and_then(Value::as_array)
            .and_then(|arr| {
                let ids: Vec<i64> = arr.iter().filter_map(Value::as_i64).collect();
                if ids.iter().any(|&x| (30..=33).contains(&x)) {
                    Some(0usize)
                } else if ids.iter().any(|&x| (34..=37).contains(&x)) {
                    Some(1usize)
                } else {
                    None
                }
            });
        // PER-PARTICLE DISSOLVE-UV OFFSET over normalized lifetime.
        //
        // The custom floats are packed, in `m_VertexStreams` order, into the texcoord payload
        // that follows the base UV, and the Ram vertex program consumes them positionally:
        //
        //     payload 1,2  -> in_TEXCOORD0.zw : added to the MAIN uv
        //     payload 3,4  -> in_TEXCOORD1.xy : added to the DISSOLVE uv   <- this block
        //     payload 5    -> in_TEXCOORD1.z  : + _Amount, the dissolve THRESHOLD (below)
        //     payload 6    -> in_TEXCOORD1.w  : disturb intensity
        //     payload 7,8  -> in_TEXCOORD2.xy : added to the DISTURB uv
        //
        // read straight out of `Ram/Disturb(CustomData)`:
        //     u_xlat16_2.xy = dissolveUV + in_TEXCOORD1.xy;
        //     vs_TEXCOORD2.xy = in_TEXCOORD1.zw;
        //
        // Wiš'adel's `stroke_01 (1)` is the case that found this. Her ending wipe is ONE quad
        // whose main texture is fully opaque (alpha 255 over every texel) and dark, so nothing
        // but the dissolve can carve it — and her only CustomData curve, Custom1.z ramping
        // 1.0 -> -0.4511 over an 0.8 s life, lands on payload 3: it SCROLLS the dissolve lookup
        // across the gradient mask, sweeping the stroke in. Without it the quad draws at its
        // static `_Amount` from the instant it spawns, stamping a flat grey veil over the whole
        // frame 0.25 s before the game darkens at all (t=12 MADC 41.7 against 6.4 either side).
        //
        // ⚠️ Positional, NOT "the first Vector stream": Civilight Eterna's `guangyun01` has the
        // identical stream list and authors its curve on Custom1.x — payload 1, the MAIN uv —
        // so it must NOT become a dissolve scroll. Deriving the position from the stream list
        // keeps both skins right and needs no per-skin rule.
        let payload: Vec<(usize, usize)> = renderer
            .and_then(|r| r.get("m_VertexStreams"))
            .and_then(Value::as_array)
            .map(|arr| {
                let mut out = Vec::new();
                for id in arr.iter().filter_map(Value::as_i64) {
                    // Custom1X..Custom1XYZW = 31..=34, Custom2X..Custom2XYZW = 35..=38.
                    //
                    // Census 2026-08-30 over all 87 bundles settles the enumeration from the
                    // data: `[5,34,35]`, `[34,36]` and `[34,37]` only parse as UV2 + C1.xyzw +
                    // C2.x, C1.xyzw + C2.xy and C1.xyzw + C2.xyz under THIS table.
                    //
                    // `DYNCHAR_VSTREAM_UV2=1`: UV2/UV3/UV4 (ids 5..=7) are 2-float streams that
                    // Unity packs into the texcoord payload BEFORE the customs (UV2 lands on
                    // TEXCOORD0.zw), so on a `[5,34]` list Custom1.z is payload 5, the `_Amount`
                    // offset, and Custom1.w the disturb intensity. Skipping them shifts every
                    // custom two slots early. 126 systems across 13 skins list one (wis 21,
                    // whitw2 13, chyue 3, cel 2 in the corpus). `DYNCHAR_VSTREAM_UV2=0` reverts.
                    let (slot, n) = match id {
                        31..=34 => (0usize, (id - 30) as usize),
                        35..=38 => (1usize, (id - 34) as usize),
                        5..=7 if vstream_uv2 => (usize::MAX, 2usize),
                        _ => continue,
                    };
                    for c in 0..n {
                        out.push((slot, c));
                    }
                }
                out
            })
            .unwrap_or_default();
        if let Some(cdm) = ps.get("CustomDataModule")
            && b(cdm, "enabled", false)
            && payload.len() >= 4
        {
            // Each axis is sampled independently, then both are interpolated onto the UNION
            // of their time bases — the two curves need not share keyframes, and in practice
            // only one axis is usually authored (the other stays 0).
            // One extractor for every payload PAIR. The positions come from the table above,
            // so adding an axis is a one-line call rather than another bespoke block.
            let axis_curve = |idx: usize| -> Vec<(f64, f64)> {
                let Some(&(slot, comp)) = payload.get(idx) else {
                    return Vec::new();
                };
                if i(cdm, &format!("mode{slot}")).unwrap_or(0) != 1 {
                    return Vec::new();
                }
                let Some(vc) = cdm.get(format!("vector{slot}_{comp}").as_str()) else {
                    return Vec::new();
                };
                let Some(mc) = vc.get("maxCurve") else {
                    return Vec::new();
                };
                let scalar = fd(vc, "scalar", 1.0);
                let pts = sample_curve(mc, if scalar == 0.0 { 1.0 } else { scalar }, 1.0);
                if pts.len() > 1 {
                    pts.iter()
                        .map(|p| (fd(p, "t", 0.0), fd(p, "v", 0.0)))
                        .collect()
                } else {
                    Vec::new()
                }
            };
            // Interpolate a pair of axes onto the UNION of their time bases — the two curves
            // need not share keyframes, and usually only one axis is authored.
            let pair = |a: &Vec<(f64, f64)>, b2: &Vec<(f64, f64)>| -> Option<Vec<Vec<f64>>> {
                if a.is_empty() && b2.is_empty() {
                    return None;
                }
                let mut times: Vec<f64> = a.iter().chain(b2.iter()).map(|&(t, _)| t).collect();
                times.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
                times.dedup_by(|x, y| (*x - *y).abs() < 1e-6);
                let at = |c: &Vec<(f64, f64)>, t: f64| -> f64 {
                    if c.is_empty() {
                        return 0.0;
                    }
                    if t <= c[0].0 {
                        return c[0].1;
                    }
                    for w in c.windows(2) {
                        if t <= w[1].0 {
                            let span = w[1].0 - w[0].0;
                            let f = if span > 1e-9 {
                                (t - w[0].0) / span
                            } else {
                                0.0
                            };
                            return w[0].1 + (w[1].1 - w[0].1) * f;
                        }
                    }
                    c[c.len() - 1].1
                };
                let out: Vec<Vec<f64>> = times
                    .iter()
                    .map(|&t| vec![t, at(a, t), at(b2, t)])
                    .collect();
                (out.len() > 1).then_some(out)
            };
            // payload 1,2 -> MAIN uv; 3,4 -> DISSOLVE uv; 7,8 -> DISTURB uv (1-indexed above).
            if let Some(v) = pair(&axis_curve(0), &axis_curve(1)) {
                sys["ramMainUVCurve"] = json!(v);
            }
            if let Some(v) = pair(&axis_curve(2), &axis_curve(3)) {
                sys["ramDissolveUVCurve"] = json!(v);
            }
            if let Some(v) = pair(&axis_curve(6), &axis_curve(7)) {
                sys["ramDisturbUVCurve"] = json!(v);
            }
            // payload 6 -> the per-particle DISTURB INTENSITY the frontend currently pins at 0.
            {
                let di = axis_curve(5);
                if !di.is_empty() {
                    sys["ramDisturbIntensityCurve"] =
                        json!(di.iter().map(|&(t, v)| vec![t, v]).collect::<Vec<_>>());
                }
            }
        }
        if let Some(cdm) = ps.get("CustomDataModule")
            && b(cdm, "enabled", false)
        {
            // `sent_stream` is an `Option`, so this loop runs 0 or 1 times by construction; kept
            // as a `for` (rather than `if let`/`while let`) because the body's `continue`/`break`
            // rely on an actual loop construct.
            // The `_Amount` offset is payload slot 5 (`vs_TEXCOORD2.x = in_TEXCOORD1.z`).
            let amount_src: Option<(usize, usize)> = if vstream_uv2 {
                payload
                    .get(4)
                    .copied()
                    .filter(|&(slot, _)| slot != usize::MAX)
            } else {
                sent_stream.map(|s| (s, 0usize))
            };
            #[allow(for_loops_over_fallibles)]
            for (stream, comp) in amount_src {
                if i(cdm, &format!("mode{stream}")).unwrap_or(0) != 1 {
                    continue;
                }
                if let Some(v0) = cdm.get(format!("vector{stream}_{comp}").as_str())
                    && let Some(mc) = v0.get("maxCurve")
                {
                    let scalar = fd(v0, "scalar", 1.0);
                    let mut pts = sample_curve(mc, if scalar == 0.0 { 1.0 } else { scalar }, 1.0);
                    // CLAMP TO THE DECLARED RANGE (`DYNCHAR_AMOUNT_CLAMP=0` reverts).
                    //
                    // This curve drives `_Amount`, the dissolve threshold, which every one of the
                    // 26 staged shaders that declares it declares as ShaderLab `Range(min, max)`
                    // (`m_Type` 3). Unity CLAMPS a Range property to those limits, so a curve
                    // running outside them is clamped in the game and is not in ours.
                    //
                    // It is not a rounding detail. The mask is
                    // `clamp((tex(_DissolveTex) - _Amount) / _BorderWidth, 0, 1)`, so a NEGATIVE
                    // `_Amount` stops the carving entirely and the quad paints its full rectangle.
                    // Wiš'adel's `sx (1)` is exactly that: its curve runs 1.0 -> 0 -> -0.018, and
                    // isolated the sheet ends a flat dark 35.09 over 100% of the frame.
                    //
                    // Corpus census: of 511 exported `ramDissolveCurve`s, **266 dip below zero**
                    // and 4 exceed one, across 19 skins; whitw2 alone carries 63 of them.
                    //
                    // Only the LOWER bound is applied, and it is read from the shader rather than
                    // chosen: all 26 declare min 0.0, while the max is 1.0 on 25 and 1.1 on one,
                    // so the upper limit is not uniform and touches only 4 curves. `shader_range`
                    // returns `None` when no shader bundle is staged, and then nothing is clamped.
                    //
                    // MEASURED: whitw2 33.432 -> 33.160 with r 0.717 -> 0.719, both signs agreeing,
                    // and the other twelve corpus keys BIT-IDENTICAL (cel and mue have 3 and 1
                    // clamped systems but do not move). It reaches 245 systems across 16 skins.
                    if std::env::var("DYNCHAR_AMOUNT_CLAMP").as_deref() != Ok("0")
                        && let Some(lo) = renderer
                            .and_then(|r| r.get("m_Materials"))
                            .and_then(Value::as_array)
                            .and_then(|a| a.first())
                            .and_then(get_path_id)
                            .and_then(|pid| all_objects.get(&pid))
                            .and_then(|(_, m)| m.get("_shaderName"))
                            .and_then(Value::as_str)
                            .and_then(|sh| super::shader_map::shader_range(sh, "_Amount"))
                            .map(|(lo, _)| lo)
                    {
                        for pt in &mut pts {
                            if pt
                                .get("v")
                                .and_then(Value::as_f64)
                                .is_some_and(|v| v < f64::from(lo))
                            {
                                pt["v"] = json!(lo);
                            }
                        }
                    }
                    if pts.len() > 1 {
                        sys["ramDissolveCurve"] = json!(pts);
                    }
                }
                break;
            }
        }
        // The `_Start` cinematic can drive an emitter's rate DIRECTLY as an animation
        // curve (`EmissionModule.rateOverTime.scalar` bindings) — Mlynar's sword clips
        // gate the confetti/star systems this way (serialized rates of 60–150/s that
        // the clips hold at 0 until the ~7–13s flourish). Without the curve those
        // systems emit at full serialized rate from t=0. Exported in absolute
        // cinematic seconds; the frontend samples it in place of the constant rate.
        if let Some(rc) = entrance_rate_curves.get(&go_pid) {
            let pts: Vec<Value> = rc.iter().map(|&(t, v)| json!({ "t": t, "v": v })).collect();
            sys["rateCurve"] = json!(pts);
        }

        // Shape.
        if let Some(shape) = ps.get("ShapeModule")
            && b(shape, "enabled", false)
        {
            let stype = shape_type_name(i(shape, "type").unwrap_or(0));
            let radius = shape
                .get("radius")
                .and_then(|r| f(r, "value"))
                .unwrap_or(0.0)
                * em_inv;
            let arc = shape
                .get("arc")
                .and_then(|a| f(a, "value"))
                .unwrap_or(360.0);
            // `m_Scale` is the shape TRANSFORM's scale (Unity 2017.2+ gave every shape
            // type a position/rotation/scale), not a box-only field. For a BOX it IS the
            // box size, so `box` keeps carrying it in px. For every RADIAL shape
            // (sphere/hemisphere/circle/cone/edge) it stretches the emission volume into
            // an ellipse around `radius` — Mlynar's `rain_left_short_01` is a radius-0.2
            // hemisphere scaled (2.0, 1.4), i.e. a 182x128px patch, and dropping the scale
            // packs the whole shower into a 91px disc (the clustered rain the archive
            // never shows). Export the raw multiplier so the frontend can widen the
            // radius; a default (1,1) scale leaves every other system byte-identical.
            let scale = shape.get("m_Scale");
            let scale_xy = [
                scale.and_then(|s| f(s, "x")).unwrap_or(1.0),
                scale.and_then(|s| f(s, "y")).unwrap_or(1.0),
            ];
            let box_wh = [scale_xy[0] * em_inv, scale_xy[1] * em_inv];
            let posv = shape.get("m_Position");
            let pos_off = [
                posv.and_then(|p| f(p, "x")).unwrap_or(0.0) * em_inv,
                posv.and_then(|p| f(p, "y")).unwrap_or(0.0) * em_inv,
            ];
            let rot_deg = shape
                .get("m_Rotation")
                .and_then(|r| f(r, "z"))
                .unwrap_or(0.0);
            sys["shape"] = json!({
                "type": stype,
                "radius": radius,
                "angleDeg": fd(shape, "angle", 25.0),
                "arcDeg": arc,
                "box": box_wh,
                "scale": scale_xy,
                "posOffset": pos_off,
                "rotDeg": rot_deg,
                "radiusThickness": fd(shape, "radiusThickness", 1.0),
            });
        } else {
            sys["shape"] = json!({ "type": "none" });
        }

        // Color over lifetime.
        if let Some(cm) = ps.get("ColorModule")
            && b(cm, "enabled", false)
            && let Some(g) = cm.get("gradient")
        {
            sys["colorOverLife"] = mmgradient(g);
        } else {
            sys["colorOverLife"] = Value::Null;
        }

        // Size over lifetime (multiplier curve, 0..1-ish).
        if let Some(sm) = ps.get("SizeModule")
            && b(sm, "enabled", false)
            && let Some(curve) = sm.get("curve")
        {
            // A single MinMaxCurve; state 1 = normalized curve × scalar.
            let scalar = fd(curve, "scalar", 1.0);
            let sampled = curve
                .get("maxCurve")
                .map(|mc| sample_curve(mc, scalar, 1.0))
                .unwrap_or_default();
            sys["sizeOverLife"] = if sampled.is_empty() {
                Value::Null
            } else {
                json!(sampled)
            };
        } else {
            sys["sizeOverLife"] = Value::Null;
        }

        // Velocity over lifetime.
        //
        // The schema's legacy shape is one constant `{x, y}` in px/s, which drops TWO
        // things Unity authors: the `z` axis (never read at all) and the per-lifetime
        // CURVE of each axis (flattened to its multiplier). Both matter as soon as an
        // emitter is tilted out of the screen plane: Skadi2 iteration's crown ring
        // (`fish_01`) authors x and z as curves that, traced against each other through
        // the emitter's basis, close a CIRCLE in screen space — flattened, it became a
        // straight 60px/s drift ("2–3 loose birds" instead of a ring).
        //
        // So when the legacy flattening is LOSSY (a non-zero `z`, or any axis authored
        // as a non-flat curve) a local-space vector is projected through the emitter's
        // FULL world basis — not the flat `matrix_z_deg`, which is a meaningless
        // Z-decomposition for a non-planar matrix — and resampled over the lifetime.
        // `space:"screen"` tells the frontend the vector is already world-aligned.
        // Everything the flattening represented faithfully keeps the byte-identical
        // legacy output.
        if let Some(vm) = ps.get("VelocityModule")
            && b(vm, "enabled", false)
        {
            let vx = vm.get("x");
            let vy = vm.get("y");
            let vz = vm.get("z");
            let nonzero = [vx, vy, vz]
                .iter()
                .any(|a| a.is_some_and(|v| !mmscalar_is_zero(v)));
            // The legacy branch reads only x/y, so it must also GATE on only x/y —
            // a world-space z-only module keeps exporting null, as it always did.
            let nonzero_xy = vx.is_some_and(|v| !mmscalar_is_zero(v))
                || vy.is_some_and(|v| !mmscalar_is_zero(v));
            let in_world = b(vm, "inWorldSpace", false);
            let flat = |a: Option<&Value>| a.is_none_or(mmscalar_is_flat);
            let lossy = !in_world
                && (vz.is_some_and(|v| !mmscalar_is_zero(v))
                    || !flat(vx)
                    || !flat(vy)
                    || !flat(vz));
            let mut emitted = false;
            if nonzero && lossy {
                // Emitter world basis (spine-root frame, Y-up px): the columns are where
                // local +X/+Y/+Z land, so a local 3-vector projects straight to screen —
                // including the foreshortening of an axis tilted toward the camera.
                let ez = world.point([0.0, 0.0, 1.0]);
                let col = |p: [f32; 3]| {
                    [
                        f64::from(p[0] - origin[0]) * inv_scale,
                        f64::from(p[1] - origin[1]) * inv_scale,
                    ]
                };
                let (bx, by, bz) = (col(ex), col(ey), col(ez));
                // How much of each local axis SURVIVES into the screen plane. An axis pointing
                // along the camera has a short screen projection relative to its 3D length:
                // motion along it is pure DEPTH in Unity and moves nothing on screen, so
                // projecting it injects drift the game never shows. Same ratio the `emitDir`
                // gate uses.
                if std::env::var("SCENE_ATTRIB").is_ok() {
                    let len3 = |p: [f32; 3]| {
                        let d = [
                            f64::from(p[0] - origin[0]),
                            f64::from(p[1] - origin[1]),
                            f64::from(p[2] - origin[2]),
                        ];
                        d[0].hypot(d[1]).hypot(d[2]) * inv_scale
                    };
                    let r = |b: [f64; 2], l: f64| if l > 1e-9 { b[0].hypot(b[1]) / l } else { 0.0 };
                    eprintln!(
                        "    [vel] {:<24} bx=({:.1},{:.1}) rx={:.3}  by=({:.1},{:.1}) ry={:.3}  bz=({:.1},{:.1}) rz={:.3}",
                        host.go_name(all_objects, go_pid),
                        bx[0],
                        bx[1],
                        r(bx, len3(ex)),
                        by[0],
                        by[1],
                        r(by, len3(ey)),
                        bz[0],
                        bz[1],
                        r(bz, len3(ez)),
                    );
                    // Does the DIRECTION turn over the lifetime, or only the magnitude? The
                    // flattening below keeps a single vector whenever the motion never
                    // REVERSES — but a direction that swings without reversing is flattened to
                    // the angle at peak speed, which is not the angle early in life.
                    let ang = |t: f64| {
                        let lx = vx.map_or(0.0, |v| mmscalar_eval(v, t, 1.0));
                        let ly = vy.map_or(0.0, |v| mmscalar_eval(v, t, 1.0));
                        let lz = vz.map_or(0.0, |v| mmscalar_eval(v, t, 1.0));
                        let sx = bx[0] * lx + by[0] * ly + bz[0] * lz;
                        let sy = bx[1] * lx + by[1] * ly + bz[1] * lz;
                        (sy.atan2(sx) * RAD_TO_DEG, sx.hypot(sy))
                    };
                    let a: Vec<String> = [0.0, 0.25, 0.5, 0.75, 1.0]
                        .iter()
                        .map(|&t| {
                            let (d, m) = ang(t);
                            format!("t{t:.2}:{d:+.1}deg|{m:.0}")
                        })
                        .collect();
                    eprintln!("          dir over life: {}", a.join("  "));
                }
                let samples: Vec<(f64, f64, f64)> = (0..=VELOCITY_CURVE_SAMPLES)
                    .map(|k| {
                        let t = f64::from(k) / f64::from(VELOCITY_CURVE_SAMPLES);
                        let lx = vx.map_or(0.0, |v| mmscalar_eval(v, t, 1.0));
                        let ly = vy.map_or(0.0, |v| mmscalar_eval(v, t, 1.0));
                        let lz = vz.map_or(0.0, |v| mmscalar_eval(v, t, 1.0));
                        (
                            t,
                            bx[0] * lx + by[0] * ly + bz[0] * lz,
                            bx[1] * lx + by[1] * ly + bz[1] * lz,
                        )
                    })
                    .collect();
                // Representative constant = the largest vector over the lifetime; the
                // frontend still uses it wherever one number is needed (streak length,
                // spawn-cull trajectory, additive pile density).
                let rep = samples
                    .iter()
                    .max_by(|a, b| a.1.hypot(a.2).total_cmp(&b.1.hypot(b.2)))
                    .copied()
                    .unwrap_or((0.0, 0.0, 0.0));
                let varies = samples.iter().any(|s| {
                    (s.1 - samples[0].1).abs() > 1e-6 || (s.2 - samples[0].2).abs() > 1e-6
                });
                // Take the per-lifetime path ONLY for a velocity that REVERSES in screen
                // space — some sample opposes the representative one. That is the case a
                // single vector cannot express at all: the particle orbits (or doubles
                // back) and its mean travel is nowhere near the constant, so flattening
                // turns a ring into a straight line. When the direction is instead
                // CONSTANT and only the magnitude modulates (Skadi2's `xian` threads:
                // their curve never changes sign), the flattened constant is already a
                // faithful direction, and measuring showed replacing it costs more than
                // it gains — so those keep the byte-identical legacy vector.
                let mag = rep.1.hypot(rep.2);
                let reverses = varies
                    && mag > 1e-9
                    && samples
                        .iter()
                        .any(|s| (s.1 * rep.1 + s.2 * rep.2) / mag < -VELOCITY_REVERSAL_FRAC * mag);
                // ...and for one that TURNS far enough that no single angle stands in for it.
                //
                // Reversal is not the only motion a constant cannot express. A direction that
                // swings wide without ever opposing itself is flattened to its angle at PEAK
                // SPEED, which is not its angle early in life — the particle is drawn on a
                // straight line at the wrong slope for most of its lifetime, and a stretched
                // billboard (aligned to velocity) is drawn at that wrong slope too.
                //
                // Skadi the Corrupting Heart's sweeping streak turns from -8 deg to -64 deg over
                // its life while the game's beam crosses the frame level; we drew it tilted for
                // the whole crossing, smearing its light down 30 rows where the game's occupies
                // 12 and so reading ~3x too faint per row.
                //
                // Magnitude-only systems are unaffected by construction: their samples are all
                // parallel, so the deviation is 0 and they keep the byte-identical constant
                // (that is the case the paragraph above measured as not worth replacing).
                let turns = varies
                    && mag > 1e-9
                    && samples.iter().any(|s| {
                        let m = s.1.hypot(s.2);
                        // Ignore samples too slow to displace the particle: their angle is
                        // numerically unstable and contributes no visible travel.
                        m > VELOCITY_TURN_MIN_FRAC * mag && {
                            let cos = (s.1 * rep.1 + s.2 * rep.2) / (m * mag);
                            cos.clamp(-1.0, 1.0).acos() * RAD_TO_DEG > VELOCITY_TURN_DEG
                        }
                    });
                if std::env::var("SCENE_ATTRIB").is_ok() {
                    let maxdev = samples
                        .iter()
                        .filter(|s| s.1.hypot(s.2) > VELOCITY_TURN_MIN_FRAC * mag)
                        .map(|s| {
                            let m = s.1.hypot(s.2);
                            ((s.1 * rep.1 + s.2 * rep.2) / (m * mag))
                                .clamp(-1.0, 1.0)
                                .acos()
                                * RAD_TO_DEG
                        })
                        .fold(0.0f64, f64::max);
                    eprintln!(
                        "    [vel2] {:<24} rep=({:.1},{:.1}) |rep|={mag:.1} maxdev={maxdev:.1}deg reverses={reverses} turns={turns}",
                        host.go_name(all_objects, go_pid),
                        rep.1,
                        rep.2
                    );
                }
                if reverses || turns {
                    sys["velocityOverLife"] = json!({
                        "x": rep.1,
                        "y": rep.2,
                        "space": "screen",
                        "curve": samples
                            .iter()
                            .map(|(t, x, y)| json!({ "t": t, "x": x, "y": y }))
                            .collect::<Vec<_>>(),
                    });
                    emitted = true;
                }
            }
            if !emitted {
                if nonzero_xy && in_world {
                    // Already in the skeleton world frame.
                    sys["velocityOverLife"] = json!({
                        "x": vx.map_or(0.0, |v| mmscalar_repr(v, inv_scale)),
                        "y": vy.map_or(0.0, |v| mmscalar_repr(v, inv_scale)),
                        "space": "world",
                    });
                } else if nonzero_xy {
                    // LOCAL-space velocity: project through the emitter's world BASIS, the
                    // same way the reversing-curve branch above does, and ship it
                    // pre-projected as "screen".
                    //
                    // It used to ship raw local x/y for the frontend to rotate by `rot`
                    // (`matrix_z_deg`) — a single Z angle recovered from the world matrix.
                    // That is the value already documented as unreliable for cone emission
                    // (hence `emitDir`), and it is wrong here for the same reason: it throws
                    // away non-uniform scale and any tilt, so it is only correct when the
                    // emitter is a pure Z rotation at uniform scale. Skadi the Corrupting
                    // Heart's `xian` threads are the visible casualty — authored local
                    // (1, 20), `rot` 153.43°:
                    //
                    //     rotate by `rot` -> ( -491.9, -872.1)   mostly DOWN  (what we drew)
                    //     world basis     -> (-1913.6, +581.4)   mostly LEFT  (the game)
                    //
                    // a 77.5° error, which is why her red threads swept vertically off-frame
                    // instead of crossing it horizontally at ~1000 px/s like the capture's.
                    // The basis also carries the emitter's NON-UNIFORM scale, which `em_inv`
                    // (a mean of the two axis lengths) had been averaging away.
                    let ez = world.point([0.0, 0.0, 1.0]);
                    let col = |q: [f32; 3]| {
                        [
                            f64::from(q[0] - origin[0]) * inv_scale,
                            f64::from(q[1] - origin[1]) * inv_scale,
                        ]
                    };
                    let (bx, by, bz) = (col(ex), col(ey), col(ez));
                    let lx = vx.map_or(0.0, |v| mmscalar_repr(v, 1.0));
                    let ly = vy.map_or(0.0, |v| mmscalar_repr(v, 1.0));
                    let lz = vz.map_or(0.0, |v| mmscalar_repr(v, 1.0));
                    sys["velocityOverLife"] = json!({
                        "x": bx[0] * lx + by[0] * ly + bz[0] * lz,
                        "y": bx[1] * lx + by[1] * ly + bz[1] * lz,
                        "space": "screen",
                    });
                } else {
                    sys["velocityOverLife"] = Value::Null;
                }
            }
        } else {
            sys["velocityOverLife"] = Value::Null;
        }

        // Force over lifetime (constant accel in px/s²). Not in the base schema
        // but visually important (drift/wind) — emitted as forceOverLife.
        // TODO: Force axes can be curves/ranges; we flatten to the representative
        // scalar per axis.
        if let Some(fm) = ps.get("ForceModule")
            && b(fm, "enabled", false)
        {
            let fx = fm.get("x");
            let fy = fm.get("y");
            let nonzero = fx.is_some_and(|v| !mmscalar_is_zero(v))
                || fy.is_some_and(|v| !mmscalar_is_zero(v));
            let fz = fm.get("z");
            let nonzero = nonzero || fz.is_some_and(|v| !mmscalar_is_zero(v));
            if nonzero {
                if b(fm, "inWorldSpace", false) {
                    sys["forceOverLife"] = json!({
                        "x": fx.map_or(0.0, |v| mmscalar_repr(v, inv_scale)),
                        "y": fy.map_or(0.0, |v| mmscalar_repr(v, inv_scale)),
                        "space": "world",
                    });
                } else {
                    // A LOCAL force is authored in the emitter's own frame, so it must be
                    // projected through the emitter's WORLD BASIS — exactly as
                    // `velocityOverLifetime` is a few lines above, and for the same reason:
                    // `rot` (`matrix_z_deg`) discards non-uniform scale and tilt, and
                    // `em_inv` averages the two axis lengths away. Shipping it pre-projected
                    // as `space:"screen"` also means the frontend needs no basis of its own.
                    let ez = world.point([0.0, 0.0, 1.0]);
                    let col = |q: [f32; 3]| {
                        [
                            f64::from(q[0] - origin[0]) * inv_scale,
                            f64::from(q[1] - origin[1]) * inv_scale,
                        ]
                    };
                    let (bx, by, bz) = (col(ex), col(ey), col(ez));
                    let lx = fx.map_or(0.0, |v| mmscalar_repr(v, 1.0));
                    let ly = fy.map_or(0.0, |v| mmscalar_repr(v, 1.0));
                    let lz = fz.map_or(0.0, |v| mmscalar_repr(v, 1.0));
                    sys["forceOverLife"] = json!({
                        "x": bx[0] * lx + by[0] * ly + bz[0] * lz,
                        "y": bx[1] * lx + by[1] * ly + bz[1] * lz,
                        "space": "screen",
                    });
                }
            }
        }

        // Limit velocity over lifetime (Unity `ClampVelocityModule`): a speed
        // ceiling the particle is damped toward each step. Mlynar's `weapon_star_*`
        // glint pops fast off the blade, then this clamp reins it into a tight,
        // stationary cluster. `magnitude` is a curve of the speed LIMIT over
        // normalized life, in the same px/s frame as `startSpeed` (em_inv for the
        // common local case; inv_scale when the clamp is world-space, matching the
        // Velocity/Force modules). Per-axis (`separateAxis`) clamps are a documented
        // gap — we only model the uniform speed clamp (`separateAxis:false`).
        if let Some(m) = ps.get("ClampVelocityModule")
            && b(m, "enabled", false)
            && !b(m, "separateAxis", false)
        {
            let cscale = if b(m, "inWorldSpace", false) {
                inv_scale
            } else {
                em_inv
            };
            sys["velocityClamp"] = json!({
                "dampen": fd(m, "dampen", 0.0),
                "magnitude": mmscalar(m.get("magnitude").unwrap_or(&Value::Null), cscale),
            });
        }

        // Rotation over lifetime (angular velocity, radians/s → deg/s).
        //
        // Exported as a full `MMScalar` — Unity authors this as a CURVE over normalized
        // particle life far more often than as a constant, and flattening it to the curve's
        // range top (`mmscalar_repr`) spins every such system at its PEAK rate from birth.
        // Virtuosa "Diversity in Oneness"'s falling apple is the clearest case: its curve is
        // 0 until 3% of life and only reaches 0.93 at 38%, so over the visible fall the game
        // turns it ~4°, while a flat 45–90°/s turned ours 50–100° and the apple visibly
        // tumbled. The error is worst for LONG lifetimes, where normalized life stays near
        // the curve's low end for the whole time on screen. A genuinely constant rotation
        // (`minMaxState` 0) still exports as `{mode:"const"}` with the identical value.
        if let Some(rm) = ps.get("RotationModule")
            && b(rm, "enabled", false)
            && let Some(curve) = rm.get("curve")
        {
            sys["rotOverLifeDegPerSec"] = mmscalar(curve, RAD_TO_DEG);
        } else {
            sys["rotOverLifeDegPerSec"] = Value::Null;
        }

        // Texture sheet animation.
        if let Some(uv) = ps.get("UVModule")
            && b(uv, "enabled", false)
        {
            // `frameOverTime` is a Unity MinMaxCurve. Only state 1 (single curve) was read, and
            // every other state fell through to null — which the renderer re-interprets as "no
            // authored frame, so animate the flipbook from life fraction". That is a different
            // animation, not an absent one.
            //
            // Civilight Eterna's seven background planes are the case that exposes it: each is a
            // 1x2 sheet whose `frameOverTime` is a CONSTANT 0.5 (minMaxState 0), i.e.
            // `floor(0.5 * cycles * tiles) = 1` — frame 1, held for the system's whole life. The
            // capture agrees exactly: correlated against each raw tile it sits on tile 1 at every
            // sample from t=5.5 to 12.0 and never advances, while a life-driven flipbook flips
            // every ~0.5 s. Emit the constant as a flat curve so the renderer's existing
            // `sampleCurve` path returns it unchanged at any life fraction.
            //
            // State 3 (two constants) is Unity's random-between-two; take the max endpoint, which
            // is what a single-particle system with maxParticles 1 lands on in practice. State 2
            // (two curves) keeps the max curve, matching the state-1 treatment.
            let frame = uv.get("frameOverTime");
            let frame_curve = match frame {
                Some(fr) => match i(fr, "minMaxState").unwrap_or(0) {
                    1 => {
                        let scalar = fd(fr, "scalar", 1.0);
                        fr.get("maxCurve")
                            .map_or(Value::Null, |mc| json!(sample_curve(mc, scalar, 1.0)))
                    }
                    2 => {
                        let scalar = fd(fr, "scalar", 1.0);
                        fr.get("maxCurve")
                            .map_or(Value::Null, |mc| json!(sample_curve(mc, scalar, 1.0)))
                    }
                    // 0 = constant, 3 = two constants. Unity stores the value in `scalar` (the
                    // curves are empty), so emit it as a two-point flat curve.
                    0 | 3 => {
                        let c = fd(fr, "scalar", 0.0);
                        json!([{ "t": 0.0, "v": c }, { "t": 1.0, "v": c }])
                    }
                    _ => Value::Null,
                },
                None => Value::Null,
            };
            sys["sheet"] = json!({
                "tilesX": i(uv, "tilesX").unwrap_or(1),
                "tilesY": i(uv, "tilesY").unwrap_or(1),
                "cycles": fd(uv, "cycles", 1.0),
                "frameOverTime": frame_curve,
            });
        } else {
            sys["sheet"] = Value::Null;
        }

        // Noise (curl-noise position/size/rotation displacement).
        if let Some(nm) = ps.get("NoiseModule")
            && b(nm, "enabled", false)
        {
            sys["noise"] = parse_noise(nm, em_inv);
        } else {
            sys["noise"] = Value::Null;
        }

        // Trail (both modes — see `parse_trail`). The trail texture is the renderer's
        // 2nd material, deduped in export.
        let (trail_tex_val, trail_alpha_val, trail_tex_pid, trail_tex_st) = if let Some(tm) =
            ps.get("TrailModule")
            && b(tm, "enabled", false)
        {
            let (tv, ta, tpid, tadd, tst) = resolve_trail_material(all_objects, renderer);
            sys["trail"] = parse_trail(tm, em_inv, tadd);
            (tv, ta, tpid, tst)
        } else {
            sys["trail"] = Value::Null;
            (None, None, None, ST_IDENTITY)
        };

        if attrib_dbg {
            // Unity `ParticleSystemScalingMode`: 0 Hierarchy, 1 Local, 2 Shape. We currently
            // scale particle SIZE by the emitter's accumulated WORLD basis (`wsx`) regardless,
            // which is only correct for Hierarchy. Print both so the mismatch is countable.
            let own = host
                .local_scale_pos_of_go(all_objects, go_pid)
                .map_or(f64::NAN, |(sc, _)| {
                    f64::midpoint(f64::from(sc[0].abs()), f64::from(sc[1].abs()))
                });
            eprintln!(
                "    [ptcl] SCALE mode={} world={world_axis_mean:.4} own={own:.4} ratio={:.4} {}",
                i(ps, "scalingMode").unwrap_or(-1),
                own / world_axis_mean,
                host.go_name(all_objects, go_pid),
            );
            eprintln!(
                "    [ptcl] KEEP  render={render_mode:<9} trail={:<4} {:<28} root={}",
                if sys["trail"].is_null() { "-" } else { "yes" },
                host.go_name(all_objects, go_pid),
                host.root_name_of_go(all_objects, go_pid)
            );
        }
        out.push(ParticleData {
            json: sys,
            tex_val,
            alpha_val,
            tex_pid,
            tex_st: main_st,
            trail_tex_val,
            trail_alpha_val,
            trail_tex_pid,
            trail_tex_st,
            ram,
        });
    }

    if entrance.is_entrance {
        apply_followbone_reveal_inheritance(&mut out);
    }

    (out, skipped)
}

/// `(main_tex, alpha_tex, main_pid, additive, main_st, tint)` of a resolved material.
type ResolvedMaterial = (
    Value,
    Option<Value>,
    i64,
    bool,
    [f64; 4],
    Option<[f64; 4]>,
    Option<[f64; 4]>,
);

/// The ×2 `_TintColor` the material multiplies its sprite by, or `None` when the
/// material is at (or has no) neutral tint and the draw is already faithful.
///
/// Torappu's ports of Unity's legacy particle shaders — the plain blend modes
/// `Torappu/Particles/<Mode>` and `Torappu/Particles-L2D/<Mode>` — sample
/// `2 × _TintColor × vertexColor × tex`, which is why every one of them declares
/// `_TintColor`'s DEFAULT as (0.5, 0.5, 0.5, 0.5): half is the neutral, doubling
/// restores white. (The same convention the sub-namespaced compositors carry under
/// `_MainColor`, mirrored in the frontend Ram GLSL as `col += col`, and already
/// applied on the scene-quad path by `spine::legacy_tint_scale`.)
///
/// The particle path multiplied by nothing at all, i.e. it assumed every material sat
/// at the neutral. Materials authored AWAY from it were then drawn at the wrong
/// amplitude — Mlynar's `bao 1` transformation blast is `_TintColor` (1, 1, 1, 1),
/// double-neutral in all four channels, so the game's additive output (`rgb × a`) is
/// 4× what we drew, which is why its full-frame white veil read as a faint haze.
///
/// Returning `None` at the neutral keeps every already-correct system byte-identical.
/// The sub-namespaced compositors (`Ram/`, `Disturb/`, `Dissolve/`, `Mask/`) are
/// excluded: they modulate by `_MainColor` and carry `_TintColor` only as an inert
/// leftover of the shader they were authored under.
///
/// ADDITIVE materials only. Under `Blend SrcAlpha One` both halves of `_TintColor` are
/// pure light scale — the sprite's contribution is `rgb × a`, so the doubling is a
/// straight amplitude correction that the half-float target carries linearly. Under
/// `Blend SrcAlpha OneMinusSrcAlpha` the alpha is COVERAGE instead: doubling it does not
/// brighten the sprite, it replaces more of the backdrop with it, and since these
/// materials are overwhelmingly authored at plain white (α 1.0 → ×2) it pins every
/// particle fully opaque for its whole life, flattening the authored `colorOverLifetime`
/// fade. Measured, not assumed: Skadi the Corrupting Heart's entrance touches ONLY
/// alpha-blend tinted systems and doubling them cost her 19.396 → 19.415 MADC, while
/// Mlynar's win came entirely from the additive side.
/// The SAME `_TintColor`, raw and undoubled, for a NON-additive plain-mode material.
///
/// `particle_tint` is gated on `is_additive`, so alpha-blend systems get no tint at all and the
/// renderer multiplies by 1. The Unity programs say that is wrong, and the two families are
/// IDENTICAL here, read from the decompiled pair rather than inferred across them:
///
/// ```glsl
/// // Particles-L2D/Additive AND Particles-L2D/AlphaBlend, vertex:
/// vs_COLOR0 = in_COLOR0 * _TintColor;
/// // ... fragment:
/// u_xlat0 = vs_COLOR0 + vs_COLOR0;          // the x2, in RGBA
/// SV_Target0.xyz = (u_xlat0 * mainTex).xyz;
/// SV_Target0.w   = clamp((u_xlat0 * mainTex).w, 0.0, 1.0);
/// ```
///
/// So the game draws `2 * _TintColor` on both, and we draw 1.0 on one of them. Corpus: 2513
/// alpha-blend materials carry a `_TintColor`, **56.1% away from the 0.502 neutral**, and on
/// those the game's `2 * maxRGB` averages **1.614** against our 1.0.
///
/// Emitted under a SEPARATE key so the default is untouched: the renderer ignores `abTint`
/// unless `?abtint=1`. Undoubled here on purpose, so the renderer owns the x2 and the alpha
/// clamp that goes with it.
fn particle_tint_nonadditive(mat: &Value) -> Option<[f64; 4]> {
    if is_additive(mat) {
        return None;
    }
    let shader = mat.get("_shaderName").and_then(Value::as_str)?;
    let plain_mode = |ns: &str| {
        shader
            .rfind(ns)
            .map(|i| &shader[i + ns.len()..])
            .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
    };
    if !plain_mode("Particles-L2D/") && !plain_mode("Particles/") {
        return None;
    }
    let tint = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.get("_TintColor"))?;
    let v = [
        fd(tint, "r", 0.5),
        fd(tint, "g", 0.5),
        fd(tint, "b", 0.5),
        fd(tint, "a", 0.5),
    ];
    let neutral = v.iter().all(|c| (c - 0.5).abs() <= 1.5 / 255.0);
    (!neutral).then_some(v)
}

fn particle_tint(mat: &Value) -> Option<[f64; 4]> {
    // GATED ON `is_additive` — RELAXING THIS WAS TRIED AND REVERTED (2026-08-03). The
    // `tex * (COLOR + COLOR)` doubling really is a property of the shader FAMILY rather than the
    // blend mode, so admitting AlphaBlend materials looked correct on paper. Measured: it changes
    // 37 systems across the three references, hands many NORMAL-blend systems a tint whose ALPHA
    // is 2.0 (doubling their opacity), and scores mly 17.439 -> 17.464 with cel/ska flat. It also
    // does NOT reach the case it was written for: Virtuosa's `bg_tint_01` / `window_bg_01` export
    // from the `Dissolve(CustomData)` variants, which `plain_mode` excludes and whose tints are
    // the 0.5 neutral anyway — the darkening (0.390, 0.395, 0.395) tint belongs to a SKIPPED
    // sibling, and that sibling is skipped CORRECTLY: `SCENE_ATTRIB=1` names its blocker as
    // 'Special Only Effects' / 'Interact Only Effects', i.e. <State> Only Effects groups that ship
    // inactive and play only on those interactions, never during the entrance. So "the skipped
    // copy stacks with the drawn one" is refuted too. See dynchar-virtuosa-t5-localised.
    if !is_additive(mat) {
        return None;
    }
    let shader = mat.get("_shaderName").and_then(Value::as_str)?;
    let plain_mode = |ns: &str| {
        shader
            .rfind(ns)
            .map(|i| &shader[i + ns.len()..])
            .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
    };
    if !plain_mode("Particles-L2D/") && !plain_mode("Particles/") {
        return None;
    }
    let tint = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.get("_TintColor"))?;
    let v = [
        fd(tint, "r", 0.5),
        fd(tint, "g", 0.5),
        fd(tint, "b", 0.5),
        fd(tint, "a", 0.5),
    ];
    // The neutral is authored as an 8-bit colour, so it arrives as 128/255 = 0.50196.
    let neutral = v.iter().all(|c| (c - 0.5).abs() <= 1.5 / 255.0);
    (!neutral).then(|| v.map(|c| c * 2.0))
}

/// Resolve one material reference into [`ResolvedMaterial`].
/// Returns `None` when the ref is null, not a Material(21), or its `_MainTex`
/// lives in another bundle (unresolvable).
fn resolve_material(
    all_objects: &HashMap<i64, (i32, Value)>,
    mat_ref: &Value,
) -> Option<ResolvedMaterial> {
    // DIAGNOSTIC (`SCENE_ATTRIB=1`): this function has FIVE silent `?` exits and they mean very
    // different things — an unbound slot is an authoring fact, while an unloaded material or
    // texture is a STAGING fault (the bundle was not passed to `-i`). A system that falls out
    // here renders untextured, so which exit fired decides whether there is anything to fix.
    let dbg = std::env::var("SCENE_ATTRIB").is_ok();
    let Some(mat_pid) = get_path_id(mat_ref).filter(|&p| p != 0) else {
        if dbg {
            eprintln!("  [ptcl-mat] no material PPtr");
        }
        return None;
    };
    let Some((21, mat)) = all_objects.get(&mat_pid) else {
        if dbg {
            eprintln!(
                "  [ptcl-mat] material {mat_pid} NOT LOADED (class={:?}) — other bundle",
                all_objects.get(&mat_pid).map(|(c, _)| *c)
            );
        }
        return None;
    };
    let mat_name = mat.get("m_Name").and_then(Value::as_str).unwrap_or("?");
    let Some(tex_envs) = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_TexEnvs"))
        .and_then(Value::as_object)
    else {
        if dbg {
            eprintln!("  [ptcl-mat] '{mat_name}' has no m_TexEnvs");
        }
        return None;
    };
    let Some(main_pid) = tex_envs
        .get("_MainTex")
        .and_then(|t| t.get("m_Texture"))
        .and_then(get_path_id)
        .filter(|&p| p != 0)
    else {
        if dbg {
            eprintln!(
                "  [ptcl-mat] '{mat_name}' shader='{}' binds NO _MainTex (slots: {:?}) dissolveBound={}",
                mat.get("_shaderName")
                    .and_then(Value::as_str)
                    .unwrap_or("?"),
                tex_envs.keys().collect::<Vec<_>>(),
                tex_envs
                    .get("_DissolveTex")
                    .and_then(|t| t.get("m_Texture"))
                    .and_then(get_path_id)
                    .is_some_and(|p| p != 0)
            );
        }
        return None;
    };
    let Some((28, tex_val)) = all_objects.get(&main_pid) else {
        if dbg {
            eprintln!(
                "  [ptcl-mat] '{mat_name}' _MainTex {main_pid} NOT LOADED (class={:?}) — other bundle",
                all_objects.get(&main_pid).map(|(c, _)| *c)
            );
        }
        return None; // texture in another bundle — unresolvable
    };
    let alpha_val = tex_envs
        .get("_AlphaTex")
        .and_then(|t| t.get("m_Texture"))
        .and_then(get_path_id)
        .filter(|&p| p != 0)
        .and_then(|p| all_objects.get(&p))
        .and_then(|(cid, v)| (*cid == 28).then(|| v.clone()));
    // _MainTex scale/offset (`_MainTex_ST`): some emitters tile/crop the sprite to
    // ONE cell of a flipbook atlas via the material UVs (not the Texture Sheet
    // module). The billboard render path must apply this or it draws the whole
    // atlas as one sprite (Hoshiguma the Breacher's wave systems → giant multi-wave
    // "plumes"). Reuse mat_texenv purely for its ST tuple.
    let (_, _, main_st) = mat_texenv(all_objects, mat, "_MainTex");
    // DIAGNOSTIC (`DYNCHAR_PSMAT`, presence-checked): print every particle material the
    // export resolves, with the shader the blend was read from, the raw blend floats and the
    // colour properties, so a system's exported `blend` and tint can be checked against the
    // bundle without re-deriving them (Pozemka's `cloud_01`, SilverAsh Alter's `slash fire`).
    if std::env::var("DYNCHAR_PSMAT").is_ok() {
        let shader = mat
            .get("_shaderName")
            .and_then(Value::as_str)
            .unwrap_or("?");
        let colors = mat
            .get("m_SavedProperties")
            .and_then(|sp| sp.get("m_Colors"))
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .map(|(k, c)| {
                        format!(
                            "{k}=({:.3},{:.3},{:.3},{:.3})",
                            fd(c, "r", 0.0),
                            fd(c, "g", 0.0),
                            fd(c, "b", 0.0),
                            fd(c, "a", 0.0)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let floats = mat
            .get("m_SavedProperties")
            .and_then(|sp| sp.get("m_Floats"))
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .map(|(k, v)| format!("{k}={}", v.as_f64().unwrap_or(f64::NAN)))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let texenvs = mat
            .get("m_SavedProperties")
            .and_then(|sp| sp.get("m_TexEnvs"))
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .map(|(k, v)| {
                        let pid = v.get("m_Texture").and_then(get_path_id).unwrap_or(0);
                        let sc = v.get("m_Scale");
                        let of = v.get("m_Offset");
                        format!(
                            "{k}:pid={pid} scale=({},{}) offset=({},{})",
                            fd(sc.unwrap_or(&Value::Null), "x", 1.0),
                            fd(sc.unwrap_or(&Value::Null), "y", 1.0),
                            fd(of.unwrap_or(&Value::Null), "x", 0.0),
                            fd(of.unwrap_or(&Value::Null), "y", 0.0)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        eprintln!(
            "PSMAT '{mat_name}' mainpid={main_pid} shader='{shader}' additive={} src={} dst={} {colors} | floats {floats} | texenvs {texenvs}",
            is_additive(mat),
            mat_float(mat, "_SrcBlend", -1.0),
            mat_float(mat, "_DstBlend", -1.0)
        );
    }
    Some((
        tex_val.clone(),
        alpha_val,
        main_pid,
        is_additive(mat),
        main_st,
        particle_tint(mat),
        particle_tint_nonadditive(mat),
    ))
}

/// `(main_texture, alpha_texture, main_pid, additive, main_st, tint, ab_tint)` of a renderer's
/// first usable material, with every optional slot unresolved.
type RendererTexture = (
    Option<Value>,
    Option<Value>,
    Option<i64>,
    bool,
    [f64; 4],
    Option<[f64; 4]>,
    Option<[f64; 4]>,
);

/// The `_Start` clip's animated MATERIAL COLOUR for a plain (non-Ram) emitter, as
/// `(t, [r,g,b,a])` samples — the particle-path twin of the scene quad's `colorCurve`.
///
/// The Ram family already gets this (`mainColorCurve` in [`resolve_ram_data`]); every other
/// emitter ignored it entirely, so a system the cinematic FADES OUT kept drawing at full
/// strength. Virtuosa is the case that found it — `SCENE_ATTRIB=1` reports
/// `bg_rain_01` alpha 0.16→0.00 over 6.2-8.0s and `spark_large` 0.50→0.00 over 9.9-10.2s,
/// both of which we drew flat — and it is the shape of the "particles over-draw late"
/// residual measured across her whole field.
///
/// Resolved against the SAME tint the system exports (`particle_tint`, the ×2 `_TintColor`
/// when additive, else the material's own colour), so curve and static value stay in one
/// convention. `tint_scale` is 1: the ×2 is already folded into the exported `tint`, and
/// doubling it again here would apply it twice.
fn particle_color_curve(
    all_objects: &HashMap<i64, (i32, Value)>,
    renderer: Option<&Value>,
    channels: Option<&[super::anim::MaterialColorChannel]>,
    additive: bool,
) -> Option<Vec<(f32, [f32; 4])>> {
    let channels = channels?;
    let materials = renderer?.get("m_Materials")?.as_array()?;
    let mat = materials.iter().find_map(|mat_ref| {
        let pid = get_path_id(mat_ref).filter(|&p| p != 0)?;
        let (21, m) = all_objects.get(&pid)? else {
            return None;
        };
        Some(m)
    })?;
    // Skip the Ram family: it carries its own `mainColorCurve` on the Ram colour path, and
    // applying both would fade it twice.
    if mat
        .get("_shaderName")
        .and_then(Value::as_str)
        .is_some_and(|s| s.contains("Ram/"))
    {
        return None;
    }
    let (props, tint_prop) = super::spine::material_color_props(mat);
    let base = particle_tint(mat).unwrap_or([1.0, 1.0, 1.0, 1.0]);
    super::anim::layer_color_curve(
        channels,
        &props,
        tint_prop.as_deref(),
        [
            base[0] as f32,
            base[1] as f32,
            base[2] as f32,
            base[3] as f32,
        ],
        1.0,
        false,
        additive,
    )
}

/// Resolve the renderer's first usable material (the particle's own texture)
/// into a [`RendererTexture`].
fn resolve_renderer_texture(
    all_objects: &HashMap<i64, (i32, Value)>,
    renderer: Option<&Value>,
) -> RendererTexture {
    let Some(materials) = renderer
        .and_then(|r| r.get("m_Materials"))
        .and_then(Value::as_array)
    else {
        return (None, None, None, false, ST_IDENTITY, None, None);
    };
    for mat_ref in materials {
        if let Some((tv, ta, pid, add, st, tint, ab)) = resolve_material(all_objects, mat_ref) {
            return (Some(tv), ta, Some(pid), add, st, tint, ab);
        }
    }
    (None, None, None, false, ST_IDENTITY, None, None)
}

// ---------------------------------------------------------------------------
// Ram shader family (`Torappu/Particles-L2D/Ram/{Disturb,VertexDisturb}`).
// A ramp-tint + dissolve + UV-disturb sprite compositor sampling 4–6 textures.
// ---------------------------------------------------------------------------

/// Read a scalar shader property from `m_SavedProperties.m_Floats`.
fn mat_float(mat: &Value, name: &str, default: f64) -> f64 {
    mat.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Floats"))
        .and_then(|m| m.get(name))
        .and_then(Value::as_f64)
        .unwrap_or(default)
}

/// Read an rgba/vector shader property from `m_SavedProperties.m_Colors`
/// (Unity stores `Color` AND `Vector` properties here). Missing → `default`.
pub(crate) fn mat_color(mat: &Value, name: &str, default: [f64; 4]) -> [f64; 4] {
    match mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|m| m.get(name))
    {
        Some(c) => [
            fd(c, "r", default[0]),
            fd(c, "g", default[1]),
            fd(c, "b", default[2]),
            fd(c, "a", default[3]),
        ],
        None => default,
    }
}

/// Read a texture slot `m_SavedProperties.m_TexEnvs.<slot>` →
/// `(path_id, Texture2D value, [scaleX,scaleY,offsetX,offsetY])`. The `path_id` /
/// value are `None` when the slot is empty or its texture is not resolvable
/// (in another bundle); the ST tuple always falls back to `[1,1,0,0]`.
pub(super) fn mat_texenv(
    all_objects: &HashMap<i64, (i32, Value)>,
    mat: &Value,
    slot: &str,
) -> (Option<i64>, Option<Value>, [f64; 4]) {
    let Some(env) = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_TexEnvs"))
        .and_then(|te| te.get(slot))
    else {
        return (None, None, ST_IDENTITY);
    };
    let scale = env.get("m_Scale");
    let offset = env.get("m_Offset");
    let st = [
        scale.and_then(|s| f(s, "x")).unwrap_or(1.0),
        scale.and_then(|s| f(s, "y")).unwrap_or(1.0),
        offset.and_then(|o| f(o, "x")).unwrap_or(0.0),
        offset.and_then(|o| f(o, "y")).unwrap_or(0.0),
    ];
    let pid = env
        .get("m_Texture")
        .and_then(get_path_id)
        .filter(|&p| p != 0);
    let tex_val = pid
        .and_then(|p| all_objects.get(&p))
        .and_then(|(cid, v)| (*cid == 28).then(|| v.clone()));
    // Only report a pid when we actually resolved its texture object; an
    // unresolved (cross-bundle) ref must yield a `null` texture index.
    match tex_val {
        Some(v) => (pid, Some(v), st),
        None => (None, None, st),
    }
}

/// Build the `"ram"` block for the renderer's first `Ram/`-family material, or
/// `None` when no such material is present. Texture-index fields are left `null`
/// (filled in [`export_particles`]); every scalar/color/vector param is baked in.
fn resolve_ram(
    all_objects: &HashMap<i64, (i32, Value)>,
    renderer: Option<&Value>,
    color_channels: Option<&[super::anim::MaterialColorChannel]>,
    additive: bool,
    go_pid: i64,
) -> Option<RamData> {
    let materials = renderer?.get("m_Materials")?.as_array()?;
    let (mat, shader) = materials.iter().find_map(|mat_ref| {
        let mat_pid = get_path_id(mat_ref).filter(|&p| p != 0)?;
        let (21, mat) = all_objects.get(&mat_pid)? else {
            return None;
        };
        let shader = mat.get("_shaderName").and_then(Value::as_str)?;
        // MEASURED — the two-map dissolve does NOT need porting to the particle path.
        //
        // The scene path was fixed to read the `Dissolve/` family's real mask names
        // (`_DissolveTex_01/_02`), because reading the inert single-name residue made those
        // layers draw their full bounding RECTANGLE. This path admits only `Ram/`, so the
        // `Dissolve/` family never reaches it, and the obvious follow-up is to widen the gate.
        //
        // Corpus scan (this diagnostic, SCENE_ATTRIB=1, over all 93 dynchar bundles): exactly
        // TWO materials carry a live two-map dissolve — `4087_ines_boc#8_star_02` and
        // `char_2015_dusk_nian#7_72` — and BOTH are `Dissolve/Dissolve Add Double`, i.e.
        // ADDITIVE. A missing mask on an additive layer adds the texture's dark field, which
        // contributes ~nothing; the rectangle artifact on the scene path came from NORMAL-blend
        // layers stamping a translucent quad. Ines (the only one whose thresholds could bite —
        // dusk's 0.030/0.085 collapse the mask to ~1) renders with no artifact of any kind.
        // ZERO `Ram/` materials carry the two-map names, so there is no narrow fix available
        // either: widening the gate would route a `Dissolve/` program through a path that
        // implements `Disturb` (ram tint, `_Opacity`, UV warp), which is misrouting, not fixing.
        //
        // Left as a diagnostic so the scan is one env var away if the corpus ever changes.
        if std::env::var("SCENE_ATTRIB").is_ok()
            && mat_float(mat, "_Amount_01", -1.0) > 0.0
            && mat_texenv(all_objects, mat, "_DissolveTex_01").0.is_some()
        {
            let edge = mat_color(mat, "_Edgecolor", [-1.0, -1.0, -1.0, -1.0]);
            eprintln!(
                "    [ptcl] DISSOLVE-TWOMAP '{}' shader={shader} amount_01={:.3} amount_02={:.3} edge=({:.3},{:.3},{:.3},{:.3}) pow={:.3}",
                mat.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                mat_float(mat, "_Amount_01", -1.0),
                mat_float(mat, "_Amount_02", -1.0),
                edge[0], edge[1], edge[2], edge[3],
                mat_float(mat, "_pow", -1.0)
            );
        }
        // Admit the `Dissolve/` family alongside `Ram/` — but ONLY when its mask is LIVE.
        //
        // Its program (decompiled from `Dissolve Add Double`) is the Ram fragment minus the
        // ramp/disturb/opacity terms plus a SECOND mask, and its pass blends `SrcAlpha, One` —
        // additive WEIGHTED BY ALPHA, so a mask that multiplies alpha really does attenuate the
        // draw. It reaches the same `RamEmitter`, where the unused terms are inert.
        //
        // The LIVE test is not pedantry. Admitting the family wholesale moves 373 systems across
        // 47 skins onto a different colour path (the Ram fragment's ×2 `_MainColor` and highlight
        // compression) — a change unrelated to the dissolve feature, and measured: cello
        // 18.815 → 18.970. At `_Amount ≈ 0` the mask is 1 and the program collapses to plain
        // `2*color*tex`, so such a material renders the same either way in the game and there is
        // no reason to reroute it. Requiring a bound `_DissolveTex_01` with a non-zero
        // `_Amount_01` keeps this to the systems the feature actually changes.
        // Admit the `Dissolve/` family alongside `Ram/`, but ONLY the TWO-MAP form with a live
        // mask (see the `_01`/`_02` reads below and the notes in `spine.rs`).
        //
        // The single-map `(CustomData)` form is deliberately NOT admitted, and both halves of
        // that decision are measured:
        //   • `_UseDissolveTex` is not a liveness test. The mask is `(bw*sw + tex - t)/bw` with
        //     `sw = 1 - roundEven(t + 0.5)`; at `t = 0`, roundEven(0.5) is 0 (banker's rounding),
        //     so `sw = 1` and the mask is `clamp(1 + tex/bw)` — identically ONE however the
        //     toggle is set. Virtuosa's `window_bg_01` is exactly that: it binds a dissolve
        //     texture with `_UseDissolveTex = 1`, `_DissolveIntensity = 0` and no CustomData
        //     curve, so its mask cuts nothing. Its long-standing "dark quad over the window" is
        //     NOT an unread dissolve. Admitting on the toggle cost cello 18.815 → 18.970.
        //   • Gating on the threshold instead (`_DissolveIntensity > 0`) is correct about
        //     liveness and STILL a regression: it admits cello's `sys35` (a 2000 px normal-blend
        //     quad whose mask does bite) and costs 18.815 → 19.068, because moving a system onto
        //     the Ram colour path (its ×2 `_MainColor` and highlight compression) outweighs what
        //     the mask recovers. The two-map form does not have this problem — those systems are
        //     additive and small.
        // Revisit only with a way to apply the mask WITHOUT changing the colour path.
        let dissolve_live = shader.contains("Dissolve/")
            && mat_float(mat, "_Amount_01", 0.0) > 0.0
            && mat_texenv(all_objects, mat, "_DissolveTex_01").0.is_some();
        // Admit the PLAIN `Disturb/` family. The two paths disagreed about it: the SCENE path's
        // `is_l2d_compositor` has always accepted `Disturb/`, while this one did not, so the
        // same shader family was composited on a mesh quad and drawn as a FLAT SPRITE on a
        // particle. That is 1834 materials across 75 of 87 bundles, the third largest family
        // after Additive and AlphaBlend and more than double `Dissolve/`, and 60 of whitw2's
        // 180 exported systems, 21 of skadi2's 44.
        //
        // ⚠️ Admitting a family here is the move the `Dissolve/` notes above warn against,
        // because rerouting changes the COLOUR path. It is safe HERE and the difference is
        // derived from the decompiled fragment, not assumed: plain `Disturb(CustomData)` is the
        // Ram fragment with the ram multiply ABSENT (no `_RamTex` sampler at all) and the
        // disturb offset masked by `_WeightTex` and scaled by `_DisturbScale`. Its colour is the
        // identical `main * _MainColor * vs_COLOR0 * 2`, the identical `roundEven` dissolve on
        // alpha, and the identical `_Opacity`. So the Ram port reproduces it exactly once
        // `uHasRam` is 0, which it already is when no ramp is bound.
        //
        // Restricted to the two spellings that are actually used (1833 of the 1834). `Anchor`
        // subtracts an anchor before the intensity and `GrabPass` needs a grab texture, so
        // both are DIFFERENT programs and stay out.
        let plain_disturb = shader.contains("/Disturb/")
            && (shader.ends_with("/Disturb") || shader.ends_with("/Disturb(CustomData)"));
        // Admit `Dissolve/Dissolve Add UVTween` (2026-09-06), from its DECOMPILED program
        // (`probe_shadersrc`, pathID 2170655151988677161 in `[uc]shaders.ab`), which is a strict
        // subset of the Ram fragment: `mask = clamp((bw * (1 - roundEven(_Amount + 0.5)) +
        // (dissolve - _Amount)) / bw)`, `colour = main * 2 * (vertexColour * _TintColor)`,
        // alpha = mask * colour.a, and both lookups scrolled by `_Time.y * _UVTween` (main by
        // .xy, dissolve by .zw) under `fract`. No CustomData, no ram, no disturb, no opacity.
        // The single-map caveat above does not apply: its colour path IS the Ram colour path.
        // It was the largest class the built-in quad could not draw (58 systems, 41 on the quad:
        // Kal'tsit sale#14's stone flows and glass, Vina Victoria epoque#50's). The `AB` spelling
        // is a different pass state and its program is not read, so it stays out.
        let uv_tween = shader.ends_with("/Dissolve/Dissolve Add UVTween");
        (shader.contains("Ram/") || dissolve_live || plain_disturb || uv_tween)
            .then_some((mat, shader))
    })?;
    // DIAGNOSTIC (`DYNCHAR_PPCENSUS`, presence-checked): the port spelling behind every emitter
    // this path admits, joined to its GameObject name. The exported `kind` collapses the two
    // plain `Disturb/` spellings into one, so this is the only place the spelling is readable.
    if std::env::var("DYNCHAR_PPCENSUS").is_ok() {
        let go_name = all_objects
            .get(&go_pid)
            .and_then(|(_, go)| go.get("m_Name"))
            .and_then(Value::as_str)
            .unwrap_or("?");
        eprintln!(
            "PPCENSUS sysshader go {go_name} mat {:?} shader {shader}",
            mat.get("m_Name").and_then(Value::as_str).unwrap_or("?")
        );
    }

    let is_vertex = shader.contains("VertexDisturb");
    let is_dissolve = !shader.contains("Ram/") && shader.contains("Dissolve/");
    // `Dissolve Add UVTween` scrolls both lookups by `_Time.y * _UVTween` (a colour property:
    // main by xy, dissolve by zw) where the other families spell speeds as `_MainUSpeed` etc.
    let is_uv_tween = shader.ends_with("/Dissolve/Dissolve Add UVTween");
    let uv_tween = if is_uv_tween {
        mat_color(mat, "_UVTween", [0.0, 0.0, 0.0, 0.0])
    } else {
        [0.0, 0.0, 0.0, 0.0]
    };
    // Recomputed here because the closure above owns its own binding.
    let is_plain_disturb = shader.contains("/Disturb/")
        && (shader.ends_with("/Disturb") || shader.ends_with("/Disturb(CustomData)"));

    let (main_pid, main_val, main_st) = mat_texenv(all_objects, mat, "_MainTex");
    let (ram_pid, ram_val, ram_st) = mat_texenv(all_objects, mat, "_RamTex");
    let (dist_pid, dist_val, dist_st) = mat_texenv(all_objects, mat, "_DisturbTex");
    // `_WeightTex` MASKS the disturb offset per axis in the game: the plain
    // `Disturb/Disturb(CustomData)` fragment reads
    // `dOff = (custom + intensity) * disturb.x * texture(_WeightTex, wUV).xy`.
    // Only that variant declares it (16 mentions against 0 in both `Ram/` variants), so on a Ram
    // material the slot is unbound residue and this resolves to None, which the frontend treats as
    // a weight of 1 and is therefore a no-op. Mlynar binds three named masks
    // (`Mlynar_##_bg_01_mask`, `_bg_03_mask_01`, `_bg_03_mask_02`) and Ch'en the Holungday four
    // external ones with an authored ST, so it is neither placeholder nor rare.
    let (weight_pid, weight_val, weight_st) = mat_texenv(all_objects, mat, "_WeightTex");
    // The `Dissolve/` family names its maps `_DissolveTex_01`/`_02`; its single-name
    // `_DissolveTex`/`_Amount`/`_BorderWidth` are inert residue from the Standard shader the
    // asset was authored against. Prefer the two-map names wherever `_01` resolves.
    let t01 = mat_texenv(all_objects, mat, "_DissolveTex_01");
    let two_map = t01.0.is_some();
    let (diss_pid, diss_val, diss_st) = if two_map {
        t01
    } else {
        mat_texenv(all_objects, mat, "_DissolveTex")
    };
    let (diss2_pid, diss2_val, diss2_st) = if two_map {
        mat_texenv(all_objects, mat, "_DissolveTex_02")
    } else {
        (None, None, ST_IDENTITY)
    };
    let (vd_pid, vd_val, _vd_st) = if is_vertex {
        mat_texenv(all_objects, mat, "_VertexDisturbTex")
    } else {
        (None, None, ST_IDENTITY)
    };
    let (vdw_pid, vdw_val, _vdw_st) = if is_vertex {
        mat_texenv(all_objects, mat, "_VertexDisturbWeightTex")
    } else {
        (None, None, ST_IDENTITY)
    };

    // DIAGNOSTIC (`SCENE_ATTRIB=1`): a particle system that resolves NO `_MainTex` draws its
    // geometry untextured — for a MESH renderer that means flat bars instead of the authored
    // strip, i.e. the right total energy in the wrong profile. Report the material, its shader
    // and which slots did resolve, so an unresolved texture can be told from a material that
    // genuinely binds none.
    if main_pid.is_none() && std::env::var("SCENE_ATTRIB").is_ok() {
        eprintln!(
            "  [ptcl-notex] mat='{}' shader='{}' ram={} dist={} diss={} | m_TexEnvs keys: {:?}",
            mat.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
            mat.get("_shaderName")
                .and_then(Value::as_str)
                .unwrap_or("?"),
            ram_pid.is_some(),
            dist_pid.is_some(),
            diss_pid.is_some(),
            mat.get("m_SavedProperties")
                .and_then(|sp| sp.get("m_TexEnvs"))
                .and_then(|t| t.as_object())
                .map(|o| o.keys().cloned().collect::<Vec<_>>()),
        );
    }

    let vd_intensity = mat_color(mat, "_VertexDisturbIntensity", [0.0, 0.0, 0.0, 0.0]);

    // `Dissolve/` multiplies `_TintColor` into the vertex colour where `Ram/` uses
    // `_MainColor` — the same role, and the shared `col += col` supplies the family's ×2, so
    // the 0.5 half-neutral convention carries over unchanged.
    let main_color = if is_dissolve {
        mat_color(mat, "_TintColor", [0.5, 0.5, 0.5, 0.5])
    } else {
        mat_color(mat, "_MainColor", [0.5, 0.5, 0.5, 0.5])
    };
    // The `_Start` clip's animated `_MainColor`, resolved onto the static colour exactly
    // as the scene-quad path resolves a layer's tint (`layer_color_curve`). `tint_scale`
    // is 1 — the Ram fragment applies the family's own `col += col` at draw time, so the
    // ×2 must NOT be baked in here — and RGB stays HDR-unclamped so an over-bright ramp
    // survives to the frontend's half-float target instead of collapsing onto a ceiling.
    let main_color_curve = color_channels.and_then(|chs| {
        let (props, _) = super::spine::material_color_props(mat);
        super::anim::layer_color_curve(
            chs,
            &props,
            Some("_MainColor"),
            [
                main_color[0] as f32,
                main_color[1] as f32,
                main_color[2] as f32,
                main_color[3] as f32,
            ],
            1.0,
            true,
            additive,
        )
    });

    let json = json!({
        "kind": if is_uv_tween { "uvTween" } else if is_dissolve { "dissolve" } else if is_vertex { "vertexDisturb" } else if is_plain_disturb { "plainDisturb" } else { "disturb" },
        "mainTex": Value::Null,     "mainST": main_st,
        "ramTex": Value::Null,      "ramST": ram_st,
        "disturbTex": Value::Null,  "disturbST": dist_st,
        "weightTex": Value::Null,   "weightST": weight_st,
        "dissolveTex": Value::Null, "dissolveST": diss_st,
        "dissolveTex2": Value::Null, "dissolveST2": diss2_st,
        "amount2": if two_map { mat_float(mat, "_Amount_02", 0.0) } else { 0.0 },
        "borderWidth2": if two_map { mat_float(mat, "_BorderWidth_02", 0.1) } else { 0.1 },
        "mainColor": main_color,
        "mainColorCurve": main_color_curve.map(|c| {
            c.into_iter()
                .map(|(t, v)| json!([t, v[0], v[1], v[2], v[3]]))
                .collect::<Vec<_>>()
        }),
        "opacity": mat_float(mat, "_Opacity", 1.0),
        "borderWidth": if two_map { mat_float(mat, "_BorderWidth_01", 0.1) } else { mat_float(mat, "_BorderWidth", 0.1) },
        "amount": if two_map { mat_float(mat, "_Amount_01", 0.5) } else { mat_float(mat, "_Amount", 0.5) },
        "intensityU": mat_float(mat, "_IntensityU", 0.0),
        "intensityV": mat_float(mat, "_IntensityV", 0.0),
        "disturbInfluenceDissolveUV": mat_float(mat, "_DisturbInfluenceDissolveUV", 0.0),
        // The SAME quantity under two names. The Ram variant scales the disturb offset into the
        // main UV with `_DisturbInfluenceMainUV`; the plain Disturb variant spells it
        // `_DisturbScale` (`u_xlat0.xy * vec2(_DisturbScale) + vs_TEXCOORD0.xy`). Reading the
        // Ram spelling on a plain material silently yields the 1.0 default.
        "disturbInfluenceMainUV": if is_plain_disturb {
            mat_float(mat, "_DisturbScale", 1.0)
        } else {
            mat_float(mat, "_DisturbInfluenceMainUV", 1.0)
        },
        "mainSpeed": if is_uv_tween { [uv_tween[0], uv_tween[1]] } else { [mat_float(mat, "_MainUSpeed", 0.0), mat_float(mat, "_MainVSpeed", 0.0)] },
        // Per-lookup UV ROTATION, `[main, dissolve, ram, disturb]` in DEGREES.
        //
        // The SCENE path has carried this since `SceneRam::uv_rot`; the particle path did not,
        // and the two are NOT redundant. Splitting the 100 `_HG_UV_ROTATION` materials by
        // renderer kind shows four skins whose rotating objects are ALL ParticleSystemRenderers
        // and which the scene-only port therefore could not reach at all: nian `cfa#1` 17
        // particle / 0 mesh, amiya3 5 / 0, texas2 `epoque#36` 1 / 0, ling `nian#12` 1 / 0.
        // Cello is 3 / 13, so she was only ever half-corrected.
        //
        // Same source as the scene side: the material never serializes `_Rotation0..3`, a
        // MonoBehaviour writes them at runtime, so this reads the component off the GameObject.
        "uvRot": super::spine::uv_rotation_of_go(all_objects, go_pid),
        "dissolveSpeed": if is_uv_tween { [uv_tween[2], uv_tween[3]] } else { [mat_float(mat, "_DissolveUSpeed", 0.0), mat_float(mat, "_DissolveVSpeed", 0.0)] },
        "disturbSpeed": [mat_float(mat, "_DisturbUSpeed", 0.0), mat_float(mat, "_DisturbVSpeed", 0.0)],
        "vertexDisturbTex": Value::Null,
        "vertexDisturbWeightTex": Value::Null,
        "vertexDisturbIntensity": [vd_intensity[0], vd_intensity[1], vd_intensity[2]],
        "vertexDisturbSpeed": [mat_float(mat, "_VertexDisturbUSpeed", 0.0), mat_float(mat, "_VertexDisturbVSpeed", 0.0)],
    });

    Some(RamData {
        json,
        texs: vec![
            ("mainTex", main_pid, main_val),
            ("ramTex", ram_pid, ram_val),
            ("disturbTex", dist_pid, dist_val),
            ("weightTex", weight_pid, weight_val),
            ("dissolveTex", diss_pid, diss_val),
            ("dissolveTex2", diss2_pid, diss2_val),
            ("vertexDisturbTex", vd_pid, vd_val),
            ("vertexDisturbWeightTex", vdw_pid, vdw_val),
        ],
    })
}

/// Resolve the renderer's trail material (index 1 in `m_Materials`) into
/// `(trail_texture, alpha_texture, pid, additive)`. A renderer with a trail
/// carries a distinct 2nd material for it; `(None, .., false)` when absent.
fn resolve_trail_material(
    all_objects: &HashMap<i64, (i32, Value)>,
    renderer: Option<&Value>,
) -> (Option<Value>, Option<Value>, Option<i64>, bool, [f64; 4]) {
    let mat_ref = renderer
        .and_then(|r| r.get("m_Materials"))
        .and_then(Value::as_array)
        .and_then(|m| m.get(1));
    match mat_ref.and_then(|mr| resolve_material(all_objects, mr)) {
        Some((tv, ta, pid, add, st, _, _)) => (Some(tv), ta, Some(pid), add, st),
        None => (None, None, None, false, ST_IDENTITY),
    }
}

/// Reduce a Unity `NoiseModule` to the schema's `noise` object. All authored
/// `MinMaxCurve`s are flattened to their representative (max-endpoint) scalar.
fn parse_noise(nm: &Value, inv_scale: f64) -> Value {
    let repr = |k: &str, scale: f64, d: f64| nm.get(k).map_or(d, |v| mmscalar_repr(v, scale));
    json!({
        // Position displacement amplitude → px.
        "strength": repr("strength", inv_scale, 0.0),
        // Spatial frequency in Unity units; left unscaled (frontend scales).
        "frequency": fd(nm, "frequency", 0.0),
        // Field scroll speed over time.
        "scrollSpeed": repr("scrollSpeed", 1.0, 0.0),
        "octaves": i(nm, "octaves").unwrap_or(1),
        "damping": b(nm, "damping", true),
        "sizeAmount": repr("sizeAmount", 1.0, 0.0),
        // Authored in radians/sec → deg/sec.
        "rotationAmount": repr("rotationAmount", RAD_TO_DEG, 0.0),
    })
}

/// Reduce a Unity `TrailModule` to the schema's `trail` object. `blend` is the trail
/// material's blend class. The `tex` index is left null here and filled by
/// [`export_particles`] after texture dedup.
///
/// Both `ParticleSystemTrailMode`s are exported. **`PerParticle`** (0) drags one ribbon
/// behind each particle. **Ribbon** (1) is a different primitive: ONE polyline threaded
/// through the system's live particles ordered by age, `ribbonCount` of them interleaved.
/// Mode 1 used to be dropped to `trail: null`, which silently deleted every emitter whose
/// renderer is also `RenderMode.None` — the head is suppressed AND the only thing that
/// would have drawn is gone, so the system is exported, gated, budgeted and simulated
/// while painting nothing. Skadi the Corrupting Heart's `hongxian_01` (红线, "red thread")
/// is one: a `colorOverTrail` coral ribbon that is the whole of her missing t=13 line.
fn parse_trail(tm: &Value, inv_scale: f64, blend: bool) -> Value {
    // widthOverTrail multiplies the trail's width ALONG the ribbon (0 = start, 1 = end),
    // on top of the particle size when `sizeAffectsWidth`. It is a ratio, so it takes no
    // px scaling. Null for the default const 1.0 (→ "just use the particle size").
    let width = tm.get("widthOverTrail").map_or(Value::Null, |v| {
        if i(v, "minMaxState").unwrap_or(0) == 0 && (fd(v, "scalar", 1.0) - 1.0).abs() < 1e-6 {
            Value::Null
        } else {
            mmscalar(v, 1.0)
        }
    });
    // A gradient/colour module reduced to the schema's colour, or null when it is the
    // default white constant (the trail then inherits the particle's own colour).
    let reduce_color = |g: &Value| match i(g, "minMaxState").unwrap_or(0) {
        0 | 2 => {
            let c = g.get("maxColor").map_or([1.0; 4], read_color);
            if c.iter().all(|x| (*x - 1.0).abs() < 1e-6) {
                Value::Null
            } else {
                json!({ "mode": "color", "r": c[0], "g": c[1], "b": c[2], "a": c[3] })
            }
        }
        _ => mmgradient(g),
    };
    json!({
        "tex": Value::Null, // resolved in export_particles
        "mode": if i(tm, "mode").unwrap_or(0) == 1 { "ribbon" } else { "perParticle" },
        // How many interleaved ribbons the live particles are split across (ribbon mode).
        "ribbonCount": i(tm, "ribbonCount").unwrap_or(1).max(1),
        "blend": if blend { "additive" } else { "normal" },
        "ratio": fd(tm, "ratio", 1.0),
        "lifetime": tm.get("lifetime").map_or(1.0, |v| mmscalar_repr(v, 1.0)),
        "minVertexDistance": fd(tm, "minVertexDistance", 0.0) * inv_scale,
        "widthOverTrail": width,
        // Along the PARTICLE's life…
        "colorOverLifetime": tm.get("colorOverLifetime").map_or(Value::Null, reduce_color),
        // …and along the RIBBON. Distinct modules: a ribbon can be banded head-to-tail
        // while every particle feeding it is white, which is exactly how the coral is
        // authored — neither the texture (grey) nor the material (`_TintColor` white)
        // carries it, so dropping this module loses the colour entirely.
        "colorOverTrail": tm.get("colorOverTrail").map_or(Value::Null, reduce_color),
        "sizeAffectsWidth": b(tm, "sizeAffectsWidth", true),
        "inheritParticleColor": b(tm, "inheritParticleColor", false),
        "dieWithParticles": b(tm, "dieWithParticles", true),
        "worldSpace": b(tm, "worldSpace", false),
    })
}

/// Write `<name>[particles].json` + deduped `<name>[particles]/<i>.png`. Returns
/// the number of files written. Textures are deduped by source `path_id`, mirror
/// of [`super::spine`]'s scene export.
#[must_use]
// Grouping these into a params struct would touch every call site for a purely cosmetic gain;
// the 8 arguments are each independently meaningful inputs to the export.
#[allow(clippy::too_many_arguments)]
pub fn export_particles(
    name: &str,
    particles: &[ParticleData],
    spine_dir: &Path,
    skel_scale: f64,
    camera_size: Option<f64>,
    character_sort: Option<i64>,
    // Sorting orders of the skeleton's SEPARATOR PARTS, ascending. A sheet whose own sort falls
    // BETWEEN two consecutive parts is drawn in that gap by the game; one above the last part is
    // genuinely in front of everything. Empty when the skin has no separator.
    separator_part_sorts: &[i64],
    resources: &HashMap<String, Vec<u8>>,
) -> usize {
    if particles.is_empty() || skel_scale == 0.0 {
        return 0;
    }
    let inv = 1.0 / skel_scale;

    let tex_dir = spine_dir.join(format!("{name}[particles]"));
    std::fs::create_dir_all(&tex_dir).ok();

    let mut tex_index: HashMap<i64, usize> = HashMap::new();
    // Unity's authored per-texture wrap mode (`m_TextureSettings.m_WrapU`), indexed the same way
    // as the saved PNGs: 0 = Repeat, 1 = Clamp, 2 = Mirror. The Ram shader SCROLLS and TILES its
    // samplers, so it reads UVs far outside [0, 1]; Pixi defaults every texture to CLAMP, which
    // edge-smears there and freezes such an emitter into a flat field. Blanket-REPEAT is not the
    // answer either — these bundles genuinely author both (Civilight Eterna's `bg_window_01_1`
    // is Clamp while her background sheets are Repeat), and forcing REPEAT everywhere costs
    // Virtuosa 17.630 -> 17.762. Ship the authored value.
    let mut tex_wrap: Vec<i64> = Vec::new();
    let mut next_idx = 0usize;
    let mut saved = 0usize;
    let mut systems: Vec<Value> = Vec::with_capacity(particles.len());

    // Decode + save a texture once per source path_id, returning its shared
    // index. Used for both the particle texture and the trail texture so they
    // share one `[particles]/<i>.png` dedup set.
    let mut resolve_tex =
        |pid: Option<i64>, tex_val: &Option<Value>, alpha_val: &Option<Value>| -> Option<usize> {
            // DIAGNOSTIC (`SCENE_ATTRIB=1`): distinguish the three ways this returns None —
            // no PPtr at all, a PPtr that did not dereference (external bundle not staged), and
            // a decode failure. A system that loses its texture draws untextured, which for a
            // MESH renderer is flat bars instead of the authored strip.
            let dbg_notex = std::env::var("SCENE_ATTRIB").is_ok();
            if dbg_notex && (pid.is_none() || tex_val.is_none()) {
                eprintln!("  [ptcl-tex] MISS pid={pid:?} deref={}", tex_val.is_some());
            }
            let (pid, tex_val) = (pid?, tex_val.as_ref()?);
            if let Some(&idx) = tex_index.get(&pid) {
                return Some(idx);
            }
            let decoded = decode_texture_object(tex_val, resources);
            if dbg_notex && !matches!(decoded, Ok(Some(_))) {
                eprintln!(
                    "  [ptcl-tex] DECODE-FAIL pid={pid} name={:?}",
                    tex_val.get("m_Name").and_then(Value::as_str)
                );
            }
            let Ok(Some(mut tex)) = decoded else {
                return None;
            };
            if let Some(alpha_val) = alpha_val
                && let Ok(Some(alpha)) = decode_texture_object(alpha_val, resources)
            {
                tex = super::alpha_merge::combine_with_alpha(&tex, &alpha);
            }
            let idx = next_idx;
            if image::save_buffer(
                tex_dir.join(format!("{idx}.png")),
                &tex.rgba,
                tex.width,
                tex.height,
                image::ColorType::Rgba8,
            )
            .is_ok()
            {
                saved += 1;
            }
            if std::env::var("SCENE_ATTRIB").is_ok() {
                eprintln!(
                    "    [ptcl-tex] idx={idx} pid={pid} name={:?} {}x{}",
                    tex_val.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                    tex.width,
                    tex.height
                );
            }
            tex_index.insert(pid, idx);
            while tex_wrap.len() < idx {
                tex_wrap.push(0);
            }
            tex_wrap.push(
                tex_val
                    .pointer("/m_TextureSettings/m_WrapU")
                    .and_then(Value::as_i64)
                    .unwrap_or(0),
            );
            next_idx += 1;
            Some(idx)
        };

    for (sys_i, p) in particles.iter().enumerate() {
        let mut sys = p.json.clone();
        let tex_idx = resolve_tex(p.tex_pid, &p.tex_val, &p.alpha_val);
        if tex_idx.is_none() && std::env::var("SCENE_ATTRIB").is_ok() {
            eprintln!(
                "  [ptcl-tex] SYSTEM {sys_i} HAS NO TEXTURE  mode={:?} pid={:?} deref={}",
                sys.get("renderMode").and_then(Value::as_str),
                p.tex_pid,
                p.tex_val.is_some()
            );
        }
        sys["tex"] = match tex_idx {
            Some(idx) => json!(idx),
            None => Value::Null,
        };
        // Fill trail.tex when the system has a trail with a distinct material.
        if sys.get("trail").is_some_and(serde_json::Value::is_object) {
            let trail_idx = resolve_tex(p.trail_tex_pid, &p.trail_tex_val, &p.trail_alpha_val);
            sys["trail"]["tex"] = match trail_idx {
                Some(idx) => json!(idx),
                None => Value::Null,
            };
        }
        // Ram block: resolve each of its texture slots into the same dedup set.
        // `mainTex` shares `_MainTex`'s path_id with the top-level `tex`, so it
        // dedups to the same index.
        if let Some(ram) = &p.ram {
            let mut ram_json = ram.json.clone();
            for (key, pid, tex_val) in &ram.texs {
                let idx = resolve_tex(*pid, tex_val, &None);
                ram_json[*key] = match idx {
                    Some(idx) => json!(idx),
                    None => Value::Null,
                };
            }
            sys["ram"] = ram_json;
        }
        systems.push(sys);
    }

    let meta = json!({
        "coordinateSystem": "spine-authored pixels, Y-up, origin at skeleton root; Unity units ×(1/skeletonScale)",
        "skeletonScale": skel_scale,
        "cameraSize": camera_size,
        "cameraSizePx": camera_size.map(|c| c * inv),
        "characterSort": character_sort,
        "separatorPartSorts": separator_part_sorts,
        "textureCount": next_idx,
        "textureWrap": tex_wrap,
        "systems": systems,
    });
    if let Ok(text) = serde_json::to_string(&meta)
        && std::fs::write(spine_dir.join(format!("{name}[particles].json")), text).is_ok()
    {
        saved += 1;
    }
    saved
}
