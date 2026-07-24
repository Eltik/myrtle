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
    /// Trail material texture (renderer's 2nd material), deduped into the same
    /// `[particles]/<i>.png` set; index written to `trail.tex`.
    pub trail_tex_val: Option<Value>,
    pub trail_alpha_val: Option<Value>,
    pub trail_tex_pid: Option<i64>,
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

/// Sample a Unity `AnimationCurve` (`m_Curve` keyframe array) into `[{t,v}]`,
/// applying `scale` to the value. Linear sample of the keyframe times/values
/// (Bezier in/out tangents are ignored — an accepted approximation).
fn sample_curve(curve: &Value, mul: f64, scale: f64) -> Vec<Value> {
    curve
        .get("m_Curve")
        .and_then(Value::as_array)
        .map(|kfs| {
            kfs.iter()
                .map(|kf| {
                    let t = fd(kf, "time", 0.0);
                    let val = fd(kf, "value", 0.0) * mul * scale;
                    json!({ "t": t, "v": val })
                })
                .collect()
        })
        .unwrap_or_default()
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
        let c = g
            .get(format!("key{idx}"))
            .map(read_color)
            .unwrap_or([1.0; 4]);
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
    // maxGradient. (For the random/two-sided states we take the "max" side.)
    match i(v, "minMaxState").unwrap_or(0) {
        0 | 2 => {
            let c = v.get("maxColor").map(read_color).unwrap_or([1.0; 4]);
            json!({ "mode": "color", "r": c[0], "g": c[1], "b": c[2], "a": c[3] })
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
fn shape_type_name(t: i64) -> &'static str {
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
fn render_mode_name(m: i64) -> &'static str {
    match m {
        1 => "stretch",
        4 => "mesh",
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
/// GameObjects (idle path), and effect-host transform (scale/position) curves. All
/// empty for the idle/main scene (which plays none of these clips).
pub(crate) struct EntranceCtx<'a> {
    pub windows: &'a HashMap<i64, super::anim::ActiveWindow>,
    pub rate_curves: &'a HashMap<i64, Vec<(f32, f32)>>,
    pub event_rate_gos: &'a std::collections::HashSet<i64>,
    pub transform_curves: &'a HashMap<i64, super::anim::EntranceTransform>,
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

        gated.sort_by(|a, b| a.total_cmp(b));
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
/// Emits are skipped (and counted by the caller) when: the InitialModule is
/// disabled, the emitter GameObject is inactive, the renderer is disabled, or the
/// EmissionModule is disabled with no bursts (nothing is emitted).
#[must_use]
pub(crate) fn collect_dynchar_particles(
    all_objects: &HashMap<i64, (i32, Value)>,
    inv_scale: f64,
    host: &BgParticleHost,
    entrance: &EntranceCtx<'_>,
    scope: &RootScope<'_>,
) -> (Vec<ParticleData>, usize) {
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
    let mut skipped = 0usize;

    for (_, ps) in systems {
        let Some(go_pid) = ps.get("m_GameObject").and_then(get_path_id) else {
            skipped += 1;
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
        let admit_cross_root = cross_root && transform_host.is_some();

        // Active up the whole hierarchy — excludes emitters under a state-gated
        // inactive group ("Start/Interact/Special Only Effects"), which otherwise
        // all play at once in the idle scene (noise).
        if !admit_cross_root && !host.effectively_active(all_objects, go_pid) {
            skipped += 1;
            continue;
        }
        if cross_root && !admit_cross_root {
            skipped += 1;
            continue;
        }

        let initial = ps.get("InitialModule").cloned().unwrap_or(Value::Null);
        if !b(&initial, "enabled", false) {
            skipped += 1;
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
            skipped += 1; // nothing is ever emitted
            continue;
        }
        // Event-clip-driven rate with no bursts: quiet at the steady state (the rate
        // is exported as 0 below) — nothing would ever spawn, so drop the system.
        if event_rate_gos.contains(&go_pid) && bursts_raw.is_empty() {
            skipped += 1;
            continue;
        }

        // Renderer (blend / sort / render mode / material texture).
        let renderer = go_to_renderer.get(&go_pid).copied();
        if let Some(r) = renderer
            && !b(r, "m_Enabled", true)
        {
            skipped += 1;
            continue;
        }

        let sort = renderer.and_then(|r| i(r, "m_SortingOrder")).unwrap_or(0);
        let render_mode =
            render_mode_name(renderer.and_then(|r| i(r, "m_RenderMode")).unwrap_or(0));

        // Resolve the first material's _MainTex (+ optional _AlphaTex), blend, tiling.
        let (tex_val, alpha_val, tex_pid, blend, main_st) =
            resolve_renderer_texture(all_objects, renderer);

        // Ram shader family (`Ram/Disturb` / `Ram/VertexDisturb`): a ramp-tint +
        // dissolve + UV-disturb compositor sampling 4–6 textures. Its params +
        // extra texture slots are exported in a `"ram"` block; the textures share
        // the same `[particles]/<i>.png` dedup set as `tex`.
        let ram = resolve_ram(all_objects, renderer);

        // Emitter world transform (spine-root frame) → position (px) + Z rot.
        let world = host.world_of_go(all_objects, go_pid);
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
        let wsx = 0.5
            * (f64::from(ex[0] - origin[0]).hypot(f64::from(ex[1] - origin[1]))
                + f64::from(ey[0] - origin[0]).hypot(f64::from(ey[1] - origin[1])));
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
        let shape_type_int = ps.get("ShapeModule").and_then(|s| i(s, "type")).unwrap_or(-1);
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
            "name": ps.get("m_Name").and_then(Value::as_str).unwrap_or(""),
            "sort": sort,
            "blend": if blend { "additive" } else { "normal" },
            "renderMode": render_mode,
            "pos": pos,
            "rot": rot,
            "duration": fd(ps, "lengthInSec", 1.0),
            "looping": b(ps, "looping", false),
            "simulationSpace": if i(ps, "moveWithTransform").unwrap_or(1) == 1 { "local" } else { "world" },
            "maxParticles": i(&initial, "maxNumParticles").unwrap_or(1000),
            "boneChain": bone_chain,
        });
        // Cinematic start delay (omitted when 0 to keep always-on scenes lean).
        if delay > 0.0 {
            sys["delay"] = json!(delay);
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
        if let Some(s) = initial.get("startSize") {
            sys["startSize"] = mmscalar(s, em_inv);
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
        // size, so the raw local geometry needs no extra scale here). Only inline
        // meshes resolve (empty resources, like the scene bg quads at spine.rs);
        // a streamed/compressed mesh yields None → the system stays skipped by the
        // frontend, exactly as before. Bounded to guard against a pathological mesh.
        if render_mode == "mesh"
            && let Some(mesh_pid) = renderer.and_then(|r| r.get("m_Mesh")).and_then(get_path_id)
            && mesh_pid != 0
            && let Some((43, mesh_val)) = all_objects.get(&mesh_pid).map(|(c, v)| (*c, v))
            && let Some(m) = super::mesh::parse_mesh(mesh_val, &HashMap::new())
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
            emission
                .get("rateOverTime")
                .map(|r| mmscalar(r, 1.0))
                .unwrap_or_else(|| json!({ "mode": "const", "v": 0.0 }))
        };
        let bursts: Vec<Value> = bursts_raw
            .iter()
            .map(|bu| {
                let count = bu
                    .get("countCurve")
                    .map(|c| mmscalar_repr(c, 1.0))
                    .unwrap_or(0.0)
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
        if let Some(cdm) = ps.get("CustomDataModule")
            && b(cdm, "enabled", false)
        {
            for stream in 0..2 {
                if i(cdm, &format!("mode{stream}")).unwrap_or(0) != 1 {
                    continue;
                }
                if let Some(v0) = cdm.get(format!("vector{stream}_0").as_str())
                    && let Some(mc) = v0.get("maxCurve")
                {
                    let scalar = fd(v0, "scalar", 1.0);
                    let pts = sample_curve(mc, if scalar != 0.0 { scalar } else { 1.0 }, 1.0);
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
            let scale = shape.get("m_Scale");
            let box_wh = [
                scale.and_then(|s| f(s, "x")).unwrap_or(1.0) * em_inv,
                scale.and_then(|s| f(s, "y")).unwrap_or(1.0) * em_inv,
            ];
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

        // Velocity over lifetime (flattened to representative x/y in px/s).
        if let Some(vm) = ps.get("VelocityModule")
            && b(vm, "enabled", false)
        {
            let vx = vm.get("x");
            let vy = vm.get("y");
            let nonzero = vx.is_some_and(|v| !mmscalar_is_zero(v))
                || vy.is_some_and(|v| !mmscalar_is_zero(v));
            if nonzero {
                // World-space velocity is already in the skeleton world frame
                // (inv_scale); local-space velocity is in the emitter frame (em_inv).
                let vscale = if b(vm, "inWorldSpace", false) {
                    inv_scale
                } else {
                    em_inv
                };
                sys["velocityOverLife"] = json!({
                    "x": vx.map(|v| mmscalar_repr(v, vscale)).unwrap_or(0.0),
                    "y": vy.map(|v| mmscalar_repr(v, vscale)).unwrap_or(0.0),
                    "space": if b(vm, "inWorldSpace", false) { "world" } else { "local" },
                });
            } else {
                sys["velocityOverLife"] = Value::Null;
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
            if nonzero {
                let fscale = if b(fm, "inWorldSpace", false) {
                    inv_scale
                } else {
                    em_inv
                };
                sys["forceOverLife"] = json!({
                    "x": fx.map(|v| mmscalar_repr(v, fscale)).unwrap_or(0.0),
                    "y": fy.map(|v| mmscalar_repr(v, fscale)).unwrap_or(0.0),
                    "space": if b(fm, "inWorldSpace", false) { "world" } else { "local" },
                });
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
        if let Some(rm) = ps.get("RotationModule")
            && b(rm, "enabled", false)
            && let Some(curve) = rm.get("curve")
        {
            sys["rotOverLifeDegPerSec"] = json!(mmscalar_repr(curve, RAD_TO_DEG));
        } else {
            sys["rotOverLifeDegPerSec"] = Value::Null;
        }

        // Texture sheet animation.
        if let Some(uv) = ps.get("UVModule")
            && b(uv, "enabled", false)
        {
            let frame = uv.get("frameOverTime");
            let frame_curve = match frame {
                Some(fr) if i(fr, "minMaxState").unwrap_or(0) == 1 => {
                    let scalar = fd(fr, "scalar", 1.0);
                    fr.get("maxCurve")
                        .map(|mc| json!(sample_curve(mc, scalar, 1.0)))
                        .unwrap_or(Value::Null)
                }
                _ => Value::Null,
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

        // Trail (PerParticle mode only; Ribbon mode 1 is skipped → null). The
        // trail texture is the renderer's 2nd material, deduped in export.
        let (trail_tex_val, trail_alpha_val, trail_tex_pid) = if let Some(tm) =
            ps.get("TrailModule")
            && b(tm, "enabled", false)
            && i(tm, "mode").unwrap_or(0) == 0
        {
            let (tv, ta, tpid, tadd) = resolve_trail_material(all_objects, renderer);
            sys["trail"] = parse_trail(tm, em_inv, tadd);
            (tv, ta, tpid)
        } else {
            sys["trail"] = Value::Null;
            (None, None, None)
        };

        out.push(ParticleData {
            json: sys,
            tex_val,
            alpha_val,
            tex_pid,
            trail_tex_val,
            trail_alpha_val,
            trail_tex_pid,
            ram,
        });
    }

    if entrance.is_entrance {
        apply_followbone_reveal_inheritance(&mut out);
    }

    (out, skipped)
}

/// `(main_tex, alpha_tex, main_pid, additive, main_st)` of a resolved material.
type ResolvedMaterial = (Value, Option<Value>, i64, bool, [f64; 4]);

/// Resolve one material reference into [`ResolvedMaterial`].
/// Returns `None` when the ref is null, not a Material(21), or its `_MainTex`
/// lives in another bundle (unresolvable).
fn resolve_material(
    all_objects: &HashMap<i64, (i32, Value)>,
    mat_ref: &Value,
) -> Option<ResolvedMaterial> {
    let mat_pid = get_path_id(mat_ref).filter(|&p| p != 0)?;
    let (21, mat) = all_objects.get(&mat_pid)? else {
        return None;
    };
    let tex_envs = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_TexEnvs"))
        .and_then(Value::as_object)?;
    let main_pid = tex_envs
        .get("_MainTex")
        .and_then(|t| t.get("m_Texture"))
        .and_then(get_path_id)
        .filter(|&p| p != 0)?;
    let (28, tex_val) = all_objects.get(&main_pid)? else {
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
    Some((
        tex_val.clone(),
        alpha_val,
        main_pid,
        is_additive(mat),
        main_st,
    ))
}

/// Resolve the renderer's first usable material (the particle's own texture)
/// into `(main_texture, alpha_texture, main_pid, additive, main_st)`.
fn resolve_renderer_texture(
    all_objects: &HashMap<i64, (i32, Value)>,
    renderer: Option<&Value>,
) -> (Option<Value>, Option<Value>, Option<i64>, bool, [f64; 4]) {
    let Some(materials) = renderer
        .and_then(|r| r.get("m_Materials"))
        .and_then(Value::as_array)
    else {
        return (None, None, None, false, ST_IDENTITY);
    };
    for mat_ref in materials {
        if let Some((tv, ta, pid, add, st)) = resolve_material(all_objects, mat_ref) {
            return (Some(tv), ta, Some(pid), add, st);
        }
    }
    (None, None, None, false, ST_IDENTITY)
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
fn mat_color(mat: &Value, name: &str, default: [f64; 4]) -> [f64; 4] {
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
/// `(path_id, Texture2D value, [scaleX,scaleY,offsetX,offsetY])`. The path_id /
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
) -> Option<RamData> {
    let materials = renderer?.get("m_Materials")?.as_array()?;
    let (mat, shader) = materials.iter().find_map(|mat_ref| {
        let mat_pid = get_path_id(mat_ref).filter(|&p| p != 0)?;
        let (21, mat) = all_objects.get(&mat_pid)? else {
            return None;
        };
        let shader = mat.get("_shaderName").and_then(Value::as_str)?;
        shader.contains("Ram/").then_some((mat, shader))
    })?;

    let is_vertex = shader.contains("VertexDisturb");

    let (main_pid, main_val, main_st) = mat_texenv(all_objects, mat, "_MainTex");
    let (ram_pid, ram_val, ram_st) = mat_texenv(all_objects, mat, "_RamTex");
    let (dist_pid, dist_val, dist_st) = mat_texenv(all_objects, mat, "_DisturbTex");
    let (diss_pid, diss_val, diss_st) = mat_texenv(all_objects, mat, "_DissolveTex");
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

    let vd_intensity = mat_color(mat, "_VertexDisturbIntensity", [0.0, 0.0, 0.0, 0.0]);

    let json = json!({
        "kind": if is_vertex { "vertexDisturb" } else { "disturb" },
        "mainTex": Value::Null,     "mainST": main_st,
        "ramTex": Value::Null,      "ramST": ram_st,
        "disturbTex": Value::Null,  "disturbST": dist_st,
        "dissolveTex": Value::Null, "dissolveST": diss_st,
        "mainColor": mat_color(mat, "_MainColor", [0.5, 0.5, 0.5, 0.5]),
        "opacity": mat_float(mat, "_Opacity", 1.0),
        "borderWidth": mat_float(mat, "_BorderWidth", 0.1),
        "amount": mat_float(mat, "_Amount", 0.5),
        "intensityU": mat_float(mat, "_IntensityU", 0.0),
        "intensityV": mat_float(mat, "_IntensityV", 0.0),
        "disturbInfluenceDissolveUV": mat_float(mat, "_DisturbInfluenceDissolveUV", 0.0),
        "disturbInfluenceMainUV": mat_float(mat, "_DisturbInfluenceMainUV", 1.0),
        "mainSpeed": [mat_float(mat, "_MainUSpeed", 0.0), mat_float(mat, "_MainVSpeed", 0.0)],
        "dissolveSpeed": [mat_float(mat, "_DissolveUSpeed", 0.0), mat_float(mat, "_DissolveVSpeed", 0.0)],
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
            ("dissolveTex", diss_pid, diss_val),
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
) -> (Option<Value>, Option<Value>, Option<i64>, bool) {
    let mat_ref = renderer
        .and_then(|r| r.get("m_Materials"))
        .and_then(Value::as_array)
        .and_then(|m| m.get(1));
    match mat_ref.and_then(|mr| resolve_material(all_objects, mr)) {
        Some((tv, ta, pid, add, _st)) => (Some(tv), ta, Some(pid), add),
        None => (None, None, None, false),
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

/// Reduce a Unity `TrailModule` (PerParticle mode only) to the schema's `trail`
/// object. `blend` is the trail material's blend class. The `tex` index is left
/// null here and filled by [`export_particles`] after texture dedup.
fn parse_trail(tm: &Value, inv_scale: f64, blend: bool) -> Value {
    // widthOverTrail: a multiplier of particle size. Emit null for the default
    // const 1.0 (→ "use particle size"), else the representative scalar ×scale.
    let width = tm.get("widthOverTrail").map_or(Value::Null, |v| {
        if i(v, "minMaxState").unwrap_or(0) == 0 && (fd(v, "scalar", 1.0) - 1.0).abs() < 1e-6 {
            Value::Null
        } else {
            json!(mmscalar_repr(v, inv_scale))
        }
    });
    // colorOverLifetime: null when it's the default white constant (trail then
    // inherits the particle colour), else the reduced gradient / colour.
    let color = tm.get("colorOverLifetime").map_or(Value::Null, |g| {
        match i(g, "minMaxState").unwrap_or(0) {
            0 | 2 => {
                let c = g.get("maxColor").map(read_color).unwrap_or([1.0; 4]);
                if c.iter().all(|x| (*x - 1.0).abs() < 1e-6) {
                    Value::Null
                } else {
                    json!({ "mode": "color", "r": c[0], "g": c[1], "b": c[2], "a": c[3] })
                }
            }
            _ => mmgradient(g),
        }
    });
    json!({
        "tex": Value::Null, // resolved in export_particles
        "blend": if blend { "additive" } else { "normal" },
        "ratio": fd(tm, "ratio", 1.0),
        "lifetime": tm.get("lifetime").map_or(1.0, |v| mmscalar_repr(v, 1.0)),
        "minVertexDistance": fd(tm, "minVertexDistance", 0.0) * inv_scale,
        "widthOverTrail": width,
        "colorOverLifetime": color,
        "dieWithParticles": b(tm, "dieWithParticles", true),
        "worldSpace": b(tm, "worldSpace", false),
    })
}

/// Write `<name>[particles].json` + deduped `<name>[particles]/<i>.png`. Returns
/// the number of files written. Textures are deduped by source `path_id`, mirror
/// of [`super::spine`]'s scene export.
#[must_use]
pub fn export_particles(
    name: &str,
    particles: &[ParticleData],
    spine_dir: &Path,
    skel_scale: f64,
    camera_size: Option<f64>,
    character_sort: Option<i64>,
    resources: &HashMap<String, Vec<u8>>,
) -> usize {
    if particles.is_empty() || skel_scale == 0.0 {
        return 0;
    }
    let inv = 1.0 / skel_scale;

    let tex_dir = spine_dir.join(format!("{name}[particles]"));
    std::fs::create_dir_all(&tex_dir).ok();

    let mut tex_index: HashMap<i64, usize> = HashMap::new();
    let mut next_idx = 0usize;
    let mut saved = 0usize;
    let mut systems: Vec<Value> = Vec::with_capacity(particles.len());

    // Decode + save a texture once per source path_id, returning its shared
    // index. Used for both the particle texture and the trail texture so they
    // share one `[particles]/<i>.png` dedup set.
    let mut resolve_tex =
        |pid: Option<i64>, tex_val: &Option<Value>, alpha_val: &Option<Value>| -> Option<usize> {
            let (pid, tex_val) = (pid?, tex_val.as_ref()?);
            if let Some(&idx) = tex_index.get(&pid) {
                return Some(idx);
            }
            let Ok(Some(mut tex)) = decode_texture_object(tex_val, resources) else {
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
            tex_index.insert(pid, idx);
            next_idx += 1;
            Some(idx)
        };

    for p in particles {
        let mut sys = p.json.clone();
        let tex_idx = resolve_tex(p.tex_pid, &p.tex_val, &p.alpha_val);
        sys["tex"] = match tex_idx {
            Some(idx) => json!(idx),
            None => Value::Null,
        };
        // Fill trail.tex when the system has a trail with a distinct material.
        if sys.get("trail").is_some_and(|t| t.is_object()) {
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
        "textureCount": next_idx,
        "systems": systems,
    });
    if let Ok(text) = serde_json::to_string(&meta)
        && std::fs::write(spine_dir.join(format!("{name}[particles].json")), text).is_ok()
    {
        saved += 1;
    }
    saved
}
