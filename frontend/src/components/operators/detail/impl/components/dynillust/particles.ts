import * as PIXI from "pixi.js";
import type { IAnimationBounds } from "../chibi/helpers";
import { sampleColorCurve } from "./sceneMesh";

/**
 * Live particle simulator for a dynamic illustration's Unity ParticleSystems.
 *
 * The `dyn_illust_*` prefabs drive most of their motion with ParticleSystems
 * (bubbles, sparkles, embers, drifting motes) rather than mesh animation. The
 * unpacker exports an approximation of each emitter to `…[particles].json`
 * (+ `…[particles]/<n>.png`); we re-simulate them here as billboarded sprites in
 * the SAME "spine-authored pixels, Y-up, origin at skeleton root" space as the
 * scene meshes, so they composite correctly with the backdrop and the spine.
 *
 * This is an approximation of Unity's system, tuned for the common modules
 * (emission, shape, start speed/size/colour, colour/size-over-life, velocity,
 * rotation). A hard global particle cap keeps it cheap.
 */

// ---- exported JSON value shapes (see assets/.../particle_export_spec.md) ----

export type MMScalar = { mode: "const"; v: number } | { mode: "range"; min: number; max: number } | { mode: "curve"; curve: ICurvePoint[] } | { mode: "rangeCurve"; min: ICurvePoint[]; max: ICurvePoint[] };

interface ICurvePoint {
    t: number;
    v: number;
}

/** Unity `MinMaxGradient`. `twoColors` is `RandomBetweenTwoColors`: each particle
 *  draws ONE colour uniformly between the two authored endpoints at birth (Skadi2
 *  iteration's `xian` threads are authored red→cyan; taking only the "max" side
 *  locked every thread cyan). Constant/gradient states are unchanged. */
export type MMColor = { mode: "color"; r: number; g: number; b: number; a: number } | { mode: "twoColors"; min: IRGBA; max: IRGBA } | { mode: "gradient"; stops: IColorStop[] };

interface IColorStop {
    t: number;
    r: number;
    g: number;
    b: number;
    a: number;
}

export interface IParticleSystemData {
    name?: string;
    sort: number;
    /** `[particles]` texture index, or null when the material carries no `_MainTex` —
     *  an UNTEXTURED material, which Unity draws in its own colour (see the mesh-render
     *  handling in {@link loadParticles}). */
    tex: number | null;
    /** Mesh-render-mode geometry (ice crystals, ribbons): flat mesh-LOCAL triangle
     *  geometry the exporter emits for `renderMode:"mesh"` systems. Each particle
     *  instances it, scaled by the particle's size. Absent for billboard/stretch. */
    mesh?: { pos: number[]; uv: number[]; idx: number[]; col?: number[] };
    blend: "additive" | "normal";
    renderMode?: string;
    /** Unity `ParticleSystemRenderer.pivot` — the quad's pivot as a MULTIPLIER of particle
     *  size, `[x, y]` in Unity's Y-UP frame. Unity holds the pivot at the particle's position
     *  and rotates the quad about it, so this both offsets the sprite and moves its rotation
     *  centre. Absent (the overwhelming majority) means the centred default. */
    pivot?: [number, number] | null;
    pos: [number, number];
    rot?: number;
    duration: number;
    looping: boolean;
    /** Cinematic start delay (s): the emitter stays dormant until the system's elapsed
     *  time reaches this, then begins as if just created. The `_Start` cinematic gates its
     *  effect groups (`_delayTime` activators + `m_IsActive` switch-ons) — e.g. the apple's
     *  golden sparks fire at ~6–7s when it falls, not at t=0. Absent/0 = emit immediately. */
    delay?: number;
    /** Unity's main-module `simulationSpeed`: the multiple of REAL time this system's own
     *  clock advances at. It scales everything downstream of the clock — the emission
     *  interval, particle age, every over-lifetime curve, rotation and velocity — so a
     *  system authored at 0.3 ran 3.3x too fast while we ignored it. Absent = 1 (Unity's
     *  default), which is the overwhelming majority. */
    simSpeed?: number;
    /** Unity's main-module `prewarm`: a LOOPING system opens already in steady state, as if
     *  it had run one full `duration` before t=0, instead of building up from empty.
     *  Absent = false. */
    prewarm?: boolean;
    simulationSpace?: "local" | "world";
    maxParticles: number;
    gravity?: MMScalar;
    lifetime: MMScalar;
    startSpeed: MMScalar;
    startSize: MMScalar;
    /** Unity `size3D`'s separate HEIGHT scalar (px). Exported only when the system
     *  authors a genuinely non-uniform particle, so the quad is `startSize` wide and
     *  `startSizeY` tall instead of square — Mlynar's `fangkuai_*` city slabs are
     *  0.5 × 2.0 units, a 1:4 bar that a single scalar renders 4× too short. */
    startSizeY?: MMScalar | null;
    startRotation?: MMScalar;
    startColor?: MMColor;
    /** The material's `_TintColor`, ALREADY doubled by the exporter — Torappu's ports of
     *  Unity's legacy particle shaders sample `2 × _TintColor × vertexColor × tex`, which
     *  is why every one of them declares the property's default as (0.5, 0.5, 0.5, 0.5).
     *  Exported only when the material is authored AWAY from that neutral (`null`
     *  otherwise), so every already-faithful system is untouched. Mlynar's `bao_01`
     *  transformation blast is (1, 1, 1, 1) — double-neutral in all four channels — so
     *  its additive output (`rgb × a`) is 4× what an untinted draw produces. */
    tint?: [number, number, number, number] | null;
    emission: { rate?: MMScalar; bursts?: { t: number; count: number }[] };
    /** The `_Start` cinematic's ANIMATED emission rate (particles/s over absolute
     *  cinematic seconds), when a clip drives `EmissionModule.rateOverTime` directly —
     *  Mlynar's sword-flourish confetti/stars have serialized rates of 60–150/s that
     *  the clips hold at 0 outside the ~7–13s flourish. Replaces the constant rate. */
    rateCurve?: ICurvePoint[] | null;
    /** `EmissionModule.rateOverDistance`, particles per PX of emitter travel — trail
     *  emission for rigs riding a moving anchor (Virtuosa's falling-apple comet dust,
     *  rate-over-TIME 0). Applied on the emitter container's per-frame movement. */
    rateOverDistance?: MMScalar | null;
    shape?: { type: string; radius?: number; radiusThickness?: number; angleDeg?: number; arcDeg?: number; box?: [number, number]; scale?: [number, number]; posOffset?: [number, number]; rotDeg?: number };
    /** Faithful screen-space (Y-up) cone emission direction for camera-facing (tilted)
     *  emitters whose flat `rot` is degenerate. Exported ONLY for such cones (Skadi2's
     *  `xiaoyu`/`guang` red streaks); when present the cone aims along it instead of the
     *  `rot`-derived "local +Y" direction. Absent → byte-identical legacy path. */
    emitDir?: [number, number] | null;
    /** Unity `StretchedBillboard` renderer scales (`renderMode:"stretch"` only). The quad is
     *  elongated along the particle's screen velocity to `|lengthScale|·size + velocityScale·speed`
     *  px (the px unit cancels, so no world→screen conversion). `lengthScale` is the size-proportional
     *  stretch (the dominant term for nearly every system — rain, embers), `velocityScale` the
     *  speed-proportional one (only Angelina's skin uses it). Absent → Unity defaults (lengthScale 2,
     *  velocityScale 0). This replaces a single invented speed factor that over-stretched fast,
     *  tiny specks into long foreground streaks. */
    stretch?: { lengthScale: number; velocityScale: number; cameraVelocityScale: number } | null;
    colorOverLife?: MMColor | null;
    sizeOverLife?: ICurvePoint[] | null;
    /** Unity `velocityOverLifetime`, in px/s. `x`/`y` are the representative (largest)
     *  vector, used where a single constant is needed (stretch length, spawn-cull
     *  trajectory, pile-density). `space:"screen"` means the exporter already projected
     *  the authored LOCAL 3-vector through the emitter's full world basis into the
     *  screen (Y-up) frame — the `rot` rotation below must NOT be re-applied. `curve`
     *  is the per-lifetime vector at normalized particle age; without it the drift is
     *  the constant `x`/`y` (the legacy shape). Skadi2 iteration's crown ring is the
     *  case both parts exist for: its x and z axes trace a CIRCLE over the lifetime,
     *  which flattening to one number turned into a straight drift ("2–3 loose birds"). */
    velocityOverLife?: { x: number; y: number; space?: string; curve?: { t: number; x: number; y: number }[] } | null;
    /** Unity `ForceModule` — a constant ACCELERATION in px/s², integrated into the
     *  particle's velocity every frame (unlike {@link velocityOverLife}, which is a
     *  velocity offset). `space:"screen"` is a local force the exporter already projected
     *  through the emitter's world basis; `space:"world"` is world-aligned already. Drives
     *  the slow lateral drift on Skadi the Corrupting Heart's thread/fish systems. */
    forceOverLife?: { x: number; y: number; space?: string } | null;
    /** Unity `ClampVelocityModule` ("Limit Velocity over Lifetime"): a per-step
     *  damped speed clamp. `magnitude` is the speed LIMIT (px/s) over normalized
     *  life; each frame a particle over the limit has its velocity lerped toward
     *  the limit by `dampen`. Mlynar's `weapon_star_*` glint uses a huge startSpeed
     *  reined in by this clamp into a tight stationary cluster — without it the
     *  burst scatters into faint motes. Absent for systems with no such module. */
    velocityClamp?: { dampen: number; magnitude: MMScalar } | null;
    /** Unity `rotationOverLifetime`, ANGULAR VELOCITY in deg/s, sampled at normalized life.
     *  Usually a CURVE (0 at birth, ramping later), so it must be integrated per frame — a
     *  constant at the curve's peak spins the particle from birth and visibly tumbles
     *  long-lived sprites (Virtuosa's falling apple). Older exports emit a bare number;
     *  {@link rotRateAt} accepts both. */
    rotOverLifeDegPerSec?: number | MMScalar | null;
    /** `_MainTex_ST` UV tiling/offset `[scaleX, scaleY, offsetX, offsetY]` from the
     *  material. When present the emitter crops its sprite to this sub-rectangle of
     *  the (flipbook-atlas) texture — the material's way of selecting ONE cell
     *  without a Texture Sheet module. Ignoring it draws the whole atlas as one
     *  giant sprite (Hoshiguma the Breacher's wave "plumes"). See {@link cropByMainST}. */
    mainST?: [number, number, number, number];
    /** Ancestor GameObject-name chain (nearest→root) from the Unity prefab. When
     *  one of these names is a real spine bone, the emitter is parented under that
     *  bone (spine-unity's `SkeletonUtilityBone`, named after the bone) and its
     *  effect drifts with the character's idle sway in-game — see {@link driftWithBone}. */
    boneChain?: string[];
    /** Explicit spine-bone attachment from a serialized spine-unity `BoneFollower`
     *  on the effect rig's clone root (the director-instantiated `_effects`, e.g.
     *  Virtuosa's entrance `start_apple_01(Clone)` → `L_C_Apple_F`). The rig's
     *  baked `pos` is only an editor pose the follower overrides at runtime — the
     *  emitter's true position is `bone(t) + followOffset`. Only used when the
     *  `boneChain` heuristic can't attach (no ancestor names a real bone); see
     *  {@link driftWithBone}. */
    followBone?: string | null;
    /** The follower's `followBoneRotation`: false = translation-only tracking. */
    followBoneRot?: boolean;
    /** Emitter offset (export px, Y-up) within the followed rig. */
    followOffset?: [number, number] | null;
    noise?: INoise | null;
    trail?: ITrail | null;
    sheet?: { tilesX: number; tilesY: number; frameOverTime?: ICurvePoint[] | null; cycles?: number } | null;
    /** Material params for the "Ram" shader family (ramp-tint + dissolve +
     *  UV-disturb sprite compositor). When present, the emitter is rendered by
     *  {@link RamEmitter} with a GLSL port of the real shader instead of as a
     *  plain tinted billboard — this is what makes SilverAsh the Reignfrost's
     *  crisp cyan energy rings/arcs (and any Ram effect) render faithfully. */
    ram?: IRamData;
    /** Per-particle Ram DISSOLVE-amount curve over normalized lifetime, from the
     *  `CustomDataModule`'s Vector stream (shader vs_TEXCOORD2.x, added to `_Amount`).
     *  Virtuosa's entrance apples crumble away with it (−0.12 → 1 by ~30% of life). */
    ramDissolveCurve?: ICurvePoint[] | null;
    /** The `_Start` cinematic's animated SCALE FACTOR on an effect-host ancestor
     *  (multiplier of the baked resting pose, keyed in absolute cinematic seconds).
     *  Virtuosa "Diversity in Oneness": the crown host scales 1.0→0.28 over 9.4–12.43s
     *  — a big golden halo shrinking into the small crown. Applied about {@link
     *  scalePivot} on the emitter container, COMPOSED with any bone-follow delta. */
    scaleCurve?: ICurvePoint[] | null;
    /** The fixed point the {@link scaleCurve} pivots about (export px, Y-up) — the host
     *  transform's origin in the baked (editor) frame, before the bone-follow delta. */
    scalePivot?: [number, number] | null;
    /** The host's animated LOCAL position as a px OFFSET of {@link scalePivot} from its
     *  resting spot (Y-up), keyed in absolute cinematic seconds. */
    posCurve?: { t: number; x: number; y: number }[] | null;
}

/** Exported `_MainTex_ST`-style tiling/offset: [scaleX, scaleY, offsetX, offsetY]. */
type RamST = [number, number, number, number];

export interface IRamData {
    /** `dissolve` = the `Torappu/Particles-L2D/Dissolve/…` family: two multiplied masks, no
     *  disturb warp, no ramp, `_TintColor` in place of `_MainColor`. Verified against the
     *  decompiled `Dissolve Add Double` program. */
    kind: "disturb" | "vertexDisturb" | "dissolve";
    mainTex: number | null;
    mainST: RamST;
    ramTex: number | null;
    ramST: RamST;
    disturbTex: number | null;
    disturbST: RamST;
    dissolveTex: number | null;
    dissolveST: RamST;
    /** SECOND dissolve map (`_DissolveTex_02`) and its own threshold/border. Null on the
     *  single-mask `Ram/` and `Disturb/` families. */
    dissolveTex2?: number | null;
    dissolveST2?: RamST;
    amount2?: number;
    borderWidth2?: number;
    mainColor: [number, number, number, number];
    /** The `_Start` cinematic's animated `_MainColor`, keyed in absolute cinematic
     *  seconds as `[t, r, g, b, a]` — the particle twin of a scene layer's
     *  `colorCurve`. Replayed each frame, REPLACING `mainColor`; absent = the static
     *  serialized colour holds for the whole shot. RGB is exported HDR-unclamped. */
    mainColorCurve?: [number, number, number, number, number][] | null;
    opacity: number;
    borderWidth: number;
    amount: number;
    intensityU: number;
    intensityV: number;
    disturbInfluenceDissolveUV: number;
    disturbInfluenceMainUV: number;
    mainSpeed: [number, number];
    dissolveSpeed: [number, number];
    disturbSpeed: [number, number];
    vertexDisturbTex: number | null;
    vertexDisturbWeightTex: number | null;
    vertexDisturbIntensity: [number, number, number];
    vertexDisturbSpeed: [number, number];
}

interface INoise {
    strength: number;
    frequency: number;
    scrollSpeed: number;
    octaves?: number;
    damping?: boolean;
    sizeAmount?: number;
    rotationAmount?: number;
}

interface ITrail {
    tex: number | null;
    /** Unity `ParticleSystemTrailMode`. `perParticle` drags a ribbon behind EACH
     *  particle; `ribbon` threads ONE polyline through the system's live particles
     *  ordered by age (see {@link RibbonTrail}). Absent in older exports → perParticle. */
    mode?: "perParticle" | "ribbon";
    /** Interleaved ribbon count (ribbon mode): particle `i` belongs to ribbon `i % n`. */
    ribbonCount?: number;
    blend: "additive" | "normal";
    ratio: number;
    /** Point lifetime as a FRACTION of the particle's lifetime. Emitted by the exporter as a
     *  bare number; older exports of `widthOverTrail` are bare numbers too. Read both through
     *  {@link scalarOf} — never `sampleScalar`, which switches on `.mode` and returns
     *  `undefined` for a plain number. */
    lifetime: number | MMScalar;
    minVertexDistance: number;
    /** Width multiplier ALONG the trail (0 = tail end, 1 = head), on top of the
     *  particle size when {@link sizeAffectsWidth}. */
    widthOverTrail?: number | MMScalar | null;
    colorOverLifetime?: MMColor | null;
    /** Colour banded along the trail — independent of the particle's own colour. */
    colorOverTrail?: MMColor | null;
    sizeAffectsWidth?: boolean;
    inheritParticleColor?: boolean;
    dieWithParticles: boolean;
    worldSpace: boolean;
}

export interface IParticlesData {
    cameraSizePx: number;
    skeletonScale: number;
    characterSort: number;
    textureCount: number;
    systems: IParticleSystemData[];
}

// ---------------------------------------------------------------------------

/** Global live-particle ceiling across every emitter (perf guard). */
const GLOBAL_MAX_PARTICLES = 1400;
/** Per-emitter hard cap, regardless of the authored maxParticles. */
const PER_SYSTEM_CAP = 250;
/** Number of ribbon points per particle trail. */
const TRAIL_POINTS = 12;

/** Spacing (px) between a trail's recorded history points.
 *
 *  Unity's Trails module bounds a ribbon by its point LIFETIME — each point lives
 *  `particleLifetime × trail.lifetime` seconds — so the ribbon spans roughly
 *  `speed × that`. `minVertexDistance` is only the SAMPLING threshold: how far the head
 *  must travel before a new vertex is laid down. It does not bound the length.
 *
 *  Recording at `minVertexDistance` into a fixed {@link TRAIL_POINTS} buffer conflated the
 *  two and capped every ribbon at `(TRAIL_POINTS − 1) × minVertexDistance`, regardless of
 *  what was authored. Across the three reference skins that truncated **22 of 28** trail
 *  systems, several catastrophically: the worst records a vertex every 1 px, so 12 points
 *  spanned 11 px where Unity draws ~2137 (194×), and Skadi's crossing red threads spanned
 *  55 px against an authored ~2000 (37×) — which is why her long red streak is missing
 *  entirely rather than merely short.
 *
 *  Spread the SAME vertex budget over the authored span instead: the ribbon keeps its
 *  length while the cost stays fixed at 12 points. Never sample finer than the authored
 *  `minVertexDistance` (that is a real floor), and clamp the span so a pathological
 *  speed×lifetime cannot stretch one ribbon across the whole scene. */
function trailSpacing(spanPx: number, minVertexDistance: number): number {
    const span = Math.min(Math.max(spanPx, 0), TRAIL_MAX_SPAN);
    return Math.max(minVertexDistance || 1, span / (TRAIL_POINTS - 1));
}

/** Raw trail-history slots per particle, as `[x, y, age]` triples newest-first. Unity
 *  expires a trail vertex by AGE, so the history has to retain every sample inside the
 *  authored window — at a 33 ms tick, 96 slots cover ~3.2 s, past every authored trail
 *  lifetime in the corpus. Beyond that the oldest samples are dropped and the ribbon is
 *  simply shorter than authored. */
const TRAIL_HIST_MAX = 96;

/** Resample the retained history polyline into a FIXED ribbon-point budget, evenly by
 *  arclength, head first.
 *
 *  The two are independent: history length is set by the authored vertex lifetime (which
 *  varies per particle and per frame rate), while the rope has {@link TRAIL_POINTS} slots.
 *  Walking the polyline decouples them, so a ribbon keeps its authored LENGTH whether that
 *  window holds 3 samples or 60, and the cost per particle stays constant.
 *
 *  `hist` is `[x, y, age]` newest-first and `n` is the count of live triples. Fewer than two
 *  live samples means the particle has not moved yet: every output collapses onto the head,
 *  which `RopeGeometry` renders as a zero-area strip (invisible) — the correct look for a
 *  trail with no history, and not a width problem. */
/** Scratch polyline (flat `[x,y,…]`, head first) reused across particles and frames so the
 *  per-frame resample allocates nothing. */
const TRAIL_LINE: number[] = [];

function resampleTrail(line: number[], n: number, out: PIXI.Point[]): void {
    if (n < 2) {
        for (const q of out) q.set(line[0], -line[1]);
        return;
    }
    let total = 0;
    for (let i = 1; i < n; i++) {
        total += Math.hypot(line[i * 2] - line[(i - 1) * 2], line[i * 2 + 1] - line[(i - 1) * 2 + 1]);
    }
    if (total < 1e-6) {
        for (const q of out) q.set(line[0], -line[1]);
        return;
    }
    const step = total / (out.length - 1);
    let seg = 1;
    let segStart = 0;
    let segLen = Math.hypot(line[2] - line[0], line[3] - line[1]);
    for (let k = 0; k < out.length; k++) {
        const target = k * step;
        while (target > segStart + segLen && seg < n - 1) {
            segStart += segLen;
            seg++;
            segLen = Math.hypot(line[seg * 2] - line[(seg - 1) * 2], line[seg * 2 + 1] - line[(seg - 1) * 2 + 1]);
        }
        const f = segLen > 1e-6 ? Math.min(1, Math.max(0, (target - segStart) / segLen)) : 0;
        const ax = line[(seg - 1) * 2];
        const ay = line[(seg - 1) * 2 + 1];
        out[k].set(ax + (line[seg * 2] - ax) * f, -(ay + (line[seg * 2 + 1] - ay) * f));
    }
}

/** Upper bound (px) on an authored trail span, in the scene's own pixel space. Guards a
 *  degenerate `speed × lifetime` product; well above any real authored ribbon (the longest
 *  across the reference skins is ~2400). */
const TRAIL_MAX_SPAN = 4000;
/** Floor (scene px) on a trail's thickness — a ribbon thinner than a pixel cannot be
 *  rasterised, and vanishing is further from the game than a hairline. */
const MIN_TRAIL_WIDTH = 1;

/** Authored thickness (scene px) of a per-particle trail at the particle's CURRENT size.
 *
 *  Unity's width is `widthOverTrail × particle size` (when `sizeAffectsWidth`).
 *  `PIXI.SimpleRope` cannot express it: `_render` reassigns the geometry width to
 *  `texture.height` every frame, and its guard fires for ANY custom width — so every ribbon
 *  drew at whatever its trail atlas page happened to be. Skadi's `xian` threads authored
 *  1.0–1.25 px drew at 64 px, a median 6.3× too wide across 129 systems in 23 skins. Hence
 *  {@link RopeMesh}. `widthOverTrail` sampled mid-trail: a range is a per-particle random
 *  (Unity RandomBetweenTwoConstants, hence `rand`); a curve varies along the ribbon and one
 *  rope has a single width, so its midpoint is the representative. */
/** Angular velocity (deg/s) at normalized life `lf`. Accepts the legacy bare number
 *  (a true constant) and the `MMScalar` the exporter emits now. */
function rotRateAt(v: number | MMScalar | undefined | null, rand: number, lf: number): number {
    if (v == null) return 0;
    return typeof v === "number" ? v : sampleScalar(v, rand, lf);
}

/** Diagnostic-only scale on every per-particle ribbon width (`?trailw=<f>`), and an
 *  absolute override (`?trailwabs=<f>`). Both are inert unless the query param is present;
 *  they exist so a width A/B can be run without editing the source out from under a
 *  comparison. Read once — the query string cannot change within a recording. */
const TRAIL_W_DIAG: { scale: number; abs: number | null; spanModel: boolean } = (() => {
    if (typeof window === "undefined") return { scale: 1, abs: null, spanModel: false };
    const q = new URLSearchParams(window.location.search);
    const s = Number(q.get("trailw"));
    const a = Number(q.get("trailwabs"));
    return { scale: Number.isFinite(s) && s > 0 ? s : 1, abs: Number.isFinite(a) && a > 0 ? a : null, spanModel: q.get("trailmodel") === "span" };
})();

function trailWidth(trail: ITrail, size: number, rand: number): number {
    if (TRAIL_W_DIAG.abs != null) return TRAIL_W_DIAG.abs;
    const mul = trail.widthOverTrail ? scalarOf(trail.widthOverTrail, rand, 0.5) : 1;
    const base = (trail.sizeAffectsWidth ?? true) ? size : 1;
    return Math.max(MIN_TRAIL_WIDTH, base * mul) * TRAIL_W_DIAG.scale;
}

/** A rope whose width is ours to set — `PIXI.SimpleRope` minus the one line that forces
 *  `width = texture.height`. `updateVertices()` is driven from the emitter update instead of
 *  PIXI's autoUpdate, so the width can track `sizeOverLife`. */
class RopeMesh extends PIXI.Mesh<PIXI.MeshMaterial> {
    constructor(texture: PIXI.Texture, points: PIXI.Point[], width: number) {
        super(new PIXI.RopeGeometry(width, points, 0), new PIXI.MeshMaterial(texture));
    }
    setWidth(w: number): void {
        const g = this.geometry as PIXI.RopeGeometry;
        g._width = w;
        g.updateVertices();
    }
}

/** Fixed step (s) the prewarm pre-roll simulates at — fine enough that a 2/s emitter lands
 *  its particles on the right ages, coarse enough that a 5 s pre-roll is 150 steps. */
const PREWARM_STEP = 1 / 30;
/** Hard cap on pre-roll steps, so a pathological `duration` cannot stall the load. */
const PREWARM_MAX_STEPS = 300;

