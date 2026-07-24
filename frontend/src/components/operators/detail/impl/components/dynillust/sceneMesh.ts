import * as PIXI from "pixi.js";

/**
 * Live renderer for a dynamic illustration's BACKGROUND mesh layers.
 *
 * Most L2D skins bake their whole scene into the spine, but some author the
 * painted backdrop (sky/water/tree/…) as separate Unity mesh layers that live
 * alongside — not inside — the character spine. Those are exported by the
 * unpacker to a `…[scene].json` (+ `…[scene]/<n>.png` textures). This module
 * rebuilds the mesh layers in Pixi and composites them behind (and, for the few
 * foreground layers, in front of) the spine, so the operator animates over its
 * real background instead of a void.
 *
 * Layer coordinates are authored in "spine-authored pixels, Y-up, origin at the
 * skeleton root" — the SAME space as the spine — so the meshes and the spine
 * align directly once both are framed to the authored camera (`cameraSizePx`).
 */

export interface ISceneLayer {
    /** Index into the scene's texture set. */
    tex: number;
    /** Flat [x0,y0,x1,y1,…] vertex positions (spine-authored pixels, Y-up). */
    pos: number[];
    /** Flat [u0,v0,u1,v1,…] UVs (Unity V, i.e. 0 at top). */
    uv: number[];
    /** Triangle indices. */
    idx: number[];
    /** Per-layer RGBA multiply, each channel 0..1. */
    tint: [number, number, number, number];
    /** Optional flat [r,g,b,a,…] per-vertex colours (straight, 0..1). Unity fades
     *  these scene quads at their edges via vertex alpha (soft light sheets, e.g.
     *  Blaze the Wildfire's white light gradients, Nearl "Evolved Art"'s spirit
     *  sheets); omitted when every vertex is opaque white. */
    col?: number[];
    /** Additive blend (else normal alpha). */
    additive: boolean;
    /** Draw order; layers below `characterSort` render behind the spine. */
    sort: number;
    /** ENTRANCE reveal time (s): the layer is HIDDEN until the entrance clip's track time
     *  reaches this (`m_IsActive` toggle-on in the `_Start` cinematic). Absent = always
     *  visible. Only `_Start` scenes carry it. */
    activeFrom?: number | null;
    /** ENTRANCE hide time (s): the layer is HIDDEN once the entrance clip's track time
     *  passes this (`m_IsActive` toggle-off — the cinematic's environment swap). Absent =
     *  never hidden. Only `_Start` scenes carry it. */
    activeUntil?: number | null;
    /** CROSS-ROOT reveal (s): the layer is the IDLE prefab's world inside an entrance
     *  scene — the game only activates that prefab at the director's transform beat
     *  (Virtuosa's white mirror-world appears at 12.0, after the blue-sea reveal).
     *  Visibility-only: unlike `activeFrom` it does NOT mark the layer as a cinematic
     *  overlay, so the backdrop/veil demotions still apply. */
    rootRevealFrom?: number | null;
    /** ENTRANCE material-colour animation: `[t, r, g, b, a]` samples of the `_Start`
     *  clip's animated material colour, resolved onto the static tint by the exporter
     *  (e.g. Mlynar's white flash `_TintColor` alpha ramping 0→0.671 over 13→15s).
     *  Replayed each frame at the entrance track time, REPLACING `tint`. Absent = the
     *  material colour is static. Only `_Start` scenes carry it. */
    colorCurve?: [number, number, number, number, number][] | null;
    /** SHADER UV-SCROLL (Capability A): per-second UV velocity `[u, v]` (Unity UV space) for
     *  a Ram-family scene layer. The frontend offsets the layer's UVs by `t · [u, v]` each
     *  frame (continuous scene clock), reproducing the shader's `_Time`-driven scroll. Absent
     *  = static layer. */
    uvScroll?: [number, number] | null;
    /** CLIP `_MainTex_ST` curve (Capability B): absolute `[t, sx, sy, ox, oy]` texture
     *  Scale/Offset samples animated by the `_Start` entrance clip. The frontend replays it at
     *  the entrance track time as `uv = meshUV·[sx,sy] + [ox,oy]` (Unity UV space). When present
     *  the exporter emits the RAW (un-ST-baked) mesh UVs. Absent = static ST. */
    stCurve?: [number, number, number, number, number][] | null;
}

export interface ISceneData {
    aspect: number;
    /** Camera half-height in spine-authored pixels; the authored viewport. */
    cameraSizePx: number;
    /** Authored display-frame centre in spine-authored px (`_adjustes[0].offset`). */
    cameraOffsetPx?: [number, number] | null;
    /** Authored display-frame square full extent in spine-authored px (`_adjustes[0].size`). */
    cameraViewPx?: number | null;
    /** TIGHT/zoomed-in display-frame centre in spine-authored px (`_adjustes[1].offset`),
     *  the close-up the in-game viewer dollies out FROM at open. Absent on ordinary skins. */
    cameraOffsetPx2?: [number, number] | null;
    /** TIGHT/zoomed-in display-frame square full extent in spine-authored px (`_adjustes[1].size`). */
    cameraViewPx2?: number | null;
    /** Sort index at which the character spine is inserted. */
    characterSort: number;
    /** ENTRANCE (`_Start`) cinematic total length in seconds (director `_params.duration`).
     *  Present only on `_Start` scenes. */
    entranceDuration?: number | null;
    /** ENTRANCE transform/reform beat in seconds (the dominant late `_delayTime` cluster) —
     *  the camera has dollied to the wide stop and the character has reformed by here, so it's
     *  the entrance→idle hand-off point AND the target time of the tight→wide camera dolly. */
    entranceTransform?: number | null;
    /** ENTRANCE close-up view in authored px (`2·ortho/skeletonScale`) — the tight frame the
     *  cinematic opens on (Virtuosa 598 vs the 1929 wide stop). `_Start` scenes only. */
    entranceViewPx?: number | null;
    /** ENTRANCE camera aim point relative to the skeleton root, authored px `[dx, dy]` (Y-up).
     *  Exported but currently unread — it does not map into the render/vis space the spine is
     *  measured in (the gamedata root and the spine's vis origin don't coincide), so framing
     *  centres on the character's own measured body instead. */
    entranceCamOffsetPx?: [number, number] | null;
    /** ENTRANCE camera dolly ZOOM — the game's actual data-driven camera motion, extracted from
     *  the `_Start` clip that animates the Main Camera's orthographic size: `[t_seconds, ortho]`
     *  keyframes (Virtuosa 1.87 hold → 1.50 zoom-in on the transform → 1.91 out). There is NO
     *  positional pan. The frontend replays it as a RELATIVE zoom (ratio to the t=0 value) on the
     *  entrance frame, so no world↔authored unit conversion is needed. `_Start` scenes only. */
    entranceOrthoCurve?: [number, number][] | null;
    /** ENTRANCE camera POSITIONAL dolly (pan): `[t_s, progress 0..1]` keyframes, extracted from
     *  the animated camera-ancestor Transform position in the `_Start` clip. Exported but currently
     *  unread — superseded by {@link entranceCamCenterCurve}. `_Start` only. */
    entrancePanCurve?: [number, number][] | null;
    /** ENTRANCE camera FRAME-CENTRE trajectory `[t_s, cxPx, cyPx]` (authored px, mesh space) —
     *  accumulated from the FULL camera rig (positions/rotations/scales) in the extractor. PURE
     *  gamedata: the entrance frame is centred here each frame, no measurement/inference. `_Start`. */
    entranceCamCenterCurve?: [number, number, number][] | null;
    /** ENTRANCE voice-line offset (s), `_params.charVoiceOffset` — when the reformed character
     *  starts talking. Exported but currently unread (the hand-off fires from the `_Start`
     *  clip's own `complete`). */
    entranceVoiceOffset?: number | null;
    skeletonScale?: number;
    textureCount: number;
    layers: ISceneLayer[];
}

