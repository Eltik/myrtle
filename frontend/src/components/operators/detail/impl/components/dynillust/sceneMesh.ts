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

/** A scene layer's Ram-family (`Torappu/Particles-L2D/Ram/…`) dissolve + disturb masking,
 *  the mesh-quad twin of the particle emitter's `IRamData`. The shader carves the layer's
 *  real silhouette out of `_DissolveTex` and warps the lookups by `_DisturbTex`; drawn as a
 *  plain tinted quad the layer instead covers its mask's whole bounding rectangle. Texture
 *  fields index the scene's shared texture list; null = the slot is unbound. */
export interface ISceneRam {
    dissolveTex?: number | null;
    dissolveST: [number, number, number, number];
    /** SECOND dissolve map. The `Dissolve/` shader family multiplies TWO masks
     *  (`_DissolveTex_01` × `_DissolveTex_02`), each with its own threshold and border
     *  width. Absent on the single-map `Ram/` and `Disturb/` families. */
    dissolveTex2?: number | null;
    dissolveST2?: [number, number, number, number];
    amount2?: number;
    borderWidth2?: number;
    /** `_Edgecolor` (straight RGBA) + `_pow` — the rim the `...edge` shader variants composite
     *  along the dissolve boundary. Absent on the variants whose program has no rim term. */
    edgeColor?: [number, number, number, number] | null;
    edgePow?: number;
    disturbTex?: number | null;
    disturbST: [number, number, number, number];
    /** Dissolve threshold and the softness of its edge. */
    amount: number;
    borderWidth: number;
    /** How far a disturb sample displaces the lookups, and which lookups it reaches. */
    intensityU: number;
    intensityV: number;
    disturbInfluenceDissolveUV: number;
    disturbInfluenceMainUV: number;
    /** UV/second scroll of each mask (Unity UV space). */
    dissolveSpeed: [number, number];
    disturbSpeed: [number, number];
}

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
    /** Ram-family DISSOLVE/DISTURB masking (see {@link ISceneRam}). Absent = the layer is a
     *  plain tinted quad. */
    ram?: ISceneRam | null;
    /** BONE ATTACHMENT: a spine-unity `BoneFollower` in the layer's Unity ancestry snaps
     *  it onto this bone at runtime, so the baked `pos` above is only an editor pose
     *  (Mlynar's sword flare bakes as a streak in the lower-left instead of a halo along
     *  the blade). Absent = a world-fixed scene quad. */
    followBone?: string | null;
    /** The follower's `followBoneRotation`: false = translation-only tracking. */
    followBoneRot?: boolean;
    /** The follower GameObject's world ORIGIN in authored px (Y-up). */
    followOrigin?: [number, number] | null;
    /** The follower GameObject's world 2×2 LINEAR basis (Y-up), row-major
     *  `[m00, m01, m10, m11]`. */
    followBasis?: [number, number, number, number] | null;
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
    /** Spine-Unity `SkeletonRenderer.separatorSlotNames` — slot names at which the GAME splits
     *  this skeleton's draw into separate submeshes so other renderers sit BETWEEN the parts.
     *  Empty/absent on skins that do not use the feature. See {@link ISceneData.characterSort}. */
    separatorSlots?: string[] | null;
    /** Sorting orders of the separator PARTS, ascending (length = separatorSlots + 1). The
     *  depth the game gives each submesh; a layer sorted BETWEEN two parts is drawn in that
     *  gap rather than over the whole skeleton. */
    separatorPartSorts?: number[] | null;
    /** ENTRANCE (`_Start`) cinematic total length in seconds (director `_params.duration`).
     *  Present only on `_Start` scenes. */
    entranceDuration?: number | null;
    /** Straight RGBA (0..1) of the director's end-of-entrance screen fade (`_params.fadeColor`).
     *  The client fades the whole view to this colour as the entrance ends, swaps to the settled
     *  idle underneath it, and lifts it again. Authored white at full alpha on every skin
     *  measured, but read from the data. Absent on non-entrance scenes. */
    entranceFade?: [number, number, number, number] | null;
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
    /** One container per GAP between the split skeleton's parts — layers the game draws over
     *  one part and under the next. Empty unless the skin ships `separatorSlots` AND
     *  `?gaplayers=1` is set. The caller seats these inside the spine, like the particle
     *  backdrop washes. */
    gaps: PIXI.Container[];
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

/** A Ram-masked layer's mask textures, resolved from the scene's shared texture list.
 *  These are DATA textures (a noise field, a flow map), so they sample the RAW image —
 *  the dark-drop/whiteness processing is silhouette guesswork for artwork and would
 *  corrupt a mask. `white` stands in for an unbound slot (1.0 everywhere = no effect). */