/** Honour Unity's `simulationSpeed` / `prewarm`? **Still OFF by default, but the reasoning
 *  below was wrong and the implementation was broken. Both are now fixed.**
 *
 *  Corpus census (`probe_simspeed`, 13069 systems / 83 bundles): **1700 systems below 1x, 403
 *  above, 3057 prewarmed** (963 both), touching **80 of 83 skins**.
 *
 *  ## RETRACTED: "both fields are inert in the client"
 *
 *  The old note here concluded that an L2D player driving emitters from a director clock
 *  (`Simulate(dt)` rather than `Play()`) triggers neither field, so our model was "correct on
 *  every count". **The client binary says otherwise.** The IL2CPP dump has TWO particle paths:
 *
 *    - `Torappu.UI.UIParticleRenderer::_Simulate` steps systems by hand with
 *      `Simulate(dt, withChildren: false, restart: false, fixedTimeStep: false)` — but its
 *      callers are all UI SCREENS (item select, missions, activity plugins), not dynchars.
 *    - Dynchar effects come from `DynIllustEffectHolder::_LoadEffect`, which just instantiates
 *      a prefab and parents it. There is NO `ParticleSystem.Play` call anywhere in the dynchar
 *      code because the systems author `playOnAwake`, and the scene is rendered by a camera
 *      into a RenderTexture. They run on Unity's NATIVE simulation, which reads both fields
 *      from the serialized data in C++ — never through the C# property. (A caller census
 *      showing `get_prewarm`/`get_simulationSpeed` with zero non-XLua callers therefore proves
 *      nothing: that is exactly what a natively-consumed field looks like.)
 *
 *  So the engine DOES apply both to dynchar systems, and the earlier measured regression was
 *  our bug, not Unity's semantics.
 *
 *  ## The two bugs that were fixed
 *
 *  1. **Clock contamination.** `this.time` is the SYSTEM clock (scaled by `speed`), yet it was
 *     also fed to `driftWithBone`/`sampleColorCurve` as "absolute seconds since `_Start`" for
 *     the DIRECTOR's curves. At `simSpeed = 0.3` those cinematic curves ran 3.3x slow. Split
 *     out as {@link Emitter.cineTime} — real timeline, never scaled, never advanced by the
 *     pre-roll.
 *  2. **Truncated pre-roll.** The roll was `ceil(duration / PREWARM_STEP)` steps capped at
 *     `PREWARM_MAX_STEPS`, so any `duration` over 10 s silently stopped short of steady state —
 *     the exact failure prewarm exists to prevent. The step now adapts so the cap can never
 *     truncate, and the last step is partial so the roll lands exactly on `duration`.
 *
 *  ## Verified: steady state is now actually reached
 *
 *  `quad_p` (Virtuosa's diamond rings, `simSpeed` 0.3, prewarmed, lifetime 5 s) drawn alone at
 *  t=12, isolated against the particle-free plate:
 *
 *      flags OFF (shipped)          17697 px   extent 466x235
 *      simSpeed ONLY                  942 px   extent 116x 89   <- collapsed, the old failure
 *      simSpeed + prewarm (fixed)   20084 px   extent 494x241   <- full size
 *
 *  Note `simSpeed` alone still collapses it, and SHOULD: a non-prewarmed system at 0.3x has
 *  genuinely not filled by t=12. The pair is what has to be judged, and the pair is correct.
 *  Measured against the game's own ring edges (same centre, metric and box), the fix also spans
 *  the right radial range for the first time — GAME->ours 7.3 -> **5.7 px**, span 43-143 ->
 *  47-175 against the game's 45-171. The shipped build misses the outermost ring entirely.
 *
 *  ## Why it is still OFF by default
 *
 *      variant          mly       cel       ska       sum
 *      baseline (off) 17.525    19.270    10.530    47.325
 *      BOTH (fixed)   17.522    19.460    10.502    47.484
 *
 *  Mlynar and Skadi improve; Virtuosa regresses, and **all of it is one beat** — t=10 costs
 *  +1.704 while her other six beats improve or are neutral (t=12 -0.168, t=14 -0.163,
 *  t=17 -0.060). t=10 is the diamond-ring beat, and what differs there is the emission PHASE,
 *  which is UNMATCHABLE by construction: a dynchar system starts on `playOnAwake` when its
 *  prefab finishes loading, then advances on real frame deltas, so its phase is set by runtime
 *  asset-load timing — not serialised, and not reproducible between two runs of the game.
 *  Turning these on trades one arbitrary phase for another, so the net MADC is measuring noise
 *  in a degree of freedom we cannot win.
 *
 *  ## THE CORPUS RENDER — run, and it settles the default
 *
 *  **77 of the 82 deployed skins carry these fields** (1171 systems with `simSpeed`, 2572
 *  prewarmed, of 4560). Every affected skin rendered twice, shipped vs candidate, at
 *  t = 2/6/10/14 — 154 renders, 616 frames. No game reference exists for 74 of them, so this is
 *  not a parity score; it asks whether the change DESTROYS or BLOWS OUT any skin.
 *
 *  **75 of 77 are neutral** (60+ move < 0.2 luma; the rest are background/fog layers at a
 *  different phase, no saturation, no content loss). **One is destroyed:**
 *  `char_4080_lin_nian#10` at t=6 goes 139.62 -> 154.85 frame mean with the saturated fraction
 *  DOUBLING, 0.145 -> 0.284, over half the frame — her snow globe blows to a white blob and the
 *  character inside disappears.
 *
 *  `psonly` attribution puts it almost entirely in her **sys0**, which is `maxParticles: 1`:
 *  ONE additive Ram-dissolve mesh at `startSize` 1938 px covering half the frame, rate 1/s,
 *  lifetime = duration = 1 s. With a single continuously-replaced particle the render IS that
 *  particle's age — `fmod(6, 1) = 0.0` at speed 1, versus `(1.0 prewarm + 0.3*6) = 2.8 -> 0.8`
 *  at speed 0.3. Same dissolve ring, different point in its cycle. **So it is the unmatchable
 *  emission phase again, not a simulation error** — a `maxParticles: 1` system has no steady
 *  state to average into, so "steady-state neutral" says nothing about it. It is still not safe
 *  to ship: the frame genuinely hides the character and no capture exists to say which phase is
 *  right.
 *
 *  ## A prewarm GATE was built, swept, and REVERTED — do not re-derive it
 *
 *  Gating `simSpeed` on `prewarmOf` (honour the speed only where the system reaches steady
 *  state) is the obvious response, and it is wrong. Built it, re-ran the full 77-skin sweep:
 *
 *      char_4080_lin_nian#10   ungated +15.23 (dSat +0.1388)   gated +15.23 (dSat +0.1387)
 *      flagged skins           ungated 2                       gated 2
 *      mean |dMean|            ungated 0.605                   gated 0.632   (WORSE)
 *      improved 11 skins, worsened 7, left 59 byte-unchanged
 *
 *  The destructive skin does not move, because 14 of lin's 15 `simSpeed` systems ALREADY
 *  prewarm — the gate excludes exactly one. Its only wins were narrow (`mlynar_epoque#28`
 *  +1.89 -> +0.08; benchmark sum 47.463 vs 47.484) and did not pay for coupling the two flags
 *  behind a non-obvious rule, so it was reverted.
 *
 *  **No gate on the data can fix this case**: the failing system is `maxParticles: 1`, so its
 *  appearance is purely its single particle's AGE. There is no steady state to reach, which is
 *  exactly why the steady-state argument the gate rests on does not apply — and the phase is
 *  not in the data at all.
 *
 *  ## Verdict: both stay OFF
 *
 *      variant          mly       cel       ska       sum
 *      baseline (off) 17.525    19.270    10.530    47.325
 *      BOTH           17.522    19.460    10.502    47.484
 *      (gated variant 17.514    19.436    10.513    47.463 — reverted, see above)
 *
 *  Mlynar and Skadi improve; Virtuosa regresses, all of it at t=10, the diamond-ring beat whose
 *  residual is the same unmatchable phase. Enabling these trades one arbitrary phase for another
 *  and destroys one skin outright. `?simspeed=1` / `?prewarm=1`, independently, to re-test. */
const SIM_SPEED_ON = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("simspeed") === "1";
function simSpeedOf(d: IParticleSystemData): number {
    if (!SIM_SPEED_ON) return 1;
    const s = d.simSpeed;
    return typeof s === "number" && Number.isFinite(s) && s > 0 ? s : 1;
}

/** See {@link simSpeedOf} — disabled by default; `?prewarm=1` re-enables. Only LOOPING
 *  systems prewarm; Unity ignores the flag on one-shots. */
const PREWARM_ON = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("prewarm") === "1";
function prewarmOf(d: IParticleSystemData): boolean {
    if (!PREWARM_ON) return false;
    return !!d.prewarm && d.looping && d.duration > 0;
}

/** Restart the emission accumulator at each loop wrap? **DISABLED — MEASURED AND REJECTED.**
 *
 *  The question was well posed: we carry the fractional `emitAcc` remainder across a looping
 *  system's duration boundary, so emission times are `k / rate` from t=0 — a GLOBAL phase. If
 *  Unity instead restarts the accumulator each cycle the times are `cycleStart + k / rate`, a
 *  phase pinned to the loop. The two differ by `duration mod (1/rate)`, which is 1.0 s on
 *  Virtuosa's `rainbow_large_01` (duration 5, rate 0.5) — the single system that carries her
 *  whole `prewarm` regression, so a loop-pinned phase was the natural explanation.
 *
 *  It is wrong, and the test is unambiguous: the reset moves that system to the SAME phase
 *  `prewarm` does, not back to the game's.
 *
 *      cel t=10        control 24.936   loopacc alone 26.708   prewarm alone 26.624
 *      cel whole       control 19.422   loopacc alone 19.617   loopacc+simSpeed+prewarm 19.559
 *      mly / ska       17.721 / 10.505  ->  17.720 / 10.514  (untouched)
 *
 *  So the GLOBAL phase we already ship is the one the game matches, and Unity carries the
 *  accumulator across loop boundaries exactly as we do. That also settles the open question
 *  left by [[simSpeedOf]]: `prewarm`'s cost is not a phase-pinning difference we could correct,
 *  it is that adding one `duration` of emission history genuinely re-phases any system whose
 *  duration is not a whole number of emission intervals.
 *
 *  `?loopacc=1` re-enables it for a re-test. */
const LOOP_ACC_RESET = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("loopacc") === "1";
function loopAccReset(): boolean {
    return LOOP_ACC_RESET;
}

/** Floor on a spawned particle's lifetime (s), guarding degenerate authored data. */
const MIN_PARTICLE_LIFE = 0.05;

/** M-c: opacity for a world-space ambient system (rain) promoted IN FRONT of the
 *  character to un-occlude it (see `unoccludeOverlap`). Drawn over the body at full
 *  strength it reads as prominent streaks; the game shows only a faint sheen there, so
 *  the over-body copy is dimmed to this while the background rain stays untouched. */
const FOREGROUND_SHEEN_ALPHA = 0.4;

const DEG = Math.PI / 180;

function lerp(a: number, b: number, t: number): number {
    return a + (b - a) * t;
}

/** Cheap smooth pseudo-noise in [-1,1] — enough for organic particle wander. */
function fbmNoise(x: number, y: number): number {
    return (Math.sin(x) + Math.sin(y * 1.3 + 1.7) + Math.sin((x + y) * 0.7 + 4.2)) / 3;
}

function sampleCurve(curve: ICurvePoint[], t: number): number {
    if (curve.length === 0) return 0;
    if (t <= curve[0].t) return curve[0].v;
    const last = curve[curve.length - 1];
    if (t >= last.t) return last.v;
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i].t) {
            const a = curve[i - 1];
            const b = curve[i];
            const f = (t - a.t) / (b.t - a.t || 1);
            return lerp(a.v, b.v, f);
        }
    }
    return last.v;
}

/** Evaluate a MinMaxCurve. `rand` in [0,1) picks within ranges; `t` in [0,1] samples curves. */
/** Read a value the exporter may emit either as a bare number or as an `MMScalar`.
 *
 *  `sampleScalar` switches on `s.mode`; handed a plain number it matches no case and returns
 *  `undefined`. That silently poisoned every per-particle trail: `trail.lifetime` is emitted
 *  as a bare number, so the ribbon span went `speed × life × undefined` = NaN, `trailSpacing`
 *  returned NaN, and the "has the head moved far enough for a new history point" test
 *  (`d² >= step²`) is FALSE against NaN forever. No history ever accumulated, all
 *  {@link TRAIL_POINTS} collapsed onto the particle, and the rope drew zero-area geometry —
 *  invisible, while still reporting a large bounding box because the emitters are spread out.
 *  That is why Skadi the Corrupting Heart's `xian` threads never appeared. */
function scalarOf(v: number | MMScalar | undefined | null, rand: number, t: number): number {
    if (v == null) return 0;
    return typeof v === "number" ? v : sampleScalar(v, rand, t);
}

function sampleScalar(s: MMScalar | undefined, rand: number, t: number): number {
    if (!s) return 0;
    switch (s.mode) {
        case "const":
            return s.v;
        case "range":
            return lerp(s.min, s.max, rand);
        case "curve":
            return sampleCurve(s.curve, t);
        case "rangeCurve":
            return lerp(sampleCurve(s.min, t), sampleCurve(s.max, t), rand);
    }
}

interface IRGBA {
    r: number;
    g: number;
    b: number;
    a: number;
}

function sampleGradient(stops: IColorStop[], t: number): IRGBA {
    if (stops.length === 0) return { r: 1, g: 1, b: 1, a: 1 };
    if (t <= stops[0].t) return stops[0];
    const last = stops[stops.length - 1];
    if (t >= last.t) return last;
    for (let i = 1; i < stops.length; i++) {
        if (t <= stops[i].t) {
            const a = stops[i - 1];
            const b = stops[i];
            const f = (t - a.t) / (b.t - a.t || 1);
            return { r: lerp(a.r, b.r, f), g: lerp(a.g, b.g, f), b: lerp(a.b, b.b, f), a: lerp(a.a, b.a, f) };
        }
    }
    return last;
}

/** Sample a `MinMaxGradient` at normalized time `t`. `rand` is the particle's own
 *  0..1 draw, used ONLY by the `twoColors` (Unity `RandomBetweenTwoColors`) state,
 *  which picks one colour per PARTICLE — pass the particle's stored `rand` for
 *  per-frame sampling so its colour is stable over its life; omit it at spawn to
 *  draw a fresh one. Every other state ignores `rand` and consumes no randomness,
 *  so single-colour/gradient systems stay byte-identical. */
function sampleColor(c: MMColor | undefined | null, t: number, rand?: number): IRGBA {
    if (!c) return { r: 1, g: 1, b: 1, a: 1 };
    if (c.mode === "twoColors") {
        const f = rand ?? Math.random();
        return { r: lerp(c.min.r, c.max.r, f), g: lerp(c.min.g, c.max.g, f), b: lerp(c.min.b, c.max.b, f), a: lerp(c.min.a, c.max.a, f) };
    }
    return c.mode === "color" ? c : sampleGradient(c.stops, t);
}

function rgbToHex(c: IRGBA): number {
    const q = (v: number) => Math.max(0, Math.min(255, Math.round(v * 255)));
    return (q(c.r) << 16) | (q(c.g) << 8) | q(c.b);
}

interface IParticle {
    x: number;
    y: number;
    vx: number;
    vy: number;
    age: number;
    life: number;
    size: number;
    /** Height / width of the quad — 1 unless the system authors `startSizeY`. */
    aspectY: number;
    rot: number;
    rotVel: number;
    rand: number;
    startCol: IRGBA;
    /** Per-particle display object: a billboard Sprite, or a Mesh instance for
     *  mesh-render systems (see {@link MeshEmitter}). */
    sprite: PIXI.Sprite | PIXI.Mesh;
    // Trail (only for particles that draw one): the rope, its points, and a
    // distance-gated position history (flat [x,y,…], newest first, Y-up).
    trailPts?: PIXI.Point[];
    rope?: RopeMesh;
    hist?: number[];
    /** Spacing (px) between this particle's recorded history points. Derived per
     *  particle from the AUTHORED trail lifetime — see {@link trailSpacing}. */
    trailSpace?: number;
    /** Seconds a trail vertex survives for THIS particle: `particleLifetime ×
     *  trail.lifetime`. This — not vertex spacing — is what bounds a Unity trail. */
    trailLife?: number;
}

/** The follow bone's LIVE world matrix this frame (pixi-spine `bone.matrix`);
 *  null when the skeleton has no such bone. */
export type FindBone = (name: string) => PIXI.Matrix | null;

/** Crop a texture to the material's `_MainTex_ST` sub-rectangle so the sprite
 *  shows the ONE atlas cell the material selects, instead of the whole flipbook
 *  atlas (the fix for Hoshiguma the Breacher's wave "plumes"). Unity UVs are
 *  bottom-left origin `u∈[ox,ox+sx], v∈[oy,oy+sy]`; Pixi frames are top-left, so
 *  `y = (1-oy-sy)·H`. Scales ≤ 1 crop; a (rare) scale > 1 would tile — we just
 *  clamp to the texture bounds rather than repeat. Returns the input unchanged
 *  for an absent or identity ST. */
function cropByMainST(tex: PIXI.Texture, st?: [number, number, number, number]): PIXI.Texture {
    if (!st) return tex;
    const [sx, sy, ox, oy] = st;
    if (Math.abs(sx - 1) < 1e-4 && Math.abs(sy - 1) < 1e-4 && Math.abs(ox) < 1e-4 && Math.abs(oy) < 1e-4) return tex;
    const base = tex.baseTexture;
    const W = base.width;
    const H = base.height;
    const rx = Math.max(0, Math.min(W - 1, ox * W));
    const ry = Math.max(0, Math.min(H - 1, (1 - oy - sy) * H));
    const rw = Math.max(1, Math.min(W - rx, sx * W));
    const rh = Math.max(1, Math.min(H - ry, sy * H));
    return new PIXI.Texture(base, new PIXI.Rectangle(rx, ry, rw, rh));
}

/** Per-emitter bone-follow state, resolved lazily on first update: the follow
 *  bone's name (null once resolved to "none") and its REFERENCE world matrix
 *  (the pose the authored `pos` was baked at) whose inverse builds the delta. */
interface IBoneAnchor {
    resolved: boolean;
    boneName: string | null;
    ref: PIXI.Matrix | null;
    /** Rebase mode (explicit `BoneFollower`): translation-only tracking. */
    transOnly?: boolean;
}

/** Explicit spine-bone attachment (exporter `followBone` fields), pre-digested. */
interface IFollow {
    bone: string;
    rot: boolean;
    off: [number, number] | null;
}

/** Build the {@link IFollow} for a system, or undefined when it has none. */
function followOf(d: IParticleSystemData): IFollow | undefined {
    return d.followBone ? { bone: d.followBone, rot: !!d.followBoneRot, off: d.followOffset ?? null } : undefined;
}

/** Longest lifetime (s) that still gets `velocityOverLifetime` drift. See below. */
const VELOCITY_MAX_LIFE = 0.8;

/** The shape transform's non-uniform SCALE (Unity `ShapeModule.scale`), which stretches a
 *  RADIAL emission volume into an ellipse around `radius`. Unity applies it to every shape
 *  type, not just boxes (whose size it already IS — `shape.box`, in px). Ignoring it packs a
 *  wide, thin shower into a small disc: Mlynar's rain hemispheres are radius-0.2 scaled up to
 *  (2.0, 1.4), so unscaled they emit ~1.6x too densely over half the authored width and read
 *  as clustered patches over the bare trees. Legacy exports (and every default (1,1) shape)
 *  return the identity, leaving those systems byte-identical. */
function shapeScale(shape: NonNullable<IParticleSystemData["shape"]>): [number, number] {
    const s = shape.scale;
    return s ? [s[0], s[1]] : [1, 1];
}

/** Unity `ShapeModule.radiusThickness`: the fraction of the radius the emission volume
 *  occupies, measured inward from the surface — 1 fills the whole disc, 0 emits on the
 *  SHELL only (Skadi2's `smoke_01`/`tri_weapon_01` bursts, which we scattered through the
 *  interior instead of ringing the rim). Returns the sampled radius as a fraction of
 *  `radius`, uniform in AREA over the annulus, consuming exactly one `Math.random()` so a
 *  full-volume (thickness 1) shape draws the identical value it always did. */
function radialFrac(shape: NonNullable<IParticleSystemData["shape"]>): number {
    const inner = 1 - Math.min(Math.max(shape.radiusThickness ?? 1, 0), 1);
    const i2 = inner * inner;
    return Math.sqrt(i2 + (1 - i2) * Math.random());
}

/** An emitter's LIVE emission rate (particles/s): the cinematic's animated rate curve
 *  when the exporter captured one (sampled at the emitter clock — rate-curve systems
 *  carry no start delay, so the clock IS cinematic time), else the serialized constant. */
/** `?bdp=0` disables the {@link isBackdropParticle} demotion (diagnostic). */
function backdropDemoteEnabled(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("bdp") !== "0";
}

function emissionRate(d: IParticleSystemData, constRate: number, time: number): number {
    // A clip-driven `rateCurve` (absolute cinematic seconds) wins wherever one exists.
    if (d.rateCurve?.length) return Math.max(0, sampleCurve(d.rateCurve, time));
    // Unity evaluates `EmissionModule.rateOverTime` at the system's NORMALIZED cycle
    // position, so a curve-mode rate must be re-sampled every frame. It was instead frozen
    // at the value the constructor took once, `sampleScalar(rate, 0.5, 0)` — the curve at
    // t=0. The standard authoring pattern ramps IN from zero, so that constant is exactly
    // 0.0 and those systems emitted NOTHING for their entire life, however large the
    // plateau. Virtuosa's four `_Start` systems each ask for a 5/s plateau over 3-6s (~15
    // particles) and produced none. Corpus census: 104 systems across 15 composites.
    const r = d.emission?.rate;
    if (r && (r.mode === "curve" || r.mode === "rangeCurve")) {
        const dur = d.duration > 0 ? d.duration : 1;
        const cyc = time / dur;
        // Looping systems wrap; a one-shot clamps so its tail holds the curve's last key.
        const n = d.looping ? cyc - Math.floor(cyc) : Math.min(1, Math.max(0, cyc));
        return Math.max(0, sampleScalar(r, 0.5, n));
    }
    return constRate;
}

/** velocityOverLifetime drift, in WORLD space (px/s).
 *
 *  Unity's `velocityOverLifetime` here is a CONSTANT push we integrate over the
 *  particle's lifetime. Over a LONG lifetime that's a huge straight-line fly-off the
 *  in-game archive never shows (Hoshiguma the Breacher's `fire_p` systems: lifetime
 *  5–6s, ~100px/s → a 500–600px sweep across the frame; that particular system has no
 *  `ClampVelocityModule` to arrest it — one that DOES (e.g. Mlynar's star glint) is now
 *  modeled via {@link IParticleSystemData.velocityClamp}). So a plain BILLBOARD without a
 *  clamp keeps a short-lifetime gate — its ambient flame
 *  stays contained. But a STREAK system (`renderMode:"stretch"` or one carrying a ribbon
 *  `trail`) is authored so the travel IS the effect (Skadi2 iteration's near-body
 *  fish-shoal / starlight streaks, vol 750–1000px/s, life 2–8s; Virtuosa's rain), so it
 *  keeps its drift at any lifetime. Property-driven, no per-skin value.
 *
 *  The exported vector lives in the emitter's frame: a `space:"local"` vector must be
 *  rotated into world by the emitter rotation `d.rot` (the same rotation `spawn` applies
 *  to the shape offset + startSpeed direction) — most of Skadi2's streak vectors are
 *  local with a non-zero `d.rot`, so unrotated they point up/off-frame and never reach
 *  her body. A `space:"world"` vector is already world-aligned and passes through raw. */
function worldVelocityOverLife(d: IParticleSystemData): { x: number; y: number } | null {
    const vol = d.velocityOverLife;
    if (!vol || (vol.x === 0 && vol.y === 0)) return null;
    const isStreak = d.renderMode === "stretch" || d.trail != null;
    if (!isStreak && scalarMax(d.lifetime) > VELOCITY_MAX_LIFE) return null;
    // `world` (authored in world space) and `screen` (a local vector the exporter already
    // projected through the emitter's world basis) are both world-aligned already.
    if (vol.space === "world" || vol.space === "screen") return { x: vol.x, y: vol.y };
    const a = (d.rot ?? 0) * DEG;
    const c = Math.cos(a);
    const s = Math.sin(a);
    return { x: vol.x * c - vol.y * s, y: vol.x * s + vol.y * c };
}

/** `forceOverLifetime` as a world-aligned acceleration (px/s²), or null when absent/zero.
 *  Both exported spaces are already world-aligned: `screen` was projected through the
 *  emitter basis by the exporter, `world` was authored that way. */
function worldForceOverLife(d: IParticleSystemData): { x: number; y: number } | null {
    const f = d.forceOverLife;
    if (!f || (f.x === 0 && f.y === 0)) return null;
    // Only the world-aligned spaces are usable. A legacy `space:"local"` export shipped the
    // RAW emitter-local axes with no basis to interpret them, so consuming it would apply an
    // arbitrarily rotated acceleration; those exports stay inert (exactly as before this
    // module was read at all) until the skin is re-extracted.
    if (f.space !== "screen" && f.space !== "world") return null;
    return { x: f.x, y: f.y };
}

/** Linear-sample a `velocityOverLife.curve` (already world/screen-aligned px/s) at
 *  normalized particle age. Only systems whose authored curves the exporter judged
 *  lossy under a single constant carry one; everything else keeps the constant
 *  {@link worldVelocityOverLife}. */
function sampleVolCurve(curve: { t: number; x: number; y: number }[], t: number): { x: number; y: number } {
    if (curve.length === 0) return { x: 0, y: 0 };
    if (t <= curve[0].t) return curve[0];
    const last = curve[curve.length - 1];
    if (t >= last.t) return last;
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i].t) {
            const a = curve[i - 1];
            const b = curve[i];
            const f = (t - a.t) / (b.t - a.t || 1);
            return { x: lerp(a.x, b.x, f), y: lerp(a.y, b.y, f) };
        }
    }
    return last;
}

/** Every bone's REFERENCE-pose world matrix, keyed by bone name — captured once
 *  (the pose the authored particle positions were baked at). Used to pick the
 *  follow bone (nearest to the emitter's spawn point) and as the delta reference.
 *  See {@link driftWithBone}. */
export type RestBone = ReadonlyMap<string, PIXI.Matrix>;
/** Setup-pose CENTRE of each spine attachment, keyed by attachment name (skeleton world,
 *  Y-down — the same frame as {@link RestBone}). Used to hand a `BoneFollower` prop off from
 *  the ART rather than the bone ORIGIN; see {@link driftWithBone}. */
export type RestAttachment = ReadonlyMap<string, { x: number; y: number }>;

/** Rigidly bond a bone-parented emitter's whole effect to the character bone it
 *  rides, so it tracks that bone through Idle AND Special animations — SilverAsh
 *  the Reignfrost's purple sword-flame stays on the blade as it sweeps, Hoshiguma's
 *  face fire tracks her head.
 *
 *  Applies the follow bone's FULL world-matrix DELTA `D = Mₙₒw · Mᵣₑf⁻¹` to the
 *  emitter container (translation + ROTATION + scale). Translation-only was wrong:
 *  it kept every spawn point at a fixed world orientation, so as the blade rotated
 *  the flame slid off the edge (the fixed-distance drift). At the reference pose
 *  `D = identity`, so the container is untouched and the flame sits exactly where
 *  its baked `pos` placed it.
 *
 *  Follow-bone selection: emitters whose exporter `boneChain` names a real spine
 *  bone are followed; additionally, a `simulationSpace:"local"` system (Unity's
 *  convention for moving with its parent transform) gets the same nearest-bone
 *  search even with no literal chain-name match. Mlynar's sword-tip sparkle burst
 *  (`weapon_star_*`) is exactly this: authored local-space, generic FX-rig chain
 *  names, no `followBone`; without it, the emitter stays frozen at its baked
 *  rest-pose position while the sword swings and camera dollies, drifting hundreds
 *  of px off-canvas by the time it fires. Un-parented world-fixed emitters — bg
 *  sparks, magic circles — use `"world"`, name none, and stay put. Among followed
 *  emitters, we track the bone PHYSICALLY NEAREST the emitter's spawn `pos`, i.e.
 *  the bone driving the visual the flame sits on — the named `Sword_Fx` is only an
 *  FX-anchor that may not rotate with the rendered blade. */