export interface ILoadedScene {
    data: ISceneData;
    /** Layers behind the character (sort < characterSort). */
    background: PIXI.Container;
    /** Layers in front of the character (sort > characterSort). */
    foreground: PIXI.Container;
    /** True when the scene owns an opaque painted backdrop that is DARKER than the studio
     *  environment gradient (see {@link STUDIO_ENV_WHITENESS}) — a self-lit painted world
     *  (Virtuosa's deep-blue mirror-world) rather than a float-over-studio illustration. Such
     *  a scene supplies its own environment, so the caller must NOT composite the light-grey
     *  studio gradient behind it: the gradient bleeds through the frame's un-covered edges
     *  (the painted backdrop doesn't reach the top after the camera plunge) and washes the
     *  deep colour to a flat grey — the dominant cause of Virtuosa's over-bright backdrop. A
     *  BRIGHT opaque backdrop (Wišʼadel's white studio wall, whiteness ≥ the gradient) is NOT
     *  flagged: the gradient can't wash a surface as bright as itself, and dropping it there
     *  would only expose black voids where the wall doesn't reach. */
    hasDarkBackdrop: boolean;
}

interface ISceneTex {
    raw: PIXI.BaseTexture;
    /** For ADDITIVE layers: an opaque glow/caustic texture (no alpha channel,
     *  dark field authored to add nothing) rebuilt with alpha = luminance so its
     *  grey field drops out instead of stamping a grey rectangle. Equals `raw`
     *  when the texture already has real transparency. */
    glow: PIXI.BaseTexture;
    /** Alpha-weighted "whiteness" in [0,1]: bright AND desaturated → ~1 (a white
     *  paint/flash veil), coloured (e.g. red halftones) → ~0. Used to attenuate
     *  face-veiling foreground overlays without touching coloured foreground fx. */
    whiteness: number;
    /** Fraction of sampled texels that are near-fully-opaque (alpha > 0.9). ~1 means
     *  a solid painted BACKDROP (a wall/room), which must sit behind everything. */
    opaqueFrac: number;
    /** Alpha-weighted mean saturation of the texture in [0,1]. */
    sat: number;
}

/** Rebuild an opaque additive texture with alpha = luminance (black-point) so its
 *  dark/grey field premultiplies to ~nothing; returns null for real-alpha sprites
 *  (nothing to drop) or on any canvas failure. */
function darkDropGlow(img: HTMLImageElement): PIXI.BaseTexture | null {
    try {
        const w = img.naturalWidth || img.width;
        const h = img.naturalHeight || img.height;
        if (!w || !h) return null;
        const canvas = document.createElement("canvas");
        canvas.width = w;
        canvas.height = h;
        const ctx = canvas.getContext("2d", { willReadFrequently: true });
        if (!ctx) return null;
        ctx.drawImage(img, 0, 0);
        const imgData = ctx.getImageData(0, 0, w, h);
        const px = imgData.data;
        // Only opaque (no usable alpha) textures need this — a soft sprite keeps
        // its own alpha.
        let opaque = 0;
        let n = 0;
        for (let i = 3; i < px.length; i += 64) {
            if (px[i] > 230) opaque++;
            n++;
        }
        if (n === 0 || opaque / n < 0.85) return null;
        const BLACK_POINT = 32;
        for (let i = 0; i < px.length; i += 4) {
            const lum = Math.max(px[i], px[i + 1], px[i + 2]);
            px[i + 3] = lum <= BLACK_POINT ? 0 : lum;
        }
        ctx.putImageData(imgData, 0, 0);
        return PIXI.BaseTexture.from(canvas);
    } catch {
        return null;
    }
}

/** Sample a texture once: alpha-weighted `whiteness` in [0,1] (`brightness ×
 *  (1 - saturation)` — a bright desaturated veil → ~1, a saturated red halftone →
 *  ~0) and `opaqueFrac` (fraction of texels with alpha > 0.9 → ~1 for a solid
 *  painted backdrop). Both default to 0 on any canvas failure. */
function analyzeTexture(img: HTMLImageElement): { whiteness: number; opaqueFrac: number; sat: number } {
    try {
        const w = img.naturalWidth || img.width;
        const h = img.naturalHeight || img.height;
        if (!w || !h) return { whiteness: 0, opaqueFrac: 0, sat: 0 };
        const canvas = document.createElement("canvas");
        canvas.width = w;
        canvas.height = h;
        const ctx = canvas.getContext("2d", { willReadFrequently: true });
        if (!ctx) return { whiteness: 0, opaqueFrac: 0, sat: 0 };
        ctx.drawImage(img, 0, 0);
        const px = ctx.getImageData(0, 0, w, h).data;
        let wsum = 0;
        let satSum = 0;
        let aw = 0;
        let opaque = 0;
        let n = 0;
        for (let i = 0; i < px.length; i += 64) {
            n++;
            const a = px[i + 3] / 255;
            if (a > 0.9) opaque++;
            if (a <= 0.02) continue;
            const r = px[i] / 255;
            const g = px[i + 1] / 255;
            const b = px[i + 2] / 255;
            const mx = Math.max(r, g, b);
            const mn = Math.min(r, g, b);
            const sat = mx > 0 ? (mx - mn) / mx : 0;
            wsum += a * (mx * (1 - sat));
            satSum += a * sat;
            aw += a;
        }
        return { whiteness: aw > 0 ? wsum / aw : 0, opaqueFrac: n > 0 ? opaque / n : 0, sat: aw > 0 ? satSum / aw : 0 };
    } catch {
        return { whiteness: 0, opaqueFrac: 0, sat: 0 };
    }
}