interface IRamSceneTex {
    dissolve: PIXI.Texture | null;
    /** Second dissolve map (the `Dissolve/` family multiplies two). Null on single-map families. */
    dissolve2: PIXI.Texture | null;
    disturb: PIXI.Texture | null;
    white: PIXI.Texture;
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
        return {
            whiteness: aw > 0 ? wsum / aw : 0,
            opaqueFrac: n > 0 ? opaque / n : 0,
            sat: aw > 0 ? satSum / aw : 0,
        };
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
/** `?gaplayers=1` seats scene layers between a split skeleton's parts (diagnostic, default OFF). */
function gapLayersEnabled(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("gaplayers") === "1";
}

const EFFECT_SCENE_GAIN = 0.3;
/** DIAGNOSTIC (`?scenegain=<f>`): override {@link EFFECT_SCENE_GAIN} so the taming applied to
 *  effect-classified scene layers can be measured rather than assumed. 1 = no taming. */
function effectSceneGain(): number {
    if (typeof window === "undefined") return EFFECT_SCENE_GAIN;
    const v = parseFloat(new URLSearchParams(window.location.search).get("scenegain") ?? "");
    return Number.isFinite(v) && v >= 0 ? v : EFFECT_SCENE_GAIN;
}
/** Max texture size treated as an effect overlay; painted backdrops are large atlases. */
const EFFECT_TEX_MAX = 512;
/** Min linear STRETCH (sqrt(meshArea / texArea)) for a small texture to count as an effect
 *  overlay. **DISABLED (0) — MEASURED AND REJECTED.**
 *
 *  The rule's stated intent is "a SMALL texture STRETCHED over a large mesh", but the test
 *  keys on texture size ALONE, so a modest texture drawn near native scale is tamed like a
 *  caustic. The stretch values do look bimodal with a gap at 2.94 → 3.99:
 *      genuine caustics   3.99, 4.87, 5.65, 5.94, 6.73, 6.78, 7.62, 7.94, 8.12, 11.25, 21.50
 *      near-native panels 1.42, 1.69 (Virtuosa's starfield sheet), 1.80, 2.94
 *  but gating on it is a NET LOSS. Applied to the gain decision alone (interior MADC):
 *      stretchmin 0 → 3.5 :  mly 15.839 → 17.105   cel 20.405 → 20.130   ska 12.056 → 12.056
 *  Virtuosa gains 0.275, Mlynar loses 1.266. The populations cannot be separated by stretch
 *  either: her starfield sheet sits at 1.69 while a Mlynar layer that genuinely NEEDS taming
 *  sits at 1.42, so any threshold that exempts hers also exempts his. Separating them would
 *  need extra properties fitted to three skins — overfitting.
 *
 *  ALSO NOTE: `isEffect` feeds THREE decisions (this gain, `hasDarkBackdrop` detection, and
 *  the `isVeil` re-sort). Gating all three at once made Virtuosa far WORSE (25.294) because
 *  flipping `hasDarkBackdrop` reconfigures the whole scene. Any future change here must be
 *  applied to ONE decision at a time. `?stretchmin=<f>` keeps it measurable. */
const EFFECT_STRETCH_MIN = 0;
function effectStretchMin(): number {
    if (typeof window === "undefined") return EFFECT_STRETCH_MIN;
    const v = parseFloat(new URLSearchParams(window.location.search).get("stretchmin") ?? "");
    return Number.isFinite(v) && v >= 0 ? v : EFFECT_STRETCH_MIN;
}
/** Does this layer TILE its texture — i.e. do its UVs span more than one full repeat on
 *  either axis?
 *
 *  This separates the two populations `isEffect` currently conflates. That rule's stated
 *  intent is "a SMALL texture STRETCHED over a large mesh" (a caustic / light dome), but it
 *  tests texture size ALONE, so a repeating PATTERN drawn at native scale — a striped light
 *  curtain, a rain sheet, a music-staff overlay — is tamed like a caustic even though it is
 *  ordinary painted coverage. A tiled layer is by construction NOT stretched: it repeats to
 *  fill the mesh at its authored density.
 *
 *  Measured on Virtuosa, whose backdrop is a tiled vertical curtain: the game's stripe
 *  pattern is in exact horizontal PHASE with ours (0 px shift at every scored beat) and at
 *  the same frequency, but 1.4x-5x our amplitude — right pattern, right place, too weak.
 *  53 of her 98 layers are tiled. */
/** Exempt tiled layers from `EFFECT_SCENE_GAIN`? **DISABLED — MEASURED AND REJECTED.**
 *
 *  The reasoning was sound and the symptom is real: Virtuosa's backdrop stripes are in exact
 *  horizontal phase with the game's and at the same frequency, but 1.4x-5x too weak (worst at
 *  her t=10, ratio 2.89 — near enough to 1/0.3 to look like the gain was the culprit), and a
 *  tiled texture genuinely is not "a small texture stretched over a large mesh". It still
 *  measures WORSE, and only on the skin it touches:
 *
 *      exemption ON   mly 30.809   cel 24.861   ska 13.754
 *      exemption OFF  mly 30.809   cel 24.583   ska 13.754
 *
 *  (Salvaged-oracle scale — A/B sign only, see scratchpad/opcheck/score3.sh.) Mlynar and
 *  Skadi are untouched, so the +0.278 is entirely Virtuosa's: un-taming her other ~50 tiled
 *  layers costs more than the stripes gain. The stripe deficit is therefore NOT the scene
 *  gain, and the next attempt should isolate WHICH layer draws the stripes (binary-search
 *  `?bg:<lo>-<hi>`) rather than reclassifying a whole population.
 *
 *  `?tiled=1` re-enables it for a re-test. */
function tiledExemptEnabled(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("tiled") === "1";
}

/** `?ordfix=0` disables the order-preserving demotion pass (see the bucketing code), so an
 *  A/B can be rendered with an identical shot list instead of by editing the source between
 *  passes — the comparison that a reclassification has to survive before it ships. */
function orderFixEnabled(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("ordfix") !== "0";
}

/** A painted light SHEET is exempt from `EFFECT_SCENE_GAIN`: whitish, non-additive, and
 *  carrying essentially NO opaque pixels, so its coverage is alpha rather than paint.
 *
 *  This is the symmetric partner of `isPaintedSurface` (see SURFACE_OPAQUE_MIN): a fully
 *  OPAQUE layer is exempt because its alpha is coverage, not intensity — and a fully
 *  TRANSPARENT whitish layer is exempt because it is painted light, not a caustic overlay
 *  stretched from a small texture. Only the middle — partly-opaque effect textures — is what
 *  the gain exists to tame.
 *
 *  Found via Virtuosa's backdrop stripe curtain, which carries essentially all of her stripe
 *  energy (isolating it reproduces 2860 of the full frame's 2809 amplitude) and measures as
 *  wanting gain ~0.86, i.e. untamed. Her stripes sit in exact horizontal phase with the
 *  game's and at the same frequency, but 1.4x-5x too weak.
 *
 *  BOTH bounds are load-bearing, and the opacity one is what makes this safe. On whiteness
 *  alone the rule is not surgical at all — over a 6-skin corpus sample it re-classifies
 *  **53% of all tamed layers** (one skin loses 99 of 141, including opaque pure-white walls
 *  the gain exists to tame). Adding the opacity cap drops that to **1.5% (3 of 205)** while
 *  still catching the target. The whiteness threshold's own safe plateau is (0.27, 0.47]:
 *  measured on Virtuosa, 0.30/0.35/0.45 all give 23.608 and 0.50 reverts to the 24.583
 *  baseline, so 0.40 sits clear of both edges. */
const PAINTED_SHEET_WHITENESS_MIN = 0.4;
const PAINTED_SHEET_OPAQUE_MAX = 0.02;

/** DIAGNOSTIC (`?sheet=<f>`): override the whiteness floor; `?sheet=0` disables the exemption. */
function paintedSheetWhitenessMin(): number {
    if (typeof window === "undefined") return PAINTED_SHEET_WHITENESS_MIN;
    const raw = new URLSearchParams(window.location.search).get("sheet");
    if (raw == null) return PAINTED_SHEET_WHITENESS_MIN;
    const v = parseFloat(raw);
    return Number.isFinite(v) && v >= 0 ? v : PAINTED_SHEET_WHITENESS_MIN;
}

function isTiledLayer(layer: ISceneLayer): boolean {
    const uv = layer.uv;
    if (!uv || uv.length < 4) return false;
    let minU = Infinity;
    let maxU = -Infinity;
    let minV = Infinity;
    let maxV = -Infinity;
    for (let i = 0; i < uv.length; i += 2) {
        minU = Math.min(minU, uv[i]);
        maxU = Math.max(maxU, uv[i]);
        minV = Math.min(minV, uv[i + 1]);
        maxV = Math.max(maxV, uv[i + 1]);
    }
    return maxU - minU > 1.001 || maxV - minV > 1.001;
}

/** Linear stretch of a layer's mesh relative to its source texture rect. */
function layerStretch(layer: ISceneLayer, effRect: { w: number; h: number }): number {
    let minx = Infinity,
        maxx = -Infinity,
        miny = Infinity,
        maxy = -Infinity;
    for (let i = 0; i < layer.pos.length; i += 2) {
        minx = Math.min(minx, layer.pos[i]);
        maxx = Math.max(maxx, layer.pos[i]);
        miny = Math.min(miny, layer.pos[i + 1]);
        maxy = Math.max(maxy, layer.pos[i + 1]);
    }
    const texA = Math.max(1, effRect.w * effRect.h);
    const meshA = Math.max(1, (maxx - minx) * (maxy - miny));
    return Math.sqrt(meshA / texA);
}
/** {@link EFFECT_TEX_MAX} keys on SIZE alone, and size alone cannot tell a caustic from
 *  paint. The overlays the gain exists to tame are LIGHT — bright, desaturated and
 *  translucent (see LIGHT_GLOW_*). A near-opaque, strongly COLOURED normal-blend sheet is
 *  the opposite: painted surface. Fading paint does not dim a light, it punches a hole and
 *  lets the void behind show through — which is what bleached Virtuosa's 128px deep-blue
 *  ground plane (opaqueFrac 0.81, sat 0.85, whiteness 0.05) from the game's indigo to a
 *  flat grey. Across the 82 shipped skins these two bounds isolate 15 layers in 7 skins,
 *  all saturated painted surfaces, and the opacity cut sits in a real gap in the data
 *  (0.81 → 0.67 with nothing between). White flash/veil panels (sat ≈ 0, a 4×4 white quad
 *  stretched over the frame) stay attenuated, as do translucent coloured caustics. */
const SURFACE_OPAQUE_MIN = 0.75;
const SURFACE_SAT_MIN = 0.4;
/** A foreground normal-blend effect panel whose whiteness exceeds this is a bright
 *  paint/flash VEIL (not a coloured comic-fx). The source art keeps that white burst
 *  BEHIND the character, so we re-sort these behind the spine — her opaque body then
 *  occludes the veil over her face while the burst still shows around her. Coloured
 *  foreground fx (red halftones/slashes, whiteness≈0.2-0.5) stay in front. */
const VEIL_WHITENESS_MIN = 0.65;
/** Fraction of the authored camera view a white foreground layer must span, on BOTH axes,
 *  to count as full-frame ATMOSPHERE rather than a localised burst — and so be exempt from
 *  the {@link VEIL_WHITENESS_MIN} demotion. 0.8 rather than 1.0 because an authored haze is
 *  sized to the shot, not to the camera box: Mlynar's covers 0.99 x 0.85 of his view. The
 *  cut is not sensitive: of the 360 layers the veil rule demotes corpus-wide, the 8 that
 *  qualify sit at coverage 0.852-2.357 and the next candidate below them is at 0.677, so
 *  every threshold in 0.70-0.85 selects exactly the same set. */
const VEIL_FRAME_COVER = 0.8;
/** Min alpha-weighted saturation for a foreground effect to count as glowing energy
 *  (→ additive over the character). White veils (~0) and pale panels fall below it. */
const GLOW_SAT_MIN = 0.4;

/** On-screen extent of a layer's quad along one axis (0 = x, 1 = y), in scene px. */
function spanOf(layer: ISceneLayer, axis: 0 | 1): number {
    let lo = Infinity;
    let hi = -Infinity;
    for (let i = axis; i < layer.pos.length; i += 2) {
        lo = Math.min(lo, layer.pos[i]);
        hi = Math.max(hi, layer.pos[i]);
    }
    return hi > lo ? hi - lo : 0;
}

/** Exempt full-frame layers from the `isVeil` demotion? **DISABLED — MEASURED AND REJECTED.**
 *
 *  The reasoning was sound and the mis-classification is real: `isEffect` measures a layer's
 *  TEXTURE RESOLUTION, which says nothing about how much of the shot it covers, so a small
 *  white quad stretched across the camera view — the ordinary way atmosphere is authored —
 *  reads as a localised burst and is pushed behind the spine. Mlynar's L14 is exactly that (a
 *  256 px texture at 2192x1893 against a 2222 px view) and the demotion does cost him: his
 *  coat stays crisp and black where the game washes it out.
 *
 *  It still measures WORSE, and catastrophically so on skins with no capture to warn us. The
 *  corpus scan said this touches only 8 layers in 6 skins, which read as reassuringly narrow;
 *  what it could not say is HOW STRONG those layers are. Spot-rendering all six at t=2/5/8/12:
 *
 *      char_113_cqbw_epoque#7    frame mean 177.2 -> 218.5   (+41, peak +52)
 *      char_003_kalts_boc#6      frame mean 103.0 -> 125.0   (+22 at every beat)
 *      char_2023_ling_2          +1.5     char_4080_lin_nian#10  -0.6
 *      char_4134_cetsyr_epoque#50 0.00 (its layer is never drawn)
 *
 *  Rendered, the first two are unmistakable: the CHARACTER DISAPPEARS under a white sheet.
 *  These layers are opaque flashes, not haze, and the demotion is what keeps a body in front
 *  of them. Against that, Mlynar gained 0.020 MADC (17.721 -> 17.701) — and only that little
 *  because his one scored beat inside the layer's fade window is t=4, where its authored alpha
 *  is already 0.06.
 *
 *  Coverage alone therefore cannot separate atmosphere from a full-frame flash. A future
 *  attempt needs a strength/opacity term as well (Mlynar's peaks at alpha 0.92 and FADES to 0;
 *  the two that broke are effectively opaque), and it needs a capture for at least one of the
 *  affected skins to calibrate against — the corpus scan counts layers, not consequences.
 *
 *  `?veilcover=1` re-enables it for a re-test. */
function veilDisabled(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("noveil") === "1";
}
function veilFrameCoverExempt(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("veilcover") === "1";
}
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
    /** DIAGNOSTIC only (`?dumplayers=1`): the layer's texture index and its position in the
     *  scene JSON's `layers` array, so a dump row can be matched back to its source without
     *  guessing from the screen box (which the camera transform makes unreliable). */
    __texIndex?: number;
    __srcIndex?: number;
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
    /** Ram masking: each mask's UV/second scroll, re-applied to the shader every frame
     *  by {@link applySceneLayerRamScroll}. Absent unless the layer carries {@link ISceneRam}. */
    __ramSpeed?: { dissolve: [number, number]; disturb: [number, number] } | null;
    /** Bone attachment (see {@link ISceneLayer.followBone}): the followed bone name, its
     *  `followBoneRotation` flag, and the follower's BAKED world pose (origin + rotation
     *  angle) in the mesh's own Y-DOWN space — the reference {@link applySceneLayerFollow}
     *  measures the live bone against. Absent for world-fixed layers. */
    __follow?: { bone: string; rot: boolean; x: number; y: number; angle: number } | null;
}