/** Linear-sample a `{t,x,y}` position curve into `[x,y]` at time `t`. */
function samplePosCurve(curve: { t: number; x: number; y: number }[], t: number): [number, number] {
    if (curve.length === 0) return [0, 0];
    if (t <= curve[0].t) return [curve[0].x, curve[0].y];
    const last = curve[curve.length - 1];
    if (t >= last.t) return [last.x, last.y];
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i].t) {
            const a = curve[i - 1];
            const b = curve[i];
            const f = (t - a.t) / (b.t - a.t || 1);
            return [lerp(a.x, b.x, f), lerp(a.y, b.y, f)];
        }
    }
    return [last.x, last.y];
}

/** The `_Start` cinematic scale-in matrix `H` (see {@link IParticleSystemData.scaleCurve}):
 *  a uniform scale `m` about the host pivot plus its animated position offset, in the
 *  emitter's SCREEN (Y-down) frame. Composed as `base · H` so it operates in the baked frame
 *  before any bone-follow delta carries the rig to its live position. Identity at rest. */
function haloMatrix(d: IParticleSystemData, ct: number): PIXI.Matrix {
    const m = d.scaleCurve?.length ? sampleCurve(d.scaleCurve, ct) : 1;
    const [cx, cy] = d.scalePivot ?? [0, 0]; // Y-up px
    let dx = 0;
    let dy = 0;
    if (d.posCurve?.length) {
        const [px, py] = samplePosCurve(d.posCurve, ct);
        dx = px;
        dy = py;
    }
    // Screen (Y-down): pivot=(cx,-cy), offset=(dx,-dy). Scale about pivot, then translate:
    // final = m·local + [pivot·(1−m) + offset].
    return new PIXI.Matrix(m, 0, 0, m, cx * (1 - m) + dx, -cy * (1 - m) - dy);
}

function driftWithBone(container: PIXI.Container, chain: string[] | undefined, pos: readonly [number, number], simSpace: string | undefined, find: FindBone | undefined, st: IBoneAnchor, restBone?: RestBone, follow?: IFollow, halo?: IParticleSystemData, ct?: number, restAtt?: RestAttachment): void {
    if (!st.resolved) {
        st.resolved = true;
        st.boneName = null;
        st.ref = null;
        if (follow && find && restBone?.has(follow.bone)) {
            // Explicit `BoneFollower` attachment (entrance effect rigs): the chain
            // names no real bone — the rig rides the follower's named bone, REBASED
            // there (the baked `pos` is only an editor pose). Doctor the reference
            // translation so the delta `T = M_now.t − ref.t` lands the emitter at
            // `bone_now + followOffset` exactly: `ref.t := pos_ydown − off_ydown`.
            const m = (restBone.get(follow.bone) as PIXI.Matrix).clone();
            const off = follow.off ?? [0, 0];
            // PROP HANDOFF. `followOffset` is measured to the bone's ORIGIN, but when the
            // followed bone also names an ATTACHMENT the rig is continuing that piece of
            // spine art (Virtuosa's apple: the entrance fades attachment `L_C_Apple_F` out
            // at the exact beat the particle copy spawns). The art sits off its bone origin,
            // so anchoring to the origin drops the particle by that offset — measured +15.1
            // screen px, constant, for the whole fall. Re-anchor onto the ART's setup centre;
            // derived from the skeleton, so it carries no per-skin constant and is a no-op
            // for every follower whose bone names no attachment.
            const art = restAtt?.get(follow.bone);
            const dax = art ? art.x - m.tx : 0;
            const day = art ? art.y - m.ty : 0;
            m.tx = pos[0] - off[0] - dax;
            m.ty = -pos[1] + off[1] - day; // Y-up export offsets → Y-down container space
            st.boneName = follow.bone;
            st.ref = m;
            st.transOnly = !follow.rot;
        } else if (find && restBone && (chain?.some((n) => restBone.has(n)) || simSpace === "world")) {
            let best: string | null = null;
            let bestDist = Number.POSITIVE_INFINITY;
            for (const [name, m] of restBone) {
                const dx = m.tx - pos[0];
                const dy = -m.ty - pos[1]; // bone matrix is Y-down; `pos` is Y-up
                const dd = dx * dx + dy * dy;
                if (dd < bestDist) {
                    bestDist = dd;
                    best = name;
                }
            }
            if (best) {
                st.boneName = best;
                st.ref = (restBone.get(best) as PIXI.Matrix).clone();
            }
        }
    }
    // Base matrix `B`: the bone-follow delta (identity when the emitter follows no bone).
    let base: PIXI.Matrix | null = null;
    if (st.boneName && st.ref && find) {
        const now = find(st.boneName);
        if (now) {
            base = st.transOnly
                ? // `followBoneRotation` off: translate the rig, don't spin it.
                  new PIXI.Matrix(1, 0, 0, 1, now.tx - st.ref.tx, now.ty - st.ref.ty)
                : // pixi-spine bone matrices are already in the skeleton's Y-DOWN world
                  // space — the SAME space the particle sprites live in — so the bone's
                  // rigid delta applies DIRECTLY. Conjugating it by a Y-flip double-flips
                  // the rotation (slides the flame off a rotating blade).
                  now.clone().append(st.ref.clone().invert());
        }
    }
    // Scale-in host (`scaleCurve`): compose `base · H` and apply fresh every frame — the
    // halo scales in the baked frame, then `base` carries it to the live bone position.
    if (halo?.scaleCurve?.length) {
        const b = base ?? PIXI.Matrix.IDENTITY.clone();
        container.transform.setFromMatrix(b.append(haloMatrix(halo, ct ?? 0)));
        return;
    }
    // No scale-in: preserve the original behaviour exactly (untouched when no bone).
    if (!base) return;
    container.transform.setFromMatrix(base);
}

/** Mutable state {@link holdWorldSpace} keeps between frames (the emitter container's
 *  transform as of the previous frame). One per emitter. */
interface IWorldSpaceState {
    prev: PIXI.Matrix | null;
}

/** The minimum a particle must expose to be carried by {@link holdWorldSpace}. */
interface IDriftable {
    x: number;
    y: number;
    vx: number;
    vy: number;
    /** Flat `[x0,y0,x1,y1,…]` trail history (Y-up), when the particle ribbons. */
    hist?: number[];
}

/** Unity `ParticleSystemSimulationSpace.World`: once emitted, a particle is INDEPENDENT
 *  of its emitter — the emitter keeps moving and the particle stays where it was born.
 *  (`Local` is the opposite: the whole cloud is parented to the emitter and rides along.)
 *
 *  Our particles are children of the emitter container, whose transform {@link driftWithBone}
 *  drives from the followed bone — so they ride it, i.e. every system simulates LOCAL. For a
 *  world-space system, re-express each live particle in the container's NEW frame so it holds
 *  the screen position it already had: `p' = M_now⁻¹ · M_prev · p`. Velocities take the same
 *  linear part, keeping their direction world-fixed as the emitter rotates.
 *
 *  This is what leaves a TRAIL behind a moving emitter: Virtuosa "Diversity in Oneness"'s
 *  falling apple sheds sparks (`rateOverDistance`) all the way down its plunge instead of
 *  dragging the whole cloud with it as a tight blob. No-op — the early return — for a static
 *  emitter, which is most of them, and never reached by a `local` system. */
function holdWorldSpace(container: PIXI.Container, st: IWorldSpaceState, particles: readonly IDriftable[]): void {
    container.transform.updateLocalTransform();
    const now = container.transform.localTransform;
    const prev = st.prev;
    st.prev = now.clone();
    if (!prev) return;
    // Untransformed emitter (identical frames) — nothing to compensate.
    if (prev.a === now.a && prev.b === now.b && prev.c === now.c && prev.d === now.d && prev.tx === now.tx && prev.ty === now.ty) return;
    const delta = now.clone().invert().append(prev);
    for (const p of particles) {
        // Particle coords are Y-UP; the container transform is Pixi's Y-DOWN screen frame.
        const x = delta.a * p.x + delta.c * -p.y + delta.tx;
        const y = delta.b * p.x + delta.d * -p.y + delta.ty;
        p.x = x;
        p.y = -y;
        const vx = delta.a * p.vx + delta.c * -p.vy;
        const vy = delta.b * p.vx + delta.d * -p.vy;
        p.vx = vx;
        p.vy = -vy;
        if (p.hist) {
            for (let i = 0; i + 1 < p.hist.length; i += 2) {
                const hx = delta.a * p.hist[i] + delta.c * -p.hist[i + 1] + delta.tx;
                const hy = delta.b * p.hist[i] + delta.d * -p.hist[i + 1] + delta.ty;
                p.hist[i] = hx;
                p.hist[i + 1] = -hy;
            }
        }
    }
}

/** Aggregate ribbon state over an emitter's pool — see {@link IEmitterProbe.rope}. */
function ropeStats(pool: IParticle[]): IEmitterProbe["rope"] {
    let n = 0;
    let vis = 0;
    let wsum = 0;
    let span = 0;
    let distinct = 0;
    for (const p of pool) {
        if (!p.rope || !p.trailPts) continue;
        n++;
        if (p.rope.visible) vis++;
        wsum += (p.rope.geometry as PIXI.RopeGeometry & { _width: number })._width;
        const pts = p.trailPts;
        let minX = Infinity;
        let maxX = -Infinity;
        let minY = Infinity;
        let maxY = -Infinity;
        let d = pts.length > 0 ? 1 : 0;
        for (let i = 0; i < pts.length; i++) {
            const q = pts[i];
            minX = Math.min(minX, q.x);
            maxX = Math.max(maxX, q.x);
            minY = Math.min(minY, q.y);
            maxY = Math.max(maxY, q.y);
            if (i > 0 && (Math.abs(q.x - pts[i - 1].x) > 1e-6 || Math.abs(q.y - pts[i - 1].y) > 1e-6)) d++;
        }
        if (pts.length > 0) span += Math.max(maxX - minX, maxY - minY);
        distinct += d;
    }
    if (n === 0) return null;
    const first = pool.find((p) => p.rope)?.rope;
    const texRes = first ? (first.shader.texture as PIXI.Texture) : null;
    let asum = 0;
    let tints = 0;
    for (const p of pool) {
        if (!p.rope) continue;
        asum += p.rope.alpha;
        tints += typeof p.rope.tint === "number" ? p.rope.tint : 0;
    }
    return {
        n,
        vis,
        w: wsum / n,
        span: span / n,
        pts: distinct / n,
        alpha: asum / n,
        tint: Math.round(tints / n)
            .toString(16)
            .padStart(6, "0"),
        tex: texRes ? `${texRes.width}x${texRes.height}${texRes.baseTexture.valid ? "" : " INVALID"}` : "none",
        blend: first ? first.blendMode : -1,
    };
}

/** One live billboard emitter: owns a sprite pool inside a container. */
class Emitter {
    protected readonly data: IParticleSystemData;
    protected readonly texture: PIXI.Texture;
    readonly container = new PIXI.Container();
    readonly pool: IParticle[] = [];
    private readonly free: IParticle[] = [];
    private time = 0;
    /** Absolute seconds since the `_Start` began, on the REAL timeline — never scaled by
     *  `simulationSpeed` and never advanced by the prewarm pre-roll. Drives the DIRECTOR's
     *  curves (`scaleCurve`, the Ram `_MainColor` ramp); {@link time} drives the simulation. */
    private cineTime = 0;
    private emitAcc = 0;
    /** Has this system EVER had a live particle? See {@link liveCount}. */
    everLive = false;
    /** One-shot latch for the {@link prewarmOf} pre-roll. */
    private prewarmed = false;
    private firedBursts = new Set<number>();
    private lastCycleT = -1;
    private readonly rate: number;

    private readonly trailTexture: PIXI.Texture | null;
    private readonly trailLayer: PIXI.Container | null;
    /** Ribbon-mode trail (see {@link RibbonTrail}); null for per-particle trails. */
    private readonly ribbon: RibbonTrail | null;

    /** Effective blend for particle sprites (may differ from data.blend). */
    protected readonly blend: "additive" | "normal";

    /** velocityOverLifetime drift, short-lived particles only (see {@link worldVelocityOverLife}). */
    private readonly volWorld: { x: number; y: number } | null;
    /** Per-lifetime velocityOverLifetime vector (px/s, world-aligned), when the exporter
     *  couldn't represent the authored curves as one constant — see
     *  {@link IParticleSystemData.velocityOverLife}. Null → the constant `volWorld` is used,
     *  exactly as before. Gated on `volWorld` so the same short-life/streak rules apply. */
    private readonly volCurve: { t: number; x: number; y: number }[] | null;
    /** Bone-follow state so the effect drifts with its parent bone (see {@link driftWithBone}). */
    private readonly boneAnchor: IBoneAnchor = { resolved: false, boneName: null, ref: null };
    /** Explicit BoneFollower attachment (see {@link followOf} / {@link driftWithBone}). */
    private readonly follow: IFollow | undefined;
    /** `rateOverDistance` (particles per px of emitter travel) and the emitter's
     *  last container position, for distance-based trail emission (Virtuosa's
     *  falling-apple comet dust). */
    private readonly rodRate: number;
    private lastEmitterPos: { x: number; y: number } | null = null;
    /** World-space simulation state — see {@link holdWorldSpace}. */
    private readonly worldSpace: IWorldSpaceState = { prev: null };
    /** Live authored display box (mesh-local px, Y-down) for the current frame — see
     *  {@link ILoadedParticles.update}. Null = no cull (e.g. no framing data yet). */
    private displayBox: IAnimationBounds | null = null;

    /** Texture Sheet Animation frames (sub-rectangles of the base texture), when
     *  the emitter flipbooks through a tile grid; empty for a plain sprite. */
    protected readonly frames: PIXI.Texture[] = [];
    /** Width of one sprite (a sheet frame's, else the whole texture) — the scale
     *  divisor so `startSize` maps to on-screen pixels regardless of tiling. */
    protected readonly spriteW: number;
    /** Height of one sprite, the divisor for a `startSizeY` quad. The sprite is NOT
     *  square in general: the material's `_MainTex_ST` crop or a sheet cell can already
     *  carry the aspect (Virtuosa's `star_large_01` samples a 512×128 strip), so a
     *  non-uniform size has to divide each axis by its own extent. */
    protected readonly spriteH: number;

    constructor(
        data: IParticleSystemData,
        texture: PIXI.Texture,
        trailTexture: PIXI.Texture | null,
        blend: "additive" | "normal",
        private readonly getBudget: () => number,
    ) {
        this.data = data;
        // Start dormant through the cinematic delay: `time` counts up from `-delay` to 0,
        // so once active all the `time`-based emission logic below sees time starting at 0.
        // `?psdt=<s>` shifts ONLY this clock (see psDt) — the camera and spine keep their own,
        // which is what makes it a phase measurement rather than a retime of the whole shot.
        this.time = -(data.delay ?? 0) + psDt();
        this.cineTime = psDt();
        // Crop to the material's `_MainTex_ST` cell first, so both the whole-sprite
        // path and the sheet-slicing below operate on the selected atlas region
        // (systems that use ST-cropping carry no Texture Sheet, so the two don't mix).
        texture = cropByMainST(texture, data.mainST);
        this.texture = texture;
        this.trailTexture = trailTexture;
        this.blend = blend;
        this.container.sortableChildren = false;
        this.follow = followOf(data);
        this.rodRate = data.rateOverDistance ? sampleScalar(data.rateOverDistance, 0.5, 0) : 0;
        this.volWorld = worldVelocityOverLife(data);
        this.volCurve = this.volWorld && data.velocityOverLife?.curve?.length ? data.velocityOverLife.curve : null;
        this.rate = data.emission?.rate ? sampleScalar(data.emission.rate, 0.5, 0) : 0;
        // Trails render behind the particle heads.
        this.trailLayer = data.trail ? new PIXI.Container() : null;
        if (this.trailLayer) this.container.addChild(this.trailLayer);
        // Ribbon mode is one polyline through the whole system, not a ribbon per
        // particle, so it replaces the per-particle rope path entirely.
        this.ribbon = data.trail?.mode === "ribbon" && this.trailLayer ? new RibbonTrail(data.trail, trailTexture ?? this.texture, this.trailLayer, Math.min(data.maxParticles || PER_SYSTEM_CAP, PER_SYSTEM_CAP)) : null;

        // Texture Sheet Animation: slice the atlas into tile frames so each
        // particle shows one animating cell, not the whole grid stamped at once
        // (e.g. SilverAsh the Reignfrost's 7×3 purple-sword flipbook `tex6`).
        const sheet = data.sheet;
        const tx = sheet?.tilesX ?? 1;
        const ty = sheet?.tilesY ?? 1;
        if (sheet && tx * ty > 1) {
            const base = texture.baseTexture;
            const fw = Math.floor(base.width / tx);
            const fh = Math.floor(base.height / ty);
            // Some flipbook atlases carry a few cells that are opaque, desaturated
            // grey FILLS (flow/fade frames, not shaped sprites) — landing a particle
            // on one stamps a hard grey rectangle over the scene (Hoshiguma Alter's
            // tex1 3×15 ice-strand sheet has 4). Detect and blank those cells so the
            // particle simply vanishes on them, like the whole-texture flow-map skip.
            const bad = sheetBadCells(base, tx, ty);
            for (let row = 0; row < ty; row++) {
                for (let col = 0; col < tx; col++) {
                    const cell = row * tx + col;
                    this.frames.push(bad.has(cell) ? PIXI.Texture.EMPTY : new PIXI.Texture(base, new PIXI.Rectangle(col * fw, row * fh, fw, fh)));
                }
            }
            this.spriteW = fw || 1;
            this.spriteH = fh || 1;
        } else {
            this.spriteW = texture.width || 1;
            this.spriteH = texture.height || 1;
        }
    }

    /** The texture frame for a particle at life-fraction `lf` (sheet flipbook). */
    private frameAt(lf: number): PIXI.Texture {
        if (this.frames.length === 0) return this.texture;
        const sheet = this.data.sheet;
        const prog = sheet?.frameOverTime ? sampleCurve(sheet.frameOverTime, lf) : lf;
        const cycles = sheet?.cycles && sheet.cycles > 0 ? sheet.cycles : 1;
        const idx = Math.min(this.frames.length - 1, Math.max(0, Math.floor(prog * cycles * this.frames.length) % this.frames.length));
        return this.frames[idx];
    }

    /** Create the per-particle display object. Overridden by {@link MeshEmitter}
     *  to instance a mesh instead of a billboard sprite. */
    protected createParticleDisp(): PIXI.Sprite | PIXI.Mesh {
        const sprite = new PIXI.Sprite(this.frames[0] ?? this.texture);
        // PIXI's anchor is the texture point pinned to the sprite's position AND the point it
        // rotates about — precisely Unity's particle pivot. Unity's +Y is up, PIXI's is down,
        // hence the sign flip on y. Default (0.5, 0.5) is the centred quad.
        const pv = this.data.pivot;
        sprite.anchor.set(0.5 + (pv ? pv[0] : 0), 0.5 - (pv ? pv[1] : 0));
        sprite.blendMode = this.blend === "additive" ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
        // Additive billboards carry Unity's ×2 additive factor via a boosted batch plugin;
        // `tint` cannot express it (it clamps at 1.0). No-op while the boost is 1.
        if (this.blend === "additive" && additiveSpriteBoost() !== 1) sprite.pluginName = ADDITIVE_BOOST_PLUGIN;
        // Unity render mode "None": the particle draws NO head sprite — only its trail
        // ribbon (Skadi2 iteration's `xian` threads). Drawing the head anyway stamps a
        // bright sprite at the ribbon's tip that the game never shows. The particle still
        // simulates (and still lays its trail); only the head is invisible.
        if (this.data.renderMode === "none") sprite.renderable = false;
        return sprite;
    }

    /** Apply the per-frame transform + colour to a particle's display object.
     *  Overridden by {@link MeshEmitter}. `sz` is the particle's current size (px),
     *  `hex`/`alpha` its tint and opacity, `lf` its life-fraction (sheet frame). */
    protected applyDisp(disp: PIXI.Sprite | PIXI.Mesh, p: IParticle, sz: number, hex: number, alpha: number, lf: number): void {
        const s = disp as PIXI.Sprite;
        if (this.frames.length) s.texture = this.frameAt(lf);
        s.position.set(p.x, -p.y);
        const base = sz / this.spriteW;
        // A STRETCHED billboard already derives its length from `lengthScale · size`
        // along the screen velocity (below), so Unity's per-axis `size3D` height is not
        // a second Y scale there — only a plain billboard uses it as the quad's height.
        // The quad is `size` wide by `size · aspectY` tall in SCENE px, so each axis
        // divides by its own sprite extent — scaling Y by the WIDTH divisor would apply
        // the sprite's own aspect a second time. Only a system that actually authors
        // `startSizeY` switches to the per-axis divisor; every other system keeps the
        // single `base` scale (which preserves the sprite's own aspect) untouched.
        const sized3D = !!this.data.startSizeY && this.data.renderMode !== "stretch";
        const baseY = sized3D ? (sz * p.aspectY) / this.spriteH : base;
        if (this.data.renderMode === "stretch") {
            // Unity "Stretched Billboard" (rain streaks, spark trails): elongate the sprite ALONG
            // its screen velocity to the AUTHORED length `|lengthScale|·size + velocityScale·speed`
            // (px; see IParticleSystemData.stretch). lengthScale carries the stretch for nearly
            // every system (a fixed multiple of the sprite, speed-independent — so fast rain stays
            // a short streak, not a long one); velocityScale adds a speed term only where authored.
            // Gated to renderMode "stretch"; no per-skin value. Absent → Unity defaults (2, 0).
            const st = this.data.stretch;
            const lenScale = st ? Math.abs(st.lengthScale) : 2;
            const velScale = st ? st.velocityScale : 0;
            const vx = p.vx + (this.volWorld?.x ?? 0);
            const vyScreen = -(p.vy + (this.volWorld?.y ?? 0)); // screen Y is -p.y
            const spd = Math.hypot(vx, vyScreen);
            const len = lenScale * sz + velScale * spd;
            if (spd > 1 && len > sz) {
                s.rotation = Math.atan2(vyScreen, vx) - Math.PI / 2; // sprite local +Y → velocity
                s.scale.set(base, (len / sz) * base);
            } else {
                s.scale.set(base, baseY);
                s.rotation = -p.rot * DEG;
            }
        } else {
            s.scale.set(base, baseY);
            s.rotation = -p.rot * DEG;
        }
        s.tint = hex;
        s.alpha = alpha;
    }