function loadTexture(url: string): Promise<ISceneTex> {
    return new Promise((resolve, reject) => {
        const img = new Image();
        img.crossOrigin = "anonymous";
        img.onload = () => {
            const raw = PIXI.BaseTexture.from(img);
            // Scene layers bake the material's `_MainTex` Scale/Offset into their UVs
            // (see the exporter's collect_dynchar_bg_quads). Many layers tile or mirror
            // (negative scale) and so reference UVs OUTSIDE [0,1] — REPEAT wrap makes those
            // sample correctly instead of edge-smearing under Pixi's default CLAMP.
            raw.wrapMode = PIXI.WRAP_MODES.REPEAT;
            const glow = darkDropGlow(img);
            if (glow) glow.wrapMode = PIXI.WRAP_MODES.REPEAT;
            const { whiteness, opaqueFrac, sat } = analyzeTexture(img);
            resolve({ raw, glow: glow ?? raw, whiteness, opaqueFrac, sat });
        };
        img.onerror = () => reject(new Error(`Failed to load scene texture: ${url}`));
        img.src = url;
    });
}

/** Attenuation for caustic / light-dome OVERLAY layers (a small texture stretched
 *  over a large mesh) — approximates the engine's absent HDR tonemap so they read
 *  as a faint ripple, not a bold white swirl over the scene. */
const EFFECT_SCENE_GAIN = 0.3;
/** Max texture size treated as an effect overlay; painted backdrops are large atlases. */
const EFFECT_TEX_MAX = 512;
/** A foreground normal-blend effect panel whose whiteness exceeds this is a bright
 *  paint/flash VEIL (not a coloured comic-fx). The source art keeps that white burst
 *  BEHIND the character, so we re-sort these behind the spine — her opaque body then
 *  occludes the veil over her face while the burst still shows around her. Coloured
 *  foreground fx (red halftones/slashes, whiteness≈0.2-0.5) stay in front. */
const VEIL_WHITENESS_MIN = 0.65;
/** Min alpha-weighted saturation for a foreground effect to count as glowing energy
 *  (→ additive over the character). White veils (~0) and pale panels fall below it. */
const GLOW_SAT_MIN = 0.4;
/** Half-width (spine-authored px) of the character's central column. A saturated
 *  foreground glow centred within it is energy over the character (additive); one
 *  outside it is side comic-fx (left normal). Spine root is at x=0. */
const CHAR_COLUMN_HALF = 300;

/** A small, VERY bright, VERY desaturated, NON-solid scene sprite is a Unity additive
 *  LIGHT-GLOW sheet (Virtuosa's mirror-world white facet glows: tex 64–128px, whiteness
 *  0.73–0.95, sat ≤0.11, opaqueFrac ≤0.6). Exported normal-blend, a stack of them
 *  MULTIPLIES the bright painted backdrop down to a dim blue (game keeps it high-key
 *  white). Rendered ADDITIVE they ADD light instead — restoring the game's bright
 *  mirror-world. The tight gate excludes: solid white walls (Wišʼadel's studio wall,
 *  opaqueFrac ≥0.9), coloured comic-fx (saturated), the mid-white haze VEILS that must
 *  stay demoted (whiteness ≤~0.6, below the 0.7 floor), and the 1024px painted
 *  backdrops (not effect-sized). Property-driven — no per-skin constant. */
const LIGHT_GLOW_WHITENESS = 0.7;
const LIGHT_GLOW_SAT_MAX = 0.15;
const LIGHT_GLOW_OPAQUE_MAX = 0.6;

/** Whiteness threshold separating a self-lit DARK painted backdrop from a bright studio
 *  wall. Derived from the studio environment gradient (see `createEnvironmentBgTexture`
 *  in SceneIllust): its darker stop is `#bfc0c4` → luminance ≈ 0xbf/0xff ≈ 0.749. An opaque
 *  backdrop whose whiteness falls BELOW this is darker than the studio gradient itself, so
 *  the gradient composited behind it would only lift/wash it (a bug); one at or above it is
 *  a bright environment the gradient can't wash. Property-derived, not per-skin. */
const STUDIO_ENV_WHITENESS = 0.749;

function tintToHex(t: [number, number, number, number]): number {
    const c = (v: number) => Math.max(0, Math.min(255, Math.round(v * 255)));
    return (c(t[0]) << 16) | (c(t[1]) << 8) | c(t[2]);
}

/** Entrance-replay state stashed on a layer mesh by {@link buildLayerMesh} and read by
 *  the entrance tick each frame: the `m_IsActive` visibility window plus the animated
 *  material-colour curve (and how to fold a sampled colour back onto the mesh). */
export interface ISceneLayerRuntime {
    __activeFrom?: number | null;
    __activeUntil?: number | null;
    /** The layer's authored Unity `m_SortingOrder` — used to hoist scene layers that
     *  outsort EVERY particle emitter above the particle container (Mlynar's sort-100
     *  white flash must cover the crystals/sparks too). */
    __sort?: number;
    __colorCurve?: [number, number, number, number, number][] | null;
    /** Mirror of the static-tint folding in {@link buildLayerMesh} (additive gain rules),
     *  so {@link applySceneLayerColor} reproduces it for every sampled colour. */
    __colorMode?: { additive: boolean; gain: number };
    /** Capability A: per-second UV velocity `[u, v]` (Unity UV space); the entrance/idle tick
     *  re-scrolls this layer's UVs each frame. Absent for static layers. */
    __uvScroll?: [number, number] | null;
    /** Capability A: a copy of the layer's un-scrolled `aUV` buffer data (the baked base UVs),
     *  so each frame's scroll is `base + t · [u, -v]` rather than accumulating drift. */
    __uvBase?: Float32Array | null;
    /** Capability B: absolute `[t, sx, sy, ox, oy]` ST curve replayed at the entrance track
     *  time; composes with `__uvScroll` in {@link applySceneLayerSt}. Absent for static layers. */
    __stCurve?: [number, number, number, number, number][] | null;
    /** Capability B: a copy of the layer's RAW mesh UVs in UNITY space (un-flipped), so the ST
     *  curve is applied in Unity space and V is flipped once at write time. */
    __stBaseUnity?: Float32Array | null;
}

