import * as PIXI from "pixi.js";

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

export type MMColor = { mode: "color"; r: number; g: number; b: number; a: number } | { mode: "gradient"; stops: IColorStop[] };

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
    tex: number;
    /** Mesh-render-mode geometry (ice crystals, ribbons): flat mesh-LOCAL triangle
     *  geometry the exporter emits for `renderMode:"mesh"` systems. Each particle
     *  instances it, scaled by the particle's size. Absent for billboard/stretch. */
    mesh?: { pos: number[]; uv: number[]; idx: number[]; col?: number[] };
    blend: "additive" | "normal";
    renderMode?: string;
    pos: [number, number];
    rot?: number;
    duration: number;
    looping: boolean;
    /** Cinematic start delay (s): the emitter stays dormant until the system's elapsed
     *  time reaches this, then begins as if just created. The `_Start` cinematic gates its
     *  effect groups (`_delayTime` activators + `m_IsActive` switch-ons) — e.g. the apple's
     *  golden sparks fire at ~6–7s when it falls, not at t=0. Absent/0 = emit immediately. */
    delay?: number;
    simulationSpace?: "local" | "world";
    maxParticles: number;
    gravity?: MMScalar;
    lifetime: MMScalar;
    startSpeed: MMScalar;
    startSize: MMScalar;
    startRotation?: MMScalar;
    startColor?: MMColor;
    emission: { rate?: MMScalar; bursts?: { t: number; count: number }[] };
    shape?: { type: string; radius?: number; angleDeg?: number; arcDeg?: number; box?: [number, number]; posOffset?: [number, number]; rotDeg?: number };
    colorOverLife?: MMColor | null;
    sizeOverLife?: ICurvePoint[] | null;
    velocityOverLife?: { x: number; y: number; space?: string } | null;
    rotOverLifeDegPerSec?: number | null;
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
    noise?: INoise | null;
    trail?: ITrail | null;
    sheet?: { tilesX: number; tilesY: number; frameOverTime?: ICurvePoint[] | null; cycles?: number } | null;
    /** Material params for the "Ram" shader family (ramp-tint + dissolve +
     *  UV-disturb sprite compositor). When present, the emitter is rendered by
     *  {@link RamEmitter} with a GLSL port of the real shader instead of as a
     *  plain tinted billboard — this is what makes SilverAsh the Reignfrost's
     *  crisp cyan energy rings/arcs (and any Ram effect) render faithfully. */
    ram?: IRamData;
}

/** Exported `_MainTex_ST`-style tiling/offset: [scaleX, scaleY, offsetX, offsetY]. */
type RamST = [number, number, number, number];