    private spawn(): void {
        if (this.pool.length - this.free.length >= Math.min(this.data.maxParticles || PER_SYSTEM_CAP, PER_SYSTEM_CAP)) return;
        if (this.getBudget() <= 0) return;

        const d = this.data;
        const nt = d.duration > 0 ? (this.time % d.duration) / d.duration : 0;

        // Shape: emit position offset + initial direction (Y-up local frame).
        const shape = d.shape;
        let ox = 0;
        let oy = 0;
        let dirAng = 90 * DEG; // default: upward
        if (shape) {
            const arc = (shape.arcDeg ?? 360) * DEG;
            const a = Math.random() * arc;
            const [sx, sy] = shapeScale(shape);
            const r = (shape.radius ?? 0) * radialFrac(shape);
            switch (shape.type) {
                case "sphere":
                case "hemisphere":
                case "circle":
                    ox = Math.cos(a) * r * sx;
                    oy = Math.sin(a) * r * sy;
                    dirAng = Math.atan2(oy, ox);
                    break;
                case "box":
                    ox = (Math.random() - 0.5) * (shape.box?.[0] ?? 0);
                    oy = (Math.random() - 0.5) * (shape.box?.[1] ?? 0);
                    dirAng = 90 * DEG;
                    break;
                case "edge":
                    ox = (Math.random() - 0.5) * (shape.radius ?? 0) * 2;
                    oy = 0;
                    dirAng = 90 * DEG;
                    break;
                default: {
                    // cone / none: spread around up by angleDeg. A cone/edge takes NO shape
                    // scale: Unity skews the cone's base ellipse AND the emission cone with
                    // it in the emitter's 3-axis frame, which our flat 2D spawn can't
                    // reproduce — applying it to the spawn offset alone measures WORSE
                    // (Virtuosa 33.626 -> 33.640). Only the flat AREA shapes above, where
                    // our disc IS the authored emission area, take it.
                    const spread = (shape.angleDeg ?? 0) * DEG;
                    dirAng = 90 * DEG + (Math.random() - 0.5) * 2 * spread;
                    const rr = (shape.radius ?? 0) * radialFrac(shape);
                    ox = Math.cos(a) * rr;
                    oy = Math.sin(a) * rr;
                }
            }
        }
        const rotDeg = (d.rot ?? 0) + (shape?.rotDeg ?? 0);
        const cos = Math.cos(rotDeg * DEG);
        const sin = Math.sin(rotDeg * DEG);
        const posOff = shape?.posOffset ?? [0, 0];

        const speed = sampleScalar(d.startSpeed, Math.random(), nt);
        const life = Math.max(MIN_PARTICLE_LIFE, sampleScalar(d.lifetime, Math.random(), nt));
        // ONE per-particle roll drives BOTH size axes, as Unity's `size3D` does — and,
        // just as importantly, it keeps the number of `Math.random()` draws per spawn
        // fixed, so a system that gains a `startSizeY` does not re-phase the seeded
        // simulation of every particle after it.
        const sizeRand = Math.random();
        const size = sampleScalar(d.startSize, sizeRand, nt);
        const rot = sampleScalar(d.startRotation ?? { mode: "const", v: 0 }, Math.random(), nt);
        const startCol = sampleColor(d.startColor, nt);

        // Rotate the local offset + direction by the emitter's world rotation.
        const lx = ox + posOff[0];
        const ly = oy + posOff[1];
        const wx = d.pos[0] + lx * cos - ly * sin;
        const wy = d.pos[1] + lx * sin + ly * cos;
        // Cone direction: legacy path rotates the local emission angle by the emitter's flat
        // `rot`. For camera-facing (tilted) cone emitters that `rot` is degenerate noise, so
        // the exporter supplies a faithful screen-space `emitDir`; aim the cone along it (with
        // the authored `angleDeg` spread), bypassing `rot`. Position offset above still uses
        // `rot`. Systems without `emitDir` keep the exact legacy direction.
        let wDir = dirAng + rotDeg * DEG;
        if (d.emitDir) {
            const spread = (shape?.angleDeg ?? 0) * DEG;
            wDir = Math.atan2(d.emitDir[1], d.emitDir[0]) + (Math.random() - 0.5) * 2 * spread;
        }

        // Edge-clip fix: a world-space system's static spawn position can be baked for an
        // older, narrower framing calibration and now fall outside the live authored display
        // box (the settled idle box, or — during the entrance — the camera's live zoom/pan
        // box) for the whole shot, rendering as particles clipped hard at the canvas edge.
        // Reject spawns whose position is ENTIRELY outside that box (small box-relative
        // margin so streaks don't visibly pop at the boundary); partial/on-screen spawns are
        // untouched — ordinary canvas clipping handles those.
        // GATE on `!this.boneAnchor.boneName` (resolved by `driftWithBone`, called earlier
        // this same `update()`), not just `simulationSpace`: a system can be authored
        // `simulationSpace:"world"` (particles drift independently once spawned) while its
        // EMITTER still rides a live bone (Virtuosa's `followBone:"L_C_Apple_F"` comet-trail
        // dust) — `wx`/`wy` here are baked in the bone's REST frame, only becoming true
        // screen coords after `this.container`'s live bone-follow transform is applied at
        // render time, so testing them against a screen-space box directly would be a
        // coordinate-space mismatch (and could wrongly thin the apple trail as it plunges).
        // A genuinely static ambient system (no chain name resolves to a real bone, e.g.
        // Mlynar's rain — its `boneChain` names are generic rig labels, not spine bones) has
        // `boneAnchor.boneName === null`, so `wx,-wy` IS already the true screen position.
        // A large-scale glow/wash (cello's crown backdrop carries several additive/normal
        // systems sized 700–1360px, anchored FAR outside any live camera box on purpose —
        // an off-frame "light source" whose bulk paints into frame, not a mis-clipped small
        // effect) is a fundamentally different authored pattern than a small, localized
        // streak/spark (Mlynar's rain tops out around 140–270px) — a flat position+margin
        // test can't safely judge "off-frame" for the former (verified against the game
        // recording: culling cello's panel removed real, game-matching deep-blue content —
        // a regression, not a fix) but correctly identifies the latter. Gate the whole cull
        // to particles small relative to the box, so it only ever touches the
        // small/localized class the fix targets; large scene-spanning glows are untouched.
        const smallEnoughToClip = this.displayBox ? size < 0.5 * Math.min(this.displayBox.width, this.displayBox.height) : false;
        if (d.simulationSpace === "local" && this.displayBox && !this.boneAnchor.boneName && smallEnoughToClip) {
            const box = this.displayBox;
            // Add the sampled particle's own half-size on top of the box-relative margin so
            // a moderately-sized sprite isn't hard-culled just for having an edge-adjacent
            // centre (streaks/sparks don't visibly pop at the boundary).
            const margin = 0.05 * Math.max(box.width, box.height) + size / 2;
            const sx = wx;
            const sy = -wy;
            // Trajectory-aware: test the whole spawn→death SEGMENT against the box, not just
            // the spawn point. A stationary particle (speed 0, no vol) degenerates to the exact
            // same point test as before (Mlynar's static rain discs: unchanged, still culled at
            // the widened edge). A particle that DRIFTS toward the box — via its initial velocity
            // AND/OR its `velocityOverLife` (`this.volWorld`, already rotated into world) — now
            // correctly survives if its trajectory ever crosses into the box, even though its
            // spawn point starts outside it (Skadi2's fish-shoal/starlight streak systems have
            // startSpeed 0, so ALL their travel is the vol push — omitting vol here point-culled
            // them at their off-frame spawn). Screen-space velocity: `wx`/`wy` grow by
            // `(cos(wDir),sin(wDir))·speed + volWorld` per second in world Y-up space (see the
            // particle push below); screen Y is negated, so `sy` moves by the negated Y rate.
            const segEndX = sx + (Math.cos(wDir) * speed + (this.volWorld?.x ?? 0)) * life;
            const segEndY = sy - (Math.sin(wDir) * speed + (this.volWorld?.y ?? 0)) * life;
            const segMinX = Math.min(sx, segEndX);
            const segMaxX = Math.max(sx, segEndX);
            const segMinY = Math.min(sy, segEndY);
            const segMaxY = Math.max(sy, segEndY);
            if (segMaxX < box.x - margin || segMinX > box.x + box.width + margin || segMaxY < box.y - margin || segMinY > box.y + box.height + margin) {
                return;
            }
        }

        let p = this.free.pop();
        if (!p) {
            const disp = this.createParticleDisp();
            this.container.addChild(disp);
            p = { x: 0, y: 0, vx: 0, vy: 0, age: 0, life: 0, size: 0, aspectY: 1, rot: 0, rotVel: 0, rand: 0, startCol, sprite: disp };
            this.pool.push(p);
        }
        p.x = wx;
        p.y = wy;
        p.vx = Math.cos(wDir) * speed;
        p.vy = Math.sin(wDir) * speed;
        p.age = 0;
        p.life = life;
        p.size = size;
        // `startSizeY` (Unity size3D): the quad is TALLER than it is wide. Carried as a
        // ratio so the sprite path can stretch only its Y scale; 1 for every uniform system.
        p.aspectY = d.startSizeY ? sampleScalar(d.startSizeY, sizeRand, nt) / Math.max(1e-6, size) : 1;
        p.rot = rot; // degrees
        p.rotVel = typeof d.rotOverLifeDegPerSec === "number" ? d.rotOverLifeDegPerSec : 0; // legacy const
        p.rand = Math.random();
        p.startCol = startCol;
        p.sprite.visible = true;

        // Trail: give this particle a ribbon if the system trails and the ratio
        // roll passes; otherwise ensure any pooled rope stays hidden.
        const trail = d.trail;
        if (trail && trail.mode !== "ribbon" && this.trailLayer && Math.random() < (trail.ratio ?? 1)) {
            if (!p.trailPts || !p.rope) {
                p.trailPts = Array.from({ length: TRAIL_POINTS }, () => new PIXI.Point(wx, -wy));
                p.rope = new RopeMesh(this.trailTexture ?? this.texture, p.trailPts, trailWidth(trail, p.size, p.rand));
                p.rope.blendMode = trail.blend === "additive" ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
                this.trailLayer.addChild(p.rope);
            }
            for (const pt of p.trailPts) pt.set(wx, -wy);
            p.hist = [wx, wy, 0];
            p.rope.visible = true;
            // Unity's Trails module bounds a ribbon by how long each VERTEX lives:
            // `particleLifetime × trail.lifetime` seconds. Sampled per particle because
            // `trail.lifetime` may be a random range.
            p.trailLife = Math.max(1e-4, p.life * scalarOf(trail.lifetime, p.rand, 0));
            // Span this ribbon over the AUTHORED trail length (see `trailSpacing`). Speed
            // includes the constant drift, matching what the stretched-billboard path uses,
            // so a system whose motion comes from `velocityOverLife` rather than
            // `startSpeed` still gets a correctly-sized ribbon.
            const sp0 = Math.hypot(p.vx + (this.volWorld?.x ?? 0), p.vy + (this.volWorld?.y ?? 0));
            p.trailSpace = trailSpacing(sp0 * p.life * scalarOf(trail.lifetime, p.rand, 0), trail.minVertexDistance);
        } else if (p.rope) {
            p.rope.visible = false;
        }
    }

    update(dt: number, findBone?: FindBone, restBone?: RestBone, displayBox?: IAnimationBounds | null, restAtt?: RestAttachment): void {
        const d = this.data;
        this.displayBox = displayBox ?? null;
        // CINEMATIC CLOCK — absolute seconds since the `_Start` began, on the REAL timeline.
        // Advanced before any early-out so it keeps running through the start delay, and NEVER
        // scaled by `simulationSpeed`. See the clock split documented on {@link simSpeedOf}.
        this.cineTime += dt;
        // MAIN-MODULE CLOCK (`simulationSpeed` + `prewarm`).
        //
        // `simulationSpeed` scales the system's OWN clock, so everything keyed on it — the
        // emission accumulator, particle age, the over-lifetime curves, rotation and velocity —
        // slows or speeds together. The cinematic `delay` deliberately stays on the REAL clock:
        // it is sequencing handed to us by the director/`_Start` gating, not part of the
        // system's simulation, so scaling it would re-time the whole cinematic.
        const speed = simSpeedOf(d);
        if (speed !== 1) {
            if (this.time < 0) {
                this.time += dt;
                if (this.time < 0) return;
                dt = this.time; // the slice of this frame left after the delay elapsed
                this.time = 0;
            }
            dt *= speed;
        }
        this.time += dt;
        // Dormant until the cinematic start delay elapses (no emission, no particles yet).
        if (this.time < 0) return;
        // A prewarmed LOOPING system opens in steady state. Pre-roll one full `duration`
        // ONCE, the first frame it is live; `duration` is a whole number of loops, so the
        // emission phase and every `time % duration` curve land exactly where they would
        // have, while the particle ages become the steady-state spread instead of empty.
        if (!this.prewarmed && prewarmOf(d)) {
            this.prewarmed = true;
            // Pre-roll EXACTLY one `duration` of SYSTEM time. The step adapts so the
            // PREWARM_MAX_STEPS cap can never truncate the roll (a truncated roll leaves the
            // system short of steady state, which is precisely the error this is here to avoid),
            // and the final step is partial so the total lands on `duration` rather than on the
            // next whole multiple of the step.
            const step = Math.max(PREWARM_STEP, d.duration / PREWARM_MAX_STEPS);
            // The recursive call re-applies `speed`, so hand it real-time steps that scale back
            // to `step` of system time.
            const clock = this.time;
            // Unity's prewarm populates particles WITHOUT advancing playback time — the system
            // still reports t=0 when the cinematic starts. Restore BOTH clocks afterwards, or
            // every curve keyed on absolute cinematic seconds (the `scaleCurve` fed through
            // `driftWithBone`) runs a whole `duration` late.
            const cine = this.cineTime;
            for (let rolled = 0; rolled < d.duration; rolled += step) {
                this.update(Math.min(step, d.duration - rolled) / speed, findBone, restBone, displayBox, restAtt);
            }
            this.time = clock;
            this.cineTime = cine;
        }
        // Cinematic time for the scale-in curves: the emitter clock counts up from
        // `-delay`, so `this.time + delay` is absolute seconds since the `_Start` began.
        driftWithBone(this.container, d.boneChain, d.pos, d.simulationSpace, findBone, this.boneAnchor, restBone, this.follow, d.scaleCurve ? d : undefined, this.cineTime, restAtt);
        // World-space systems leave their particles behind as the emitter travels on.
        if (d.simulationSpace === "world") holdWorldSpace(this.container, this.worldSpace, this.pool);

        // Retire particles that expire THIS frame, before emitting.
        //
        // Unity's ParticleSystem ages and retires particles ahead of emission, so a slot
        // freed this frame is refillable the same frame. Our step loop runs AFTER emission
        // (it has to — it applies the frame's forces), which meant a system sitting at
        // `maxParticles` saw its cap still occupied by a corpse: `spawn()` refused, the
        // accumulator had already been consumed, and the system waited a WHOLE accumulator
        // period. At `rate = 1/s` that is a one-second late respawn.
        //
        // For a sparse emitter the lag is the entire cadence, not a detail. Skadi the
        // Corrupting Heart's sweeping streak is `maxParticles:1`, rate 1/s, lifetime 4s: its
        // particle dies at t≈5 and the replacement should be born the same instant, but we
        // deferred it to t≈6 — so at t=6 the streak is still off the right edge while the
        // game already has it crossing mid-frame, and every later cycle inherits the drift.
        //
        // Emission that arrives while the cap is genuinely full is still DROPPED, not
        // queued (`spawn()` returns early and the accumulator stays spent) — that is Unity's
        // behaviour and the reason this is an ordering fix rather than a queueing one.
        for (const p of RETIRE_BEFORE_EMIT ? this.pool : []) {
            if (!p.sprite.visible || p.age + dt < p.life) continue;
            p.sprite.visible = false;
            if (p.rope) p.rope.visible = false;
            p.age = p.life;
            this.free.push(p);
        }

        // Emission (rate + bursts), only while the system is "playing".
        const playing = d.looping || this.time <= d.duration;
        // `rateOverDistance`: trail emission per px of emitter travel (the container
        // moves when the rig rides a bone — Virtuosa's comet sheds dust down the
        // shaft). Measured on the container position, whichever follow path drives it.
        const ep = { x: this.container.position.x, y: this.container.position.y };
        const moved = this.lastEmitterPos ? Math.hypot(ep.x - this.lastEmitterPos.x, ep.y - this.lastEmitterPos.y) : 0;
        this.lastEmitterPos = ep;
        if (playing) {
            // Loop-wrap accumulator restart (see loopAccReset). Compared against the PREVIOUS
            // frame's cycle position, so it fires exactly once per wrap, before this frame's
            // contribution is added.
            if (loopAccReset() && d.looping && d.duration > 0 && this.time % d.duration < this.lastCycleT) this.emitAcc = 0;
            this.emitAcc += emissionRate(d, this.rate, this.time) * dt;
            if (this.rodRate > 0 && moved > 0) this.emitAcc += this.rodRate * moved;
            while (this.emitAcc >= 1) {
                this.emitAcc -= 1;
                this.spawn();
            }
            const cycleT = d.duration > 0 ? this.time % d.duration : this.time;
            // A magnitude check false-fires immediately after delayed activation;
            // only a genuine modulo wrap starts a new burst cycle.
            if (d.duration > 0 && cycleT < this.lastCycleT) this.firedBursts.clear();
            this.lastCycleT = cycleT;
            for (const burst of d.emission?.bursts ?? []) {
                const key = Math.floor(this.time / (d.duration || 1)) * 1000 + burst.t;
                if (cycleT >= burst.t && !this.firedBursts.has(key)) {
                    this.firedBursts.add(key);
                    for (let i = 0; i < Math.min(burst.count, PER_SYSTEM_CAP); i++) this.spawn();
                }
            }
        }

        // gravity is exported as gravityModifier×100; true accel = value×9.81 px/s², downward (−Y).
        const grav = d.gravity ? sampleScalar(d.gravity, 0.5, 0) * 9.81 : 0;
        const force = worldForceOverLife(d);
        const noise = d.noise;
        const trail = d.trail;
        const trailMinD = trail?.minVertexDistance || 1;
        const volConst = this.volWorld;
        const volCurve = this.volCurve;
        for (const p of this.pool) {
            if (!p.sprite.visible) continue;
            p.age += dt;
            if (p.age >= p.life) {
                p.sprite.visible = false;
                if (p.rope) p.rope.visible = false;
                this.free.push(p);
                continue;
            }
            const lf = p.age / p.life;
            // gravity pulls -Y (down) in our Y-up space.
            p.vy -= grav * dt;
            if (force) {
                p.vx += force.x * dt;
                p.vy += force.y * dt;
            }
            // Limit velocity over lifetime: damp the particle's speed toward the
            // curve-sampled ceiling (Unity ClampVelocityModule). Reins the star
            // glint's hot startSpeed burst into a tight stationary cluster.
            if (d.velocityClamp) {
                const limit = sampleScalar(d.velocityClamp.magnitude, 0.5, lf);
                const spd = Math.hypot(p.vx, p.vy);
                if (spd > limit && spd > 0) {
                    const k = limit / spd;
                    p.vx = lerp(p.vx, p.vx * k, d.velocityClamp.dampen);
                    p.vy = lerp(p.vy, p.vy * k, d.velocityClamp.dampen);
                }
            }
            // The drift push: per-lifetime when the exporter supplied a curve (the vector
            // TURNS over the particle's life — Skadi2 iteration's crown ring closes a
            // circle a single constant flattened into a straight line), else the constant.
            const vol = volCurve ? sampleVolCurve(volCurve, lf) : volConst;
            p.x += (p.vx + (vol?.x ?? 0)) * dt;
            p.y += (p.vy + (vol?.y ?? 0)) * dt;
            // Noise: an organic wander sampled from a time-scrolling field.
            if (noise) {
                const f = noise.frequency * 0.002;
                const ph = this.time * noise.scrollSpeed;
                p.x += fbmNoise(p.x * f + ph, p.y * f + p.rand * 17) * noise.strength * dt;
                p.y += fbmNoise(p.y * f + ph + 3.1, p.x * f + p.rand * 17 + 9.3) * noise.strength * dt;
            }
            // Integrate the authored angular-velocity curve; `rotVel` carries the legacy
            // bare-number constant so old exports are bit-for-bit unchanged.
            p.rot += (p.rotVel || rotRateAt(d.rotOverLifeDegPerSec, p.rand, lf)) * dt;

            const sz = p.size * (d.sizeOverLife ? sampleCurve(d.sizeOverLife, lf) : 1);
            // Unity multiplies startColor × colorOverLifetime (both RGB and alpha).
            // Using colorOverLife alone discards the emitter's authored tint and
            // alpha — for Logos/Pozëmka that turned dim coloured glows (startColor
            // alpha 0.2–0.68) into full-bright white, blowing out the character.
            const lifeCol = d.colorOverLife ? sampleColor(d.colorOverLife, lf, p.rand) : { r: 1, g: 1, b: 1, a: 1 };
            // …and by the material's doubled `_TintColor` (see IParticleSystemData.tint);
            // absent (the neutral) for all but a handful of systems.
            const mt = d.tint;
            const col = {
                r: p.startCol.r * lifeCol.r * (mt ? mt[0] : 1),
                g: p.startCol.g * lifeCol.g * (mt ? mt[1] : 1),
                b: p.startCol.b * lifeCol.b * (mt ? mt[2] : 1),
                a: p.startCol.a * lifeCol.a * (mt ? mt[3] : 1),
            };
            // NOTE — "over-bright rescue" MEASURED AND REJECTED (2026-08-03). `rgbToHex` feeds
            // PIXI's `Sprite.tint`, an 8-bit RGB word, so any channel the material's ×2
            // `_TintColor` pushes past 1.0 is truncated: Mlynar's campfire emitters are
            // startColor (1.00, 0.25, 0.12) × tint 2.0 = (2.0, 0.5, 0.24) and draw as
            // (1.0, 0.5, 0.24), losing the whole red doubling. Under ADDITIVE blending that
            // excess can be moved into the float alpha with no change to the product
            // (`rgb /= max`, `alpha *= max`, applied only where `a × max` still fits so the
            // trade stays an identity). It works exactly as intended and is still WRONG:
            // recovering that energy costs Mlynar 17.525 → 17.687. The clamp is not what stops
            // his lights glaring — the game simply does not show the extra energy, which is the
            // same over-saturation these glare items keep showing. Don't re-derive it.
            const hex = rgbToHex(col);
            const alpha = Math.max(0, Math.min(1, col.a));
            this.applyDisp(p.sprite, p, sz, hex, alpha, lf);

            // Trail: lay down a vertex once the head has moved `minVertexDistance` (Unity's
            // SAMPLING threshold), drop vertices older than the authored vertex lifetime
            // (Unity's LENGTH bound), then resample what survives into the fixed rope budget.
            //
            // Spacing alone cannot express this. With one sample per frame max, a ribbon
            // recorded at a fixed spacing always spans TRAIL_POINTS FRAMES once the particle
            // outruns that spacing — 0.4 s at a 33 ms tick, whatever was authored. Skadi's
            // `xian` threads author 2.0 s × 0.05 = 0.1 s and so drew 4× too long, and the
            // error scaled with frame rate rather than with the data.
            // (Ribbon mode has no per-particle history — see the rebuild below.)
            if (trail && trail.mode !== "ribbon" && p.rope?.visible && p.hist && p.trailPts) {
                const dx = p.x - p.hist[0];
                const dy = p.y - p.hist[1];
                // `?trailmodel=span` restores the superseded frame-bounded model for a same-batch
                // A/B. Diagnostic only; the age model is the shipped default.
                const spanMode = TRAIL_W_DIAG.spanModel;
                const step = spanMode ? (p.trailSpace ?? trailMinD) : trailMinD;
                if (dx * dx + dy * dy >= step * step) {
                    p.hist.unshift(p.x, p.y, p.age);
                    if (p.hist.length > (spanMode ? TRAIL_POINTS : TRAIL_HIST_MAX) * 3) p.hist.length = (spanMode ? TRAIL_POINTS : TRAIL_HIST_MAX) * 3;
                }
                // Polyline = the LIVE head, then every committed vertex still inside its
                // lifetime. The head is kept separate from `hist` so the distance test above
                // keeps measuring against the last COMMITTED vertex; folding it in would
                // re-arm the test every frame and defeat `minVertexDistance`. One expired
                // vertex is retained so the tail interpolates to where it is rather than
                // snapping back a whole sample interval.
                if (spanMode) {
                    p.trailPts[0].set(p.x, -p.y);
                    for (let i = 1; i < TRAIL_POINTS; i++) {
                        const hi = i * 3;
                        if (hi + 1 < p.hist.length) p.trailPts[i].set(p.hist[hi], -p.hist[hi + 1]);
                        else p.trailPts[i].copyFrom(p.trailPts[i - 1]);
                    }
                } else {
                    const cutoff = p.age - (p.trailLife ?? 0);
                    const committed = p.hist.length / 3;
                    TRAIL_LINE[0] = p.x;
                    TRAIL_LINE[1] = p.y;
                    let n = 1;
                    for (let i = 0; i < committed; i++) {
                        TRAIL_LINE[n * 2] = p.hist[i * 3];
                        TRAIL_LINE[n * 2 + 1] = p.hist[i * 3 + 1];
                        n++;
                        if (p.hist[i * 3 + 2] < cutoff) break;
                    }
                    resampleTrail(TRAIL_LINE, n, p.trailPts);
                }
                // Width follows the particle's CURRENT size so a ribbon grows and fades with
                // `sizeOverLife`, and drives the geometry rebuild SimpleRope's autoUpdate did.
                p.rope.setWidth(trailWidth(trail, sz, p.rand));
                // The ribbon takes the particle's START colour, NOT its faded current colour.
                //
                // Unity's trail carries its OWN colour (`colorOverLifetime` / `colorOverTrail`);
                // `inheritParticleColor` multiplies in the emitter's particle colour, not the
                // head's `colorOverLifetime` fade. Handing the rope the sprite's current `hex`/
                // `alpha` instead makes the whole ribbon vanish the moment the head fades out —
                // even while metres of it are still on screen.
                //
                // Skadi the Corrupting Heart's red beam is exactly that: a ~3000px ribbon whose
                // head has faded (its gradient hits alpha 0 at 69% of life and stays there), so
                // we drew the entire streak at alpha 0.000 through t=6..7 while the game shows
                // two bright red beams crossing the full frame. The other trail implementation
                // in this file already does the right thing — `inheritParticleColor ? p.startCol
                // : white` — so this was an inconsistency between the two, not a design choice.
                const towned = (trail.inheritParticleColor ?? true) && TRAIL_START_COLOR;
                const tlife = trail.colorOverLifetime ? sampleColor(trail.colorOverLifetime, lf, p.rand) : null;
                const tcol = TRAIL_START_COLOR
                    ? {
                          r: (towned ? p.startCol.r : 1) * (tlife ? tlife.r : 1) * (mt ? mt[0] : 1),
                          g: (towned ? p.startCol.g : 1) * (tlife ? tlife.g : 1) * (mt ? mt[1] : 1),
                          b: (towned ? p.startCol.b : 1) * (tlife ? tlife.b : 1) * (mt ? mt[2] : 1),
                          a: (towned ? p.startCol.a : 1) * (tlife ? tlife.a : 1) * (mt ? mt[3] : 1),
                      }
                    : { r: col.r, g: col.g, b: col.b, a: col.a };
                p.rope.tint = rgbToHex(tcol);
                p.rope.alpha = Math.max(0, Math.min(1, tcol.a));
            }
        }

        // Ribbon mode: one polyline through the system's live particles, OLDEST first
        // (Unity's threading order, and the direction `colorOverTrail`/`widthOverTrail`
        // are sampled along). Rebuilt from scratch each frame — the control points are
        // the particles themselves, so there is no history to keep.
        if (this.ribbon) {
            const live = this.pool.filter((q) => q.sprite.visible);
            live.sort((q1, q2) => q2.age - q1.age);
            this.ribbon.rebuild(live);
        }
    }

    /** DIAGNOSTIC (`?dumplayers=1`): why is this system silent? Reports the emitter's own
     *  clock, the emission rate the data evaluates to RIGHT NOW, and the cap/pool state, so a
     *  system that never spawns can be told apart from one that is merely dormant. */
    dbg(): Record<string, number | string | boolean | null> {
        const d = this.data;
        return {
            t: Number(this.time.toFixed(3)),
            cine: Number(this.cineTime.toFixed(3)),
            dur: d.duration,
            loop: !!d.looping,
            rateNow: Number(emissionRate(d, this.rate, this.time).toFixed(3)),
            acc: Number(this.emitAcc.toFixed(3)),
            bone: d.boneChain ? String(d.boneChain[d.boneChain.length - 1]) : null,
            boneName: this.boneAnchor.boneName, boneOk: this.boneAnchor.boneName ? !!this.boneAnchor.ref : null, resolved: this.boneAnchor.resolved,
            simSpace: String(d.simulationSpace),
        };
    }

    liveCount(): number {
        const n = this.pool.length - this.free.length;
        // Latch, not a sample. A fixed beat list cannot answer "does this system EVER emit":
        // a 0.3s burst 13s into a 14.5s entrance falls between beats and reads as dead. The
        // module calls `liveCount()` every frame, so latching here makes the answer exact for
        // the cost of one comparison.
        if (n > 0) this.everLive = true;
        return n;
    }

    destroy(): void {
        this.container.destroy({ children: true });
    }
}

/**
 * Mesh-render-mode emitter: each particle instances a shared MESH (ice-crystal
 * shards, ribbons, blades) transformed by its position/size/rotation, instead of a
 * camera-facing billboard. The exporter emits the mesh's local-space geometry in
 * `data.mesh`. Reuses the full billboard SIMULATION (spawn/shape/movement/lifetime/
 * colour) from {@link Emitter}; only the per-particle display object differs.
 */
// Mesh-render particles bake an edge-falloff into per-vertex ALPHA (RGB stays
// white; only aColor.a varies 0..1 to soften the shard rim). Pixi's stock
// MeshMaterial ignores vertex colour and would render every crystal as a hard
// glassy panel, so we run a tiny vcolor shader: sample the texture, multiply by
// the per-particle tint and the per-vertex mask, and emit premultiplied so ADD
// and NORMAL blend correctly (same convention as the Ram emitter).
const MESH_VERT = `
precision highp float;
attribute vec2 aVertexPosition;
attribute vec2 aUV;
attribute vec4 aColor;
uniform mat3 translationMatrix;
uniform mat3 projectionMatrix;
varying vec2 vUV;
varying vec4 vColor;
void main() {
    gl_Position = vec4((projectionMatrix * translationMatrix * vec3(aVertexPosition, 1.0)).xy, 0.0, 1.0);
    vUV = aUV;
    vColor = aColor;
}
`;

const MESH_FRAG = `
precision highp float;
varying vec2 vUV;
varying vec4 vColor;
uniform sampler2D uSampler;
uniform vec3 uTint;   // per-particle startColour × lifetime tint
uniform float uAlpha; // per-particle opacity (fade over life)
uniform float uBoost; // additive HDR boost (ADDITIVE_MESH_BOOST for ADD); 1.0 for
                      // NORMAL so painted frames don't over-brighten.
void main() {
    vec4 tex = texture2D(uSampler, vUV);       // premultiplied
    float cov = vColor.a * uAlpha;             // per-vertex edge mask × particle alpha
    gl_FragColor = vec4(tex.rgb * uTint * vColor.rgb * cov * uBoost, tex.a * cov);
}
`;

/** HDR boost for ADDITIVE mesh particles: Unity's ×2 additive particle shader plus
 *  headroom so thin glowing shards survive the tonemap. */
const ADDITIVE_MESH_BOOST = 2.5;