/** Linear-sample a `[t, r, g, b, a]` colour curve at time `t`, clamped to its endpoints. */
export function sampleColorCurve(curve: [number, number, number, number, number][], t: number): [number, number, number, number] {
    const first = curve[0];
    if (t <= first[0]) return [first[1], first[2], first[3], first[4]];
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i][0]) {
            const [t0, r0, g0, b0, a0] = curve[i - 1];
            const [t1, r1, g1, b1, a1] = curve[i];
            const f = t1 > t0 ? (t - t0) / (t1 - t0) : 0;
            return [r0 + (r1 - r0) * f, g0 + (g1 - g0) * f, b0 + (b1 - b0) * f, a0 + (a1 - a0) * f];
        }
    }
    const last = curve[curve.length - 1];
    return [last[1], last[2], last[3], last[4]];
}

function uvBufferOf(mesh: PIXI.DisplayObject): PIXI.Buffer {
    const geo = (mesh as unknown as PIXI.Mesh).geometry;
    // PIXI.MeshGeometry names the UV attribute 'aTextureCoord'; the custom vcolor
    // geometry (buildVColorMesh) uses 'aUV'. getBuffer() throws on a missing attribute,
    // so pick whichever this geometry actually declares.
    return geo.getBuffer(geo.getAttribute("aUV") ? "aUV" : "aTextureCoord");
}

/** Re-tint a built scene-layer mesh from a sampled entrance colour — the runtime
 *  counterpart of the static tint set in {@link buildLayerMesh} (same additive/gain
 *  folding), covering both the MeshMaterial path (tint/alpha) and the per-vertex-colour
 *  shader (premultiplied `uColor` uniform). No-op for meshes built without a curve. */
export function applySceneLayerColor(mesh: PIXI.DisplayObject, rgba: [number, number, number, number]): void {
    const mm = mesh as unknown as ISceneLayerRuntime & { shader?: PIXI.Shader };
    const mode = mm.__colorMode;
    const shader = mm.shader;
    if (!mode || !shader) return;
    const g = mode.gain;
    const rgb: [number, number, number, number] = mode.additive ? [rgba[0] * g, rgba[1] * g, rgba[2] * g, rgba[3]] : rgba;
    const alpha = mode.additive ? rgba[3] : rgba[3] * g;
    if (shader instanceof PIXI.MeshMaterial) {
        shader.tint = tintToHex(rgb);
        // Drive the DISPLAY-OBJECT alpha, not MeshMaterial.alpha: PIXI batches small
        // meshes (a 4-vertex quad like Mlynar's white-flash plane), and the batch
        // path reads `worldAlpha` while silently ignoring the material's alpha — the
        // replayed fade was a no-op and the plane rendered at FULL opacity from its
        // first frame. `worldAlpha` is honoured by BOTH the batch and default paths
        // (the default path copies it into the material each render).
        (mesh as unknown as { alpha: number }).alpha = alpha;
    } else {
        shader.uniforms.uColor = [rgb[0] * alpha, rgb[1] * alpha, rgb[2] * alpha, alpha];
    }
}

/** Re-scroll a scene layer's UVs for Capability A: rewrite the `aUV` buffer as
 *  `base + [u, -v] · t`. V is NEGATED because {@link buildLayerMesh} stores `1 - v`,
 *  inverting Unity's V axis (Unity scrolls `_MainVSpeed` in the un-flipped space). Cheap CPU
 *  rewrite (scene quads are a handful of verts); no-op for meshes built without a scroll. */
export function applySceneLayerUvScroll(mesh: PIXI.DisplayObject, t: number): void {
    const rt = mesh as unknown as ISceneLayerRuntime;
    const scroll = rt.__uvScroll;
    const base = rt.__uvBase;
    if (!scroll || !base) return;
    const buf = uvBufferOf(mesh);
    const d = buf.data as unknown as Float32Array;
    const du = scroll[0] * t;
    const dv = scroll[1] * t;
    for (let i = 0; i < base.length; i += 2) {
        d[i] = base[i] + du;
        d[i + 1] = base[i + 1] - dv;
    }
    buf.update();
}

/** Linear-sample an ST curve `[t, sx, sy, ox, oy]` at time `t` (clamped to endpoints). */
function sampleStCurve(curve: [number, number, number, number, number][], t: number): [number, number, number, number] {
    const first = curve[0];
    if (t <= first[0]) return [first[1], first[2], first[3], first[4]];
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i][0]) {
            const [t0, sx0, sy0, ox0, oy0] = curve[i - 1];
            const [t1, sx1, sy1, ox1, oy1] = curve[i];
            const f = t1 > t0 ? (t - t0) / (t1 - t0) : 0;
            return [sx0 + (sx1 - sx0) * f, sy0 + (sy1 - sy0) * f, ox0 + (ox1 - ox0) * f, oy0 + (oy1 - oy0) * f];
        }
    }
    const last = curve[curve.length - 1];
    return [last[1], last[2], last[3], last[4]];
}

/** Capability B: replay a scene layer's animated `_MainTex_ST` at entrance track time `tt`,
 *  composing with the optional continuous UV scroll (`clock`). Works in UNITY UV space —
 *  `uv = base·[sx,sy] + [ox,oy] + [uSpeed,vSpeed]·clock` — then flips V once at write time
 *  (`1 - v`), matching {@link buildLayerMesh}'s stored convention and Unity's shader order
 *  (`meshUV·ST + _Time·mainSpeed`). No-op for meshes built without an ST curve. */
export function applySceneLayerSt(mesh: PIXI.DisplayObject, tt: number, clock: number): void {
    const rt = mesh as unknown as ISceneLayerRuntime;
    const curve = rt.__stCurve;
    const base = rt.__stBaseUnity;
    if (!curve || !base) return;
    const [sx, sy, ox, oy] = sampleStCurve(curve, tt);
    const su = rt.__uvScroll ? rt.__uvScroll[0] * clock : 0;
    const sv = rt.__uvScroll ? rt.__uvScroll[1] * clock : 0;
    const buf = uvBufferOf(mesh);
    const d = buf.data as unknown as Float32Array;
    for (let i = 0; i < base.length; i += 2) {
        const u = base[i] * sx + ox + su;
        const v = base[i + 1] * sy + oy + sv;
        d[i] = u;
        d[i + 1] = 1 - v;
    }
    buf.update();
}