export interface IRamData {
    kind: "disturb" | "vertexDisturb";
    mainTex: number | null;
    mainST: RamST;
    ramTex: number | null;
    ramST: RamST;
    disturbTex: number | null;
    disturbST: RamST;
    dissolveTex: number | null;
    dissolveST: RamST;
    mainColor: [number, number, number, number];
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
    blend: "additive" | "normal";
    ratio: number;
    lifetime: MMScalar;
    minVertexDistance: number;
    widthOverTrail?: MMScalar | null;
    colorOverLifetime?: MMColor | null;
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
/** Floor on a spawned particle's lifetime (s), guarding degenerate authored data. */
const MIN_PARTICLE_LIFE = 0.05;

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

function sampleColor(c: MMColor | undefined | null, t: number): IRGBA {
    if (!c) return { r: 1, g: 1, b: 1, a: 1 };
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
    rope?: PIXI.SimpleRope;
    hist?: number[];
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
}

/** Longest lifetime (s) that still gets `velocityOverLifetime` drift. See below. */
const VELOCITY_MAX_LIFE = 0.8;

/** Streak length (px) added per unit screen-speed (px/s) for `renderMode:"stretch"`
 *  particles — ~one 20fps frame of motion blur, so a 500px/s rain speck draws a ~25px
 *  streak. Purely visual; scales with each particle's own speed. */
const STRETCH_LEN_PER_SPEED = 0.05;

/** velocityOverLifetime drift, applied ONLY to SHORT-lived particles.
 *
 *  Unity's `velocityOverLifetime` here is a CONSTANT push we integrate over the
 *  particle's lifetime. Over a LONG lifetime that's a huge straight-line fly-off the
 *  in-game archive never shows (Hoshiguma the Breacher's `fire_p` systems: lifetime
 *  5–6s, ~100px/s → a 500–600px sweep across the frame; no drag/`ClampVelocity` in the
 *  data arrests it). For SHORT-lived particles the total travel is BOUNDED and the
 *  motion IS the effect: Virtuosa's rain (`velocityOverLife.y = −500`, life 0.3–0.4s →
 *  ~150–200px fall) and spark streaks. So apply it only below a short lifetime — rain
 *  falls, long-lived flames stay contained. Property-driven, no per-skin value. The
 *  velocity is in the emitter's LOCAL frame (matching each particle's container-local
 *  `p.x/p.y`), so no world rotation is needed. */
function worldVelocityOverLife(d: IParticleSystemData): { x: number; y: number } | null {
    const vol = d.velocityOverLife;
    if (!vol || (vol.x === 0 && vol.y === 0)) return null;
    if (scalarMax(d.lifetime) > VELOCITY_MAX_LIFE) return null;
    return { x: vol.x, y: vol.y };
}

/** Every bone's REFERENCE-pose world matrix, keyed by bone name — captured once
 *  (the pose the authored particle positions were baked at). Used to pick the
 *  follow bone (nearest to the emitter's spawn point) and as the delta reference.
 *  See {@link driftWithBone}. */
export type RestBone = ReadonlyMap<string, PIXI.Matrix>;

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
 *  Follow-bone selection: only emitters whose exporter `boneChain` names a real
 *  spine bone are followed (un-parented world-fixed emitters — bg sparks, magic
 *  circles — name none and stay put). Among those, we track the bone PHYSICALLY
 *  NEAREST the emitter's spawn `pos`, i.e. the bone driving the visual the flame
 *  sits on — the named `Sword_Fx` is only an FX-anchor that may not rotate with the
 *  rendered blade. */
function driftWithBone(container: PIXI.Container, chain: string[] | undefined, pos: readonly [number, number], find: FindBone | undefined, st: IBoneAnchor, restBone?: RestBone): void {
    if (!st.resolved) {
        st.resolved = true;
        st.boneName = null;
        st.ref = null;
        if (find && restBone && chain && chain.some((n) => restBone.has(n))) {
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
    if (!st.boneName || !st.ref || !find) return;
    const now = find(st.boneName);
    if (!now) return;
    // pixi-spine bone matrices are already in the skeleton's Y-DOWN world space —
    // the SAME space the particle sprites live in (each sprite draws at `-p.y`, so a
    // sprite's container position IS Y-down). So the bone's rigid delta applies
    // DIRECTLY to the emitter container. Conjugating it by a Y-flip (as if the bone
    // were Y-up) double-flips the rotation: it reads as identity at idle but slides
    // the flame off the blade the moment the Special rotates the sword.
    const d = now.clone().append(st.ref.clone().invert());
    container.transform.setFromMatrix(d);
}

/** One live billboard emitter: owns a sprite pool inside a container. */
class Emitter {
    protected readonly data: IParticleSystemData;
    protected readonly texture: PIXI.Texture;
    readonly container = new PIXI.Container();
    private readonly pool: IParticle[] = [];
    private readonly free: IParticle[] = [];
    private time = 0;
    private emitAcc = 0;
    private firedBursts = new Set<number>();
    private readonly rate: number;

    private readonly trailTexture: PIXI.Texture | null;
    private readonly trailLayer: PIXI.Container | null;

    /** Effective blend for particle sprites (may differ from data.blend). */
    protected readonly blend: "additive" | "normal";

    /** velocityOverLifetime drift, short-lived particles only (see {@link worldVelocityOverLife}). */
    private readonly volWorld: { x: number; y: number } | null;
    /** Bone-follow state so the effect drifts with its parent bone (see {@link driftWithBone}). */
    private readonly boneAnchor: IBoneAnchor = { resolved: false, boneName: null, ref: null };

    /** Texture Sheet Animation frames (sub-rectangles of the base texture), when
     *  the emitter flipbooks through a tile grid; empty for a plain sprite. */
    protected readonly frames: PIXI.Texture[] = [];
    /** Width of one sprite (a sheet frame's, else the whole texture) — the scale
     *  divisor so `startSize` maps to on-screen pixels regardless of tiling. */
    protected readonly spriteW: number;

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
        this.time = -(data.delay ?? 0);
        // Crop to the material's `_MainTex_ST` cell first, so both the whole-sprite
        // path and the sheet-slicing below operate on the selected atlas region
        // (systems that use ST-cropping carry no Texture Sheet, so the two don't mix).
        texture = cropByMainST(texture, data.mainST);
        this.texture = texture;
        this.trailTexture = trailTexture;
        this.blend = blend;
        this.container.sortableChildren = false;
        this.volWorld = worldVelocityOverLife(data);
        this.rate = data.emission?.rate ? sampleScalar(data.emission.rate, 0.5, 0) : 0;
        // Trails render behind the particle heads.
        this.trailLayer = data.trail ? new PIXI.Container() : null;
        if (this.trailLayer) this.container.addChild(this.trailLayer);

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
        } else {
            this.spriteW = texture.width || 1;
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
        sprite.anchor.set(0.5);
        sprite.blendMode = this.blend === "additive" ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
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
        if (this.data.renderMode === "stretch") {
            // Unity "Stretched Billboard" (rain streaks, spark trails): elongate the sprite
            // ALONG its screen velocity, length growing with speed, so a fast 1–3px rain speck
            // reads as a thin streak instead of an invisible dot. Gated to renderMode "stretch"
            // so ordinary billboards are untouched; applies to any stretch system, no per-skin value.
            const vx = p.vx + (this.volWorld?.x ?? 0);
            const vyScreen = -(p.vy + (this.volWorld?.y ?? 0)); // screen Y is -p.y
            const spd = Math.hypot(vx, vyScreen);
            if (spd > 1) {
                const len = sz + spd * STRETCH_LEN_PER_SPEED;
                s.rotation = Math.atan2(vyScreen, vx) - Math.PI / 2; // sprite local +Y → velocity
                s.scale.set(base, (len / sz) * base);
            } else {
                s.scale.set(base);
                s.rotation = -p.rot * DEG;
            }
        } else {
            s.scale.set(base);
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
            const r = (shape.radius ?? 0) * Math.sqrt(Math.random());
            switch (shape.type) {
                case "sphere":
                case "hemisphere":
                case "circle":
                    ox = Math.cos(a) * r;
                    oy = Math.sin(a) * r;
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
                    // cone / none: spread around up by angleDeg
                    const spread = (shape.angleDeg ?? 0) * DEG;
                    dirAng = 90 * DEG + (Math.random() - 0.5) * 2 * spread;
                    const rr = (shape.radius ?? 0) * Math.sqrt(Math.random());
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
        const size = sampleScalar(d.startSize, Math.random(), nt);
        const rot = sampleScalar(d.startRotation ?? { mode: "const", v: 0 }, Math.random(), nt);
        const startCol = sampleColor(d.startColor, nt);

        // Rotate the local offset + direction by the emitter's world rotation.
        const lx = ox + posOff[0];
        const ly = oy + posOff[1];
        const wx = d.pos[0] + lx * cos - ly * sin;
        const wy = d.pos[1] + lx * sin + ly * cos;
        const wDir = dirAng + rotDeg * DEG;

        let p = this.free.pop();
        if (!p) {
            const disp = this.createParticleDisp();
            this.container.addChild(disp);
            p = { x: 0, y: 0, vx: 0, vy: 0, age: 0, life: 0, size: 0, rot: 0, rotVel: 0, rand: 0, startCol, sprite: disp };
            this.pool.push(p);
        }
        p.x = wx;
        p.y = wy;
        p.vx = Math.cos(wDir) * speed;
        p.vy = Math.sin(wDir) * speed;
        p.age = 0;
        p.life = life;
        p.size = size;
        p.rot = rot; // degrees
        p.rotVel = d.rotOverLifeDegPerSec ?? 0; // degrees/sec (kept in degrees to match p.rot)
        p.rand = Math.random();
        p.startCol = startCol;
        p.sprite.visible = true;

        // Trail: give this particle a ribbon if the system trails and the ratio
        // roll passes; otherwise ensure any pooled rope stays hidden.
        const trail = d.trail;
        if (trail && this.trailLayer && Math.random() < (trail.ratio ?? 1)) {
            if (!p.trailPts || !p.rope) {
                p.trailPts = Array.from({ length: TRAIL_POINTS }, () => new PIXI.Point(wx, -wy));
                p.rope = new PIXI.SimpleRope(this.trailTexture ?? this.texture, p.trailPts);
                p.rope.blendMode = trail.blend === "additive" ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
                this.trailLayer.addChild(p.rope);
            }
            for (const pt of p.trailPts) pt.set(wx, -wy);
            p.hist = [wx, wy];
            p.rope.visible = true;
        } else if (p.rope) {
            p.rope.visible = false;
        }
    }

    update(dt: number, findBone?: FindBone, restBone?: RestBone): void {
        const d = this.data;
        this.time += dt;
        // Dormant until the cinematic start delay elapses (no emission, no particles yet).
        if (this.time < 0) return;
        driftWithBone(this.container, d.boneChain, d.pos, findBone, this.boneAnchor, restBone);

        // Emission (rate + bursts), only while the system is "playing".
        const playing = d.looping || this.time <= d.duration;
        if (playing) {
            this.emitAcc += this.rate * dt;
            while (this.emitAcc >= 1) {
                this.emitAcc -= 1;
                this.spawn();
            }
            const cycleT = d.duration > 0 ? this.time % d.duration : this.time;
            for (const burst of d.emission?.bursts ?? []) {
                const key = Math.floor(this.time / (d.duration || 1)) * 1000 + burst.t;
                if (cycleT >= burst.t && !this.firedBursts.has(key)) {
                    this.firedBursts.add(key);
                    for (let i = 0; i < Math.min(burst.count, PER_SYSTEM_CAP); i++) this.spawn();
                }
            }
            if (d.duration > 0 && cycleT < 0.05) this.firedBursts.clear();
        }

        // gravity is exported as gravityModifier×100; true accel = value×9.81 px/s², downward (−Y).
        const grav = d.gravity ? sampleScalar(d.gravity, 0.5, 0) * 9.81 : 0;
        const noise = d.noise;
        const trail = d.trail;
        const trailMinD = trail?.minVertexDistance || 1;
        const vol = this.volWorld;
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
            p.x += (p.vx + (vol?.x ?? 0)) * dt;
            p.y += (p.vy + (vol?.y ?? 0)) * dt;
            // Noise: an organic wander sampled from a time-scrolling field.
            if (noise) {
                const f = noise.frequency * 0.002;
                const ph = this.time * noise.scrollSpeed;
                p.x += fbmNoise(p.x * f + ph, p.y * f + p.rand * 17) * noise.strength * dt;
                p.y += fbmNoise(p.y * f + ph + 3.1, p.x * f + p.rand * 17 + 9.3) * noise.strength * dt;
            }
            p.rot += p.rotVel * dt;

            const sz = p.size * (d.sizeOverLife ? sampleCurve(d.sizeOverLife, lf) : 1);
            // Unity multiplies startColor × colorOverLifetime (both RGB and alpha).
            // Using colorOverLife alone discards the emitter's authored tint and
            // alpha — for Logos/Pozëmka that turned dim coloured glows (startColor
            // alpha 0.2–0.68) into full-bright white, blowing out the character.
            const lifeCol = d.colorOverLife ? sampleColor(d.colorOverLife, lf) : { r: 1, g: 1, b: 1, a: 1 };
            const col = {
                r: p.startCol.r * lifeCol.r,
                g: p.startCol.g * lifeCol.g,
                b: p.startCol.b * lifeCol.b,
                a: p.startCol.a * lifeCol.a,
            };
            const hex = rgbToHex(col);
            const alpha = Math.max(0, Math.min(1, col.a));
            this.applyDisp(p.sprite, p, sz, hex, alpha, lf);

            // Trail: record a new point once the head has moved far enough, then
            // lay the fixed ribbon points along the recent history (head first).
            if (trail && p.rope?.visible && p.hist && p.trailPts) {
                const dx = p.x - p.hist[0];
                const dy = p.y - p.hist[1];
                if (dx * dx + dy * dy >= trailMinD * trailMinD) {
                    p.hist.unshift(p.x, p.y);
                    if (p.hist.length > TRAIL_POINTS * 2) p.hist.length = TRAIL_POINTS * 2;
                }
                p.trailPts[0].set(p.x, -p.y);
                for (let i = 1; i < TRAIL_POINTS; i++) {
                    const hi = i * 2;
                    if (hi + 1 < p.hist.length) p.trailPts[i].set(p.hist[hi], -p.hist[hi + 1]);
                    else p.trailPts[i].copyFrom(p.trailPts[i - 1]);
                }
                p.rope.tint = hex;
                p.rope.alpha = alpha;
            }
        }
    }

    liveCount(): number {
        return this.pool.length - this.free.length;
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

class MeshEmitter extends Emitter {
    private readonly geometry: PIXI.Geometry;
    private readonly meshTexture: PIXI.Texture;

    constructor(data: IParticleSystemData, texture: PIXI.Texture, blend: "additive" | "normal", getBudget: () => number) {
        super(data, texture, null, blend, getBudget);
        this.meshTexture = texture;
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
            uBoost: this.blend === "additive" ? ADDITIVE_MESH_BOOST : 1,
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
        m.rotation = -p.rot * DEG;
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
uniform vec4 uDisturbST;
uniform vec4 uRamST;
uniform vec2 uMainScroll;
uniform vec2 uDissolveScroll;
uniform vec2 uDisturbScroll;
varying vec2 vMainUV;
varying vec2 vDissolveUV;
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
    vDisturbUV = aUV * uDisturbST.xy + uDisturbST.zw + uDisturbScroll;
    vRamUV = aUV * uRamST.xy + uRamST.zw;
    vMainUV.y = 1.0 - vMainUV.y;
    vDissolveUV.y = 1.0 - vDissolveUV.y;
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
varying vec2 vDisturbUV;
varying vec2 vRamUV;
varying vec4 vColor;
varying vec2 vCustom;
uniform sampler2D uMainTex;
uniform sampler2D uRamTex;
uniform sampler2D uDisturbTex;
uniform sampler2D uDissolveTex;
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
    private emitAcc = 0;
    private firedBursts = new Set<number>();
    private readonly rate: number;

    private readonly blend: "additive" | "normal";
    private readonly volWorld: { x: number; y: number } | null;
    private readonly boneAnchor: IBoneAnchor = { resolved: false, boneName: null, ref: null };
    private readonly mesh: PIXI.Mesh<PIXI.Shader>;
    private readonly shader: PIXI.Shader;
    private readonly posData: Float32Array;
    private readonly uvData: Float32Array;
    private readonly colData: Float32Array;
    private readonly customData: Float32Array;
    private readonly posBuf: PIXI.Buffer;
    private readonly colBuf: PIXI.Buffer;
    private readonly customBuf: PIXI.Buffer;

    constructor(
        data: IParticleSystemData,
        ram: IRamData,
        tex: { main: PIXI.Texture | null; ram: PIXI.Texture | null; disturb: PIXI.Texture | null; dissolve: PIXI.Texture | null },
        blend: "additive" | "normal",
        private readonly getBudget: () => number,
    ) {
        this.blend = blend;
        this.data = data;
        // Start dormant through the cinematic delay (see the billboard system's ctor).
        this.time = -(data.delay ?? 0);
        this.ram = ram;
        this.volWorld = worldVelocityOverLife(data);
        this.rate = data.emission?.rate ? sampleScalar(data.emission.rate, 0.5, 0) : 0;
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
            // Static per-corner base UV (0..1), transformed per-slot in the shader.
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
        const geometry = new PIXI.Geometry();
        geometry.addAttribute("aVertexPosition", this.posBuf, 2);
        geometry.addAttribute("aUV", new PIXI.Buffer(this.uvData as unknown as BufArg), 2);
        geometry.addAttribute("aColor", this.colBuf, 4);
        geometry.addAttribute("aCustom", this.customBuf, 2);
        geometry.addIndex(new PIXI.Buffer(idx as unknown as BufArg));

        const mainT = tex.main ?? WHITE_TEX;
        this.shader = PIXI.Shader.from(RAM_VERT, RAM_FRAG, {
            uMainTex: mainT,
            uRamTex: tex.ram ?? WHITE_TEX,
            uDisturbTex: tex.disturb ?? WHITE_TEX,
            uDissolveTex: tex.dissolve ?? WHITE_TEX,
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
            uHasRam: tex.ram ? 1 : 0,
            uMainST: ram.mainST,
            uDissolveST: ram.dissolveST,
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
            switch (shape.type) {
                case "circle": {
                    const rr = (shape.radius ?? 0) * Math.sqrt(Math.random());
                    ox = Math.cos(a) * rr;
                    oy = Math.sin(a) * rr;
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
                    const spread = (shape.angleDeg ?? 0) * DEG;
                    dirAng = 90 * DEG + (Math.random() - 0.5) * 2 * spread;
                    const rr = (shape.radius ?? 0) * Math.sqrt(Math.random());
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
        const wDir = dirAng + rotDeg * DEG;
        this.particles.push({
            x: wx,
            y: wy,
            vx: Math.cos(wDir) * speed,
            vy: Math.sin(wDir) * speed,
            age: 0,
            life: Math.max(MIN_PARTICLE_LIFE, sampleScalar(d.lifetime, Math.random(), nt)),
            size: sampleScalar(d.startSize, Math.random(), nt),
            rot: sampleScalar(d.startRotation ?? { mode: "const", v: 0 }, Math.random(), nt),
            rotVel: d.rotOverLifeDegPerSec ?? 0,
            rand: Math.random(),
            col: sampleColor(d.startColor, nt),
        });
    }

    update(dt: number, findBone?: FindBone, restBone?: RestBone): void {
        const d = this.data;
        this.time += dt;
        // Dormant until the cinematic start delay elapses (no emission, no particles yet).
        if (this.time < 0) return;
        driftWithBone(this.container, d.boneChain, d.pos, findBone, this.boneAnchor, restBone);

        const playing = d.looping || this.time <= d.duration;
        if (playing) {
            this.emitAcc += this.rate * dt;
            while (this.emitAcc >= 1) {
                this.emitAcc -= 1;
                this.spawn();
            }
            const cycleT = d.duration > 0 ? this.time % d.duration : this.time;
            for (const burst of d.emission?.bursts ?? []) {
                const key = Math.floor(this.time / (d.duration || 1)) * 1000 + burst.t;
                if (cycleT >= burst.t && !this.firedBursts.has(key)) {
                    this.firedBursts.add(key);
                    for (let i = 0; i < Math.min(burst.count, this.cap); i++) this.spawn();
                }
            }
            if (d.duration > 0 && cycleT < 0.05) this.firedBursts.clear();
        }

        const grav = d.gravity ? sampleScalar(d.gravity, 0.5, 0) * 9.81 : 0;
        const noise = d.noise;
        const vol = this.volWorld;

        // Step + cull, compacting the array in place.
        let w = 0;
        for (let r = 0; r < this.particles.length; r++) {
            const p = this.particles[r];
            p.age += dt;
            if (p.age >= p.life) continue; // drop
            p.vy -= grav * dt;
            p.x += (p.vx + (vol?.x ?? 0)) * dt;
            p.y += (p.vy + (vol?.y ?? 0)) * dt;
            if (noise) {
                const f = noise.frequency * 0.002;
                const ph = this.time * noise.scrollSpeed;
                p.x += fbmNoise(p.x * f + ph, p.y * f + p.rand * 17) * noise.strength * dt;
                p.y += fbmNoise(p.y * f + ph + 3.1, p.x * f + p.rand * 17 + 9.3) * noise.strength * dt;
            }
            p.rot += p.rotVel * dt;
            this.particles[w++] = p;
        }
        this.particles.length = w;

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
        const n = Math.min(this.particles.length, this.cap);
        for (let q = 0; q < n; q++) {
            const p = this.particles[q];
            const lf = p.age / p.life;
            const sz = p.size * (d.sizeOverLife ? sampleCurve(d.sizeOverLife, lf) : 1);
            const lifeCol = d.colorOverLife ? sampleColor(d.colorOverLife, lf) : { r: 1, g: 1, b: 1, a: 1 };
            const cr = p.col.r * lifeCol.r;
            const cg = p.col.g * lifeCol.g;
            const cb = p.col.b * lifeCol.b;
            const ca = Math.max(0, Math.min(1, p.col.a * lifeCol.a));
            const h = sz / 2;
            const th = -p.rot * DEG;
            const c = Math.cos(th);
            const s = Math.sin(th);
            const cx = p.x;
            const cy = -p.y;
            const vp = q * 8;
            // 4 corners: TL(-h,-h) TR(h,-h) BR(h,h) BL(-h,h)
            const cxs = [-h, h, h, -h];
            const cys = [-h, -h, h, h];
            for (let k = 0; k < 4; k++) {
                const lxk = cxs[k];
                const lyk = cys[k];
                pos[vp + k * 2] = cx + lxk * c - lyk * s;
                pos[vp + k * 2 + 1] = cy + lxk * s + lyk * c;
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
            // intensity). Every svash2 Ram system that shapes itself via a dissolve
            // mask has its CustomData module DISABLED, so the amount is a static 0
            // and the mask (ring / star-glint / etc.) shapes the fill directly.
            for (let k = 0; k < 4; k++) {
                cst[vk + k * 2] = 0; // dissolve amount
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
    }

    liveCount(): number {
        return this.particles.length;
    }

    destroy(): void {
        this.container.destroy({ children: true });
        this.shader.destroy();
    }
}

export interface ILoadedParticles {
    data: IParticlesData;
    /** Emitters whose sort is behind the character. */
    background: PIXI.Container;
    /** Emitters whose sort is in front of the character. */
    foreground: PIXI.Container;
    /** `findBone` (pixi-spine `skeleton.findBone`) lets bone-parented emitters
     *  drift with the character; omit for no bone-following. */
    update(dt: number, findBone?: FindBone, restBone?: RestBone): void;
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
        const brightCore = maxLum > 200 && maxLum > meanLum * 4;
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
    if (scalarMax(sys.startSpeed) > STATIONARY_SPEED) return 1; // moving → spreads, no pile
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

export async function loadParticles(url: string, textureBaseUrl: string, bust = ""): Promise<ILoadedParticles | null> {
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
    let liveEstimate = 0;
    const budget = () => GLOBAL_MAX_PARTICLES - liveEstimate;

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
     *  preserving a bright fill (SilverAsh's dissolve-shaped reticle rings). */
    const ramMainTex = (i: number | null): PIXI.Texture | null => {
        const b = i != null ? bases[i] : null;
        return b ? new PIXI.Texture(b.darkDropBase) : null;
    };

    for (const sys of data.systems) {
        // Ram-shader emitters render via the ported ramp/dissolve/disturb shader
        // (RamEmitter), using RAW textures. Needs a main tex. A Ram system carrying
        // a texture-sheet flipbook (e.g. SilverAsh's purple sword flame) falls
        // through to the sprite path, which slices the atlas into animated frames —
        // the Ram mesh shader has no per-frame UV, so a whole-atlas sample would
        // stamp the grid. Mesh-render-mode Ram systems are skipped (their real mesh
        // geometry isn't exported; billboarding a large one stamps a raw quad).
        if (sys.ram && !(sys.sheet && sys.sheet.tilesX * sys.sheet.tilesY > 1)) {
            if (sys.renderMode === "mesh") continue;
            const main = ramMainTex(sys.ram.mainTex);
            if (!main) continue;
            const emitter = new RamEmitter(sys, sys.ram, { main, ram: rawTex(sys.ram.ramTex), disturb: rawTex(sys.ram.disturbTex), dissolve: rawTex(sys.ram.dissolveTex) }, sys.blend, budget);
            emitters.push(emitter);
            (sys.sort < data.characterSort ? background : foreground).addChild(emitter.container);
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
        const isBackdropParticle = effBlend === "normal" && (bgNamed || (tex.hazePanel && scalarMax(sys.startSize) > 400));
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
            const meshOK = meshBlend === "additive" || !tex.desatPanel;
            if (sys.mesh && sys.mesh.idx.length >= 3 && meshOK) {
                const emitter = new MeshEmitter(sys, new PIXI.Texture(tex.base), meshBlend, budget);
                emitters.push(emitter);
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
        if (blend === "additive") emitter.container.alpha = additivePileGain(sys);
        // bg_* / haze atmospherics are demoted behind the spine (see isBackdropParticle above).
        (sys.sort < data.characterSort || isBackdropParticle ? background : foreground).addChild(emitter.container);
    }
    if (emitters.length === 0) return null;

    return {
        data,
        background,
        foreground,
        update(dt: number, findBone?: FindBone, restBone?: RestBone) {
            // Recompute the shared live count once per frame for the budget.
            liveEstimate = 0;
            for (const e of emitters) liveEstimate += e.liveCount();
            for (const e of emitters) e.update(dt, findBone, restBone);
        },
        destroy() {
            for (const e of emitters) e.destroy();
        },
    };
}