/** How the two independent additive attenuations combine.
 *
 *  `additivePileGain` corrects OVER-PILING (many coincident particles counted many times);
 *  `EFFECT_PARTICLE_GAIN` tames a LARGE additive sheet on a self-lit dark backdrop. They
 *  answer different questions, and the shipped behaviour MULTIPLIES them — which on cello
 *  (the only skin where the second fires) leaves her 1313px `star_large` flares at
 *  0.16-0.24 of their authored energy. `?gainmode=` selects the rule so the choice can be
 *  measured instead of assumed:
 *    compound (default, shipped)  pile x effect
 *    min                          the STRONGER single attenuation, never their product
 *    effect                       effect supersedes pile where it applies
 *    pile                         pile only (no large-sheet taming)
 *  Only systems where `temperLargeAdditive` fires differ between modes, so every other skin
 *  is byte-identical in all four. */
function combineAdditiveGains(pile: number, temper: boolean): number {
    const mode = typeof window === "undefined" ? "compound" : (new URLSearchParams(window.location.search).get("gainmode") ?? "compound");
    if (!temper) return pile;
    switch (mode) {
        case "min":
            return Math.min(pile, EFFECT_PARTICLE_GAIN);
        case "effect":
            return EFFECT_PARTICLE_GAIN;
        case "pile":
            return pile;
        default:
            return pile * EFFECT_PARTICLE_GAIN;
    }
}

/** DIAGNOSTIC (`?psdt=<seconds>`): offset every particle system's own clock, leaving the camera
 *  and the spine on theirs. Virtuosa's diamond rings (sys55) are emitted at a constant 2/s with a
 *  5 s lifetime, so ten rings are alive at once, 0.5 s apart in age, and their radii follow
 *  `sizeOverLife(age/5)`. Their PHASE is therefore `(t - t0) mod 0.5` — and a phase error cannot
 *  be measured by re-rendering at a different time, because at t=10 cello pans at 435 px/s and
 *  the whole frame moves with it. This moves the rings alone. Inert (0) by default. */
/** DIAGNOSTIC (`?umgain=<f>`): override the UNTEXTURED-mesh gain (default 2 — the `col += col`
 *  half-neutral convention). Sweeping it tests whether that ×2 still holds on the current
 *  baseline; it was calibrated on one that scored 35.x. */
function untexMeshGain(): number {
    if (typeof window === "undefined") return 2;
    const v = parseFloat(new URLSearchParams(window.location.search).get("umgain") ?? "");
    return Number.isFinite(v) && v >= 0 ? v : 2;
}

function psDt(): number {
    if (typeof window === "undefined") return 0;
    const v = parseFloat(new URLSearchParams(window.location.search).get("psdt") ?? "");
    return Number.isFinite(v) ? v : 0;
}

/** DIAGNOSTIC (`?psonly=<i>` / `?psoff=<i>`, comma lists): isolate or drop individual
 *  particle SYSTEMS by their index in `data.systems`.
 *
 *  MUST be called at EVERY emitter-construction path. There are FOUR (RamEmitter, the two
 *  MeshEmitter branches, and the sprite Emitter), each with its own `addChild` and its own
 *  `continue`s — wiring only the sprite path silently leaves mesh/ram systems unfilterable,
 *  which reads as "the filter does nothing" and invalidates any conclusion drawn from it. */
function applyPsDiag(data: IParticlesData, sys: IParticleSystemData, container: PIXI.Container): void {
    if (typeof window === "undefined") return;
    const q = new URLSearchParams(window.location.search);
    const only = q.get("psonly");
    const off = q.get("psoff");
    if (!only && !off) return;
    const idx = data.systems.indexOf(sys);
    if (only && !only.split(",").map(Number).includes(idx)) container.renderable = false;
    if (off?.split(",").map(Number).includes(idx)) container.renderable = false;
}

/** Batch plugin name for ADDITIVE particle SPRITES (see {@link ensureAdditiveSpriteBoost}). */
const ADDITIVE_BOOST_PLUGIN = "dynAdditiveBoost";

/** The same ×2 Unity additive factor {@link ADDITIVE_MESH_BOOST} applies on the MESH path,
 *  but for BILLBOARD sprites — which had no equivalent because PIXI's `Sprite.tint` clamps
 *  at 1.0, so an additive billboard could never contribute more than its authored colour.
 *  The asymmetry is visible: Skadi's crown shoal is a `renderMode:"billboard"` additive
 *  emitter whose white texture and warm authored tint render warm-orange, where the game's
 *  same fish read white because a ×2 contribution clips R and lifts G/B toward neutral.
 *
 *  MEASURED AND REJECTED as a default (2026-08-01): enabling it together with the exporter's
 *  `scalingMode` size fix regressed every reference skin (mly 30.816 -> 32.536,
 *  cel 23.367 -> 24.139, ska 13.789 -> 13.994). Kept OFF; `?spriteboost=<f>` turns it on. */
/* REAL BUG, MEASURED NEGLIGIBLE (2026-08-03) — recorded so it is not re-derived.
 * PIXI's batcher packs tint x alpha into an 8-bit word, so a particle alpha above 1 is silently
 * clamped. Unity clamps the PRODUCT instead: the Additive fragment writes
 * `SV_Target0.w = clamp(tex.a x 2 x startColor.a x _TintColor.a, 0, 1)`, per texel. So a system
 * with `startColor.a x tint[3] > 1` is drawn up to 2x too dim by us — and that is 517 of the 887
 * tinted additive systems, across 54 skins.
 *
 * An exact fix exists and was built: the CONSTANT excess (`startColor.a x tint[3]`) rides the
 * boost plugin, which multiplies AFTER the texture sample and lets the framebuffer clamp, while
 * `sprite.alpha` carries only the per-particle `colorOverLifetime` factor. The split is an
 * identity: `min(tex x tint x excess x lifeAlpha, 1)`.
 *
 * NOT SHIPPED because it is visually inert: the affected systems are small and sparse. On
 * `char_2023_ling_2` (39 affected systems, one of the worst in the corpus) it moves 14-20 PIXELS,
 * and all three benchmarks are bit-identical (17.525 / 19.270 / 10.530). A complete version would
 * need a plugin per quantised excess (the corpus spreads 1.0-2.0, with 208 systems at exactly
 * 2.0) — machinery that buys nothing measurable. */

function additiveSpriteBoost(): number {
    if (typeof window === "undefined") return 1;
    const v = parseFloat(new URLSearchParams(window.location.search).get("spriteboost") ?? "");
    return Number.isFinite(v) && v > 0 ? v : 1;
}

/** Unity retires expired particles BEFORE emitting, so a slot freed this frame is refillable
 *  the same frame — see the retirement pre-pass in `Emitter.update` for why that matters.
 *
 *  `?retire=0` restores the previous emit-then-age order. It exists so the change can be A/B'd
 *  in ONE build: `particles.ts` carries a large uncommitted delta, so reverting it with
 *  `git checkout` to measure a baseline destroys unrelated work and renders garbage. */
function retireBeforeEmit(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("retire") !== "0";
}

/** Resolved once: the query string cannot change without a reload. */
const RETIRE_BEFORE_EMIT = retireBeforeEmit();

/** A per-particle ribbon takes the particle's START colour rather than its faded CURRENT colour
 *  (`?trailcol=0` restores the old behaviour).
 *
 *  Unity stores each trail vertex's colour AS IT WAS LAID, so an old vertex stays bright while
 *  the head fades. A PIXI rope carries ONE tint+alpha for the whole ribbon, so neither extreme is
 *  exact — but "start colour" is far closer than "current colour", which zeroes the entire ribbon
 *  the instant the head's `colorOverLifetime` reaches 0 and made Skadi's ~3000px red beam vanish
 *  outright. True per-vertex colour would need a vertex-coloured rope.
 *
 *  DEFAULT OFF, opt in with `?trailcol=1`. The mechanism is right and the restored ribbon is
 *  real content the game shows — but it currently lands ~70 rows BELOW the game's beam, so
 *  enabling it measures WORSE (mly 30.816→30.826, ska 13.795→13.830) by adding correct content
 *  in the wrong place. It should become the default once the streak's position/tilt is fixed;
 *  until then this stays off rather than trading a visible defect for a measured regression. */
const TRAIL_START_COLOR = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("trailcol") === "1";

let additiveBoostReady = false;
/** Register a batch plugin identical to PIXI's default except that it scales RGB by the
 *  additive factor. MUST run before the `Renderer` is constructed — renderer plugins are
 *  instantiated with the renderer, so a later `extensions.add` never reaches it. */
export function ensureAdditiveSpriteBoost(): void {
    if (additiveBoostReady) return;
    additiveBoostReady = true;
    const boost = additiveSpriteBoost();
    if (boost === 1) return; // inert: leave every sprite on the stock batch plugin
    const frag = PIXI.BatchRenderer.defaultFragmentTemplate.replace("gl_FragColor = color * vColor;", `vec4 c = color * vColor;\n    gl_FragColor = vec4(c.rgb * ${boost.toFixed(4)}, c.a);`);
    class AdditiveBoostRenderer extends PIXI.BatchRenderer {
        constructor(renderer: PIXI.Renderer) {
            super(renderer);
            this.shaderGenerator = new PIXI.BatchShaderGenerator(PIXI.BatchRenderer.defaultVertexSrc, frag);
        }
    }
    PIXI.extensions.add({
        name: ADDITIVE_BOOST_PLUGIN,
        type: PIXI.ExtensionType.RendererPlugin,
        ref: AdditiveBoostRenderer,
    });
}

/**
 * Unity Trails in **Ribbon** mode: not one ribbon per particle, but ONE polyline
 * threaded through all of the system's live particles ordered by age, `ribbonCount`
 * of them interleaved (particle `i` → ribbon `i % ribbonCount`). The emitter becomes
 * a way of laying down a moving LINE rather than a spray of sprites — which is why
 * such a system is usually paired with `RenderMode.None`: the particles themselves
 * are invisible control points and the ribbon is the entire visible effect.
 *
 * Rendered as a triangle strip so width and colour can vary along the ribbon, which
 * `PIXI.SimpleRope` (uniform width, single tint) cannot express — and the variation is
 * the whole look: Skadi the Corrupting Heart's `hongxian_01` is banded coral by
 * `colorOverTrail` over a GREY texture and a white material, so a single-tint rope
 * would draw it grey.
 */
class RibbonTrail {
    private readonly meshes: PIXI.Mesh<PIXI.Shader>[] = [];
    private readonly pos: Float32Array[] = [];
    private readonly uv: Float32Array[] = [];
    private readonly col: Float32Array[] = [];
    private readonly posBuf: PIXI.Buffer[] = [];
    private readonly uvBuf: PIXI.Buffer[] = [];
    private readonly colBuf: PIXI.Buffer[] = [];
    private readonly count: number;

    /** `cap` is the emitter's per-system particle ceiling — the most control points a
     *  single ribbon can ever have, so the geometry is allocated once and never grows. */
    constructor(
        private readonly trail: ITrail,
        texture: PIXI.Texture,
        layer: PIXI.Container,
        private readonly cap: number,
    ) {
        this.count = Math.max(1, Math.min(8, trail.ribbonCount ?? 1));
        type BufArg = ConstructorParameters<typeof PIXI.Buffer>[0];
        for (let r = 0; r < this.count; r++) {
            // Two vertices (±half-width across the ribbon) per control point.
            const nv = this.cap * 2;
            const pos = new Float32Array(nv * 2);
            const uv = new Float32Array(nv * 2);
            const col = new Float32Array(nv * 4);
            const idx = new Uint16Array(Math.max(0, this.cap - 1) * 6);
            for (let s = 0; s + 1 < this.cap; s++) {
                const v = s * 2;
                const o = s * 6;
                idx[o] = v;
                idx[o + 1] = v + 1;
                idx[o + 2] = v + 2;
                idx[o + 3] = v + 1;
                idx[o + 4] = v + 3;
                idx[o + 5] = v + 2;
            }
            const pb = new PIXI.Buffer(pos as unknown as BufArg);
            const ub = new PIXI.Buffer(uv as unknown as BufArg);
            const cb = new PIXI.Buffer(col as unknown as BufArg);
            const geo = new PIXI.Geometry();
            geo.addAttribute("aVertexPosition", pb, 2);
            geo.addAttribute("aUV", ub, 2);
            geo.addAttribute("aColor", cb, 4);
            geo.addIndex(new PIXI.Buffer(idx as unknown as BufArg));
            const additive = trail.blend === "additive";
            const shader = PIXI.Shader.from(MESH_VERT, MESH_FRAG, {
                uSampler: texture,
                uTint: [1, 1, 1],
                uAlpha: 1,
                uBoost: additive ? ADDITIVE_MESH_BOOST : 1,
            });
            const mesh = new PIXI.Mesh(geo, shader as unknown as PIXI.MeshMaterial);
            mesh.blendMode = additive ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
            mesh.visible = false;
            layer.addChild(mesh);
            this.meshes.push(mesh as unknown as PIXI.Mesh<PIXI.Shader>);
            this.pos.push(pos);
            this.uv.push(uv);
            this.col.push(col);
            this.posBuf.push(pb);
            this.uvBuf.push(ub);
            this.colBuf.push(cb);
        }
    }

    /** Rewrite every ribbon's geometry from the emitter's live particles.
     *  `live` must be ordered OLDEST first — that is the direction Unity threads the
     *  ribbon, and `colorOverTrail`/`widthOverTrail` are sampled along it. */
    rebuild(live: IParticle[]): void {
        const t = this.trail;
        const sizeAffects = t.sizeAffectsWidth ?? true;
        for (let r = 0; r < this.count; r++) {
            const mesh = this.meshes[r];
            // Interleave: this ribbon takes every `count`-th particle.
            const pts: IParticle[] = [];
            for (let i = r; i < live.length && pts.length < this.cap; i += this.count) pts.push(live[i]);
            // A single point has no segment to draw across.
            if (pts.length < 2) {
                mesh.visible = false;
                continue;
            }
            mesh.visible = true;
            const pos = this.pos[r];
            const uv = this.uv[r];
            const col = this.col[r];
            const n = pts.length;
            for (let i = 0; i < n; i++) {
                const p = pts[i];
                const u = i / (n - 1);
                // Tangent from the neighbours (one-sided at the ends), then the
                // across-ribbon normal. Coincident neighbours would give a zero
                // tangent and collapse the quad, so fall back to +X.
                const a = pts[Math.max(0, i - 1)];
                const b = pts[Math.min(n - 1, i + 1)];
                let tx = b.x - a.x;
                let ty = -b.y - -a.y;
                const len = Math.hypot(tx, ty);
                if (len > 1e-6) {
                    tx /= len;
                    ty /= len;
                } else {
                    tx = 1;
                    ty = 0;
                }
                const half = 0.5 * (sizeAffects ? p.size : 1) * (t.widthOverTrail ? scalarOf(t.widthOverTrail, p.rand, u) : 1);
                const nx = -ty * half;
                const ny = tx * half;
                const x = p.x;
                const y = -p.y;
                const v = i * 4;
                pos[v] = x + nx;
                pos[v + 1] = y + ny;
                pos[v + 2] = x - nx;
                pos[v + 3] = y - ny;
                uv[v] = u;
                uv[v + 1] = 0;
                uv[v + 2] = u;
                uv[v + 3] = 1;
                // Unity multiplies the along-RIBBON band by the along-LIFETIME colour,
                // and by the particle's own colour only when `inheritParticleColor`.
                const band = t.colorOverTrail ? sampleColor(t.colorOverTrail, u, p.rand) : { r: 1, g: 1, b: 1, a: 1 };
                const lifeCol = t.colorOverLifetime ? sampleColor(t.colorOverLifetime, Math.min(1, p.age / p.life), p.rand) : { r: 1, g: 1, b: 1, a: 1 };
                const own = t.inheritParticleColor ? p.startCol : { r: 1, g: 1, b: 1, a: 1 };
                const cr = band.r * lifeCol.r * own.r;
                const cg = band.g * lifeCol.g * own.g;
                const cb2 = band.b * lifeCol.b * own.b;
                const ca = Math.max(0, Math.min(1, band.a * lifeCol.a * own.a));
                const c = i * 8;
                for (let k = 0; k < 2; k++) {
                    col[c + k * 4] = cr;
                    col[c + k * 4 + 1] = cg;
                    col[c + k * 4 + 2] = cb2;
                    col[c + k * 4 + 3] = ca;
                }
            }
            // Degenerate the unused tail so stale points from a longer previous frame
            // cannot leave a stray quad hanging across the scene.
            for (let i = n; i < this.cap; i++) {
                const v = i * 4;
                pos[v] = pos[v + 1] = pos[v + 2] = pos[v + 3] = 0;
                const c = i * 8;
                for (let k = 0; k < 8; k++) col[c + k] = 0;
            }
            this.posBuf[r].update();
            this.uvBuf[r].update();
            this.colBuf[r].update();
        }
    }
}

class MeshEmitter extends Emitter {
    private readonly geometry: PIXI.Geometry;
    private readonly meshTexture: PIXI.Texture;
    /** Shader-family COLOUR gain applied to a NORMAL-blend mesh — the Torappu
     *  compositors' ×2 half-neutral convention (see `RAM_FRAG`'s `col += col`). Colour
     *  only: the particle's own alpha stays authored, so the ribbon keeps the ~25 %
     *  see-through the capture shows. 1 (the default) leaves every existing mesh
     *  emitter byte-identical. */
    private readonly gain: number;
    /** Extra screen rotation (deg) on top of the particle's own — the emitter transform's
     *  z-angle, which poses a LOCAL-space mesh particle. 0 (the default) for every textured
     *  mesh emitter, whose current orientation measures correct as it stands. */
    private readonly rotOffsetDeg: number;

    constructor(data: IParticleSystemData, texture: PIXI.Texture, blend: "additive" | "normal", getBudget: () => number, gain = 1, rotOffsetDeg = 0) {
        super(data, texture, null, blend, getBudget);
        this.meshTexture = texture;
        this.gain = gain;
        this.rotOffsetDeg = rotOffsetDeg;
        // Build the shared geometry ONCE: mesh-local positions with Y negated and
        // UV V-flipped (authored Y-up / Unity-V → Pixi), exactly like the scene-mesh
        // layers (sceneMesh.buildLayerMesh), plus the per-vertex RGBA edge mask.
        const m = data.mesh ?? { pos: [], uv: [], idx: [] };
        const n = m.pos.length / 2;
        const vertices = new Float32Array(m.pos.length);
        const uvs = new Float32Array(n * 2);
        const colors = new Float32Array(n * 4);
        const hasCol = Array.isArray(m.col) && m.col.length >= n * 4;
        for (let i = 0; i < n; i++) {
            vertices[i * 2] = m.pos[i * 2];
            vertices[i * 2 + 1] = -m.pos[i * 2 + 1];
            uvs[i * 2] = m.uv[i * 2] ?? 0;
            uvs[i * 2 + 1] = 1 - (m.uv[i * 2 + 1] ?? 0);
            colors[i * 4] = hasCol ? (m.col as number[])[i * 4] : 1;
            colors[i * 4 + 1] = hasCol ? (m.col as number[])[i * 4 + 1] : 1;
            colors[i * 4 + 2] = hasCol ? (m.col as number[])[i * 4 + 2] : 1;
            colors[i * 4 + 3] = hasCol ? (m.col as number[])[i * 4 + 3] : 1;
        }
        type BufArg = ConstructorParameters<typeof PIXI.Buffer>[0];
        this.geometry = new PIXI.Geometry();
        this.geometry.addAttribute("aVertexPosition", new PIXI.Buffer(vertices as unknown as BufArg), 2);
        this.geometry.addAttribute("aUV", new PIXI.Buffer(uvs as unknown as BufArg), 2);
        this.geometry.addAttribute("aColor", new PIXI.Buffer(colors as unknown as BufArg), 4);
        this.geometry.addIndex(new PIXI.Buffer(new Uint16Array(m.idx) as unknown as BufArg));
    }

    protected override createParticleDisp(): PIXI.Mesh {
        // One shader instance per particle so uTint/uAlpha vary independently; the
        // program itself is cached by source, so this only clones the uniform group.
        const shader = PIXI.Shader.from(MESH_VERT, MESH_FRAG, {
            uSampler: this.meshTexture,
            uTint: new Float32Array([1, 1, 1]),
            uAlpha: 1,
            uBoost: this.blend === "additive" ? ADDITIVE_MESH_BOOST : this.gain,
        });
        const mesh = new PIXI.Mesh(this.geometry, shader);
        mesh.blendMode = this.blend === "additive" ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
        return mesh as unknown as PIXI.Mesh;
    }

    protected override applyDisp(disp: PIXI.Sprite | PIXI.Mesh, p: IParticle, sz: number, hex: number, alpha: number): void {
        const m = disp as PIXI.Mesh;
        m.position.set(p.x, -p.y);
        // The geometry is raw mesh-local; Unity scales the mesh by the particle
        // size, and startSize is already emitter-scaled to px → finalPx = local × sz.
        m.scale.set(sz);
        m.rotation = (this.rotOffsetDeg - p.rot) * DEG;
        const u = (m.shader as PIXI.Shader).uniforms;
        (u.uTint as Float32Array)[0] = ((hex >> 16) & 0xff) / 255;
        (u.uTint as Float32Array)[1] = ((hex >> 8) & 0xff) / 255;
        (u.uTint as Float32Array)[2] = (hex & 0xff) / 255;
        // Custom shader ignores worldAlpha, so fold the container-level additive
        // pile-gain (set on this.container.alpha) into the per-particle opacity.
        u.uAlpha = alpha * this.container.alpha;
    }

    override destroy(): void {
        super.destroy();
        this.geometry.destroy();
    }
}

// ---- Ram-shader emitter (faithful GLSL port) -----------------------------
//
// The "Ram" (ramp) shader family is a ramp-tint + dissolve + UV-disturb sprite
// compositor (extracted from the game's GLES3 shader blob). A plain tinted
// billboard loses everything that makes these effects read as crisp cyan energy:
// the per-pixel colour RAMP (`_RamTex`), the soft dissolve edge (`_DissolveTex`
// + `_Amount`/`_BorderWidth`), and the flow UV disturbance (`_DisturbTex`). We
// render each Ram emitter's live particles as a single PIXI.Mesh whose fragment
// shader is a direct port of the game shader, so the real look is reproduced.
//
// Vertex-disturb displacement (a vertex texture-fetch wobble) is approximated as
// the identity — the dominant look is the fragment compositor, which is shared.

const RAM_VERT = `
precision highp float;
attribute vec2 aVertexPosition;
attribute vec2 aUV;
attribute vec4 aColor;
attribute vec2 aCustom;
uniform mat3 translationMatrix;
uniform mat3 projectionMatrix;
uniform vec4 uMainST;
uniform vec4 uDissolveST;
uniform vec4 uDissolveST2;
uniform vec4 uDisturbST;
uniform vec4 uRamST;
uniform vec2 uMainScroll;
uniform vec2 uDissolveScroll;
uniform vec2 uDisturbScroll;
varying vec2 vMainUV;
varying vec2 vDissolveUV;
varying vec2 vDissolveUV2;
varying vec2 vDisturbUV;
varying vec2 vRamUV;
varying vec4 vColor;
varying vec2 vCustom;
void main() {
    gl_Position = vec4((projectionMatrix * translationMatrix * vec3(aVertexPosition, 1.0)).xy, 0.0, 1.0);
    // Unity samples textures V-bottom-up but the exported PNGs are top-down, so
    // TRANSFORM_TEX offsets land in the mirrored half of the atlas. Flip V after
    // applying each _ST so partial-region samples (e.g. the reticle crosshair
    // arms carved from a shared glint atlas) hit the intended sub-rect.
    vMainUV = aUV * uMainST.xy + uMainST.zw + uMainScroll;
    vDissolveUV = aUV * uDissolveST.xy + uDissolveST.zw + uDissolveScroll;
    vDissolveUV2 = aUV * uDissolveST2.xy + uDissolveST2.zw + uDissolveScroll;
    vDisturbUV = aUV * uDisturbST.xy + uDisturbST.zw + uDisturbScroll;
    vRamUV = aUV * uRamST.xy + uRamST.zw;
    vMainUV.y = 1.0 - vMainUV.y;
    vDissolveUV.y = 1.0 - vDissolveUV.y;
    vDissolveUV2.y = 1.0 - vDissolveUV2.y;
    vDisturbUV.y = 1.0 - vDisturbUV.y;
    vRamUV.y = 1.0 - vRamUV.y;
    vColor = aColor;
    vCustom = aCustom;
}
`;

const RAM_FRAG = `
precision highp float;
varying vec2 vMainUV;
varying vec2 vDissolveUV;
varying vec2 vDissolveUV2;
varying vec2 vDisturbUV;
varying vec2 vRamUV;
varying vec4 vColor;
varying vec2 vCustom;
uniform sampler2D uMainTex;
uniform sampler2D uRamTex;
uniform sampler2D uDisturbTex;
uniform sampler2D uDissolveTex;
uniform sampler2D uDissolveTex2;
uniform vec4 uMainColor;
uniform float uOpacity;
uniform float uBorderWidth;
uniform float uAmount;
uniform float uIntensityU;
uniform float uIntensityV;
uniform float uDisturbInfluenceDissolveUV;
uniform float uDisturbInfluenceMainUV;
uniform float uHasDisturb;
uniform float uHasDissolve;
uniform float uHasDissolve2;
uniform float uAmount2;
uniform float uBorderWidth2;
uniform float uHasRam;
void main() {
    float disturbSample = uHasDisturb > 0.5 ? texture2D(uDisturbTex, vDisturbUV).x : 0.0;
    vec2 dOff = (vCustom.y + vec2(uIntensityU, uIntensityV)) * disturbSample;
    vec2 mUV = dOff * uDisturbInfluenceMainUV + vMainUV;
    vec2 dsUV = dOff * uDisturbInfluenceDissolveUV + vDissolveUV;
    vec4 col = texture2D(uMainTex, mUV) * uMainColor * vColor;
    col += col; // faithful *2 (also makes _MainColor.a=0.5 neutral on alpha)
    float dissolveTex = uHasDissolve > 0.5 ? texture2D(uDissolveTex, dsUV).x : 1.0;
    float threshold = vCustom.x + uAmount;
    // Game shader: sw = 1 - roundEven(threshold + 0.5). roundEven(y) ~= floor(y +
    // 0.5), so this is floor(threshold + 1.0). (Mis-porting this as floor(t+0.5)
    // disables the dissolve for low thresholds — the whole quad shows as a block
    // instead of the dissolve mask's shape, e.g. the targeting-ring reticles.)
    float sw = 1.0 - floor(threshold + 1.0);
    float bw = max(uBorderWidth, 1e-4);
    float edge = bw * sw + (dissolveTex - threshold);
    float dAlpha = clamp(edge / bw, 0.0, 1.0);
    // SECOND MASK - the Dissolve/ family multiplies two, each with its own threshold,
    // border and ST (verified against the decompiled Dissolve Add Double program). The
    // Ram/ and Disturb/ families bind only one, so uHasDissolve2 is 0 and this is exactly 1.
    if (uHasDissolve2 > 0.5) {
        float d2 = texture2D(uDissolveTex2, vDissolveUV2).x;
        float sw2 = 1.0 - floor(uAmount2 + 1.0);
        float bw2 = max(uBorderWidth2, 1e-4);
        dAlpha *= clamp((bw2 * sw2 + (d2 - uAmount2)) / bw2, 0.0, 1.0);
    }
    if (uHasRam > 0.5) {
        col *= texture2D(uRamTex, vRamUV);
    }
    float a = clamp(dAlpha * col.a * uOpacity, 0.0, 1.0);
    // The game feeds this into an HDR bloom+tonemap pass we don't run; the *2
    // above then just blows out. Compress highlights with a gentle Reinhard so
    // mids are untouched but the over-bright (esp. opaque normal-blend) tames.
    vec3 rgb = col.rgb / (1.0 + 0.5 * max(col.rgb, 0.0));
    // Premultiplied output: ADD (One,One) adds rgb*a; NORMAL over-blends by a.
    gl_FragColor = vec4(rgb * a, a);
}
`;