// Per-vertex-colour mesh shader for layers that carry `col`. PIXI's built-in
// MeshMaterial only applies a uniform tint, so a light sheet whose SHAPE is
// authored as a vertex-alpha gradient (opaque core → transparent edges over a
// flat-white texture) renders as a hard opaque block (e.g. Blaze "Wildfire"'s
// white light wedge stamped a solid triangle). This program multiplies the
// (premultiplied) texture by the premultiplied per-layer tint AND the
// premultiplied per-vertex colour — a faithful extension of MeshMaterial, so a
// pure-white vertex colour reduces to the exact same result. Only used when
// `layer.col` is present; the common path keeps MeshMaterial untouched.
const VCOLOR_VERT = `
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
const VCOLOR_FRAG = `
precision highp float;
varying vec2 vUV;
varying vec4 vColor;
uniform sampler2D uSampler;
uniform vec4 uColor; // premultiplied tint*alpha
void main() {
    vec4 tex = texture2D(uSampler, vUV); // premultiplied
    vec4 vc = vec4(vColor.rgb * vColor.a, vColor.a); // premultiply the straight vertex colour
    gl_FragColor = tex * uColor * vc;
}
`;

/** Build a per-vertex-colour mesh (see {@link VCOLOR_FRAG}). */
function buildVColorMesh(layer: ISceneLayer, base: PIXI.BaseTexture, rgb: [number, number, number, number], alpha: number): PIXI.Mesh<PIXI.Shader> | null {
    const vertexCount = layer.pos.length / 2;
    const col = layer.col;
    if (!col || col.length < vertexCount * 4) return null;
    const vertices = new Float32Array(vertexCount * 2);
    const uvs = new Float32Array(vertexCount * 2);
    const colors = new Float32Array(vertexCount * 4);
    for (let i = 0; i < vertexCount; i++) {
        vertices[i * 2] = layer.pos[i * 2];
        vertices[i * 2 + 1] = -layer.pos[i * 2 + 1];
        uvs[i * 2] = layer.uv[i * 2];
        uvs[i * 2 + 1] = 1 - layer.uv[i * 2 + 1];
        colors[i * 4] = col[i * 4];
        colors[i * 4 + 1] = col[i * 4 + 1];
        colors[i * 4 + 2] = col[i * 4 + 2];
        colors[i * 4 + 3] = col[i * 4 + 3];
    }
    type BufArg = ConstructorParameters<typeof PIXI.Buffer>[0];
    const geometry = new PIXI.Geometry();
    geometry.addAttribute("aVertexPosition", new PIXI.Buffer(vertices as unknown as BufArg), 2);
    geometry.addAttribute("aUV", new PIXI.Buffer(uvs as unknown as BufArg), 2);
    geometry.addAttribute("aColor", new PIXI.Buffer(colors as unknown as BufArg), 4);
    geometry.addIndex(new PIXI.Buffer(new Uint16Array(layer.idx) as unknown as BufArg));
    // uColor: premultiplied per-layer tint*alpha (matches MeshMaterial's uniform).
    const uColor = [rgb[0] * alpha, rgb[1] * alpha, rgb[2] * alpha, alpha];
    const shader = PIXI.Shader.from(VCOLOR_VERT, VCOLOR_FRAG, { uSampler: new PIXI.Texture(base), uColor });
    return new PIXI.Mesh(geometry, shader);
}

/** Pixel footprint of just THIS layer's sampled sub-rect of its (possibly shared/atlas)
 *  texture. A layer packed into a large shared atlas page samples only a small UV sub-rect;
 *  the raw texture dimensions then mis-describe it as a big backdrop. Provably a no-op for a
 *  layer that samples its texture's full extent (uv spans [0,1] → returns the raw dims). */
function layerUvRectPx(layer: ISceneLayer, tex: ISceneTex): { w: number; h: number } {
    let uMin = Infinity;
    let uMax = -Infinity;
    let vMin = Infinity;
    let vMax = -Infinity;
    for (let i = 0; i < layer.uv.length; i += 2) {
        uMin = Math.min(uMin, layer.uv[i]);
        uMax = Math.max(uMax, layer.uv[i]);
        vMin = Math.min(vMin, layer.uv[i + 1]);
        vMax = Math.max(vMax, layer.uv[i + 1]);
    }
    return { w: (uMax - uMin) * tex.raw.width, h: (vMax - vMin) * tex.raw.height };
}

/**
 * Build a Pixi mesh for one layer. Y is flipped (authored Y-up → Pixi Y-down)
 * and V is flipped (Unity → Pixi UV), matching the exporter's coordinate note.
 */
function buildLayerMesh(layer: ISceneLayer, tex: ISceneTex, forceAdditive = false, fullGain = false, temperLargeAdditive = false): PIXI.Mesh | null {
    const vertexCount = layer.pos.length / 2;
    if (vertexCount < 3 || layer.idx.length < 3) return null;
    const additive = layer.additive || forceAdditive;
    // Dark-field-dropped texture (`tex.glow`: opaque texel → alpha=luminance, so a
    // black/dark field drops out) is ONLY valid for ADDITIVE blending, where black
    // adds nothing and the drop just spares the grey rectangle. For NORMAL-blend
    // surfaces a texel's dark colour is REAL paint, not "nothing": dark-dropping a
    // solid painted backdrop or a fully-opaque crystal sheet punches its dark regions
    // transparent, turning a continuous wall/crystal into floating bright-edge facets
    // over the void — the "shattered backdrop" bug (Virtuosa's grey vista + tex-9
    // crystal throne). Normal-blend layers must sample the raw texture. A normal-blend
    // caustic that genuinely needs its black field gone is promoted to ADD upstream
    // (`forceAdditive`), so it still reaches the glow path when it should.
    const base = additive ? tex.glow : tex.raw;
    // A SMALL texture stretched over a large mesh is a caustic / light-dome OVERLAY
    // (not a painted backdrop, which is a large atlas). In-engine those are subtle
    // HDR-tonemapped light; composited raw they blow out into bold swirls, so
    // attenuate. Additive is darkened via tint (reliable for ADD blend); normal via
    // alpha. Painted backdrops (large textures) are untouched.
    const effRect = layerUvRectPx(layer, tex);
    const isEffect = effRect.w <= EFFECT_TEX_MAX && effRect.h <= EFFECT_TEX_MAX;
    // `EFFECT_SCENE_GAIN` tames small caustic overlays (they'd otherwise over-cover);
    // but an additive LIGHT-GLOW sheet (fullGain) must contribute its full energy — the
    // HDR float target + tonemap handle the peaks — else it can't lift the backdrop.
    // A LARGE (non-effect) additive layer was previously always full-gain (1), uncapped
    // even on a scene that owns its own dark backdrop (`temperLargeAdditive`, threaded from
    // the scene-level `hasDarkBackdrop` flag below) — that's exactly the studio-gradient
    // fallback scenario `EFFECT_SCENE_GAIN` exists to tame in the first place (a large
    // additive sheet summing against the now-always-on grey fallback, see `loadSceneMeshes`
    // / `hasDarkBackdrop`). Fold it into the same tamed-gain bucket, still exempting
    // `fullGain` layers (light-glow sheets / animated colour curves) — a no-op for every
    // scene that doesn't own a dark backdrop (every skin except Virtuosa today).
    const gain = !fullGain && (isEffect || (additive && temperLargeAdditive)) ? EFFECT_SCENE_GAIN : 1;

    const vertices = new Float32Array(vertexCount * 2);
    const uvs = new Float32Array(vertexCount * 2);
    for (let i = 0; i < vertexCount; i++) {
        vertices[i * 2] = layer.pos[i * 2];
        vertices[i * 2 + 1] = -layer.pos[i * 2 + 1];
        uvs[i * 2] = layer.uv[i * 2];
        uvs[i * 2 + 1] = 1 - layer.uv[i * 2 + 1];
    }

    // PIXI's MeshGeometry expects its own IArrayBuffer; TS 5.7's generic typed
    // arrays don't structurally match, so cast to the constructor's param type.
    type GeomBuf = ConstructorParameters<typeof PIXI.MeshGeometry>[0];
    const rgb: [number, number, number, number] = additive ? [layer.tint[0] * gain, layer.tint[1] * gain, layer.tint[2] * gain, layer.tint[3]] : layer.tint;
    const alpha = additive ? layer.tint[3] : layer.tint[3] * gain;
    // A layer whose SHAPE is a vertex-alpha gradient (soft light sheet over a
    // flat texture) needs per-vertex colour, or it stamps a hard opaque block.
    const stashRuntime = (m: PIXI.DisplayObject) => {
        const rt = m as unknown as ISceneLayerRuntime;
        // Visibility window = the clip-authored reveal AND the cross-root prefab
        // activation (whichever is later). `isRevealOverlay` above deliberately keys
        // on the clip window alone — a root-revealed layer is scenery, not an overlay.
        const from = layer.activeFrom ?? null;
        const rootFrom = layer.rootRevealFrom ?? null;
        rt.__activeFrom = from != null || rootFrom != null ? Math.max(from ?? 0, rootFrom ?? 0) : null;
        rt.__activeUntil = layer.activeUntil ?? null;
        rt.__sort = layer.sort;
        if (layer.colorCurve?.length) {
            rt.__colorCurve = layer.colorCurve;
            rt.__colorMode = { additive, gain };
        }
        if (layer.uvScroll && (layer.uvScroll[0] !== 0 || layer.uvScroll[1] !== 0) && !layer.stCurve?.length) {
            rt.__uvScroll = layer.uvScroll;
            const buf = uvBufferOf(m);
            rt.__uvBase = Float32Array.from(buf.data as unknown as Float32Array);
        }
        if (layer.stCurve?.length) {
            rt.__stCurve = layer.stCurve;
            rt.__stBaseUnity = Float32Array.from(layer.uv);
            if (layer.uvScroll && (layer.uvScroll[0] !== 0 || layer.uvScroll[1] !== 0)) rt.__uvScroll = layer.uvScroll;
        }
    };
    if (layer.col && layer.col.length >= vertexCount * 4) {
        const vmesh = buildVColorMesh(layer, base, rgb, alpha);
        if (vmesh) {
            vmesh.blendMode = additive ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
            stashRuntime(vmesh);
            return vmesh as unknown as PIXI.Mesh;
        }
    }
    const geometry = new PIXI.MeshGeometry(vertices as unknown as GeomBuf, uvs as unknown as GeomBuf, new Uint16Array(layer.idx) as unknown as GeomBuf);
    // Static layer alpha lives on the DISPLAY OBJECT, not MeshMaterial.alpha — the
    // batch path small meshes take ignores the material's alpha (see
    // {@link applySceneLayerColor}); worldAlpha works on every path.
    const material = new PIXI.MeshMaterial(new PIXI.Texture(base), { tint: tintToHex(rgb) });
    const mesh = new PIXI.Mesh(geometry, material);
    mesh.alpha = alpha;
    mesh.blendMode = additive ? PIXI.BLEND_MODES.ADD : PIXI.BLEND_MODES.NORMAL;
    stashRuntime(mesh);
    return mesh;
}

/** The authored camera/display frame — where the static illustration maps in
 *  spine coordinates. Present even for skins with no mesh layers (a bare
 *  `[scene].json` carrying only the frame), so the static-art backdrop can be
 *  aligned for spine-only skins too. */
export interface ISceneFrame {
    /** Display-frame centre in spine-authored px (Y-up). */
    offsetPx: [number, number];
    /** Display-frame square full extent in spine-authored px. */
    viewPx: number;
    /** Camera half-height in spine-authored px — the static illustration spans
     *  `2 × cameraSizePx` of the scene, so this sets the backdrop's scale. */
    cameraSizePx: number;
    /** TIGHT/zoomed-in endpoint centre (`_adjustes[1].offset`), when the skin ships one. */
    offsetPx2?: [number, number] | null;
    /** TIGHT/zoomed-in endpoint square extent (`_adjustes[1].size`), when present. */
    viewPx2?: number | null;
    /** ENTRANCE close-up view extent in authored px (`2·ortho/skeletonScale`) — the tight
     *  frame the `_Start` cinematic opens on before dollying out. `_Start` scenes only. */
    entranceViewPx?: number | null;
}

/** Sample the entrance camera dolly at time `t` (seconds) and return the ZOOM RATIO relative to
 *  the curve's first keyframe — 1.0 at the open, <1 while zoomed in, >1 while zoomed out. Linear
 *  interpolation between keyframes; clamps to the endpoints. Returns 1 (no zoom) for an absent or
 *  degenerate curve. This is the game's data-driven camera motion (there is no positional pan). */
export function orthoZoomRatio(curve: [number, number][] | null | undefined, t: number): number {
    if (!curve || curve.length < 2) return 1;
    const base = curve[0][1];
    if (!(base > 0)) return 1;
    let size = curve[curve.length - 1][1];
    if (t <= curve[0][0]) {
        size = curve[0][1];
    } else {
        for (let i = 1; i < curve.length; i++) {
            if (t <= curve[i][0]) {
                const [t0, s0] = curve[i - 1];
                const [t1, s1] = curve[i];
                const f = t1 > t0 ? (t - t0) / (t1 - t0) : 0;
                size = s0 + (s1 - s0) * f;
                break;
            }
        }
    }
    return size / base;
}

/** Linear-sample a `[t, x, y]` keyframe curve at time `t`, clamped to the endpoints. Returns
 *  `[x, y]`, or `null` for an absent/empty curve. Used for the entrance camera frame-centre curve. */
export function sampleCurveXY(curve: [number, number, number][] | null | undefined, t: number): [number, number] | null {
    if (!curve || curve.length === 0) return null;
    if (t <= curve[0][0]) return [curve[0][1], curve[0][2]];
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i][0]) {
            const [t0, x0, y0] = curve[i - 1];
            const [t1, x1, y1] = curve[i];
            const f = t1 > t0 ? (t - t0) / (t1 - t0) : 0;
            return [x0 + (x1 - x0) * f, y0 + (y1 - y0) * f];
        }
    }
    const last = curve[curve.length - 1];
    return [last[1], last[2]];
}

/** Frame data taken directly from a loaded {@link ISceneData}. */
export function sceneFrameOf(data: ISceneData): ISceneFrame | null {
    if (!data.cameraOffsetPx || !data.cameraViewPx || !data.cameraSizePx) return null;
    return {
        offsetPx: data.cameraOffsetPx,
        viewPx: data.cameraViewPx,
        cameraSizePx: data.cameraSizePx,
        offsetPx2: data.cameraOffsetPx2 ?? null,
        viewPx2: data.cameraViewPx2 ?? null,
        entranceViewPx: data.entranceViewPx ?? null,
    };
}

/** Fetch just the authored camera frame from a scene JSON (no textures). Returns
 *  null when there's no scene JSON or it lacks the frame fields. */
export async function loadSceneFrame(sceneUrl: string): Promise<ISceneFrame | null> {
    try {
        const res = await fetch(sceneUrl);
        if (!res.ok) return null;
        const data = (await res.json()) as ISceneData;
        return sceneFrameOf(data);
    } catch {
        return null;
    }
}

/**
 * Fetch and build the background/foreground mesh containers for a scene.
 * Returns null when there is no scene JSON (the common case — the skin's scene
 * is fully in the spine), so callers fall back to spine-only rendering.
 */
export async function loadSceneMeshes(sceneUrl: string, textureBaseUrl: string, bust = ""): Promise<ILoadedScene | null> {
    let data: ISceneData;
    try {
        const res = await fetch(sceneUrl);
        if (!res.ok) return null;
        data = (await res.json()) as ISceneData;
    } catch {
        return null;
    }
    if (!data.layers?.length || !data.cameraSizePx) {
        // No mesh layers (the backdrop is baked into the spine atlas — e.g. Chongyue), but the
        // scene JSON may still carry the entrance CAMERA track. Return an empty-mesh scene so that
        // data still reaches the entrance camera (SceneIllust sources entranceCamCenterCurve etc.
        // from scene.data); returning null here would discard the whole entrance camera move.
        if (data.entranceCamCenterCurve?.length) return { data, background: new PIXI.Container(), foreground: new PIXI.Container(), hasDarkBackdrop: false };
        return null;
    }

    const bases = await Promise.all(Array.from({ length: data.textureCount }, (_, i) => loadTexture(`${textureBaseUrl}${i}.png${bust}`)));

    // Does this scene own a DARK opaque painted backdrop (a self-lit painted world, not a
    // bright studio wall)? Mirrors the per-layer `isBackdrop` predicate below plus a darkness
    // bound: a large (non-effect), near-opaque, desaturated backdrop layer whose whiteness is
    // below the studio-gradient brightness. Returned so the caller can drop the studio
    // gradient behind it (see {@link ILoadedScene.hasDarkBackdrop}).
    const hasDarkBackdrop = data.layers.some((l) => {
        const b = bases[l.tex];
        if (!b || l.additive) return false;
        const isRevealOverlay = l.activeFrom != null || l.activeUntil != null;
        const effRect = layerUvRectPx(l, b);
        const isEffect = effRect.w <= EFFECT_TEX_MAX && effRect.h <= EFFECT_TEX_MAX;
        return !isRevealOverlay && !isEffect && b.opaqueFrac >= 0.9 && b.whiteness >= 0.4 && b.whiteness < STUDIO_ENV_WHITENESS;
    });

    const background = new PIXI.Container();
    const foreground = new PIXI.Container();
    // Solid painted BACKDROPS (large + near-fully-opaque, e.g. Wiš'adel's white
    // studio wall) must sit at the very back: an opaque panel occludes everything
    // Unity authored behind it (the red "WISADEL" graffiti at a lower sort), so
    // those layers never render. Moving the backdrop to the back only REVEALS those
    // previously-dead layers (never hides a visible one). Added to `background`
    // first so it renders behind the rest.
    const backdropMeshes: PIXI.Mesh[] = [];
    const otherBg: PIXI.Mesh[] = [];
    // A layer whose full geometry (texture index + vertex positions + UVs + triangle
    // indices) is a byte-identical duplicate of an EARLIER layer already classified as
    // background is an author-side duplicate mesh (the same backdrop baked twice — once
    // correctly placed behind the character, once again at/above `characterSort`, e.g.
    // Skadi2's layers[0] at sort -5 and layers[5] at sort 0, tied with characterSort=0).
    // It inherits the earlier layer's background classification outright, regardless of
    // its own sort or how close its own texture's saturation sits to the
    // `isBackdropMisSorted` cutoff. Pure equality over data already read in this loop —
    // no new constant, no per-skin value; a no-op unless two layers' geometry is
    // literally identical (verified zero such pairs in cello's/mlynar's scene JSONs).
    const bgGeometrySignatures = new Set<string>();
    for (const layer of [...data.layers].sort((a, b) => a.sort - b.sort)) {
        const base = bases[layer.tex];
        if (!base) continue;
        const geomKey = `${layer.tex}|${layer.pos.join(",")}|${layer.uv.join(",")}|${layer.idx.join(",")}`;
        const isDuplicateOfBackground = bgGeometrySignatures.has(geomKey);
        // A layer authored above the character's sort normally renders in front of
        // her. But a DESATURATED PAINTED sheet that's either fully opaque or a large
        // semi-opaque atlas is a scene BACKDROP element mis-sorted into the
        // foreground, not an effect that crosses the character: an opaque sheet in
        // front would simply occlude her (never the intent), and a large grey painted
        // sheet is a wall/vista. In the source art she stands in FRONT of those —
        // Virtuosa "Diversity Oneness"'s reflected building & piano-key platforms
        // (opaqueFrac≈1.0) and her white crystal mandala (1024px, opaqueFrac 0.49,
        // sat 0.15) otherwise bury her body. Demote them behind the character. The
        // `sat < 0.25` gate spares genuine foreground EFFECTS: coloured light sheets
        // (Nearl "Evolved Art"'s prismatic spirit sheets, sat 0.8-0.99) and
        // Hoshiguma's teal ice (sat 0.28) stay in front; additive glows are excluded
        // outright.
        const texLarge = base.raw.width >= 512 || base.raw.height >= 512;
        // An entrance-REVEALED overlay (an `m_IsActive` window from the `_Start` clip)
        // plays a deliberate cinematic beat at its AUTHORED sort — e.g. Mlynar's white
        // transition flash, a solid-white plane that must white-out the frame from the
        // FRONT. Its texture reads exactly like a mis-sorted backdrop/veil to the
        // heuristics below (opaque, desaturated, white), so exempt windowed layers from
        // every demotion.
        const isRevealOverlay = layer.activeFrom != null || layer.activeUntil != null;
        // Demote when: fully opaque anywhere; OR a large semi-opaque atlas; OR a large
        // WHITISH translucent haze/light sheet (whiteness ≥ 0.45, opaqueFrac ≥ 0.25) —
        // Virtuosa's white glass-panel sheet (tex7: 1024px, opaqueFrac 0.33,
        // whiteness 0.53) hazes over her body from the front. Dark crystal shards that
        // legitimately cross her are far less white (≈0.27) and stay in front.
        const isBackdropMisSorted = isDuplicateOfBackground || (!isRevealOverlay && !layer.additive && base.sat < 0.25 && (base.opaqueFrac >= 0.95 || (texLarge && base.opaqueFrac >= 0.4) || (texLarge && base.opaqueFrac >= 0.25 && base.whiteness >= 0.45)));
        const isForeground = layer.sort >= data.characterSort && !isBackdropMisSorted;
        // A bright-white foreground effect panel is a paint/flash VEIL the source art
        // keeps BEHIND the character — re-sort it to the background so her body
        // occludes it (see VEIL_WHITENESS_MIN). Coloured foreground fx stay in front.
        const effRect = layerUvRectPx(layer, base);
        const isEffect = effRect.w <= EFFECT_TEX_MAX && effRect.h <= EFFECT_TEX_MAX;
        const isVeil = isForeground && !isRevealOverlay && !layer.additive && isEffect && base.whiteness >= VEIL_WHITENESS_MIN;
        // A SATURATED foreground effect panel sitting over the character's central
        // column is glowing energy the game blends additively (Hoshiguma's blue
        // ice-flame around her oni-mask "shield"). Exported as normal-blend (Unity
        // uses premultiplied "over" for the illustration), a stack of them piles into
        // an opaque blob that HIDES the character element beneath — her silver mask
        // vanishes under solid blue. Rendering them additive lets the mask show
        // through with a blue glow, matching the source. Gated to the character
        // column (|cx| < CHAR_COLUMN_HALF) so it never touches side comic-fx like
        // Wiš'adel's red halftone patches, which must stay solid.
        let cxMin = 1e9;
        let cxMax = -1e9;
        for (let i = 0; i < layer.pos.length; i += 2) {
            cxMin = Math.min(cxMin, layer.pos[i]);
            cxMax = Math.max(cxMax, layer.pos[i]);
        }
        const overCharacter = Math.abs((cxMin + cxMax) / 2) < CHAR_COLUMN_HALF;
        // Bright desaturated non-solid light-glow sheets → additive (un-darken the
        // high-key mirror-world backdrop). Applies wherever they sit (fg or demoted bg),
        // since additive light brightens from any depth. See LIGHT_GLOW_* above.
        const isLightGlowSheet = !layer.additive && isEffect && base.whiteness >= LIGHT_GLOW_WHITENESS && base.sat <= LIGHT_GLOW_SAT_MAX && base.opaqueFrac <= LIGHT_GLOW_OPAQUE_MAX;
        // V-b: these bright, desaturated, non-solid sheets (Virtuosa's Ram lattice/
        // cross-hatch, authored `additive:0` with TRANSPARENT fields) were force-promoted
        // to ADDITIVE on the premise that normal blend "multiplies the backdrop down to a
        // dim blue". That premise is inverted: the game holds a crisp DEEP-BLUE field +
        // white diamond, and it's the additive path — ~34 overlapping copies, now also
        // UV-scrolling (Capability A) and summed through the HDR bloom — that stacks into
        // the milky white wash that erases the blue. Their fields are already transparent
        // (own alpha), so NORMAL blend composites the white lines OVER the surviving blue
        // without summing. Respect the authored blend; keep full gain so the lines stay
        // crisp (not the 0.3 caustic attenuation). The other clause (a SATURATED fg glow
        // over the character — Hoshiguma's blue ice-flame) is unchanged.
        const forceAdditive = isForeground && !layer.additive && isEffect && base.sat >= GLOW_SAT_MIN && overCharacter;
        // A layer with an authored colour curve carries its EXACT animated alpha — the
        // effect-overlay gain (which tames caustics frozen without their animation)
        // would wrongly damp it (Mlynar's 0.671 white-out would peak at ~0.2).
        const mesh = buildLayerMesh(layer, base, forceAdditive, isLightGlowSheet || !!layer.colorCurve?.length, hasDarkBackdrop);
        if (!mesh) continue;
        if (isForeground && !isVeil) {
            foreground.addChild(mesh);
            continue;
        }
        // Large, near-fully-opaque, desaturated panel = a solid backdrop wall.
        const isBackdrop = !isRevealOverlay && !layer.additive && !isEffect && base.opaqueFrac >= 0.9 && base.whiteness >= 0.4;
        (isBackdrop ? backdropMeshes : otherBg).push(mesh);
        bgGeometrySignatures.add(geomKey);
    }
    for (const m of backdropMeshes) background.addChild(m);
    for (const m of otherBg) background.addChild(m);

    // A scene with no background layer isn't a backdrop — it's a foreground-only
    // fx overlay. Those layers are static snapshots of animated Unity effects and
    // render as garbage over the character (e.g. Nian "Unfettered Freedom"'s ink
    // smears), while the spine already carries the full illustration. Skip it and
    // let the self-contained spine render on its own.
    if (background.children.length === 0) {
        background.destroy({ children: true });
        foreground.destroy({ children: true });
        return null;
    }

    return { data, background, foreground, hasDarkBackdrop };
}