/** Rebase a bone-following scene layer onto its live bone.
 *
 *  spine-unity's `BoneFollower` snaps its GameObject to the bone's world POSITION each
 *  frame, plus the bone's world ROTATION when `followBoneRotation` is set; it leaves the
 *  follower's own SCALE alone. So the runtime motion of everything beneath it — this
 *  layer's baked geometry included — is the RIGID delta from the follower's baked editor
 *  pose to the bone's live pose. Applying the bone's full matrix instead would drag the
 *  bone's scale in and drop the follower's own (Mlynar's flare rigs carry a ~10%
 *  non-uniform scale), so take position and angle only.
 *
 *  pixi-spine bone matrices are already in the skeleton's Y-DOWN world space — the same
 *  space the baked vertices live in after the exporter's Y flip — so the delta applies
 *  directly (conjugating it by a Y-flip would double-flip the rotation and slide the
 *  effect off its bone). No-op for layers built without a `followBone`. */
export function applySceneLayerFollow(mesh: PIXI.DisplayObject, findBone: (name: string) => PIXI.Matrix | null): void {
    const f = (mesh as unknown as ISceneLayerRuntime).__follow;
    if (!f) return;
    const now = findBone(f.bone);
    if (!now) return;
    // `followBoneRotation` off: the follower tracks the bone's POSITION only.
    if (!f.rot) {
        mesh.transform.setFromMatrix(new PIXI.Matrix(1, 0, 0, 1, now.tx - f.x, now.ty - f.y));
        return;
    }
    // Spine's `WorldRotationX` is the angle of the bone matrix's first column.
    const d = Math.atan2(now.b, now.a) - f.angle;
    const [c, s] = [Math.cos(d), Math.sin(d)];
    // Rotate about the follower's baked origin, then translate it onto the bone.
    mesh.transform.setFromMatrix(new PIXI.Matrix(c, s, -s, c, now.tx - (c * f.x - s * f.y), now.ty - (s * f.x + c * f.y)));
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

/** Scroll a Ram-masked scene layer's dissolve/disturb lookups to the scene clock, the
 *  shader's `_Time`-driven mask drift. Uniform-only (the masks are sampled from a static
 *  attribute), so it composes with the CPU `_MainTex` scroll without touching it. No-op for
 *  a layer built without {@link ISceneRam}. */
export function applySceneLayerRamScroll(mesh: PIXI.DisplayObject, t: number): void {
    const rt = mesh as unknown as ISceneLayerRuntime & { shader?: PIXI.Shader };
    const sp = rt.__ramSpeed;
    if (!sp || !rt.shader) return;
    rt.shader.uniforms.uDissolveScroll = [sp.dissolve[0] * t, sp.dissolve[1] * t];
    rt.shader.uniforms.uDisturbScroll = [sp.disturb[0] * t, sp.disturb[1] * t];
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

// Per-vertex-colour mesh shader for layers that carry `col`, and for OVER-BRIGHT
// layers (see {@link isOverbrightLayer}). PIXI's built-in MeshMaterial only applies
// a uniform tint, so a light sheet whose SHAPE is authored as a vertex-alpha gradient
// (opaque core → transparent edges over a flat-white texture) renders as a hard opaque
// block (e.g. Blaze "Wildfire"'s white light wedge stamped a solid triangle) — and its
// `tint` is an 8-bit RGB word, so a colour above 1.0 is silently truncated to white.
// This program multiplies the (premultiplied) texture by the premultiplied per-layer
// tint AND the premultiplied per-vertex colour — a faithful extension of MeshMaterial,
// so a pure-white vertex colour reduces to the exact same result — while carrying the
// tint as a float `uColor` uniform that keeps values above 1 intact all the way to the
// half-float HDR target. The common LDR path keeps MeshMaterial untouched.
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
uniform float uWorldAlpha; // scene-graph alpha (MeshMaterial gets this for free)
void main() {
    vec4 tex = texture2D(uSampler, vUV); // premultiplied
    vec4 vc = vec4(vColor.rgb * vColor.a, vColor.a); // premultiply the straight vertex colour
    gl_FragColor = tex * uColor * vc * uWorldAlpha;
}
`;

// Ram-family compositor for SCENE layers (see {@link ISceneRam}) — the mesh-quad port of
// the particle `RAM_FRAG`, minus the parts a mesh quad has no source for. There is no
// per-particle custom data, so the dissolve threshold is the material's `_Amount` alone and
// the disturb intensity is the material's `_IntensityU/V`; and the family's `col += col` is
// already baked into the exported tint (`ram_tint_scale` in the scene exporter), so doubling
// again here would blow the layer out.
//
// The masks are sampled from `aBaseUV`, the layer's UNTOUCHED authored UVs, not from `aUV`:
// `aUV` is rewritten CPU-side each frame by the `_MainTex` scroll / ST curve, and Unity
// derives each mask's lookup from the raw mesh UV with that mask's OWN ST and speed. Reading
// `aUV` would leak the main texture's scroll into the mask and make the silhouette crawl.
const RAM_SCENE_VERT = `
precision highp float;
attribute vec2 aVertexPosition;
attribute vec2 aUV;
attribute vec2 aBaseUV;
attribute vec4 aColor;
uniform mat3 translationMatrix;
uniform mat3 projectionMatrix;
uniform vec4 uDissolveST;
uniform vec4 uDissolveST2;
uniform vec4 uDisturbST;
uniform vec2 uDissolveScroll;
uniform vec2 uDisturbScroll;
varying vec2 vUV;
varying vec2 vDissolveUV;
varying vec2 vDissolveUV2;
varying vec2 vDisturbUV;
varying vec4 vColor;
void main() {
    gl_Position = vec4((projectionMatrix * translationMatrix * vec3(aVertexPosition, 1.0)).xy, 0.0, 1.0);
    vUV = aUV;
    vColor = aColor;
    // aBaseUV carries Unity's V already flipped (see buildLayerMesh); undo the flip, apply
    // the mask's ST + scroll in Unity space, then flip once more to sample.
    vec2 unity = vec2(aBaseUV.x, 1.0 - aBaseUV.y);
    vec2 ds = unity * uDissolveST.xy + uDissolveST.zw + uDissolveScroll;
    vec2 dt = unity * uDisturbST.xy + uDisturbST.zw + uDisturbScroll;
    vec2 ds2 = unity * uDissolveST2.xy + uDissolveST2.zw + uDissolveScroll;
    vDissolveUV = vec2(ds.x, 1.0 - ds.y);
    vDissolveUV2 = vec2(ds2.x, 1.0 - ds2.y);
    vDisturbUV = vec2(dt.x, 1.0 - dt.y);
}
`;
const RAM_SCENE_FRAG = `
precision highp float;
varying vec2 vUV;
varying vec2 vDissolveUV;
varying vec2 vDissolveUV2;
varying vec2 vDisturbUV;
varying vec4 vColor;
uniform sampler2D uSampler;
uniform sampler2D uDissolveTex;
uniform sampler2D uDissolveTex2;
uniform sampler2D uDisturbTex;
uniform vec4 uColor; // premultiplied tint*alpha
uniform float uWorldAlpha;
uniform float uAmount;
uniform float uBorderWidth;
uniform float uIntensityU;
uniform float uIntensityV;
uniform float uDisturbInfluenceDissolveUV;
uniform float uDisturbInfluenceMainUV;
uniform float uHasDissolve;
uniform float uHasDissolve2;
uniform float uAmount2;
uniform float uBorderWidth2;
uniform vec4 uEdgeColor;
uniform float uEdgePow;
uniform float uHasEdge;
uniform float uHasDisturb;
void main() {
    float disturbSample = uHasDisturb > 0.5 ? texture2D(uDisturbTex, vDisturbUV).x : 0.0;
    vec2 dOff = vec2(uIntensityU, uIntensityV) * disturbSample;
    vec4 tex = texture2D(uSampler, dOff * uDisturbInfluenceMainUV + vUV); // premultiplied
    float dissolveTex = uHasDissolve > 0.5 ? texture2D(uDissolveTex, dOff * uDisturbInfluenceDissolveUV + vDissolveUV).x : 1.0;
    // Game shader: sw = 1 - roundEven(_Amount + 0.5), i.e. floor(_Amount + 1.0). Porting
    // this as floor(t + 0.5) disables the dissolve for low thresholds and the whole quad
    // shows as a block instead of the mask's shape.
    float sw = 1.0 - floor(uAmount + 1.0);
    float bw = max(uBorderWidth, 1e-4);
    float dAlpha = clamp((bw * sw + (dissolveTex - uAmount)) / bw, 0.0, 1.0);
    // The Dissolve/ family multiplies a SECOND mask with its own threshold and border
    // (the Ram/ and Disturb/ families bind only the first, so uHasDissolve2 is 0 and this
    // term is exactly 1). Same profile as above, sampled through the same disturb warp.
    // NOTE: no backticks in here - this is inside a template literal.
    if (uHasDissolve2 > 0.5) {
        float d2 = texture2D(uDissolveTex2, dOff * uDisturbInfluenceDissolveUV + vDissolveUV2).x;
        float sw2 = 1.0 - floor(uAmount2 + 1.0);
        float bw2 = max(uBorderWidth2, 1e-4);
        dAlpha *= clamp((bw2 * sw2 + (d2 - uAmount2)) / bw2, 0.0, 1.0);
    }
    vec4 vc = vec4(vColor.rgb * vColor.a, vColor.a);
    vec4 pm = tex * uColor * vc * uWorldAlpha;   // premultiplied layer colour, pre-mask
    // EDGE RIM. The ...edge variants light the dissolve boundary:
    //   e   = clamp(mask / edge.a, 0, 1)
    //   rim = pow(1 - smoothstep(0, 1, e), pow)
    //   rgb = mix(rgb, edge.rgb, rim)      a = mask * a * mix(1, edge.a, rim)
    // Recovered to straight alpha because this shader composites premultiplied. With rim = 0
    // the whole branch reduces exactly to pm * dAlpha, so a layer with no rim is untouched.
    // edge.a == 0 means the rim is authored OFF: the game divides by it, e saturates to 1,
    // smoothstep reaches 1 and pow(0, _pow) is 0. The max() guard below reproduces that
    // exactly rather than producing a NaN - most authored _Edgecolor values are (0,0,0,0).
    float aBase = pm.a;
    vec3 cRgb = aBase > 1e-5 ? pm.rgb / aBase : vec3(0.0);
    float rim = 0.0;
    if (uHasEdge > 0.5) {
        float e = clamp(dAlpha / max(uEdgeColor.a, 1e-4), 0.0, 1.0);
        float sm = e * e * (3.0 - 2.0 * e);
        rim = pow(max(1.0 - sm, 0.0), max(uEdgePow, 1e-4));
    }
    vec3 outRgb = mix(cRgb, uEdgeColor.rgb, rim);
    float outA = dAlpha * aBase * mix(1.0, uEdgeColor.a, rim);
    gl_FragColor = vec4(outRgb * outA, outA);
}
`;

/** A Mesh on a CUSTOM shader: PIXI only folds `worldAlpha` into `MeshMaterial`, so a
 *  custom-shader layer would ignore every container fade above it (the entrance→idle
 *  cross-dissolve ramps the whole entrance root's alpha to 0). Feed it to the shader
 *  instead. Premultiplied throughout, so a scalar multiply IS the fade. */
class VColorMesh extends PIXI.Mesh<PIXI.Shader> {
    protected override _renderDefault(renderer: PIXI.Renderer): void {
        this.shader.uniforms.uWorldAlpha = this.worldAlpha;
        super._renderDefault(renderer);
    }
}

/** Build a per-vertex-colour mesh (see {@link VCOLOR_FRAG}). `col` is the layer's
 *  straight per-vertex colour, or null for an over-bright layer that has none (every
 *  vertex is opaque white, so the vertex term is the identity). */
function buildVColorMesh(layer: ISceneLayer, base: PIXI.BaseTexture, rgb: [number, number, number, number], alpha: number, col: number[] | null, ramTex: IRamSceneTex | null): PIXI.Mesh<PIXI.Shader> | null {
    const vertexCount = layer.pos.length / 2;
    if (col && col.length < vertexCount * 4) return null;
    const vertices = new Float32Array(vertexCount * 2);
    const uvs = new Float32Array(vertexCount * 2);
    const colors = new Float32Array(vertexCount * 4).fill(1);
    for (let i = 0; i < vertexCount; i++) {
        vertices[i * 2] = layer.pos[i * 2];
        vertices[i * 2 + 1] = -layer.pos[i * 2 + 1];
        uvs[i * 2] = layer.uv[i * 2];
        uvs[i * 2 + 1] = 1 - layer.uv[i * 2 + 1];
        if (col) {
            colors[i * 4] = col[i * 4];
            colors[i * 4 + 1] = col[i * 4 + 1];
            colors[i * 4 + 2] = col[i * 4 + 2];
            colors[i * 4 + 3] = col[i * 4 + 3];
        }
    }
    type BufArg = ConstructorParameters<typeof PIXI.Buffer>[0];
    const geometry = new PIXI.Geometry();
    geometry.addAttribute("aVertexPosition", new PIXI.Buffer(vertices as unknown as BufArg), 2);
    geometry.addAttribute("aUV", new PIXI.Buffer(uvs as unknown as BufArg), 2);
    geometry.addAttribute("aColor", new PIXI.Buffer(colors as unknown as BufArg), 4);
    geometry.addIndex(new PIXI.Buffer(new Uint16Array(layer.idx) as unknown as BufArg));
    // uColor: premultiplied per-layer tint*alpha (matches MeshMaterial's uniform).
    const uColor = [rgb[0] * alpha, rgb[1] * alpha, rgb[2] * alpha, alpha];
    if (ramTex && layer.ram) {
        const r = layer.ram;
        // Untouched authored UVs for the mask lookups (see {@link RAM_SCENE_VERT}).
        geometry.addAttribute("aBaseUV", new PIXI.Buffer(Float32Array.from(uvs) as unknown as BufArg), 2);
        const shader = PIXI.Shader.from(RAM_SCENE_VERT, RAM_SCENE_FRAG, {
            uSampler: new PIXI.Texture(base),
            uDissolveTex: ramTex.dissolve ?? ramTex.white,
            uDissolveTex2: ramTex.dissolve2 ?? ramTex.white,
            uDisturbTex: ramTex.disturb ?? ramTex.white,
            uColor,
            uWorldAlpha: 1,
            uAmount: r.amount,
            uBorderWidth: r.borderWidth,
            uIntensityU: r.intensityU,
            uIntensityV: r.intensityV,
            uDisturbInfluenceDissolveUV: r.disturbInfluenceDissolveUV,
            uDisturbInfluenceMainUV: r.disturbInfluenceMainUV,
            uHasDissolve: ramTex.dissolve ? 1 : 0,
            uHasDissolve2: ramTex.dissolve2 ? 1 : 0,
            uAmount2: r.amount2 ?? 0,
            uBorderWidth2: r.borderWidth2 ?? 0.1,
            uEdgeColor: r.edgeColor ?? [1, 1, 1, 1],
            uEdgePow: r.edgePow ?? 1,
            uHasEdge: r.edgeColor ? 1 : 0,
            uHasDisturb: ramTex.disturb ? 1 : 0,
            uDissolveST: r.dissolveST,
            uDissolveST2: r.dissolveST2 ?? [1, 1, 0, 0],
            uDisturbST: r.disturbST,
            uDissolveScroll: [0, 0],
            uDisturbScroll: [0, 0],
        });
        return new VColorMesh(geometry, shader);
    }
    const shader = PIXI.Shader.from(VCOLOR_VERT, VCOLOR_FRAG, { uSampler: new PIXI.Texture(base), uColor, uWorldAlpha: 1 });
    return new VColorMesh(geometry, shader);
}

/** True when the layer's exported colour leaves the 0..1 LDR range. The Ram /
 *  `Particles-L2D` compositors sample `2 × _MainColor × tex`, so the exporter bakes
 *  that ×2 into the tint/colour curve — Mlynar's `bg01` entrance backdrop ramps
 *  0.824 → 1.176. `MeshMaterial.tint` is an 8-bit RGB word and truncates anything
 *  above 1.0, which flattens that whole ramp onto white; such a layer has to take the
 *  float-uniform shader path so the over-bright energy survives into the half-float
 *  HDR target (see `hdrTonemap.ts`). Purely value-driven: an LDR layer never matches. */
function isOverbrightLayer(layer: ISceneLayer): boolean {
    if (layer.tint[0] > 1 || layer.tint[1] > 1 || layer.tint[2] > 1) return true;
    return !!layer.colorCurve?.some((c) => c[1] > 1 || c[2] > 1 || c[3] > 1);
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
/** Authored XY bounds `[minX, minY, maxX, maxY]` of a layer's mesh, from its flat
 *  `[x0,y0,x1,y1,…]` position array. Used to tell whether a demoted layer actually
 *  COVERS another one — see the order-preserving demotion pass. */
function boundsOf(pos: number[]): [number, number, number, number] {
    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    for (let i = 0; i + 1 < pos.length; i += 2) {
        if (pos[i] < minX) minX = pos[i];
        if (pos[i] > maxX) maxX = pos[i];
        if (pos[i + 1] < minY) minY = pos[i + 1];
        if (pos[i + 1] > maxY) maxY = pos[i + 1];
    }
    return [minX, minY, maxX, maxY];
}

function buildLayerMesh(layer: ISceneLayer, tex: ISceneTex, ramTex: IRamSceneTex | null, forceAdditive = false, fullGain = false, temperLargeAdditive = false): PIXI.Mesh | null {
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
    const isEffect = effRect.w <= EFFECT_TEX_MAX && effRect.h <= EFFECT_TEX_MAX && layerStretch(layer, effRect) >= effectStretchMin();
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
    // A painted SURFACE is never an overlay, whatever its texture measures (see
    // SURFACE_OPAQUE_MIN): its alpha carries coverage, not intensity, so scaling it only
    // reveals the void behind.
    const isPaintedSurface = !additive && tex.opaqueFrac >= SURFACE_OPAQUE_MIN && tex.sat >= SURFACE_SAT_MIN;
    // A TILED layer is exempt from the taming (see isTiledLayer): it is repeating painted
    // coverage, not a small texture stretched into a caustic, so scaling it only dims art the
    // game draws at full strength. Applied to the GAIN decision ONLY — `isEffect` also feeds
    // `hasDarkBackdrop` detection and the `isVeil` re-sort, and gating all three at once
    // reconfigures the whole scene (it previously drove Virtuosa to 25.294). `?tiled=0`
    // restores the old behaviour for an A/B.
    const tiledExempt = isTiledLayer(layer) && tiledExemptEnabled();
    // A painted light SHEET keeps its full energy — see PAINTED_SHEET_WHITENESS_MIN. Distinct
    // from `isLightGlowSheet`, which needs >=0.70 whiteness and <=0.15 sat and deliberately
    // excludes the mid-white veils that must stay demoted.
    const sheetMin = paintedSheetWhitenessMin();
    const sheetExempt = sheetMin > 0 && !additive && tex.whiteness >= sheetMin && tex.opaqueFrac <= PAINTED_SHEET_OPAQUE_MAX;
    const gain = !fullGain && !isPaintedSurface && ((isEffect && !tiledExempt && !sheetExempt) || (additive && temperLargeAdditive)) ? effectSceneGain() : 1;
    // DIAGNOSTIC (`?dumpclass=1`): record every input the effect/gain classifier uses, so a
    // mis-classified layer can be separated from correctly-tamed ones on real properties.
    // Collected into a global rather than logged: the dev server rewrites `console.log` into
    // a styled "Go to Source" message, which destroys the payload.
    if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("dumpclass")) {
        let minx = Infinity,
            maxx = -Infinity,
            miny = Infinity,
            maxy = -Infinity;
        for (let i = 0; i < layer.pos.length; i += 2) {
            minx = Math.min(minx, layer.pos[i]);
            maxx = Math.max(maxx, layer.pos[i]);
            miny = Math.min(miny, layer.pos[i + 1]);
            maxy = Math.max(maxy, layer.pos[i + 1]);
        }
        const meshW = maxx - minx,
            meshH = maxy - miny;
        const texA = Math.max(1, effRect.w * effRect.h),
            meshA = Math.max(1, meshW * meshH);
        const w = window as unknown as { __classDump?: unknown[] };
        w.__classDump ??= [];
        w.__classDump.push({
            sort: layer.sort,
            add: additive ? 1 : 0,
            texW: Math.round(effRect.w),
            texH: Math.round(effRect.h),
            meshW: Math.round(meshW),
            meshH: Math.round(meshH),
            stretch: Number(Math.sqrt(meshA / texA).toFixed(2)),
            opaque: Number(tex.opaqueFrac.toFixed(3)),
            sat: Number(tex.sat.toFixed(3)),
            white: Number(tex.whiteness.toFixed(3)),
            isEffect: isEffect ? 1 : 0,
            painted: isPaintedSurface ? 1 : 0,
            fullGain: fullGain ? 1 : 0,
            gain: Number(gain.toFixed(2)),
        });
    }

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
        rt.__texIndex = layer.tex;
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
        // Bone attachment: reduce the follower's baked world frame to a Y-DOWN origin and
        // rotation angle. `followBasis` is row-major Y-up `[m00, m01, m10, m11]`, so its
        // first column is `(m00, m10)` and the Y flip negates both the angle and origin Y.
        if (ramTex && layer.ram) rt.__ramSpeed = { dissolve: layer.ram.dissolveSpeed, disturb: layer.ram.disturbSpeed };
        if (layer.followBone && layer.followOrigin && layer.followBasis) {
            const [m00, , m10] = layer.followBasis;
            const [ox, oy] = layer.followOrigin;
            rt.__follow = { bone: layer.followBone, rot: layer.followBoneRot !== false, x: ox, y: -oy, angle: -Math.atan2(m10, m00) };
        }
    };
    const hasVertexColor = !!layer.col && layer.col.length >= vertexCount * 4;
    if (hasVertexColor || isOverbrightLayer(layer) || ramTex) {
        const vmesh = buildVColorMesh(layer, base, rgb, alpha, hasVertexColor ? (layer.col ?? null) : null, ramTex);
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
    // DIAGNOSTIC: record what this layer actually resolved to, so a dump can tell a true additive
    // layer from one that merely carries the data flag, and whether it sampled the
    // dark-field-dropped `glow` texture or the raw one.
    (mesh as unknown as { __blendDbg?: unknown }).__blendDbg = {
        additive,
        dataAdditive: !!layer.additive,
        forced: !!forceAdditive,
        usedGlow: base === tex.glow,
        glowIsRaw: tex.glow === tex.raw,
        gain: Number(gain.toFixed(3)),
        texWhite: Number(tex.whiteness.toFixed(3)),
        texOpaque: Number(tex.opaqueFrac.toFixed(3)),
    };
    stashRuntime(mesh);
    return mesh;
}

/** The authored camera/display frame — where the static illustration maps in
 *  spine coordinates. Present even for skins with no mesh layers (a bare
 *  `[scene].json` carrying only the frame), so the static-art backdrop can be
 *  aligned for spine-only skins too. */
export interface ISceneFrame {
    /** Display-frame centre in spine-authored px (Y-up). NULL when the controller's
     *  `_adjustes` stop is uninitialised — see {@link sceneFrameOf}. */
    offsetPx: [number, number] | null;
    /** Display-frame square full extent in spine-authored px. Null on the same skins. */
    viewPx: number | null;
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

/** How many times the curve's own MEDIAN segment speed a single segment must exceed to read
 *  as a HARD CUT rather than a smooth dolly — a scale-free outlier factor, applied identically
 *  to every skin (never a per-skin value). Skadi the Corrupting Heart's rig repositions spike to
 *  ~430× the median; Virtuosa's fastest genuine pan stays ~1×, so the two never overlap. */
const CAM_CUT_SPEED_FACTOR = 8;

/** Detect HARD-CUT segments in an entrance camera-centre curve. The `_Start` camera track is
 *  baked per-frame (~30 fps); a hard cut (Skadi the Corrupting Heart's rig snaps to a new
 *  position at t≈3.67 / 11.0 / 14.7 s) shows up as a single baked frame whose positional jump is
 *  orders of magnitude larger than the surrounding smooth motion. Linearly interpolating across
 *  such a segment at the viewer's (higher) render rate smears that instant cut into a visible
 *  fast pan — the "camera doesn't move immediately, it jolts" artifact. Returning the segment's
 *  END-index lets {@link sampleCurveXY} snap instead of lerp there.
 *
 *  Detection is fully data-derived: a segment qualifies when its per-second speed is at least
 *  {@link CAM_CUT_SPEED_FACTOR}× the curve's OWN median segment speed AND its positional jump
 *  exceeds a tenth of the frame extent (`frameSize`, so the floor scales with the shot, not an
 *  absolute pixel constant). A smooth-but-fast pan (Virtuosa/cello: high speed, tiny per-frame
 *  delta) clears neither test, so cut-free curves return an empty set and the sampler stays
 *  byte-identical. */
export function detectCurveCuts(curve: [number, number, number][] | null | undefined, frameSize: number | null | undefined): Set<number> {
    const cuts = new Set<number>();
    if (!curve || curve.length < 3 || !frameSize || frameSize <= 0) return cuts;
    const seg: { i: number; speed: number; delta: number }[] = [];
    for (let i = 1; i < curve.length; i++) {
        const [t0, x0, y0] = curve[i - 1];
        const [t1, x1, y1] = curve[i];
        const dt = t1 - t0;
        if (dt <= 0) continue;
        const delta = Math.hypot(x1 - x0, y1 - y0);
        seg.push({ i, speed: delta / dt, delta });
    }
    if (!seg.length) return cuts;
    const sorted = seg.map((s) => s.speed).sort((a, b) => a - b);
    const median = sorted[Math.floor(sorted.length / 2)] || 0;
    const deltaFloor = 0.1 * frameSize;
    for (const s of seg) {
        if (s.speed >= CAM_CUT_SPEED_FACTOR * median && s.delta > deltaFloor) cuts.add(s.i);
    }
    return cuts;
}

/** Linear-sample a `[t, x, y]` keyframe curve at time `t`, clamped to the endpoints. Returns
 *  `[x, y]`, or `null` for an absent/empty curve. Used for the entrance camera frame-centre curve.
 *  `cuts` (from {@link detectCurveCuts}) marks hard-cut segments the game snaps through: inside
 *  one, STEP to the nearer keyframe instead of interpolating, so an instant reposition is never
 *  smeared into a pan. Omit `cuts` (or pass an empty set) for pure linear behaviour. */
export function sampleCurveXY(curve: [number, number, number][] | null | undefined, t: number, cuts?: Set<number>): [number, number] | null {
    if (!curve || curve.length === 0) return null;
    if (t <= curve[0][0]) return [curve[0][1], curve[0][2]];
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i][0]) {
            const [t0, x0, y0] = curve[i - 1];
            const [t1, x1, y1] = curve[i];
            // Hard cut: snap to the nearer endpoint (step at the segment midpoint — within half a
            // baked frame of the true cut, so imperceptible) rather than lerp across the jump.
            if (cuts?.has(i)) return t - t0 <= t1 - t ? [x0, y0] : [x1, y1];
            const f = t1 > t0 ? (t - t0) / (t1 - t0) : 0;
            return [x0 + (x1 - x0) * f, y0 + (y1 - y0) * f];
        }
    }
    const last = curve[curve.length - 1];
    return [last[1], last[2]];
}

/** Frame data taken directly from a loaded {@link ISceneData}. */
export function sceneFrameOf(data: ISceneData): ISceneFrame | null {
    // Only `cameraSizePx` is REQUIRED. It is the camera's own half-height and every scene
    // authors it; the other two come from the controller's `_adjustes` display stops, which a
    // skin may ship uninitialised. Demanding all three threw the authored camera away whenever a
    // stop was missing and fell back to inflating the character's measured bounds — which framed
    // Nearl "Epoque" (a valid cameraSizePx of 1300, both stops unset) entirely off-screen: she
    // rendered as bare environment fill with one corner of cloud, no character at all. Every
    // consumer of `offsetPx`/`viewPx` already guards for absence (`?.`, `usableExtent`), so the
    // authored camera extent survives on its own.
    if (!data.cameraSizePx) return null;
    return {
        offsetPx: data.cameraOffsetPx ?? null,
        viewPx: data.cameraViewPx ?? null,
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
        if (data.entranceCamCenterCurve?.length) return { data, background: new PIXI.Container(), foreground: new PIXI.Container(), gaps: [], hasDarkBackdrop: false };
        return null;
    }

    const bases = await Promise.all(Array.from({ length: data.textureCount }, (_, i) => loadTexture(`${textureBaseUrl}${i}.png${bust}`)));

    // Ram mask lookup (see {@link IRamSceneTex}). Null unless the layer carries a mask that
    // actually loaded, which is what switches it onto the Ram compositor at all.
    const ramTexOf = (layer: ISceneLayer): IRamSceneTex | null => {
        const r = layer.ram;
        if (!r) return null;
        const slot = (i: number | null | undefined) => (i != null && bases[i] ? new PIXI.Texture(bases[i].raw) : null);
        const dissolve = slot(r.dissolveTex);
        const dissolve2 = slot(r.dissolveTex2);
        const disturb = slot(r.disturbTex);
        return dissolve || dissolve2 || disturb ? { dissolve, dissolve2, disturb, white: PIXI.Texture.WHITE } : null;
    };

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
    /** Background-bound meshes carry their authored sort so the final order can be rebuilt
     *  after the order-preserving demotion pass below. */
    // GAP CONTAINERS. The game splits the skeleton at `separatorSlots` and gives each part its
    // own sorting order, so a scene layer sorted BETWEEN two parts is drawn in that gap — over
    // the earlier part, under the later one. Drawing it in `foreground` puts it over the whole
    // character; drawing it in `background` puts it behind the scenery it should cover. The
    // particle sheets already got this treatment; these are the scene layers, 516 of them
    // across 33 composites.
    //
    // MEASURED AND REFUTED (`?gaplayers=1`, kept inert so the idea is not re-derived):
    //     ska 10.404 -> 49.922      cel 17.167 -> 19.795      mly 17.405 (no separator)
    // Decisive. The lesson is what separates this from the wash: seating the `isBackdropParticle`
    // sheets WORKED because it RESTORED layers our own heuristic had force-demoted behind the
    // whole spine to the depth the data already gave them. These scene layers were never
    // displaced — they are where the author put them — so re-sorting them by the part depths
    // moves correct geometry. A part's sortingOrder is evidently not comparable to a scene
    // layer's in the way it is to a particle system's. Do not retry without first establishing
    // that the two sorts live in the same space.
    const partSorts = data.separatorPartSorts ?? [];
    const gapsOn = partSorts.length >= 2 && gapLayersEnabled();
    const gapMeshes: { mesh: PIXI.Mesh; sort: number }[][] = Array.from({ length: Math.max(0, partSorts.length - 1) }, () => []);
    /** Which gap does this sort fall in? -1 = behind every part, gaps.length = in front of all. */
    const gapOf = (sort: number): number => {
        let j = -1;
        for (let i = 0; i < partSorts.length; i++) if (partSorts[i] <= sort) j = i;
        return j;
    };
    const backdropMeshes: { mesh: PIXI.Mesh; sort: number }[] = [];
    const otherBg: { mesh: PIXI.Mesh; sort: number }[] = [];
    /** Foreground candidates, bucketed only once the demoted set is known. */
    const fgPending: { mesh: PIXI.Mesh; sort: number; box: [number, number, number, number] }[] = [];
    /** Layers the author put in FRONT of the character that we pushed behind it (the veil
     *  demotion), with the authored bounds needed to tell which foreground layers they cover. */
    const demotedFg: { sort: number; box: [number, number, number, number] }[] = [];
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
    // Source index per layer, captured BEFORE the sort below reorders them — diagnostic only
    // (`?dumplayers=1`), so a dump row names its scene-JSON layer instead of being guessed at
    // from its screen box.
    const srcIndexOf = new Map<ISceneLayer, number>(data.layers.map((l, i) => [l, i] as const));
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
        // ...unless it covers the WHOLE FRAME, in which case it is atmosphere, not a panel.
        // `isEffect` measures the layer's TEXTURE resolution, which says nothing about how
        // much of the shot the layer occupies: a 4x4 or 256px white quad stretched across the
        // camera view reads as a small "effect" and gets demoted behind the spine. That is
        // right for a burst that should sit behind a face and wrong for a full-frame haze,
        // which must tint the character along with everything else. Mlynar's L14 is the case
        // in point — a 256px quad stretched to 2192x1893 against a 2222px view, holding an
        // authored fade from alpha 0.92 down to 0. Demoted, his coat stays crisp and black
        // while the game veils it: the frame runs ~10 luma dark through t=3-6, peaking at
        // -11.7 at t=4 (his worst beat, MADC 22.80), and the deficit tracks this layer's
        // fade exactly — we are too BRIGHT while it is up (t<=2) and too DARK once it lands.
        // Coverage is the discriminator the texture size cannot provide. Corpus-wide this
        // exempts 8 layers in 6 skins out of the 360 the veil rule demotes, and touches
        // neither of the other two reference skins.
        const camExtent = 2 * (data.cameraSizePx || 0);
        const coversFrame = camExtent > 0 && spanOf(layer, 0) >= VEIL_FRAME_COVER * camExtent && spanOf(layer, 1) >= VEIL_FRAME_COVER * camExtent;
        // DIAGNOSTIC (`?noveil=1`): disable the demotion outright, so "should this layer be in
        // FRONT of the character at all?" can be measured on its own. Distinct from
        // `?veilcover=1`, which only exempts layers whose span beats `VEIL_FRAME_COVER` of the
        // AUTHORED camera box — an extent the entrance plunge is much wider than, which is why
        // that flag is bit-identical on cello even though her veil manifestly fills the shot.
        // Touches ONLY the re-sort, never `isEffect`'s other two consumers (see the note above).
        const isVeil = !veilDisabled() && isForeground && !isRevealOverlay && !layer.additive && isEffect && base.whiteness >= VEIL_WHITENESS_MIN && !(coversFrame && veilFrameCoverExempt());
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
        const mesh = buildLayerMesh(layer, base, ramTexOf(layer), forceAdditive, isLightGlowSheet || !!layer.colorCurve?.length, hasDarkBackdrop);
        if (!mesh) continue;
        (mesh as unknown as ISceneLayerRuntime).__srcIndex = srcIndexOf.get(layer);
        const box = boundsOf(layer.pos);
        if (isForeground && !isVeil) {
            // Deferred: whether this can STAY in front depends on which layers the veil
            // demotion moves behind it, which is only known once every layer is classified.
            fgPending.push({ mesh, sort: layer.sort, box });
            continue;
        }
        // Large, near-fully-opaque, desaturated panel = a solid backdrop wall.
        const isBackdrop = !isRevealOverlay && !layer.additive && !isEffect && base.opaqueFrac >= 0.9 && base.whiteness >= 0.4;
        // Record anything pushed BEHIND the character that the author put in FRONT of it —
        // the veil demotion. See the order-preservation pass below.
        // A FULL-FRAME veil is excluded as a coverer. Its demotion is not a statement about
        // scene depth — it is the character-specific hack that keeps a whitish wash from
        // erasing her — so it must not drag the scene behind her with it. Mlynar's sort-10
        // veil spans his whole camera view and would otherwise demote every layer he draws
        // in front (0.40 MADC). A LOCALISED demoted layer genuinely sits above its
        // neighbours, and that ordering is worth preserving.
        const spansView = Math.max(box[2] - box[0], box[3] - box[1]) >= (data.cameraSizePx ?? Number.POSITIVE_INFINITY);
        if (isForeground && !spansView) demotedFg.push({ sort: layer.sort, box });
        // A solid backdrop WALL stays at the very back whatever its sort — it is the scenery
        // every part is drawn against, not something to interleave.
        const gj = gapsOn && !isBackdrop ? gapOf(layer.sort) : -1;
        if (gj >= 0 && gj < gapMeshes.length) gapMeshes[gj].push({ mesh, sort: layer.sort });
        else (isBackdrop ? backdropMeshes : otherBg).push({ mesh, sort: layer.sort });
        bgGeometrySignatures.add(geomKey);
    }
    // ORDER-PRESERVING DEMOTION. Moving a layer behind the character does not only change
    // its relation to HER — it changes its relation to every other scene layer, and the veil
    // demotion silently inverts those. Virtuosa seats 34 layers at sort 5-100 behind the
    // character as veils while five dark cross-hatched quads at sort 4 stay in front, so
    // quads the author put UNDERNEATH get painted ON TOP of the bright crystal faces that
    // should cover them. That is why those quads read as dark hatching across a frame the
    // game keeps bright: measured over their own footprints we darken by 21-32 luma where
    // the game darkens by 6-7, and the covering layers overlap them by 288-404% of area.
    //
    // The correction is pairwise and GEOMETRIC, not a global floor: a foreground layer is
    // demoted only when a layer authored ABOVE it was demoted AND actually covers it. A
    // global "demote everything below the lowest demoted sort" is both too weak and too
    // strong — on Virtuosa the lowest demoted sort is 1, so it moves nothing, while on
    // Mlynar it shuffles unrelated layers and costs 0.40 MADC. Requiring real coverage
    // fixes the inversion exactly where it exists and is a no-op everywhere else.
    const COVER = 0.5;
    for (const p of orderFixEnabled() ? fgPending : []) {
        const area = Math.max(1e-6, (p.box[2] - p.box[0]) * (p.box[3] - p.box[1]));
        const covered = demotedFg.some((d) => {
            if (d.sort <= p.sort) return false; // authored below us — no inversion possible
            const w = Math.min(d.box[2], p.box[2]) - Math.max(d.box[0], p.box[0]);
            const h = Math.min(d.box[3], p.box[3]) - Math.max(d.box[1], p.box[1]);
            return w > 0 && h > 0 && (w * h) / area >= COVER;
        });
        if (covered) otherBg.push(p);
    }
    // Background draws back-to-front: solid backdrop walls first, then everything else in
    // authored sort order. The demoted layers must MERGE into that order by sort rather than
    // append, or they would land above the very veils they were demoted to stay under.
    otherBg.sort((a, b) => a.sort - b.sort);
    for (const m of backdropMeshes) background.addChild(m.mesh);
    for (const m of otherBg) background.addChild(m.mesh);
    for (const p of fgPending) {
        if (otherBg.includes(p)) continue;
        const gj = gapsOn ? gapOf(p.sort) : -1;
        if (gj >= 0 && gj < gapMeshes.length) gapMeshes[gj].push(p);
        else foreground.addChild(p.mesh);
    }
    for (const g of gapMeshes) g.sort((a, b) => a.sort - b.sort);
    const gaps = gapMeshes.map((g) => {
        const c = new PIXI.Container();
        for (const m of g) c.addChild(m.mesh);
        return c;
    });

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

    return { data, background, foreground, gaps, hasDarkBackdrop };
}