/** A Ram particle: sim state only (no per-particle display object). */
interface IRamParticle {
    x: number;
    y: number;
    vx: number;
    vy: number;
    age: number;
    life: number;
    size: number;
    /** Quad HEIGHT (px) — differs from `size` only for a `startSizeY` system. */
    sizeY: number;
    rot: number;
    rotVel: number;
    rand: number;
    col: IRGBA;
}

const WHITE_TEX = PIXI.Texture.WHITE;

/** One Ram-shader emitter: simulates particles and draws them as a single mesh
 *  running the ported ramp/dissolve/disturb shader. */
class RamEmitter {
    readonly container = new PIXI.Container();
    private readonly data: IParticleSystemData;
    private readonly ram: IRamData;
    private readonly particles: IRamParticle[] = [];
    private readonly cap: number;
    private time = 0;
    /** Absolute seconds since the `_Start` began, on the REAL timeline — never scaled by
     *  `simulationSpeed` and never advanced by the prewarm pre-roll. Drives the DIRECTOR's
     *  curves (`scaleCurve`, the Ram `_MainColor` ramp); {@link time} drives the simulation. */
    private cineTime = 0;
    private emitAcc = 0;
    /** Has this system EVER had a live particle? See {@link liveCount}. */
    everLive = false;
    /** One-shot latch for the {@link prewarmOf} pre-roll. */
    private prewarmed = false;
    private firedBursts = new Set<number>();
    private lastCycleT = -1;
    private readonly rate: number;

    private readonly blend: "additive" | "normal";
    private readonly volWorld: { x: number; y: number } | null;
    private readonly boneAnchor: IBoneAnchor = { resolved: false, boneName: null, ref: null };
    /** Explicit BoneFollower attachment (see {@link followOf} / {@link driftWithBone}). */
    private readonly follow: IFollow | undefined;
    /** `rateOverDistance` (particles per px of emitter travel) and the emitter's
     *  last container position, for distance-based trail emission (Virtuosa's
     *  falling-apple comet dust). */
    private readonly rodRate: number;
    private lastEmitterPos: { x: number; y: number } | null = null;
    /** World-space simulation state — see {@link holdWorldSpace}. */
    private readonly worldSpace: IWorldSpaceState = { prev: null };
    /** Live authored display box (mesh-local px, Y-down) for the current frame — see
     *  {@link ILoadedParticles.update}. Null = no cull (e.g. no framing data yet). */
    private displayBox: IAnimationBounds | null = null;
    private readonly mesh: PIXI.Mesh<PIXI.Shader>;
    private readonly shader: PIXI.Shader;
    private readonly posData: Float32Array;
    private readonly uvData: Float32Array;
    private readonly colData: Float32Array;
    private readonly customData: Float32Array;
    private readonly posBuf: PIXI.Buffer;
    private readonly colBuf: PIXI.Buffer;
    private readonly customBuf: PIXI.Buffer;
    /** Only rewritten (and re-uploaded) for a texture-sheet system — see `sheetTiles`. */
    private readonly uvBuf: PIXI.Buffer;
    /** `tilesX * tilesY` when the system drives a Texture Sheet flipbook, else 0. */
    private readonly sheetTiles: number;

    constructor(
        data: IParticleSystemData,
        ram: IRamData,
        tex: { main: PIXI.Texture | null; ram: PIXI.Texture | null; disturb: PIXI.Texture | null; dissolve: PIXI.Texture | null; dissolve2?: PIXI.Texture | null },
        blend: "additive" | "normal",
        private readonly getBudget: () => number,
    ) {
        this.blend = blend;
        this.data = data;
        // Start dormant through the cinematic delay (see the billboard system's ctor).
        this.time = -(data.delay ?? 0) + psDt();
        this.cineTime = psDt();
        this.ram = ram;
        this.follow = followOf(data);
        this.rodRate = data.rateOverDistance ? sampleScalar(data.rateOverDistance, 0.5, 0) : 0;
        this.volWorld = worldVelocityOverLife(data);
        this.rate = data.emission?.rate ? sampleScalar(data.emission.rate, 0.5, 0) : 0;
        this.sheetTiles = data.sheet ? Math.max(0, data.sheet.tilesX * data.sheet.tilesY) : 0;
        this.cap = Math.max(1, Math.min(PER_SYSTEM_CAP, data.maxParticles > 0 ? data.maxParticles : PER_SYSTEM_CAP));

        // Preallocate geometry for `cap` quads (4 verts, 6 indices each).
        const nv = this.cap * 4;
        this.posData = new Float32Array(nv * 2);
        this.uvData = new Float32Array(nv * 2);
        this.colData = new Float32Array(nv * 4);
        this.customData = new Float32Array(nv * 2);
        const idx = new Uint16Array(this.cap * 6);
        for (let q = 0; q < this.cap; q++) {
            const v = q * 4;
            const o = q * 6;
            idx[o] = v;
            idx[o + 1] = v + 1;
            idx[o + 2] = v + 2;
            idx[o + 3] = v;
            idx[o + 4] = v + 2;
            idx[o + 5] = v + 3;
            // Base per-corner UV (0..1), transformed per-slot in the shader. A
            // texture-sheet system overwrites this per particle each frame with the
            // live flipbook TILE's sub-rect (see {@link writeGeometry}) — exactly
            // Unity's order, where the Texture Sheet module rewrites the vertex UV
            // and each sampler's `_ST` then applies on top of the tile.
            const u = v * 2;
            this.uvData[u] = 0;
            this.uvData[u + 1] = 0;
            this.uvData[u + 2] = 1;
            this.uvData[u + 3] = 0;
            this.uvData[u + 4] = 1;
            this.uvData[u + 5] = 1;
            this.uvData[u + 6] = 0;
            this.uvData[u + 7] = 1;
        }

        // PIXI.Buffer's constructor param type doesn't structurally match TS 5.7's
        // typed arrays; cast as sceneMesh.ts does for MeshGeometry.
        type BufArg = ConstructorParameters<typeof PIXI.Buffer>[0];
        this.posBuf = new PIXI.Buffer(this.posData as unknown as BufArg);
        this.colBuf = new PIXI.Buffer(this.colData as unknown as BufArg);
        this.customBuf = new PIXI.Buffer(this.customData as unknown as BufArg);
        this.uvBuf = new PIXI.Buffer(this.uvData as unknown as BufArg);
        const geometry = new PIXI.Geometry();
        geometry.addAttribute("aVertexPosition", this.posBuf, 2);
        geometry.addAttribute("aUV", this.uvBuf, 2);
        geometry.addAttribute("aColor", this.colBuf, 4);
        geometry.addAttribute("aCustom", this.customBuf, 2);
        geometry.addIndex(new PIXI.Buffer(idx as unknown as BufArg));

        const mainT = tex.main ?? WHITE_TEX;
        this.shader = PIXI.Shader.from(RAM_VERT, RAM_FRAG, {
            uMainTex: mainT,
            uRamTex: tex.ram ?? WHITE_TEX,
            uDisturbTex: tex.disturb ?? WHITE_TEX,
            uDissolveTex: tex.dissolve ?? WHITE_TEX,
            uDissolveTex2: tex.dissolve2 ?? WHITE_TEX,
            uMainColor: ram.mainColor,
            uOpacity: ram.opacity,
            uBorderWidth: ram.borderWidth,
            uAmount: ram.amount,
            uIntensityU: ram.intensityU,
            uIntensityV: ram.intensityV,
            uDisturbInfluenceDissolveUV: ram.disturbInfluenceDissolveUV,
            uDisturbInfluenceMainUV: ram.disturbInfluenceMainUV,
            uHasDisturb: tex.disturb ? 1 : 0,
            uHasDissolve: tex.dissolve ? 1 : 0,
            uHasDissolve2: tex.dissolve2 ? 1 : 0,
            uAmount2: ram.amount2 ?? 0,
            uBorderWidth2: ram.borderWidth2 ?? 0.1,
            uHasRam: tex.ram ? 1 : 0,
            uMainST: ram.mainST,
            uDissolveST: ram.dissolveST,
            uDissolveST2: ram.dissolveST2 ?? [1, 1, 0, 0],
            uDisturbST: ram.disturbST,
            uRamST: ram.ramST,
            uMainScroll: [0, 0],
            uDissolveScroll: [0, 0],
            uDisturbScroll: [0, 0],
        });
        this.mesh = new PIXI.Mesh(geometry, this.shader);
        // Premultiplied output → ADD (One,One) for additive, NORMAL (over) for
        // alpha-blend materials (the shader's _DstBlend: 1=Add, 10=AlphaBlend).
        this.mesh.blendMode = this.blend === "additive" ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
        this.container.addChild(this.mesh);
    }

    private spawn(): void {
        if (this.particles.length >= this.cap || this.getBudget() <= 0) return;
        const d = this.data;
        const nt = d.duration > 0 ? (this.time % d.duration) / d.duration : 0;
        const shape = d.shape;
        let ox = 0;
        let oy = 0;
        let dirAng = 90 * DEG;
        const a = Math.random() * Math.PI * 2;
        if (shape) {
            const [sx, sy] = shapeScale(shape);
            switch (shape.type) {
                case "circle": {
                    const rr = (shape.radius ?? 0) * radialFrac(shape);
                    ox = Math.cos(a) * rr * sx;
                    oy = Math.sin(a) * rr * sy;
                    dirAng = a;
                    break;
                }
                case "box":
                    ox = (Math.random() - 0.5) * (shape.box?.[0] ?? 0);
                    oy = (Math.random() - 0.5) * (shape.box?.[1] ?? 0);
                    break;
                case "edge":
                    ox = (Math.random() - 0.5) * (shape.radius ?? 0) * 2;
                    break;
                default: {
                    // cone / edge take no shape scale — see the billboard spawn above.
                    const spread = (shape.angleDeg ?? 0) * DEG;
                    dirAng = 90 * DEG + (Math.random() - 0.5) * 2 * spread;
                    const rr = (shape.radius ?? 0) * radialFrac(shape);
                    ox = Math.cos(a) * rr;
                    oy = Math.sin(a) * rr;
                }
            }
        }
        const rotDeg = (d.rot ?? 0) + (shape?.rotDeg ?? 0);
        const cos = Math.cos(rotDeg * DEG);
        const sin = Math.sin(rotDeg * DEG);
        const posOff = shape?.posOffset ?? [0, 0];
        const lx = ox + posOff[0];
        const ly = oy + posOff[1];
        const wx = d.pos[0] + lx * cos - ly * sin;
        const wy = d.pos[1] + lx * sin + ly * cos;
        const speed = sampleScalar(d.startSpeed, Math.random(), nt);
        // ONE per-particle roll drives BOTH size axes, as Unity's `size3D` does — and,
        // just as importantly, it keeps the number of `Math.random()` draws per spawn
        // fixed, so a system that gains a `startSizeY` does not re-phase the seeded
        // simulation of every particle after it.
        const sizeRand = Math.random();
        const size = sampleScalar(d.startSize, sizeRand, nt);
        const wDir = dirAng + rotDeg * DEG;
        const life = Math.max(MIN_PARTICLE_LIFE, sampleScalar(d.lifetime, Math.random(), nt));
        // Edge-clip fix: a world-space system's static spawn position can be baked for an
        // older, narrower framing calibration and now fall outside the live authored display
        // box (the settled idle box, or — during the entrance — the camera's live zoom/pan
        // box) for the whole shot, rendering as particles clipped hard at the canvas edge.
        // Reject spawns whose full trajectories are ENTIRELY outside that box (small
        // box-relative margin so streaks don't visibly pop at the boundary); particles that
        // drift into view are untouched — ordinary canvas clipping handles those.
        // GATE on `!this.boneAnchor.boneName` (resolved by `driftWithBone`, called earlier
        // this same `update()`), not just `simulationSpace`: a system can be authored
        // `simulationSpace:"world"` (particles drift independently once spawned) while its
        // EMITTER still rides a live bone (Virtuosa's `followBone:"L_C_Apple_F"` comet-trail
        // dust) — `wx`/`wy` here are baked in the bone's REST frame, only becoming true
        // screen coords after `this.container`'s live bone-follow transform is applied at
        // render time, so testing them against a screen-space box directly would be a
        // coordinate-space mismatch (and could wrongly thin the apple trail as it plunges).
        // A genuinely static ambient system (no chain name resolves to a real bone, e.g.
        // Mlynar's rain — its `boneChain` names are generic rig labels, not spine bones) has
        // `boneAnchor.boneName === null`, so `wx,-wy` IS already the true screen position.
        // See the Emitter.spawn() twin of this check: a large-scale glow/wash anchored far
        // outside the box on purpose (verified regression on cello's crown backdrop) needs
        // to stay untouched — gate the cull to particles small relative to the box.
        const smallEnoughToClip = this.displayBox ? size < 0.5 * Math.min(this.displayBox.width, this.displayBox.height) : false;
        if (d.simulationSpace === "local" && this.displayBox && !this.boneAnchor.boneName && smallEnoughToClip) {
            const box = this.displayBox;
            // A moderately-sized sprite isn't hard-culled just for having an edge-adjacent
            // centre — widen the margin by the particle's own half-size too.
            const margin = 0.05 * Math.max(box.width, box.height) + size / 2;
            const sx = wx;
            const sy = -wy;
            // See the Emitter.spawn() twin of this check: test the whole spawn→death segment,
            // so stationary particles retain the previous point-cull behavior while particles
            // that drift toward the box (via startSpeed AND/OR `velocityOverLife`, `this.volWorld`)
            // survive if their trajectory crosses into it. Screen Y is negated from world Y-up,
            // so `sy` moves by the negated Y rate per second.
            const segEndX = sx + (Math.cos(wDir) * speed + (this.volWorld?.x ?? 0)) * life;
            const segEndY = sy - (Math.sin(wDir) * speed + (this.volWorld?.y ?? 0)) * life;
            const segMinX = Math.min(sx, segEndX);
            const segMaxX = Math.max(sx, segEndX);
            const segMinY = Math.min(sy, segEndY);
            const segMaxY = Math.max(sy, segEndY);
            if (segMaxX < box.x - margin || segMinX > box.x + box.width + margin || segMaxY < box.y - margin || segMinY > box.y + box.height + margin) {
                return;
            }
        }
        this.particles.push({
            x: wx,
            y: wy,
            vx: Math.cos(wDir) * speed,
            vy: Math.sin(wDir) * speed,
            age: 0,
            life,
            size,
            sizeY: d.startSizeY ? sampleScalar(d.startSizeY, sizeRand, nt) : size,
            rot: sampleScalar(d.startRotation ?? { mode: "const", v: 0 }, Math.random(), nt),
            rotVel: typeof d.rotOverLifeDegPerSec === "number" ? d.rotOverLifeDegPerSec : 0, // legacy const
            rand: Math.random(),
            col: sampleColor(d.startColor, nt),
        });
    }

    update(dt: number, findBone?: FindBone, restBone?: RestBone, displayBox?: IAnimationBounds | null, restAtt?: RestAttachment): void {
        const d = this.data;
        this.displayBox = displayBox ?? null;
        // CINEMATIC CLOCK — absolute seconds since the `_Start` began, on the REAL timeline.
        // Advanced before any early-out so it keeps running through the start delay, and NEVER
        // scaled by `simulationSpeed`. See the clock split documented on {@link simSpeedOf}.
        this.cineTime += dt;
        // MAIN-MODULE CLOCK (`simulationSpeed` + `prewarm`).
        //
        // `simulationSpeed` scales the system's OWN clock, so everything keyed on it — the
        // emission accumulator, particle age, the over-lifetime curves, rotation and velocity —
        // slows or speeds together. The cinematic `delay` deliberately stays on the REAL clock:
        // it is sequencing handed to us by the director/`_Start` gating, not part of the
        // system's simulation, so scaling it would re-time the whole cinematic.
        const speed = simSpeedOf(d);
        if (speed !== 1) {
            if (this.time < 0) {
                this.time += dt;
                if (this.time < 0) return;
                dt = this.time; // the slice of this frame left after the delay elapsed
                this.time = 0;
            }
            dt *= speed;
        }
        this.time += dt;
        // Dormant until the cinematic start delay elapses (no emission, no particles yet).
        if (this.time < 0) return;
        // A prewarmed LOOPING system opens in steady state. Pre-roll one full `duration`
        // ONCE, the first frame it is live; `duration` is a whole number of loops, so the
        // emission phase and every `time % duration` curve land exactly where they would
        // have, while the particle ages become the steady-state spread instead of empty.
        if (!this.prewarmed && prewarmOf(d)) {
            this.prewarmed = true;
            // Pre-roll EXACTLY one `duration` of SYSTEM time. The step adapts so the
            // PREWARM_MAX_STEPS cap can never truncate the roll (a truncated roll leaves the
            // system short of steady state, which is precisely the error this is here to avoid),
            // and the final step is partial so the total lands on `duration` rather than on the
            // next whole multiple of the step.
            const step = Math.max(PREWARM_STEP, d.duration / PREWARM_MAX_STEPS);
            // The recursive call re-applies `speed`, so hand it real-time steps that scale back
            // to `step` of system time.
            const clock = this.time;
            // Unity's prewarm populates particles WITHOUT advancing playback time — the system
            // still reports t=0 when the cinematic starts. Restore BOTH clocks afterwards, or
            // every curve keyed on absolute cinematic seconds (the `scaleCurve` fed through
            // `driftWithBone`) runs a whole `duration` late.
            const cine = this.cineTime;
            for (let rolled = 0; rolled < d.duration; rolled += step) {
                this.update(Math.min(step, d.duration - rolled) / speed, findBone, restBone, displayBox, restAtt);
            }
            this.time = clock;
            this.cineTime = cine;
        }
        // Cinematic time for the scale-in curves: the emitter clock counts up from
        // `-delay`, so `this.time + delay` is absolute seconds since the `_Start` began.
        driftWithBone(this.container, d.boneChain, d.pos, d.simulationSpace, findBone, this.boneAnchor, restBone, this.follow, d.scaleCurve ? d : undefined, this.cineTime, restAtt);
        // World-space systems leave their particles behind as the emitter travels on.
        if (d.simulationSpace === "world") holdWorldSpace(this.container, this.worldSpace, this.particles);

        const playing = d.looping || this.time <= d.duration;
        // `rateOverDistance`: trail emission per px of emitter travel (the container
        // moves when the rig rides a bone — Virtuosa's comet sheds dust down the
        // shaft). Measured on the container position, whichever follow path drives it.
        const ep = { x: this.container.position.x, y: this.container.position.y };
        const moved = this.lastEmitterPos ? Math.hypot(ep.x - this.lastEmitterPos.x, ep.y - this.lastEmitterPos.y) : 0;
        this.lastEmitterPos = ep;
        // Retire this frame's expiring particles before emitting — same Unity ordering fix
        // as `Emitter.update`, where the reasoning is written out in full.
        if (RETIRE_BEFORE_EMIT && this.particles.length > 0) {
            let keep = 0;
            for (let r = 0; r < this.particles.length; r++) {
                const p = this.particles[r];
                if (p.age + dt >= p.life) continue;
                this.particles[keep++] = p;
            }
            this.particles.length = keep;
        }
        if (playing) {
            // Loop-wrap accumulator restart (see loopAccReset). Compared against the PREVIOUS
            // frame's cycle position, so it fires exactly once per wrap, before this frame's
            // contribution is added.
            if (loopAccReset() && d.looping && d.duration > 0 && this.time % d.duration < this.lastCycleT) this.emitAcc = 0;
            this.emitAcc += emissionRate(d, this.rate, this.time) * dt;
            if (this.rodRate > 0 && moved > 0) this.emitAcc += this.rodRate * moved;
            while (this.emitAcc >= 1) {
                this.emitAcc -= 1;
                this.spawn();
            }
            const cycleT = d.duration > 0 ? this.time % d.duration : this.time;
            // A magnitude check false-fires immediately after delayed activation;
            // only a genuine modulo wrap starts a new burst cycle.
            if (d.duration > 0 && cycleT < this.lastCycleT) this.firedBursts.clear();
            this.lastCycleT = cycleT;
            for (const burst of d.emission?.bursts ?? []) {
                const key = Math.floor(this.time / (d.duration || 1)) * 1000 + burst.t;
                if (cycleT >= burst.t && !this.firedBursts.has(key)) {
                    this.firedBursts.add(key);
                    for (let i = 0; i < Math.min(burst.count, this.cap); i++) this.spawn();
                }
            }
        }

        const grav = d.gravity ? sampleScalar(d.gravity, 0.5, 0) * 9.81 : 0;
        const force = worldForceOverLife(d);
        const noise = d.noise;
        const vol = this.volWorld;

        // Step + cull, compacting the array in place.
        let w = 0;
        for (let r = 0; r < this.particles.length; r++) {
            const p = this.particles[r];
            p.age += dt;
            if (p.age >= p.life) continue; // drop
            p.vy -= grav * dt;
            if (force) {
                p.vx += force.x * dt;
                p.vy += force.y * dt;
            }
            // Limit velocity over lifetime (Unity ClampVelocityModule) — see the
            // sprite Emitter loop. Damp speed toward the curve-sampled ceiling.
            if (d.velocityClamp) {
                const lf = p.age / p.life;
                const limit = sampleScalar(d.velocityClamp.magnitude, 0.5, lf);
                const spd = Math.hypot(p.vx, p.vy);
                if (spd > limit && spd > 0) {
                    const k = limit / spd;
                    p.vx = lerp(p.vx, p.vx * k, d.velocityClamp.dampen);
                    p.vy = lerp(p.vy, p.vy * k, d.velocityClamp.dampen);
                }
            }
            p.x += (p.vx + (vol?.x ?? 0)) * dt;
            p.y += (p.vy + (vol?.y ?? 0)) * dt;
            if (noise) {
                const f = noise.frequency * 0.002;
                const ph = this.time * noise.scrollSpeed;
                p.x += fbmNoise(p.x * f + ph, p.y * f + p.rand * 17) * noise.strength * dt;
                p.y += fbmNoise(p.y * f + ph + 3.1, p.x * f + p.rand * 17 + 9.3) * noise.strength * dt;
            }
            // See the sprite Emitter loop: integrate the authored angular-velocity curve.
            p.rot += (p.rotVel || rotRateAt(d.rotOverLifeDegPerSec, p.rand, p.age / p.life)) * dt;
            this.particles[w++] = p;
        }
        this.particles.length = w;

        // The cinematic's animated `_MainColor`, replayed at the absolute entrance time
        // (the emitter clock counts up from `-delay`). The scene-quad path has always
        // replayed its twin (`colorCurve`); without this a Ram emitter the `_Start` clip
        // brightens held its static serialized colour for the whole shot.
        const curve = this.ram.mainColorCurve;
        if (curve?.length) {
            this.shader.uniforms.uMainColor = sampleColorCurve(curve, this.cineTime);
        }

        // Scroll offsets for the three animated UV sets (fract(time * speed)).
        const frac = (v: number) => v - Math.floor(v);
        this.shader.uniforms.uMainScroll = [frac(this.time * this.ram.mainSpeed[0]), frac(this.time * this.ram.mainSpeed[1])];
        this.shader.uniforms.uDissolveScroll = [frac(this.time * this.ram.dissolveSpeed[0]), frac(this.time * this.ram.dissolveSpeed[1])];
        this.shader.uniforms.uDisturbScroll = [frac(this.time * this.ram.disturbSpeed[0]), frac(this.time * this.ram.disturbSpeed[1])];

        this.writeGeometry();
    }

    /** Fill the vertex buffers from live particles; collapse unused quads. */
    private writeGeometry(): void {
        const d = this.data;
        const pos = this.posData;
        const col = this.colData;
        const cst = this.customData;
        const uv = this.uvData;
        const sheet = d.sheet;
        const n = Math.min(this.particles.length, this.cap);
        for (let q = 0; q < n; q++) {
            const p = this.particles[q];
            const lf = p.age / p.life;
            const grow = d.sizeOverLife ? sampleCurve(d.sizeOverLife, lf) : 1;
            const lifeCol = d.colorOverLife ? sampleColor(d.colorOverLife, lf, p.rand) : { r: 1, g: 1, b: 1, a: 1 };
            const cr = p.col.r * lifeCol.r;
            const cg = p.col.g * lifeCol.g;
            const cb = p.col.b * lifeCol.b;
            const ca = Math.max(0, Math.min(1, p.col.a * lifeCol.a));
            const hx = (p.size * grow) / 2;
            const hy = (p.sizeY * grow) / 2;
            const th = -p.rot * DEG;
            const c = Math.cos(th);
            const s = Math.sin(th);
            const cx = p.x;
            const cy = -p.y;
            const vp = q * 8;
            // 4 corners: TL(-hx,-hy) TR(hx,-hy) BR(hx,hy) BL(-hx,hy), shifted so the Unity
            // PIVOT lands on the particle's position — the corner offsets are what the
            // rotation below is applied to, so shifting them here also makes the quad rotate
            // ABOUT the pivot, matching Unity. `hx`/`hy` are half-extents, so a pivot of 1.0
            // displaces by the full size. Unity's +Y is up, this buffer is Y-down.
            const pv = d.pivot;
            const ppx = pv ? pv[0] * 2 * hx : 0;
            const ppy = pv ? pv[1] * 2 * hy : 0;
            const cxs = [-hx - ppx, hx - ppx, hx - ppx, -hx - ppx];
            const cys = [-hy + ppy, -hy + ppy, hy + ppy, hy + ppy];
            for (let k = 0; k < 4; k++) {
                const lxk = cxs[k];
                const lyk = cys[k];
                pos[vp + k * 2] = cx + lxk * c - lyk * s;
                pos[vp + k * 2 + 1] = cy + lxk * s + lyk * c;
            }
            // Texture Sheet Animation: point the quad at the live flipbook TILE, in the
            // same raster (row-major, top-down) order the sprite path slices frames in.
            // Every `_ST` in the shader then applies ON TOP of the tile, which is Unity's
            // own order — without it a sheet system samples the WHOLE atlas per quad and
            // stamps the grid, which is why such systems used to be barred from this path.
            if (sheet && this.sheetTiles > 1) {
                const prog = sheet.frameOverTime ? sampleCurve(sheet.frameOverTime, lf) : lf;
                const cycles = sheet.cycles && sheet.cycles > 0 ? sheet.cycles : 1;
                const fi = Math.min(this.sheetTiles - 1, Math.max(0, Math.floor(prog * cycles * this.sheetTiles) % this.sheetTiles));
                const cw = 1 / sheet.tilesX;
                const ch = 1 / sheet.tilesY;
                const u0 = (fi % sheet.tilesX) * cw;
                const v0 = Math.floor(fi / sheet.tilesX) * ch;
                const us = [u0, u0 + cw, u0 + cw, u0];
                const vs = [v0, v0, v0 + ch, v0 + ch];
                for (let k = 0; k < 4; k++) {
                    uv[vp + k * 2] = us[k];
                    uv[vp + k * 2 + 1] = vs[k];
                }
            }
            const vc = q * 16;
            for (let k = 0; k < 4; k++) {
                col[vc + k * 4] = cr;
                col[vc + k * 4 + 1] = cg;
                col[vc + k * 4 + 2] = cb;
                col[vc + k * 4 + 3] = ca;
            }
            const vk = q * 8;
            // CustomData (vs_TEXCOORD2 = per-particle dissolve amount + disturb
            // intensity). Systems with the CustomData module DISABLED (every svash2
            // Ram mask effect) hold a static 0 and the dissolve mask shapes the fill
            // directly; an exported `ramDissolveCurve` (Virtuosa's crumbling apples)
            // replays the per-particle dissolve amount over normalized lifetime.
            const dis = d.ramDissolveCurve ? sampleCurve(d.ramDissolveCurve, lf) : 0;
            for (let k = 0; k < 4; k++) {
                cst[vk + k * 2] = dis; // dissolve amount
                cst[vk + k * 2 + 1] = 0; // disturb intensity (no _DisturbTex bound here)
            }
        }
        // Collapse unused quads to a degenerate point so they draw nothing.
        for (let q = n; q < this.cap; q++) {
            const vp = q * 8;
            for (let k = 0; k < 8; k++) pos[vp + k] = 0;
        }
        this.posBuf.update();
        this.colBuf.update();
        this.customBuf.update();
        if (this.sheetTiles > 1) this.uvBuf.update();
    }

    liveCount(): number {
        if (this.particles.length > 0) this.everLive = true;
        return this.particles.length;
    }

    destroy(): void {
        this.container.destroy({ children: true });
        this.shader.destroy();
    }
}

/** Per-emitter diagnostic snapshot from {@link ILoadedParticles.probe}.
 *
 *  Exists because "this effect is missing" is not answerable from the JSON: a system can be
 *  authored, exported, pass every activation gate, sit inside the particle budget and still put
 *  nothing on screen — because it emitted nothing, or because it emitted off-frame. Those need
 *  opposite fixes and only live state distinguishes them. Colour masks cannot: they measure the
 *  composite, not the emitter. */
export interface IEmitterProbe {
    /** Index into {@link IParticlesData.systems}. Systems the loader skipped are absent. */
    sys: number;
    /** Live (non-recycled) particles this frame. 0 means it is emitting nothing at all. */
    live: number;
    /** Emitter container position in the scene's local space (Pixi Y-down). */
    x: number;
    y: number;
    /** Container ORIGIN in global/screen px. Note `pos` is baked into each particle's spawn
     *  position rather than the container, so this is the emitter's anchor, NOT where its
     *  particles are — use {@link box} for that. Null when the emitter is not on the stage. */
    sx: number | null;
    sy: number | null;
    /** Screen-space bounding box of the emitter's LIVE, VISIBLE particles (`getBounds`, which
     *  asks the renderer rather than re-deriving the transform chain). This is what localises a
     *  missing effect: a box wholly outside 0..w / 0..h means the particles exist but are drawn
     *  off-frame. Null when nothing is currently drawn. */
    box: { x: number; y: number; w: number; h: number } | null;
    /** How many of this emitter's live, drawable children have their ORIGIN inside the viewport.
     *  This is the measure {@link box} cannot give: a union bbox spanning the canvas says nothing
     *  about whether any particle is actually in view — 13 particles at opposite corners produce a
     *  box covering everything between them. "34 live, 0 onScreen" is unambiguous.
     *  Counts child origins, so a particle straddling an edge may read as outside, and a trail's
     *  rope counts once at the emitter origin rather than along its length. */
    onScreen: number;
    /** Total canvas area (px^2) this emitter's live children actually cover, summed as each
     *  child's DRAWN rect intersected with the viewport.
     *
     *  This is the measure that settles "is the effect visible": unlike {@link onScreen} it
     *  accounts for a particle whose origin is off-canvas but whose sprite still paints inside it
     *  (large stretched billboards routinely do), and unlike {@link box} it cannot be inflated by
     *  two distant particles bounding an empty middle. **0 means genuinely nothing is drawn.**
     *  Overlapping particles are counted twice — it is coverage, not distinct pixels. */
    paintedPx: number;
    /** Per-particle trail ribbon state, or null when the system has no per-particle trail.
     *  `span` is the mean bounding extent of a ribbon's control points: a ribbon whose points
     *  have collapsed onto the head has span 0, and `RopeGeometry.updateVertices` then zeroes
     *  the perpendicular (`perpLength < 1e-6`), producing a zero-AREA strip that is invisible
     *  **at every width**. That is the signature to check before blaming ribbon width. */
    rope?: { n: number; vis: number; w: number; span: number; pts: number; alpha: number; tint: string; tex: string; blend: number } | null;
}

export interface ILoadedParticles {
    data: IParticlesData;
    /** Emitters whose sort is behind the character. */
    background: PIXI.Container;
    /** Emitters whose sort is in front of the character. */
    foreground: PIXI.Container;
    /** `findBone` (pixi-spine `skeleton.findBone`) lets bone-parented emitters
     *  drift with the character; omit for no bone-following. */
    update(dt: number, findBone?: FindBone, restBone?: RestBone, displayBox?: IAnimationBounds | null, restAtt?: RestAttachment): void;
    /** Live per-emitter state for parity diagnosis — see {@link IEmitterProbe}. Read-only and
     *  side-effect free; call it after `update` from a harness, never from the render path.
     *  Pass the viewport size so `onScreen` can be counted. */
    probe(viewW?: number, viewH?: number): IEmitterProbe[];
    destroy(): void;
}

interface ILoadedTex {
    base: PIXI.BaseTexture;
    /** A fully-opaque glow/starfield texture (no alpha channel) authored for
     *  additive blending — its `base` has been rebuilt with alpha = luminance so
     *  the dark field drops out. Render these additive. */
    glow: boolean;
    /** The UNPROCESSED texture (no glow/vignette mangling). Ram-shader emitters
     *  need the raw pixels — their shader does its own ramp/dissolve/disturb, so
     *  the opaque-glow heuristic would destroy the flow/ramp/mask inputs. */
    rawBase: PIXI.BaseTexture;
    /** Ram MAIN-slot texture: for opaque glow textures, alpha = luminance (dark
     *  field → 0) but with NO border vignette / radial falloff, so a bright fill
     *  (SilverAsh's reticle stripes, shaped by a dissolve mask) is preserved while
     *  a dark starfield/bokeh field (Logos, Nian) drops out instead of boxing.
     *  Equals `rawBase` for real-alpha sprites (nothing to drop). */
    darkDropBase: PIXI.BaseTexture;
    /** A DESATURATED, substantially-FILLED texture (mean saturation < 0.2 over a
     *  covered ≥25% of the sheet): a blue-grey flow/distortion "background" map a
     *  Unity shader samples to warp the scene, not a drawable sprite. Unlike `skip`
     *  (which also requires the field be DARK), this catches the BRIGHT-grey panels
     *  (`meanLum`≈230) — used only to gate normal-blend MESH rendering so those
     *  panels don't stamp grey rectangles, while COLOURED effect meshes (lightning
     *  bolts sat≈0.28, ice-flames sat≈0.4) and THIN white sparks (low coverage)
     *  still render. */
    desatPanel: boolean;
    /** A fully-opaque, DARK/MID-grey, low-contrast field: a flow / distortion /
     *  cloud MAP that a Unity shader samples to warp the scene, never a drawable
     *  sprite. Billboarded raw it stamps a translucent grey rectangle that flickers
     *  as the emitter bursts (Hoshiguma Alter's "top wave artifacts"). The
     *  billboard/mesh DRAW paths skip these; the Ram-shader path (which legitimately
     *  consumes them as flow inputs via rawBase/darkDropBase) runs earlier and is
     *  unaffected. A genuine glow has real transparency (never reaches the opaque
     *  branch) or is uniformly bright (a light fill, kept). */
    skip: boolean;
    /** A BRIGHT, desaturated, SOFT (mostly-translucent) sheet: an atmospheric
     *  haze/fog/glow cloud. When such a texture drives a LARGE normal-blend emitter
     *  authored in front of the character, it fogs her opaque body into a translucent
     *  ghost (Virtuosa "Diversity Oneness"'s `air_01`, tex10: white cloud, startSize
     *  ~1160). The game keeps that ambient haze in the scene DEPTH behind the focal
     *  character; we demote those emitters behind the spine (see the split below). */
    hazePanel: boolean;
}

/** Memoised per-(baseTexture, tiling) analysis of a flipbook atlas: returns the set
 *  of cell indices (row-major, `row*tx + col`) that are opaque, desaturated grey
 *  FILLS — flow/fade frames that stamp a hard grey rectangle rather than a shaped
 *  sprite. Empty set on any failure (all cells kept). Cheap: one canvas readback per
 *  distinct atlas+grid, shared across the systems that reuse it. */
const sheetBadCellCache = new WeakMap<PIXI.BaseTexture, Map<string, Set<number>>>();
function sheetBadCells(base: PIXI.BaseTexture, tx: number, ty: number): Set<number> {
    const key = `${tx}x${ty}`;
    let byGrid = sheetBadCellCache.get(base);
    if (byGrid?.has(key)) return byGrid.get(key) as Set<number>;
    if (!byGrid) {
        byGrid = new Map();
        sheetBadCellCache.set(base, byGrid);
    }
    const bad = new Set<number>();
    try {
        const src = (base.resource as { source?: CanvasImageSource } | undefined)?.source;
        const w = base.width;
        const h = base.height;
        if (src && w && h) {
            const canvas = document.createElement("canvas");
            canvas.width = w;
            canvas.height = h;
            const ctx = canvas.getContext("2d", { willReadFrequently: true });
            if (ctx) {
                ctx.drawImage(src, 0, 0, w, h);
                const fw = Math.floor(w / tx);
                const fh = Math.floor(h / ty);
                for (let row = 0; row < ty; row++) {
                    for (let col = 0; col < tx; col++) {
                        const px = ctx.getImageData(col * fw, row * fh, fw, fh).data;
                        let covered = 0;
                        let satSum = 0;
                        let aw = 0;
                        let n = 0;
                        for (let i = 0; i < px.length; i += 16) {
                            n++;
                            const a = px[i + 3] / 255;
                            if (a > 0.5) covered++;
                            if (a <= 0.1) continue;
                            const r = px[i];
                            const g = px[i + 1];
                            const b = px[i + 2];
                            const mx = Math.max(r, g, b);
                            satSum += mx > 0 ? (mx - Math.min(r, g, b)) / mx : 0;
                            aw++;
                        }
                        // A cell that covers ≥40% of its rectangle with desaturated
                        // (grey/white) pixels is a flow/fade FILL frame, not a shaped
                        // sprite — blanking it drops the rectangle it would stamp.
                        const coverage = n > 0 ? covered / n : 0;
                        const sat = aw > 0 ? satSum / aw : 0;
                        if (coverage >= 0.4 && sat < 0.25) bad.add(row * tx + col);
                    }
                }
            }
        }
    } catch {
        bad.clear();
    }
    byGrid.set(key, bad);
    return bad;
}

/**
 * A fully-opaque particle texture has no usable alpha channel: it's a
 * glow / light / starfield authored for ADDITIVE blending, where the (near-)black
 * or dark-grey field is meant to add nothing. Composited straight — or even
 * additive as-is — that field stamps a faint/black square (its non-zero
 * luminance adds a uniform block; e.g. Logos' starfield sits at grey ~23/255).
 * Rebuild the texture with `alpha = max(r,g,b)` so, premultiplied under additive
 * blend, dark texels contribute ~nothing and only the light glows. Textures that
 * already carry real transparency (soft sprites) are returned untouched.
 */
function processGlowTexture(img: HTMLImageElement): ILoadedTex {
    const plain = (skip = false, desatPanel = false, hazePanel = false): ILoadedTex => {
        const base = PIXI.BaseTexture.from(img);
        return { base, glow: false, rawBase: base, darkDropBase: base, skip, desatPanel, hazePanel };
    };
    try {
        const w = img.naturalWidth || img.width;
        const h = img.naturalHeight || img.height;
        if (!w || !h) return plain();
        const canvas = document.createElement("canvas");
        canvas.width = w;
        canvas.height = h;
        const ctx = canvas.getContext("2d", { willReadFrequently: true });
        if (!ctx) return plain();
        ctx.drawImage(img, 0, 0);
        const imgData = ctx.getImageData(0, 0, w, h);
        const px = imgData.data;
        // Opaque fraction (sample every 16th pixel): a soft sprite has lots of
        // transparency, a glow atlas is near-fully opaque. In the same pass gather
        // coverage/luminance/saturation over the covered pixels so we can spot a
        // DARK DISTORTION ORB — a big dark desaturated filled blob that Unity samples
        // as a refraction/heat-warp lens (never drawn as a solid sprite). Billboarded
        // raw it stamps a black cloud; in-game it's near-invisible. Virtuosa E2's
        // `tex5` (cov≈0.44, meanLum≈42, meanSat≈0.13) is exactly this. The dark
        // SHARDS (tex0/tex3, cov≤0.2) and grey crystals (tex8, meanLum≈151) stay,
        // so real debris is untouched.
        let opaque = 0;
        let n = 0;
        let cov = 0;
        let lumS = 0;
        let satS = 0;
        for (let i = 3; i < px.length; i += 64) {
            if (px[i] > 230) opaque++;
            if (px[i] > 128) {
                cov++;
                const r = px[i - 3];
                const g = px[i - 2];
                const b = px[i - 1];
                const mx = Math.max(r, g, b);
                lumS += mx;
                satS += mx > 0 ? (mx - Math.min(r, g, b)) / mx : 0;
            }
            n++;
        }
        const covFrac = n ? cov / n : 0;
        const darkOrb = cov > 0 && covFrac >= 0.4 && lumS / cov < 90 && satS / cov < 0.2;
        // Filled + desaturated (regardless of brightness) → a flow/distortion panel
        // (gates normal-blend MESH; see ILoadedTex.desatPanel). Thin coloured effects
        // (lightning/ice-flame) and thin white sparks fall below covFrac and pass.
        const desatPanel = cov > 0 && covFrac >= 0.25 && satS / cov < 0.2;
        // A BRIGHT, desaturated, present-but-soft sheet = atmospheric haze/fog (see
        // ILoadedTex.hazePanel). Bright fill (meanLum ≥ 150) rules out the dark
        // distortion orb; low saturation rules out coloured light sheets. The split
        // only demotes it when it also drives a LARGE normal-blend emitter, so small
        // white sparks (same texture profile) are never affected.
        const hazePanel = cov > 0 && covFrac >= 0.15 && lumS / cov >= 150 && satS / cov < 0.2;
        if (n === 0 || opaque / n < 0.85) return plain(darkOrb, desatPanel, hazePanel);
        // alpha = luminance, with a black point so the near-black/dark-grey field
        // (Logos' starfield sits at ~23/255) drops to fully transparent instead of
        // adding a faint residual block; genuine glow/stars sit well above it.
        const BLACK_POINT = 32;
        let highAlpha = 0;
        let alphaN = 0;
        let lumSum = 0;
        let satSum = 0;
        let maxLum = 0;
        for (let i = 0; i < px.length; i += 4) {
            const r = px[i];
            const g = px[i + 1];
            const b = px[i + 2];
            const lum = Math.max(r, g, b);
            const a = lum <= BLACK_POINT ? 0 : lum;
            px[i + 3] = a;
            if (a > 128) highAlpha++;
            if (lum > maxLum) maxLum = lum;
            lumSum += lum;
            satSum += lum > 0 ? (lum - Math.min(r, g, b)) / lum : 0;
            alphaN++;
        }
        // Snapshot the luminance-alpha result (dark field dropped, NO vignette or
        // radial) for the Ram MAIN slot — see ILoadedTex.darkDropBase.
        const ddCanvas = document.createElement("canvas");
        ddCanvas.width = w;
        ddCanvas.height = h;
        const ddCtx = ddCanvas.getContext("2d");
        let darkDropBase: PIXI.BaseTexture;
        if (ddCtx) {
            ddCtx.putImageData(imgData, 0, 0);
            darkDropBase = PIXI.BaseTexture.from(ddCanvas);
        } else {
            darkDropBase = PIXI.BaseTexture.from(img);
        }
        // An opaque texture billboarded as-is stamps a hard square (dark
        // distortion/cloud maps, solid fills). Two shape corrections:
        //  • ALWAYS fade the outer border to 0 so the hard texture-edge softens
        //    away — no box outline, regardless of content.
        //  • A uniform-bright fill (still mostly opaque after luminance-alpha —
        //    e.g. SilverAsh the Reignfrost's flat cyan `tex0`) carries no shape,
        //    so ALSO give it a full radial falloff (a soft glow orb).
        // A genuine glow-on-black sprite (starfield, soft flare) has real
        // transparency and never reaches here (returned plain above).
        const uniformBright = highAlpha / Math.max(1, alphaN) > 0.5;
        // A fully-opaque field that is DESATURATED (grey) and dark/mid in luminance is
        // a flow/distortion/cloud MAP, not a drawable sprite — mark it skipped by the
        // draw paths (see ILoadedTex.skip). Hoshiguma Alter's grey-rectangle "wave
        // artifacts" are tex 4/14/15/22 (lumMean 64-138, ~grey) — all caught; brightness
        // alone must not exempt a UNIFORM-bright-but-grey fill like tex15.
        // The saturation gate protects genuine glows: a coloured light fill (SilverAsh's
        // cyan) stays even when dim, and a bright white flare has meanLum ≥ 150; soft
        // transparent sprites returned `plain` above and never reach here.
        const meanLum = lumSum / Math.max(1, alphaN);
        const meanSat = satSum / Math.max(1, alphaN);
        // A BRIGHT-CORE-on-dark texture (a sparkle/glint/star: near-black field with a small
        // intense white core, e.g. Virtuosa's tex13 — meanLum ~27 but a 255 peak) reads as
        // low-mean/low-sat and was wrongly lumped with flow/distortion maps and skipped. A real
        // flow map is UNIFORM mid-grey: no concentrated peak (its max luminance is close to its
        // mean). So exempt textures whose peak far exceeds the mean AND is genuinely bright —
        // that's a drawable sparkle, not a shader warp-input. Property-driven, no per-skin value.
        // ...and a DARK glow sprite is the same idea one stop down. Mlynar's sword glow is a
        // 128px near-black sheet (meanLum 10.1) whose core peaks at **189** — a concentrated
        // flare by any reading, but 11 levels short of the absolute 200 above, so it was
        // classified as a flow map and skipped.
        //
        // NOT the sword-shine fix, despite being written while chasing it: that turned out to be
        // the exporter halving `_TintColor` on sub-namespaced `Particles-L2D` families, and it is
        // fixed there. This rule is metric-NEUTRAL on all three reference skins (bit-identical
        // mly 17.525 / cel 19.270 / ska 10.505) because it governs textures none of them draw.
        // It is kept on its own merits — the population split below is real — not on a measured
        // win, and it is strictly additive, so the downside is bounded.
        //
        // The property that actually separates the two populations is the PEAK-TO-MEAN RATIO:
        // a flow map is uniform, a glow is concentrated. Measured over every opaque grey/dark
        // scene texture in the corpus that this rule can reach (320 textures, 246 currently
        // skipped), the ratio splits them with a wide empty gap:
        //
        //     peak/mean  <2 : 101 skipped      4-8 :   1 skipped
        //                2-4: 138 skipped     >=8 :   6 skipped   <- all genuine glow sprites
        //
        // Nothing sits between 4.4 and 18.6. The six are Mlynar's sword glow (x2, ratio 18.6),
        // Ines' boc#8 (x2), Hoshiguma Alter (39.5) and Ling nian#9 (27.0).
        //
        // Written as an OR so it can only ever UN-skip: the original clause is untouched, so no
        // texture that draws today can start being dropped. The `maxLum >= 64` floor keeps
        // near-black noise (where a huge ratio is meaningless) out.
        const brightCore = (maxLum > 200 && maxLum > meanLum * 4) || (maxLum >= 64 && maxLum > meanLum * 8);
        const skip = meanLum < 150 && meanSat < 0.2 && !brightCore;
        const cx = (w - 1) / 2;
        const cy = (h - 1) / 2;
        const rMax = Math.min(w, h) / 2;
        const border = Math.max(2, Math.min(w, h) * 0.16);
        const smooth = (t: number) => t * t * (3 - 2 * t);
        for (let y = 0; y < h; y++) {
            for (let x = 0; x < w; x++) {
                const edge = Math.min(x, w - 1 - x, y, h - 1 - y);
                let f = edge < border ? smooth(edge / border) : 1;
                if (uniformBright) {
                    const rn = Math.hypot(x - cx, y - cy) / rMax;
                    f *= rn >= 1 ? 0 : smooth(1 - rn);
                }
                const idx = (y * w + x) * 4 + 3;
                px[idx] = Math.round(px[idx] * f);
            }
        }
        ctx.putImageData(imgData, 0, 0);
        // Opaque glow textures render additive, so desatPanel (a normal-blend mesh
        // gate) never applies — but keep the field consistent.
        return { base: PIXI.BaseTexture.from(canvas), glow: true, rawBase: PIXI.BaseTexture.from(img), darkDropBase, skip, desatPanel: false, hazePanel: false };
    } catch {
        return plain();
    }
}

function loadTexture(url: string): Promise<ILoadedTex> {
    return new Promise((resolve, reject) => {
        const img = new Image();
        img.crossOrigin = "anonymous";
        img.onload = () => resolve(processGlowTexture(img));
        img.onerror = () => reject(new Error(`Failed to load particle texture: ${url}`));
        img.src = url;
    });
}

/** Max value a MinMaxScalar can take (for "is it ~stationary?" / life-span tests). */
function scalarMax(s: MMScalar | undefined): number {
    if (!s) return 0;
    switch (s.mode) {
        case "const":
            return s.v;
        case "range":
            return Math.max(s.min, s.max);
        case "curve":
            return Math.max(...s.curve.map((p) => p.v), 0);
        case "rangeCurve":
            return Math.max(...s.min.map((p) => p.v), ...s.max.map((p) => p.v), 0);
    }
}

/** Intensity scale for an ADDITIVE billboard emitter, correcting for OVER-PILING.
 *
 *  The HDR pass stops additive stacks from clipping to white and keeps their hue, but
 *  it can't soften a stack that's genuinely bright-white: a STATIONARY emitter
 *  (`startSpeed≈0`) with a CONTINUOUS emission rate piles every particle it ever emits
 *  onto the same point — Wiš'adel "Supernova"'s `rate=60/s`, `life=3s` bloom stacks
 *  ~180 identical 450px sprites into one spot, a hard opaque white blob. Physically
 *  that's ONE glow over-counted ~180×. So when a stationary additive emitter's expected
 *  concurrent count (`rate × life`) exceeds a small target, scale it down to that
 *  target — the glow reads as soft light the backdrop shows through, instead of a solid
 *  shape. MOVING emitters (speed > 0) spread their particles over distinct positions, so
 *  they don't pile and aren't touched; BURST emitters (Hoshiguma's ice-flames: `rate=0`,
 *  `life=0.2s`, one particle per burst) have ~0 concurrent pile and stay full-bright. */
function additivePileGain(sys: IParticleSystemData): number {
    const STATIONARY_SPEED = 1; // authored-px/s; these blooms are exactly 0
    const TARGET_STACK = 0.8; // allowed overlapping-particle "brightness" at the core
    const MIN_GAIN = 0.02;
    // Motion is not only `startSpeed`: a system can sit still at birth and drift entirely on
    // `velocityOverLifetime`. Skadi the Corrupting Heart's crown shoal is exactly that —
    // startSpeed 0, but 60 px/s of vol over a 4.2 s life = ~250 px of travel — so it spreads
    // and never piles, yet the startSpeed-only test dimmed it to 9.5 % and erased the ring of
    // fish the game draws around her crown. Both terms mean the same thing here: the particles
    // move apart, so their brightness does not stack at a point.
    const vol = sys.velocityOverLife;
    const volSpeed = vol ? Math.hypot(vol.x ?? 0, vol.y ?? 0) : 0;
    if (scalarMax(sys.startSpeed) > STATIONARY_SPEED || volSpeed > STATIONARY_SPEED) return 1; // moving → spreads, no pile
    const rate = sys.emission.rate ? scalarMax(sys.emission.rate) : 0;
    // Expected concurrent particles at the point — but the pile can NEVER exceed the system's
    // `maxParticles`, so cap by it. Virtuosa's warm ambient glows are `rate=1000` but
    // `maxParticles=1` (a single re-spawning glow, ZERO pile); the uncapped `rate×life` read
    // 1000 and clamped them to the 0.02 floor, erasing the reform's gold mist. A genuine
    // over-pile (Wiš'adel "Supernova": ~180 stacked sprites) needs many concurrent particles,
    // so its `maxParticles` is high and the cap leaves it untouched.
    const maxP = sys.maxParticles && sys.maxParticles > 0 ? sys.maxParticles : Number.POSITIVE_INFINITY;
    const density = Math.min(rate * scalarMax(sys.lifetime), maxP); // concurrent particles at the point
    if (density <= TARGET_STACK) return 1; // burst/sparse/single → no meaningful pile
    return Math.max(MIN_GAIN, TARGET_STACK / density);
}

/** Mirror of sceneMesh's `EFFECT_SCENE_GAIN` / `EFFECT_TEX_MAX` on the PARTICLE path.
 *  On a scene that owns its own DARK painted backdrop (`hasDarkBackdrop`, cello only), a
 *  LARGE additive billboard feeds the HDR bloom at full gain and blooms into the crown halo,
 *  washing the white diamond lattice (Virtuosa's two 1313px `star_large` flares, tex5). The
 *  scene-layer path already tempers a large additive layer to `EFFECT_SCENE_GAIN` on such
 *  scenes (`sceneMesh.buildLayerMesh`); the particle path had no equivalent. Apply the same
 *  tamed gain to a large additive billboard, ATTENUATING (not deleting — a subtle central
 *  flare survives) and gated on the data-derived flag so every other skin is byte-identical.
 *  "Large" = billboard bigger than the effect-overlay size (mirrors `EFFECT_TEX_MAX`), so
 *  small additive sparks (the amber orb, comet trail, glow motes) are untouched. */
const EFFECT_PARTICLE_GAIN = 0.3; // = sceneMesh EFFECT_SCENE_GAIN
const EFFECT_PARTICLE_MAX = 512; // = sceneMesh EFFECT_TEX_MAX (effect-overlay vs large boundary)

/** DIAGNOSTIC registry of every particles module loaded on this page.
 *
 *  A corpus census wants one question answered per system: does it EVER emit? Sampling the
 *  live probe at fixed beats cannot answer it, and gets it wrong in both directions — a
 *  system whose window falls between beats reads dead (23 of Wisadel's 93 did, all of them
 *  fine), and an ENTRANCE composite is destroyed at the hand-off, so any probe scheduled
 *  after that sees nothing at all while a probe before its late systems fire calls them dead
 *  (Ch'en the Holungday's four systems are delayed to 8.0s of a 9.767s entrance). The
 *  emitters' `everLive` latch is authoritative and survives destruction inside this closure,
 *  so a SINGLE dump at the end of the clip reports every composite exactly.
 *
 *  One entry per composite per page load (at most a handful), so retaining them is free. */
const PARTICLE_CENSUS: { emitters: number; snapshot(): { sys: number; everLive: boolean }[] }[] = [];

/** Finalised never-emitted census for every composite this page built, live or destroyed. */
export function particleCensus(): { emitters: number; systems: { sys: number; everLive: boolean }[] }[] {
    return PARTICLE_CENSUS.map((c) => ({ emitters: c.emitters, systems: c.snapshot() }));
}

export async function loadParticles(url: string, textureBaseUrl: string, bust = "", characterBounds: IAnimationBounds | null = null, hasDarkBackdrop = false): Promise<ILoadedParticles | null> {
    let data: IParticlesData;
    try {
        const res = await fetch(url);
        if (!res.ok) return null;
        data = (await res.json()) as IParticlesData;
    } catch {
        return null;
    }
    if (!data.systems?.length) return null;

    const bases = await Promise.all(Array.from({ length: data.textureCount }, (_, i) => loadTexture(`${textureBaseUrl}${i}.png${bust}`).catch(() => null)));

    const background = new PIXI.Container();
    const foreground = new PIXI.Container();
    const emitters: Array<Emitter | RamEmitter> = [];
    /** `emitters[i]` came from `data.systems[emitterSys[i]]` — skipped systems leave no entry. */
    const emitterSys: number[] = [];
    let liveEstimate = 0;
    const budget = () => GLOBAL_MAX_PARTICLES - liveEstimate;

    /** Does this world-space system's STATIC spawn disc substantially overlap the
     *  character's own body bounds? A per-system geometric test (not a blanket flag) —
     *  only systems that would render mostly/fully hidden behind the character get
     *  promoted out of `background`; ambient background props positioned away from the
     *  character (embers, ground mist, magic circles) are untouched. `sys.pos` is
     *  authored export px, Y-up; negate Y to match `characterBounds`'s Y-down (Pixi
     *  local-bounds) convention — the same negation `applyDisp`/spawn already use for
     *  every particle's own screen position. No per-skin constant: the 0.5 threshold and
     *  the disc-vs-box geometry are generic. */
    const overlapsCharacter = (sys: IParticleSystemData): boolean => {
        if (!characterBounds) return false;
        const r = sys.shape?.radius ?? 0;
        const cx = sys.pos[0];
        const cy = -sys.pos[1];
        const discW = Math.max(2 * r, 1);
        const discH = Math.max(2 * r, 1);
        const dx0 = cx - discW / 2;
        const dy0 = cy - discH / 2;
        const ix0 = Math.max(dx0, characterBounds.x);
        const iy0 = Math.max(dy0, characterBounds.y);
        const ix1 = Math.min(dx0 + discW, characterBounds.x + characterBounds.width);
        const iy1 = Math.min(dy0 + discH, characterBounds.y + characterBounds.height);
        const iw = Math.max(0, ix1 - ix0);
        const ih = Math.max(0, iy1 - iy0);
        return (iw * ih) / (discW * discH) >= 0.5;
    };

    /** Raw (unprocessed) texture for a Ram shader slot, by [particles] index.
     *  Ram, disturb and dissolve inputs are DATA textures (ramp / flow / mask)
     *  that must stay pixel-exact. */
    const rawTex = (i: number | null): PIXI.Texture | null => {
        const b = i != null ? bases[i] : null;
        return b ? new PIXI.Texture(b.rawBase) : null;
    };

    /** Main texture for a Ram slot: the dark-field-dropped base (see
     *  ILoadedTex.darkDropBase). An opaque glow main (dark starfield / bokeh,
     *  alpha=1) would otherwise keep its near-black field and, ×2-boosted, stamp a
     *  faint grey box on the black canvas; luminance-alpha drops that field while
     *  preserving a bright fill (SilverAsh's dissolve-shaped reticle rings).
     *
     *  Sampling the RAW texture for a material that carries a `_DissolveTex` (which
     *  already carves the shape, so the fill's own alpha is genuinely 1) is what the
     *  Unity shader does, and it recovers 2.3× the light on Mlynar's city slabs — but
     *  it measured WORSE against the game on all three reference skins, because our
     *  render already sits well ABOVE the capture's luminance through that whole shot.
     *  Kept as-is until that DC excess is understood; see the round-11 notes. */
    const ramMainTex = (i: number | null): PIXI.Texture | null => {
        const b = i != null ? bases[i] : null;
        return b ? new PIXI.Texture(b.darkDropBase) : null;
    };

    for (const [sysIndex, sys] of data.systems.entries()) {
        // Ram-shader emitters render via the ported ramp/dissolve/disturb shader
        // (RamEmitter), using RAW textures. Needs a main tex. Mesh-render-mode Ram
        // systems are skipped (their real mesh geometry isn't exported; billboarding
        // a large one stamps a raw quad).
        //
        // A Ram system carrying a texture-sheet flipbook (SilverAsh's purple sword
        // flame) prefers the SPRITE path: it slices the atlas into per-frame textures,
        // and that's the shipped, tuned look. But when the sprite path would DROP the
        // system outright — its `_MainTex` measures as a flow/cloud map (`tex.skip`),
        // which a Ram main slot is by construction: the shape comes from `_DissolveTex`,
        // not the main sampler — the system renders nothing at all. Mlynar's 14
        // `fangkuai_*` city slabs are exactly that case (a bubble normal-map main, a
        // 4-tile slab silhouette dissolve): both paths rejected them, so the whole
        // brightening beat of his transformation was missing. Fall back to the Ram
        // path, which now drives the flipbook tile through the vertex UVs.
        const ramSheet = !!(sys.sheet && sys.sheet.tilesX * sys.sheet.tilesY > 1);
        const spriteWouldDrop = sys.ram?.mainTex != null && !!bases[sys.ram.mainTex]?.skip;
        if (sys.ram && (!ramSheet || spriteWouldDrop)) {
            if (sys.renderMode === "mesh") continue;
            const main = ramMainTex(sys.ram.mainTex);
            if (!main) continue;
            const emitter = new RamEmitter(sys, sys.ram, { main, ram: rawTex(sys.ram.ramTex), disturb: rawTex(sys.ram.disturbTex), dissolve: rawTex(sys.ram.dissolveTex), dissolve2: rawTex(sys.ram.dissolveTex2 ?? null) }, sys.blend, budget);
            emitters.push(emitter);
            emitterSys.push(sysIndex);
            applyPsDiag(data, sys, emitter.container);
            (sys.sort < data.characterSort ? background : foreground).addChild(emitter.container);
            continue;
        }
        // UNTEXTURED MESH: a `renderMode:"mesh"` system whose material has no `_MainTex`
        // is not a missing asset — Unity draws the mesh in the material's own COLOUR,
        // which the exporter already ships as `startColor` / `colorOverLife`. Virtuosa
        // "Diversity in Oneness"'s `scene_02/window/quad_p` is exactly that: an untextured
        // square RING (12 verts / 8 tris, ±0.25 outer, ±0.215 inner) spawned twice a second
        // at 716 px, growing across a 5 s life and turning 45°/s — the nested white diamond
        // outlines the game draws behind her head from t≈9.8. Every path below indexes
        // `bases[sys.tex]` first, so a null texture dropped the system whole.
        // Corpus-wide this admits that one system (plus its idle-scene twin) and nothing
        // else: the remaining texture-less systems are billboards, which have no geometry
        // to draw, or Ram-shader meshes the branch above already consumes.
        if (sys.tex == null) {
            if (sys.renderMode === "mesh" && sys.mesh && sys.mesh.idx.length >= 3) {
                // ×2 COLOUR: a material with no `_MainTex` samples Unity's built-in white,
                // so the draw is purely the material colour — and these Torappu compositors
                // carry the ×2 half-neutral convention the Ram GLSL port already implements
                // (`col += col`). Measured on the game capture at t=14/17, where the ribbons
                // cross a mid backdrop: they read 215-236 luma, which the ×2 reaches and the
                // plain ×1 composite (≈87) does not. ×1 measured 35.878 MADC against ×2's
                // 35.321 on Virtuosa's seven beats.
                // EMITTER ROTATION: `quad_p`'s square ring is authored axis-aligned and posed
                // by its emitter transform's 160.63° z-angle, so the rings only land as the
                // game's ~45° diamonds once that angle is applied (independently confirmed by
                // projecting the size curve: the three visible rings sit at life 0.9/0.8/0.7,
                // whose own 45°/s spin puts them at 22.5°/0°/67.5° — 41.9°/-19.4°/25.6° once
                // the emitter angle is folded in, which is what the capture shows). Passed
                // only here: applying it to the TEXTURED mesh emitters as well measured worse
                // (Virtuosa t=2 32.19 → 34.75, her `window_bg_01` backdrop panel carries a
                // 180° emitter angle the game plainly does not draw it with).
                const emitter = new MeshEmitter(sys, PIXI.Texture.WHITE, sys.blend, budget, untexMeshGain(), sys.rot ?? 0);
                emitters.push(emitter);
                emitterSys.push(sysIndex);
                emitter.container.alpha = sys.blend === "additive" ? additivePileGain(sys) : 1;
                applyPsDiag(data, sys, emitter.container);
                (sys.sort < data.characterSort ? background : foreground).addChild(emitter.container);
            }
            continue;
        }
        const tex = bases[sys.tex];
        if (!tex) continue;
        // A flow/distortion/cloud map (opaque dark-grey field) is a shader input, not
        // a drawable sprite; billboarding/mesh-stamping it leaves flickering grey
        // rectangles (Hoshiguma Alter's top "wave artifacts"). Skip it here — the
        // Ram-shader path above already consumed such maps as flow inputs.
        if (tex.skip) continue;
        // Scene-DEPTH atmospherics exported at a particle sort ABOVE the character —
        // Unity "bg_*" GameObjects (tint/rain/reflection washes: Virtuosa "Diversity
        // Oneness"'s bg_tint_01 / bg_rain_01) and large soft haze clouds (its air_01,
        // see ILoadedTex.hazePanel). Drawn in front, a full-scene normal-blend sheet
        // fogs the focal character's opaque body into a translucent ghost. The game
        // keeps them in the scene depth; demote behind the spine. Applies to BOTH the
        // mesh-render and billboard-sprite paths below (these emitters take either).
        // Additive glows (flames, light motes) and small crisp effects are untouched.
        const effBlend: "additive" | "normal" = tex.glow ? "additive" : sys.blend;
        const bgNamed = (sys.boneChain ?? []).some((n) => /^bg[_ -]/i.test(n));
        // DIAGNOSTIC `?bdp=0`: keep these sheets at their AUTHORED depth instead of demoting
        // them. Every one of them is authored ABOVE `characterSort` (cello: bg_tint_01 sort 10,
        // bg_rain_01 25, bg_ref 3, air_01 12, against characterSort 0), so Unity draws them in
        // FRONT of the spine — the demotion is the deviation, not the fidelity.
        const isBackdropParticle = effBlend === "normal" && (bgNamed || (tex.hazePanel && scalarMax(sys.startSize) > 400)) && backdropDemoteEnabled();
        // A large STATIC PROP faked as a particle: a single burst that persists ~the whole
        // cinematic (not a live emitting effect), large enough to span much of the frame.
        // Virtuosa's draped `chair` pedestal is one such (tex 447px ≈ 0.43× the 1050px camera
        // view, rate 0, one burst, 17s life) — the seat she sits ON, which the game never shows
        // over her (it draws the real backdrop/stage as scene mesh). Property-driven (emission
        // shape + size vs camera view), no per-skin value; applied billboard-only (below), so
        // the mesh path's own layering is untouched.
        const emRate = sys.emission?.rate ? scalarMax(sys.emission.rate) : 0;
        const burstTotal = (sys.emission?.bursts ?? []).reduce((a, b) => a + b.count, 0);
        const isStaticProp = effBlend === "normal" && !sys.looping && emRate < 1 && burstTotal >= 1 && burstTotal <= 2 && scalarMax(sys.lifetime) >= 10 && scalarMax(sys.startSize) > 0.4 * (data.cameraSizePx || 1050);
        // The SECOND spelling of the same "static geometry faked as a particle system" idea,
        // for a MESH emitter. Where `isStaticProp` above recognises the BURST spelling (rate 0,
        // one or two bursts, a lifetime spanning the cinematic), this recognises the
        // CAP-SATURATED one: `maxParticles: 1` with an emission rate that overruns that cap by
        // orders of magnitude, so the cap — not the rate — governs and exactly one particle is
        // ever alive; a lifetime spanning the whole emitter cycle, so that particle never dies
        // within it; and ZERO dynamics of any kind — no start speed, no gravity, no spawn shape,
        // and no velocity / size / colour / rotation curve, noise or trail. A system authored
        // that way has no time-varying output at all: it draws one mesh, unchanging, forever.
        // That is scenery, not an effect.
        //
        // Virtuosa "Diversity in Oneness"'s ten `wing_p` rigs are the case that forced this.
        // They alpha-blend a hot-pink membrane (tex 250,81,140) over her wing blades, which the
        // game does not draw ANYWHERE in frame — hue-matching the capture at t=12, its pink
        // pixel count and centroid equal our particles-OFF render (8761 @ (433,73) vs 6784 @
        // (433,94)), i.e. all of the game's pink is her own artwork, while ours jumps to 22531
        // @ (492,134). Our render of them is nonetheless FAITHFUL — rasterising the mesh
        // triangles and weighting by screen area, the authored alpha is 0.156 against the 0.169
        // we measure off the frame (1.08×) — so the mismatch is not an amplitude bug to fix but
        // content the game withholds at runtime (see the `_HGExternalCtrl` kill switch in its
        // shader, which is keyword-selected and never serialised). Everything else was excluded
        // with measurements: the fragment program (`Torappu/Particles-L2D/AlphaBlend` computes
        // `tex × (COLOR+COLOR)` with `_TintColor` at the 0.502 half-neutral, alpha included, so
        // the doubling cancels), renderer `m_Enabled`/`m_RenderMode`, the director's `_effects`
        // activation, an `m_IsActive` OFF-window (none exists — every window ends `until=None`),
        // the GameObject layer (entrance-root vs idle-root, not a culling mask), depth demotion
        // (68 % of the harm is OFF her silhouette) and a placement shift (correlation 0.173 at
        // zero, best only 0.226). Worth −0.340 MADC, improving every live beat.
        // BONE-ATTACHED AND IN FRONT OF THE CHARACTER. The zero-dynamics shape above also fits
        // painted SCENERY faked as a mesh particle, which the game genuinely draws and which the
        // `isBackdropPanel` carve-out below exists to protect (skipping such a panel leaves the
        // dark environment backdrop showing through as a flat black rectangle). Two kinds turned
        // up in the corpus scan: unparented sheets (Wiš'adel's 3522 px `sort: -18`, Siege's
        // 1920 px `sort: -50` and 1050 px `sort: -71`), excluded by requiring a followed BONE;
        // and bone-attached scenery (Nian's six `BG_Screen_*` panels at `sort: -133…-73`),
        // excluded by the same `sort < characterSort ⇒ scenery` line the mesh path already
        // draws. What remains is rig decoration pinned to the skeleton and drawn OVER the
        // character — which is what pokes out of a silhouette the game keeps clean.
        const noDynamics = scalarMax(sys.startSpeed) === 0 && scalarMax(sys.gravity) === 0 && (sys.shape?.type ?? "none") === "none" && !sys.velocityOverLife && !sys.sizeOverLife && !sys.colorOverLife && !sys.noise && !sys.rotOverLifeDegPerSec && !sys.trail && !sys.tint;
        const isStaticMeshProp = effBlend === "normal" && sys.renderMode === "mesh" && !!sys.followBone && sys.sort >= data.characterSort && sys.looping && sys.maxParticles === 1 && emRate * scalarMax(sys.lifetime) >= 100 && sys.duration > 0 && scalarMax(sys.lifetime) >= sys.duration && noDynamics;
        // Mesh render mode emits particles as arbitrary textured MESHES (whose
        // geometry we don't export), often a single large quad with a non-sprite
        // texture (a galaxy band, a mask, a distortion map). Billboarding those
        // shows the raw texture as a hard rectangle/band — the "black block" and
        // "rainbow streak" artifacts — so skip them. EXCEPT when the emitter drives
        // a texture-sheet flipbook: that's a genuine animated effect sprite (e.g.
        // SilverAsh the Reignfrost's purple-sword flame), which billboards cleanly
        // as a single cropped frame.
        const hasSheet = !!(sys.sheet && sys.sheet.tilesX * sys.sheet.tilesY > 1);
        // Mesh-render systems: instance the exported mesh geometry per particle
        // (Hoshiguma's ice-crystal shards). If no geometry was exported (a streamed/
        // compressed mesh the unpacker couldn't decode) and it's not a flipbook,
        // skip as before — billboarding it would stamp the texture as a hard quad.
        if (sys.renderMode === "mesh" && !hasSheet) {
            const meshBlend: "additive" | "normal" = tex.glow ? "additive" : sys.blend;
            // Only ADDITIVE mesh particles are actual glowing geometry (Hoshiguma's
            // ice-crystal shards): additive adds light, so the mesh reads as a shard.
            // NORMAL-blend mesh systems are flow/distortion panels (blue-flame flow
            // maps on flat quads) meant to warp what's behind them, not to be drawn
            // directly — rendering their geometry stamps translucent grey rectangles
            // over the scene (the "animated top artifacts"). Skip them, as the
            // pre-mesh code skipped all mesh-render billboards.
            // Render the mesh when it ADDS light (additive glow shards) OR when it's a
            // COLOURED effect mesh — jagged lightning bolts, ice-flame tips, water
            // ribbons (Hoshiguma the Breacher's `lighting_*`/`fire_tip_*`/`water_*`,
            // normal-blend, sat 0.28–0.50). Only a DESATURATED, FILLED panel
            // (`desatPanel`: sat < 0.2 over a covered ≥25% texture — the blue-grey
            // flow/distortion "bg" maps, tex sat≈0.12) is a shader warp-input that
            // stamps a grey rectangle — those stay skipped. (Skipping ALL normal-blend
            // mesh instead would throw out the real lightning bolt and leave only its
            // additive glow halo.)
            // …EXCEPT a painted BACKDROP PANEL faked as a mesh particle. Same authored
            // shape `isStaticProp` recognises (rate 0, one or two bursts, a lifetime
            // spanning the whole cinematic — set dressing, not a live effect) but sorted
            // BEHIND the character, so it is scenery the game does show, not a prop that
            // would poke out around her. Virtuosa "Diversity in Oneness"'s
            // `scene_01/little/window_bg_01` (634px, one 17.5s burst, sort −2) paints the
            // soft grey window light behind her halo; the scene mesh is transparent there,
            // so skipping it leaves the dark environment backdrop showing through as a flat
            // BLACK rectangle. Such a panel measures `desatPanel` (soft, low-saturation
            // wash) exactly like the flow maps the skip targets — sort + burst shape is what
            // separates them.
            const isBackdropPanel = !sys.looping && emRate < 1 && burstTotal >= 1 && burstTotal <= 2 && scalarMax(sys.lifetime) >= 10 && sys.sort < data.characterSort;
            const meshOK = (meshBlend === "additive" || !tex.desatPanel || isBackdropPanel) && !isStaticMeshProp;
            if (sys.mesh && sys.mesh.idx.length >= 3 && meshOK) {
                const emitter = new MeshEmitter(sys, new PIXI.Texture(tex.base), meshBlend, budget);
                emitters.push(emitter);
                emitterSys.push(sysIndex);
                // A LARGE additive glow mesh (Hoshiguma the Breacher's lightning-bolt
                // halos, startSize≈476) spreads a lot of additive light, and several
                // co-fire on the same burst — through the HDR bloom they stack into one
                // washed-out white BLOB that swallows the crisp normal-blend bolt drawn
                // under it. Dim large additive glow meshes so the bolt reads as a defined
                // shape (the in-game look) instead of a flare. Small additive shards
                // (crystals) spread little light and stay full-bright.
                const bigGlow = meshBlend === "additive" && scalarMax(sys.startSize) > 300;
                // `additivePileGain` corrects ADDITIVE over-piling (many stacked glows reading as
                // one opaque blob) — it must ONLY touch additive meshes, exactly as the billboard
                // path gates it below. NORMAL-blend meshes alpha-composite (they don't accumulate
                // brightness, so there's no pile to correct); dimming them clamps a full-frame
                // effect panel to ~2% and it vanishes (Virtuosa's mirror-world/chevron overlay was
                // invisible through the whole reform because of this). Normal-blend → full alpha.
                emitter.container.alpha = (meshBlend === "additive" ? additivePileGain(sys) : 1) * (bigGlow ? 0.4 : 1);
                applyPsDiag(data, sys, emitter.container);
                (sys.sort < data.characterSort || isBackdropParticle ? background : foreground).addChild(emitter.container);
            }
            continue;
        }
        // A large opaque STATIC PROP faked as a 1-particle billboard is a set-dressing
        // element the game layers in the scene, never a foreground effect — Virtuosa's
        // draped `chair` (tex 447px, one 17s burst) is the seat she sits on. Even demoted
        // behind the spine it POKES OUT around her body (it's wider than she is), which the
        // game never shows. Skip it outright, matching the mesh path's desatPanel skip — the
        // real backdrop/stage is drawn by the scene-mesh layer, not this prop billboard.
        if (isStaticProp) continue;
        // A fully-opaque glow/starfield texture (luminance baked into alpha above)
        // must render additive so its dark field drops out rather than stamping a
        // square — regardless of the authored blend.
        const blend: "additive" | "normal" = tex.glow ? "additive" : sys.blend;
        // Trail uses its own texture index when the exporter found a distinct
        // trail material; otherwise it reuses the particle texture.
        const trailTex = sys.trail && sys.trail.tex != null ? bases[sys.trail.tex] : null;
        const trailTexture = trailTex ? new PIXI.Texture(trailTex.base) : null;
        const emitter = new Emitter(sys, new PIXI.Texture(tex.base), trailTexture, blend, budget);
        emitters.push(emitter);
        emitterSys.push(sysIndex);
        // Tame a LARGE additive billboard on a self-lit dark-backdrop scene (mirrors the
        // scene-layer `EFFECT_SCENE_GAIN` temper — see EFFECT_PARTICLE_GAIN above). Composes
        // with `additivePileGain`; still additive, so a subtle central flare remains. No-op
        // for every non-`hasDarkBackdrop` skin, and for small additive sparks.
        const temperLargeAdditive = hasDarkBackdrop && blend === "additive" && scalarMax(sys.startSize) > EFFECT_PARTICLE_MAX;
        if (blend === "additive") emitter.container.alpha = combineAdditiveGains(additivePileGain(sys), temperLargeAdditive);
        // bg_* / haze atmospherics are demoted behind the spine (see isBackdropParticle above).
        // A world-space system that would be bucketed background purely by sort (NOT already
        // demoted as a deliberate backdrop atmospheric) but whose static spawn disc sits mostly
        // ON the character's own body renders 100% hidden in our flat 2D compositor (the real
        // Unity renderer gives it genuine depth parallax around the silhouette; ours can't) —
        // e.g. Mlynar's dominant `rain_left_short_01` disc sits on his torso. Promote only that
        // narrow, geometrically-overlapping case to foreground; everything else keeps the
        // original sort-driven bucketing untouched.
        const wouldBeBackground = sys.sort < data.characterSort || isBackdropParticle;
        const unoccludeOverlap = wouldBeBackground && !isBackdropParticle && sys.simulationSpace === "world" && overlapsCharacter(sys);
        // M-c: the un-occlude promotion draws this ambient system (Mlynar's torso rain) IN
        // FRONT of the character. At full strength its streaks read as prominent/"weird" over
        // his coat at the tight entrance zoom; the game shows only a faint sheen. Dim just the
        // promoted over-body copy — the background rain (and every other system) is untouched.
        if (unoccludeOverlap) emitter.container.alpha *= FOREGROUND_SHEEN_ALPHA;
        applyPsDiag(data, sys, emitter.container);
        (wouldBeBackground && !unoccludeOverlap ? background : foreground).addChild(emitter.container);
    }
    if (emitters.length === 0) return null;

    PARTICLE_CENSUS.push({
        emitters: emitters.length,
        snapshot: () => emitters.map((e, i) => ({ sys: emitterSys[i] ?? -1, everLive: !!(e as unknown as { everLive?: boolean }).everLive })),
    });
    return {
        data,
        background,
        foreground,
        update(dt: number, findBone?: FindBone, restBone?: RestBone, displayBox?: IAnimationBounds | null, restAtt?: RestAttachment) {
            // Recompute the shared live count once per frame for the budget.
            liveEstimate = 0;
            for (const e of emitters) liveEstimate += e.liveCount();
            for (const e of emitters) e.update(dt, findBone, restBone, displayBox, restAtt);
        },
        probe(viewW = 0, viewH = 0): IEmitterProbe[] {
            const gp = new PIXI.Point();
            return emitters.map((e, i) => {
                const pos = e.container.position;
                const g = e.container.parent ? e.container.getGlobalPosition(new PIXI.Point()) : null;
                // getBounds() reflects what the renderer will actually draw (visibility,
                // transforms, culling) — cheaper to trust than rebuilding the chain by hand.
                const b = e.container.getBounds();
                const box = Number.isFinite(b.width) && b.width > 0 && b.height > 0 ? { x: b.x, y: b.y, w: b.width, h: b.height } : null;
                let onScreen = 0;
                let paintedPx = 0;
                if (viewW > 0 && viewH > 0) {
                    for (const ch of e.container.children) {
                        if (!ch.visible || !ch.renderable) continue;
                        ch.getGlobalPosition(gp);
                        if (gp.x >= 0 && gp.x <= viewW && gp.y >= 0 && gp.y <= viewH) onScreen++;
                        // Drawn rect ∩ viewport. getBounds() is the child's world AABB, so size,
                        // scale and the stretched-billboard rotation are already folded in.
                        const cb = ch.getBounds();
                        if (!Number.isFinite(cb.width) || !Number.isFinite(cb.height)) continue;
                        const iw = Math.min(viewW, cb.x + cb.width) - Math.max(0, cb.x);
                        const ih = Math.min(viewH, cb.y + cb.height) - Math.max(0, cb.y);
                        if (iw > 0 && ih > 0) paintedPx += iw * ih;
                    }
                }
                const dbg = "dbg" in e && typeof (e as { dbg?: unknown }).dbg === "function" ? (e as unknown as { dbg(): unknown }).dbg() : null;
                return { sys: emitterSys[i] ?? -1, dbg, everLive: (e as unknown as { everLive?: boolean }).everLive ?? null, live: e.liveCount(), x: pos.x, y: pos.y, sx: g?.x ?? null, sy: g?.y ?? null, box, onScreen, paintedPx, rope: "pool" in e ? ropeStats(e.pool) : null };
            });
        },
        destroy() {
            for (const e of emitters) e.destroy();
        },
    };
}
