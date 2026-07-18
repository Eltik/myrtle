import * as PIXI from "pixi.js";
import { useEffect, useRef, useState } from "react";
import { Spinner } from "#/components/ui/spinner";
import type { IChibiSpineFiles } from "#/lib/api/chibis";
import { cn } from "#/lib/utils";
import { ANIMATION_SPEED } from "../chibi/constants";
import { chibiAssetURL, DEFAULT_SPINE_FIT, type IAnimationBounds, type ISpineFit, layoutSpine, loadSpineWithEncodedURLs, measureAnimationBounds } from "../chibi/helpers";
import { createHDRScene, type IHDRScene } from "./hdrTonemap";
import { type FindBone, type ILoadedParticles, loadParticles } from "./particles";
import { type ISceneFrame, loadSceneFrame, loadSceneMeshes, orthoZoomRatio, sampleCurveXY, sceneFrameOf } from "./sceneMesh";

interface ISceneIllustProps {
    files: IChibiSpineFiles;
    server?: "en" | "cn";
    fit?: ISpineFit;
    /**
     * How to frame a mesh-backed scene. "character" (default) frames to the
     * character's visible bounds — prominent character, good for the narrow card.
     * "authored" uses the game's `_adjustes` display frame (full-scene, square
     * viewport) — for the large fullscreen viewer.
     */
    framing?: "character" | "authored";
    /**
     * Static skin illustration URL. Some dynamic assets omit the full painted
     * backdrop (sky/interior) that the static art has — the game's archive
     * viewer composites the static illustration behind the animated spine. When
     * given, we draw it as a backdrop layer aligned to the authored camera frame
     * and frame the whole scene to that frame, so the animated spine overlays its
     * own static counterpart and the missing backdrop fills in.
     */
    backdrop?: string;
    onReady?: () => void;
}

const IDLE_ANIMATION = "Idle";

/** Visible-pixel bounds of the illustration at rest, plus alpha-mass landmarks
 *  (centroid, solid head/feet rows) — see {@link measureVisibleBounds}. */
type IVisBounds = IAnimationBounds & {
    centroid?: { cx: number; cy: number };
    /** Y (container space) of the character's SOLID head top — the first row (from the
     *  top) whose alpha mass exceeds a fraction of the densest row. Unlike the bbox top,
     *  this SKIPS thin protrusions above the head (a raised rifle, wispy hair, effects),
     *  so it's a pose-robust anchor for the head. */
    headTop?: number;
    /** Y (container space) of the character's SOLID base — the last dense row (from the
     *  bottom), skipping thin trailing hair/dress tails. */
    feetBottom?: number;
};
// A row counts as "solid" character (vs a thin protrusion) once its alpha mass reaches a
// fraction of the densest row. The head uses a HIGHER bar so a held weapon or effect ABOVE
// the head (Executor's raised rifle) is skipped; the feet use a LOWER bar so a thin extended
// leg/base BELOW the body is KEPT (only the wispy trailing hair/dress tail is skipped).
const HEAD_ROW_FRAC = 0.18;
const FEET_ROW_FRAC = 0.07;
/**
 * Bounds of the actually-VISIBLE pixels of the illustration at its resting frame.
 * The skeleton's geometry bounds (`getLocalBounds`) can be wildly inflated by
 * invisible/off-screen attachments (bounding boxes, huge transparent effect
 * quads) — that shrinks the illustration into a corner. So we render the resting
 * pose to a small offscreen texture, read back its alpha, and take the tight
 * bounding box of non-transparent pixels — the real illustration extent, centred.
 * Returns null (caller falls back to geometry bounds) if it can't be measured.
 */
function measureVisibleBounds(renderer: PIXI.IRenderer, spine: import("pixi-spine").Spine, idleAnim: string): IVisBounds | null {
    spine.state.timeScale = 1;
    spine.state.clearTracks();
    spine.skeleton.setToSetupPose();
    if (spine.spineData.findAnimation(idleAnim)) spine.state.setAnimation(0, idleAnim, true);
    spine.update(0);
    const geom = spine.getLocalBounds();
    if (geom.width <= 0 || geom.height <= 0) return null;

    const MAX = 512;
    const scale = Math.min(MAX / geom.width, MAX / geom.height, 4);
    const rw = Math.max(1, Math.ceil(geom.width * scale));
    const rh = Math.max(1, Math.ceil(geom.height * scale));
    const rt = PIXI.RenderTexture.create({ width: rw, height: rh });
    try {
        const m = new PIXI.Matrix();
        m.scale(scale, scale);
        m.translate(-geom.x * scale, -geom.y * scale);
        renderer.render(spine, { renderTexture: rt, transform: m, clear: true });
        const pixels = renderer.extract.pixels(rt);
        let x0 = rw;
        let y0 = rh;
        let x1 = -1;
        let y1 = -1;
        let sx = 0;
        let sy = 0;
        let sw = 0;
        const rowMass = new Float64Array(rh); // total alpha per pixel row → solid-body profile
        for (let y = 0; y < rh; y++) {
            let rowSum = 0;
            for (let x = 0; x < rw; x++) {
                const a = pixels[(y * rw + x) * 4 + 3];
                if (a > 8) {
                    if (x < x0) x0 = x;
                    if (x > x1) x1 = x;
                    if (y < y0) y0 = y;
                    if (y > y1) y1 = y;
                    // Alpha-weighted centroid: the dense character/props dominate over
                    // thin, spread-out backdrop elements (arch, railing), giving the
                    // illustration's true focal point rather than its bounding-box centre.
                    sx += x * a;
                    sy += y * a;
                    sw += a;
                    rowSum += a;
                }
            }
            rowMass[y] = rowSum;
        }
        if (x1 < x0 || y1 < y0) return { x: geom.x, y: geom.y, width: geom.width, height: geom.height };
        const centroid = sw > 0 ? { cx: geom.x + sx / sw / scale, cy: geom.y + sy / sw / scale } : undefined;
        // Pose-robust head/feet: the first/last row (top-down / bottom-up) whose alpha mass
        // reaches SOLID_ROW_FRAC of the densest row — skipping thin protrusions (a raised
        // rifle above the head, wispy hair/dress tails below the feet) that the bbox includes.
        let maxRow = 0;
        for (let y = y0; y <= y1; y++) if (rowMass[y] > maxRow) maxRow = rowMass[y];
        let headTopPx = y0;
        for (let y = y0; y <= y1; y++) {
            if (rowMass[y] >= maxRow * HEAD_ROW_FRAC) {
                headTopPx = y;
                break;
            }
        }
        let feetBotPx = y1;
        for (let y = y1; y >= y0; y--) {
            if (rowMass[y] >= maxRow * FEET_ROW_FRAC) {
                feetBotPx = y;
                break;
            }
        }
        return {
            x: geom.x + x0 / scale,
            y: geom.y + y0 / scale,
            width: (x1 - x0 + 1) / scale,
            height: (y1 - y0 + 1) / scale,
            ...(centroid ? { centroid } : {}),
            headTop: geom.y + headTopPx / scale,
            feetBottom: geom.y + feetBotPx / scale,
        };
    } catch {
        return { x: geom.x, y: geom.y, width: geom.width, height: geom.height };
    } finally {
        rt.destroy(true);
    }
}

/** How much the "authored" framing zooms OUT from the character's visible bounds,
 * so the subject reads at ~half the frame with the surrounding scene around it —
 * matching the in-game archive view. Character-adaptive: a lone character gets
 * breathing room, while an E2 whose creatures/scene ARE the visible bounds is shown
 * whole at the same factor. 1.7 frames the skin character at ~40% with the FULL
 * mirror-world scene around her (1.3 cropped the scene sides — read as "too zoomed";
 * a fixed 2× camera-height framed her too small at ~35%). */
const SCENE_ZOOM_OUT = 1.7;

/** Expand a bounds box around its centre by `factor` (>1 zooms the framing out). */
function inflateBounds(bounds: IAnimationBounds | null, factor: number): IAnimationBounds | null {
    if (!bounds) return null;
    const w = bounds.width * factor;
    const h = bounds.height * factor;
    return { x: bounds.x - (w - bounds.width) / 2, y: bounds.y - (h - bounds.height) / 2, width: w, height: h };
}

/** A global calibration constant, overridable via a URL search param (e.g. `?rcal=0.6`)
 *  for on-page tuning against reference recordings. */
function calibrationParam(name: string, fallback: number): number {
    try {
        return parseFloat(new URLSearchParams(window.location.search).get(name) || String(fallback)) || fallback;
    } catch {
        return fallback;
    }
}

/** A spine BACKDROP element that is a dark shadow / smoke silhouette (named as
 *  such and behind the character). */
const DARK_BACKDROP_SLOT = /^bg_.*(shadow|smoke|somke|silhou|black|hei|ying)/i;

/** When a static illustration is composited behind the spine, the static art
 *  already contains every backdrop element, PROPERLY layered by the game (e.g.
 *  Pepe's black shadow-cat sits BEHIND the purple nebula in the art). But the
 *  spine re-draws those dark elements as flat, opaque slots ON TOP of the static,
 *  blotting out the nebula/wall behind them. Since the static provides them
 *  faithfully — and the animated character still renders over the static — we hide
 *  the spine's dark backdrop silhouettes so the correctly-composited static shows
 *  through. Light, well-aligned backdrop elements (doves, streamers, petals) are
 *  left animating; the character's own attached shadows aren't `BG_`-prefixed, so
 *  they're untouched. (Verified harmless on Hoshiguma-alter, whose `BG_Door_*_shadow`
 *  mirror-panel shadows are re-supplied identically by the static art.) */
function hasShadowSlots(spine: import("pixi-spine").Spine): boolean {
    return spine.skeleton.slots.some((s) => DARK_BACKDROP_SLOT.test((s as unknown as { data: { name: string } }).data.name));
}

/** Make the matched slots' per-slot display objects non-renderable. pixi-spine
 *  rebuilds the geometry from the (re-attached) attachments on every `update`, so
 *  this must run each frame AFTER `update`; flipping the already-built mesh's
 *  `renderable` flag is what actually keeps them hidden. */
function hideRedundantShadowSlots(spine: import("pixi-spine").Spine): void {
    for (const slotU of spine.skeleton.slots) {
        const slot = slotU as unknown as { data: { name: string }; currentMesh?: { renderable: boolean }; currentSprite?: { renderable: boolean }; currentGraphics?: { renderable: boolean } };
        if (!DARK_BACKDROP_SLOT.test(slot.data.name)) continue;
        if (slot.currentMesh) slot.currentMesh.renderable = false;
        if (slot.currentSprite) slot.currentSprite.renderable = false;
        if (slot.currentGraphics) slot.currentGraphics.renderable = false;
    }
}

interface ILoadedBackdrop {
    texture: PIXI.Texture;
    /** Alpha-weighted centroid of the illustration, normalised to [0,1] of the
     *  texture (0.5,0.5 when it can't be measured). */
    centroid: { nx: number; ny: number };
}

/**
 * A fully-built, framed dynamic-illustration composite (character spine + its
 * optional mesh-scene layers + particle systems), ready to be shown but not yet
 * attached to the stage or started. Building it as a self-contained unit lets the
 * loader assemble MORE THAN ONE — a special "_Start" ENTRANCE composite and the
 * settled main L2D — and hand off between them (play the entrance once, then swap
 * to the main idle cycle) without duplicating the compositing/framing machinery.
 */
interface IComposite {
    spine: import("pixi-spine").Spine;
    /** The object to add to the stage (or point the HDR pass at) and lay out — the
     *  scene container when there are mesh layers/particles, else the bare spine. */
    root: PIXI.Container;
    /** True when `root` is a scene container (mesh layers/particles), false when it
     *  is the bare spine (spine-only art with no separate scene). */
    isScene: boolean;
    bounds: IAnimationBounds | null;
    /** The game's AUTHORED display frame in frontend coords (`cameraViewPx` square
     *  centred on the Y-flipped `cameraOffsetPx`) — the exact camera the in-game
     *  viewer frames these skins with. Used to frame the special-entry entrance skins
     *  (and their idle) to match the game rather than the character-bounds heuristic.
     *  Null for spine-only art with no authored camera. */
    authoredDisplayBounds: IAnimationBounds | null;
    /** The game's TIGHT/zoomed-in display frame (`_adjustes[1]`, e.g. Virtuosa 1246 vs
     *  the 1929 wide) in frontend coords — the close-up the in-game viewer opens on and
     *  dollies OUT from. Same vis centre as {@link authoredDisplayBounds}, shifted by the
     *  authored offset delta (the game's small pan). Null when the skin has no tight
     *  endpoint; the opener then derives the tight box from the 0.646 game ratio. */
    authoredTightBounds: IAnimationBounds | null;
    /** The TIGHT `_adjustes[1]` stop as a FRACTION of the wide display view (`viewPx2/viewPx`).
     *  Lets the settled main idle open on the SAME close-up the entrance held before pulling
     *  out to the wide frame. Null when the skin ships no tight stop. */
    entranceViewRatio: number | null;
    /** ENTRANCE (`_Start`) timing from the scene gamedata (seconds): total `duration`
     *  and the `transform` reform beat. Non-null only for a "_Start" entrance composite;
     *  drives the tight→wide camera dolly length and the hand-off time. */
    entranceDuration: number | null;
    entranceTransform: number | null;
    /** The `_Start` camera dolly ZOOM curve (`[t_s, ortho]` keyframes) — the game's actual
     *  data-driven camera motion (extracted from the clip animating the Main Camera's orthographic
     *  size). Scales the entrance frame relative to its base extent as the shot pushes in/out. */
    entranceOrthoCurve: [number, number][] | null;
    /** The `_Start` camera's ABSOLUTE frame-centre track (`[t_s, cx, cy]`, mesh px) — the game's
     *  own camera rig (the animated camera-parent Transform) accumulated by the Rust exporter into
     *  a world-space centre curve. Drives the entrance pan/dolly directly; no measured bounds. */
    entranceCamCenterCurve: [number, number, number][] | null;
    /** The entrance frame EXTENT in mesh px (`entranceViewPx`, the `_Start` camera's own view,
     *  falling back to the tight `_adjustes[1]` stop) — the base frame size the live camera holds,
     *  scaled by the ortho-size ratio each tick. Same 100 px/world-unit scale as
     *  {@link entranceCamCenterCurve}, so they compose directly. */
    entranceFrameSize: number | null;
    /** The entrance's scene-mesh containers (background + foreground) — carry the per-layer
     *  `m_IsActive` gating meshes. Entrance-only. */
    sceneLayers: PIXI.Container[] | null;
    particles: ILoadedParticles | null;
    /** Setup-pose bone matrices for bone-following particle emitters (scene only). */
    boneRest: Map<string, PIXI.Matrix> | null;
    /** Whether the spine's redundant dark shadow slots are being hidden each frame. */
    hasShadow: boolean;
    /** Start this composite's playback: the idle+specials cycle for the main L2D, or
     *  the one-shot entrance for a "_Start" composite. */
    play: (opts?: { skipStart?: boolean }) => void;
    /** Free the composite's own GPU resources (particles + container/ spine). */
    destroy: () => void;
}

/** Load the static illustration as a texture AND compute its opaque centroid, so
 *  it can be registered centroid-to-centroid against the spine (both depict the
 *  same art, so their mass centres correspond — robust to composition, unlike a
 *  bounding-box centre which the arch/railing skew). */
function loadImageTexture(url: string): Promise<ILoadedBackdrop> {
    return new Promise((resolve, reject) => {
        const img = new Image();
        img.crossOrigin = "anonymous";
        img.onload = () => {
            const texture = PIXI.Texture.from(img);
            let centroid = { nx: 0.5, ny: 0.5 };
            try {
                const w = img.naturalWidth || img.width;
                const h = img.naturalHeight || img.height;
                const S = 200;
                const scale = Math.min(S / w, S / h, 1);
                const cw = Math.max(1, Math.round(w * scale));
                const ch = Math.max(1, Math.round(h * scale));
                const canvas = document.createElement("canvas");
                canvas.width = cw;
                canvas.height = ch;
                const ctx = canvas.getContext("2d", { willReadFrequently: true });
                if (ctx) {
                    ctx.drawImage(img, 0, 0, cw, ch);
                    const px = ctx.getImageData(0, 0, cw, ch).data;
                    let sx = 0;
                    let sy = 0;
                    let sw = 0;
                    for (let y = 0; y < ch; y++) {
                        for (let x = 0; x < cw; x++) {
                            const a = px[(y * cw + x) * 4 + 3];
                            if (a > 8) {
                                sx += x * a;
                                sy += y * a;
                                sw += a;
                            }
                        }
                    }
                    if (sw > 0) centroid = { nx: sx / sw / cw, ny: sy / sw / ch };
                }
            } catch {
                /* keep default 0.5,0.5 */
            }
            resolve({ texture, centroid });
        };
        img.onerror = () => reject(new Error(`Failed to load backdrop: ${url}`));
        img.src = url;
    });
}

/** Build the static-illustration backdrop sprite. The static art is the camera's
 *  view of the scene — it spans `2 × cameraSizePx` of scene height — so we scale
 *  it to that and centre it on the animated character's visible bounds (the focal
 *  point both the static art and the spine are composed around). This aligns the
 *  spine over its own static counterpart so the missing backdrop fills in behind. */
function makeBackdropSprite(backdrop: ILoadedBackdrop, frame: ISceneFrame, spineCentroid: { x: number; y: number }): PIXI.Sprite {
    const { texture, centroid } = backdrop;
    const sprite = new PIXI.Sprite(texture);
    // Anchor at the illustration's own centroid so `position` places THAT point;
    // register it onto the spine's centroid so the two illustrations overlap by
    // mass (robust where a bounding-box centre is skewed by an arch/railing).
    sprite.anchor.set(centroid.nx, centroid.ny);
    sprite.scale.set((2 * frame.cameraSizePx) / Math.max(texture.width || 1, texture.height || 1));
    sprite.position.set(spineCentroid.x, spineCentroid.y);
    return sprite;
}

/** The in-game L2D viewer frames the illustration over a bright STUDIO ENVIRONMENT — a
 *  light-grey backdrop lit from above — NOT black. That environment is a generic viewer
 *  asset, not part of any skin's extracted scene, so without it the illustration reads far
 *  too dark against the page. We reproduce it as a full-viewport radial gradient (brighter
 *  toward the top, where the studio light falls) rendered BEHIND the scene, so the mesh
 *  layers composite over it at the game's brightness. `resizeEnvironmentBg` keeps it
 *  covering the viewport. */
// Fresh per-app texture (the app is destroyed with `texture: true`, so a shared/cached
// texture would be torn down under later mounts).
function createEnvironmentBgTexture(): PIXI.Texture {
    const S = 512;
    const cvs = document.createElement("canvas");
    cvs.width = S;
    cvs.height = S;
    const ctx = cvs.getContext("2d");
    if (ctx) {
        const g = ctx.createRadialGradient(S / 2, S * 0.3, S * 0.08, S / 2, S * 0.5, S * 0.78);
        g.addColorStop(0, "#dddee2");
        g.addColorStop(1, "#bfc0c4");
        ctx.fillStyle = g;
        ctx.fillRect(0, 0, S, S);
    }
    return PIXI.Texture.from(cvs);
}
function resizeEnvironmentBg(sprite: PIXI.Sprite, w: number, h: number): void {
    sprite.width = w;
    sprite.height = h;
}

/**
 * Plays an operator's dynamic (L2D) illustration. The `dyn_illust_*.skel`
 * skeleton already contains the ENTIRE scene — painted background, effect/glass
 * layers, sparkles — as Spine slots (with their blend modes baked in) and mesh
 * attachments, so we simply render the skeleton and frame it to its own animation
 * bounds (the same approach as the reference aklive2d renderer). pixi-spine
 * honours each slot's blend mode, reproducing the game's compositing.
 */
export function SceneIllust({ files, server, fit = DEFAULT_SPINE_FIT, framing = "character", backdrop, onReady }: ISceneIllustProps) {
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<import("pixi-spine").Spine | null>(null);
    const boundsRef = useRef<IAnimationBounds | null>(null);
    // When the skin's backdrop lives in separate mesh layers (not the spine), the
    // spine is nested with them in this container and the whole scene is framed
    // together (see buildComposite); null for spine-only art.
    const sceneContainerRef = useRef<PIXI.Container | null>(null);
    // Live-simulated Unity particle systems (bubbles/sparkles/embers), when the
    // skin exports a `[particles].json`; composited among the scene layers.
    const particlesRef = useRef<ILoadedParticles | null>(null);
    // Setup-pose world position of every spine bone, captured before the idle
    // animation starts. Bone-parented emitters reference this (see driftWithBone):
    // their `pos` is baked at the bind/setup pose, so the follow-delta must be
    // measured from setup, not the first animated frame.
    const boneRestRef = useRef<Map<string, PIXI.Matrix> | null>(null);
    // HDR bloom pass: when float render targets are available, the scene is drawn
    // into `hdr.target` (half-float, additive stacks don't clip) and tonemapped to
    // screen via `hdr.mesh`. `hdrSceneRef` is the container rendered into it.
    const hdrRef = useRef<IHDRScene | null>(null);
    const hdrSceneRef = useRef<PIXI.Container | null>(null);
    const envBgRef = useRef<PIXI.Sprite | null>(null);
    // Every composite built for the current skin (usually one; two while a "_Start"
    // entrance is playing before it hands off to the settled main L2D). Tracked so
    // cleanup frees them all — under the HDR pass their containers live OUTSIDE the
    // stage, so app.destroy() can't reach them.
    const compositesRef = useRef<IComposite[]>([]);
    // Set when the entrance's "Start" completes; the tick runs it AFTER the spine
    // update returns (the completion fires mid-update, so freeing the entrance spine
    // there would corrupt the in-progress update).
    const doSwapRef = useRef<(() => void) | null>(null);
    // Camera dolly ZOOM, driven by GAMEDATA: the `_Start` clip animates the Main Camera's
    // orthographic size (verified by extracting the curve — Virtuosa holds 1.87, zooms in to 1.50
    // on the transform at ~8.6s, out to 1.91 for the standing reveal). The camera itself has NO
    // positional curve — the pan lives in the animated camera-PARENT rig (`camCenter`). Each
    // frame we sample the curve at the clip's track time and scale the entrance frame by the ratio
    // to its t=0 value (so no world↔authored unit conversion), zooming around the frame centre.
    const entranceFollowRef = useRef<{
        spine: import("pixi-spine").Spine;
        root: PIXI.Container;
        ortho: [number, number][] | null;
        camCenter: [number, number, number][];
        frameSize: number;
        sceneLayers: PIXI.Container[];
    } | null>(null);
    // Crossfade the `_Start` cinematic OUT (frozen at its dissolved final frame) while the main
    // gala idle fades IN — so the character is never absent during the transformation (the game
    // overlaps the reform with the dissolve). Both roots are wrapped in one container the HDR
    // pass renders; the tick ramps their alphas, then detaches the main and frees the entrance.
    const crossfadeRef = useRef<{ wrapper: PIXI.Container; mainRoot: PIXI.Container; entRoot: PIXI.Container; ent: IComposite; elapsed: number; duration: number } | null>(null);
    // The opening zoom: the in-game viewer opens on a tight close-up of the character
    // and zooms OUT to the steady framing over a fraction of a second, then holds. This
    // interpolates the entrance's framing from `from` (close) to `to` (steady) over
    // `duration` seconds of REAL time (not animation progress — the zoom is a fixed
    // ~0.7s regardless of the 17s entrance), then clears itself so the framing holds.
    const entranceZoomRef = useRef<{ container: PIXI.Container; from: IAnimationBounds; to: IAnimationBounds; elapsed: number; duration: number; delay: number } | null>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const mountedRef = useRef(true);
    const loadIdRef = useRef(0);
    const onReadyRef = useRef(onReady);
    onReadyRef.current = onReady;
    // "authored" (the in-game viewer) scales the crop to the container HEIGHT — the character
    // fills a constant fraction of the frame height on ANY container aspect (tall mobile card,
    // wide desktop card, fullscreen dialog), and the excess/short width shows more/less scene.
    // A square+`cover` would instead inflate the character with the container's aspect ratio
    // (game-sized only at ~2.16:1, ~2× too small on a tall mobile card). Character-framing
    // contexts keep the caller's fit (contain).
    const fitRef = useRef(fit);
    fitRef.current = framing === "authored" ? { mode: "height", align: "center" } : fit;
    const framingRef = useRef(framing);
    framingRef.current = framing;
    // When compositing a static backdrop, the spine's dark shadow silhouettes are
    // re-hidden every frame (the idle clip re-attaches them each `update`).
    const hideShadowsRef = useRef(false);

    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    // The skin's L2D has no displayable resting frame (an entrance-only skeleton);
    // we silently keep the static charart rather than show a broken animation.
    const [unsupported, setUnsupported] = useState(false);

    useEffect(() => {
        const container = containerRef.current;
        if (!container) return;
        if (!files.skel || !files.atlas || !files.png) return;

        const skelPath = files.skel;
        const atlasPath = files.atlas;

        mountedRef.current = true;
        const currentLoadId = ++loadIdRef.current;
        let animationFrameId: number | null = null;

        const cleanup = () => {
            if (animationFrameId) cancelAnimationFrame(animationFrameId);
            hideShadowsRef.current = false;
            spineRef.current = null;
            sceneContainerRef.current = null;
            // Particles are owned by their composite (freed below); just drop the ref.
            particlesRef.current = null;
            boneRestRef.current = null;
            doSwapRef.current = null;
            entranceZoomRef.current = null;
            entranceFollowRef.current = null;
            if (crossfadeRef.current) {
                // Mid-crossfade: detach the main (freed via compositesRef below) and drop the wrapper.
                crossfadeRef.current.wrapper.removeChild(crossfadeRef.current.mainRoot);
                crossfadeRef.current.wrapper.destroy({ children: false });
                crossfadeRef.current = null;
            }
            // Free every built composite. Under the HDR pass a composite's container
            // lives OUTSIDE the stage, so app.destroy() won't reach it. Detach each
            // from its parent FIRST so the later app.destroy() can't double-free an
            // on-stage one, then destroy it (particles-before-container, the order the
            // display objects can be torn down safely).
            for (const c of compositesRef.current) {
                c.root.parent?.removeChild(c.root);
                c.destroy();
            }
            compositesRef.current = [];
            hdrRef.current?.destroy();
            hdrRef.current = null;
            hdrSceneRef.current = null;
            envBgRef.current = null;
            if (appRef.current) {
                appRef.current.destroy(true, { children: true, texture: true });
                appRef.current = null;
            }
        };

        cleanup();
        while (container.firstChild) container.removeChild(container.firstChild);
        setIsLoading(true);
        setError(null);
        setUnsupported(false);

        const app = new PIXI.Application({
            width: container.clientWidth || 600,
            height: container.clientHeight || 450,
            backgroundAlpha: 0,
            antialias: true,
            resolution: typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1,
            autoDensity: true,
        });
        appRef.current = app;
        container.appendChild(app.view as HTMLCanvasElement);

        let lastTick = performance.now();
        const tick = (now: number) => {
            if (!mountedRef.current) return;
            const dt = Math.min((now - lastTick) / 1000, 0.1);
            lastTick = now;
            if (spineRef.current) {
                spineRef.current.update(dt);
                // `update` rebuilds the dark shadow slots' meshes each frame; flip them
                // back to non-renderable so the static backdrop's version shows instead.
                if (hideShadowsRef.current) hideRedundantShadowSlots(spineRef.current);
            }
            // ENTRANCE camera = the game's OWN camera rig, replayed straight from gamedata.
            // The `_Start` prefab animates the camera's parent Transform (a position curve) as the
            // seated form reforms into the standing form ~3300px higher; the Rust exporter accumulates
            // that camera chain into an ABSOLUTE frame-centre curve (`entranceCamCenterCurve`, mesh px)
            // and pairs it with the authored ortho-size zoom. We sample the centre and scale the frame
            // by the ortho ratio — no measured bounds, no easing, no tuning: purely the rig's motion.
            const ef = entranceFollowRef.current;
            if (ef && appRef.current && spineRef.current === ef.spine) {
                const tr = ef.spine.state.tracks[0] as unknown as { trackTime?: number } | null;
                const tt = tr?.trackTime ?? 0;
                const { width: sw, height: sh } = appRef.current.screen;
                // Camera CENTRE: sample the accumulated gamedata camera track at the clip time. Its
                // keyframe timing IS the game's dolly/pan — replay it directly.
                const c = sampleCurveXY(ef.camCenter, tt) ?? [0, 0];
                // Camera SIZE: the gamedata frame extent (`_adjustes` view px) × the ortho-size ratio,
                // so the character grows into the frame exactly as the authored zoom dictates.
                const size = ef.frameSize * orthoZoomRatio(ef.ortho, tt);
                layoutSpine(ef.root, sw, sh, { x: c[0] - size / 2, y: c[1] - size / 2, width: size, height: size }, fitRef.current);
                // Per-layer `m_IsActive` window from the `_Start` clips (gamedata): a layer with
                // `activeFrom`/`activeUntil` renders only while `activeFrom <= t < activeUntil`.
                // Absent = always visible (Virtuosa's backdrop is entirely always-on).
                for (const c of ef.sceneLayers) {
                    for (const m of c.children) {
                        const mm = m as unknown as { __activeFrom?: number | null; __activeUntil?: number | null };
                        const af = mm.__activeFrom;
                        const au = mm.__activeUntil;
                        if (af != null || au != null) m.renderable = (af == null || tt >= af) && (au == null || tt < au);
                    }
                }
            }
            // Hand off from the entrance to the main L2D, if its "Start" just finished.
            // Deferred to here (out of the entrance spine's `complete` listener) so the
            // entrance can be freed without corrupting the update it fired from. Runs
            // before the particle update below so that reads the swapped-in main refs.
            if (doSwapRef.current) {
                const swap = doSwapRef.current;
                doSwapRef.current = null;
                swap();
            }
            // Opening zoom-out (matches the in-game viewer, measured from the reference
            // recording): open TIGHT on the character, hold for `delay`s, then dolly OUT to
            // the steady framing over `duration`s of real time, then clear so it holds.
            const ez = entranceZoomRef.current;
            if (ez && appRef.current) {
                ez.elapsed += dt;
                const active = Math.max(0, ez.elapsed - ez.delay); // hold at `from` during the delay
                const t = Math.min(1, active / ez.duration);
                const e = t * t * (3 - 2 * t); // smoothstep — symmetric ease for the dolly-out
                const from = ez.from;
                const to = ez.to;
                const b = {
                    x: from.x + (to.x - from.x) * e,
                    y: from.y + (to.y - from.y) * e,
                    width: from.width + (to.width - from.width) * e,
                    height: from.height + (to.height - from.height) * e,
                };
                const { width: sw, height: sh } = appRef.current.screen;
                layoutSpine(ez.container, sw, sh, b, fitRef.current);
                if (t >= 1) entranceZoomRef.current = null;
            }
            // Entrance→main crossfade: ramp the main IN and the dissolved entrance OUT, then
            // detach the main (so it survives), free the entrance, and point HDR back at the main.
            const cf = crossfadeRef.current;
            if (cf) {
                cf.elapsed += dt;
                const t = Math.min(1, cf.elapsed / cf.duration);
                cf.mainRoot.alpha = t;
                cf.entRoot.alpha = 1 - t;
                if (t >= 1) {
                    crossfadeRef.current = null;
                    cf.mainRoot.alpha = 1;
                    cf.wrapper.removeChild(cf.mainRoot); // keep the main; it renders directly again
                    hdrSceneRef.current = cf.mainRoot;
                    cf.ent.destroy(); // frees the entrance root (detaches it from the wrapper)
                    const i = compositesRef.current.indexOf(cf.ent);
                    if (i >= 0) compositesRef.current.splice(i, 1);
                    cf.wrapper.destroy({ children: false });
                }
            }
            if (particlesRef.current) {
                // Let bone-parented emitters drift with the character's idle sway:
                // pass a live spine-bone lookup (spine.update above already ran, so
                // bone world transforms are current this frame).
                const sp = spineRef.current;
                const findBone: FindBone | undefined = sp
                    ? (name: string) => {
                          const b = sp.skeleton.findBone(name) as unknown as { matrix: PIXI.Matrix } | null;
                          return b ? b.matrix : null;
                      }
                    : undefined;
                particlesRef.current.update(dt, findBone, boneRestRef.current ?? undefined);
            }
            const currentApp = appRef.current;
            if (currentApp?.renderer) {
                // HDR path: draw the scene into the half-float target (additive stacks
                // accumulate past 1 without clipping), then let the stage's tonemap quad
                // blit it to screen. Otherwise render the stage straight (8-bit).
                if (hdrRef.current && hdrSceneRef.current) {
                    currentApp.renderer.render(hdrSceneRef.current, { renderTexture: hdrRef.current.target, clear: true });
                    // Update the bloom texture from the freshly-drawn target before the
                    // stage's tonemap quad (which samples both) blits to screen.
                    hdrRef.current.prepare(currentApp.renderer);
                }
                currentApp.renderer.render(currentApp.stage);
            }
            animationFrameId = requestAnimationFrame(tick);
        };
        animationFrameId = requestAnimationFrame(tick);

        const aborted = () => currentLoadId !== loadIdRef.current || !mountedRef.current || !appRef.current;

        // In dev, cache-bust the scene/particle assets so re-extracted data is picked
        // up immediately (they're served with a long Cache-Control that otherwise masks
        // changes). No-op in prod.
        const bust = import.meta.env.DEV ? `?v=${Date.now()}` : "";

        // Build one framed dynamic-illustration composite (character spine + optional
        // mesh-scene layers + particles) from a skel/atlas pair. It is fully laid out
        // but NOT yet attached to the stage or started — the orchestrator below decides
        // attach + playback so a special "_Start" ENTRANCE composite can play first and
        // hand off to the settled main L2D. Returns "unsupported" when the MAIN set has
        // no resting frame (entrance-only skeleton → keep the static charart), or null
        // when the load is superseded/aborted or an optional entrance set is absent.
        const buildComposite = async (cSkel: string, cAtlas: string, opts: { mode: "main" | "entrance"; framingOverride?: IAnimationBounds | null; onEntranceEnd?: () => void }): Promise<IComposite | "unsupported" | null> => {
            let spine: import("pixi-spine").Spine;
            try {
                spine = await loadSpineWithEncodedURLs(cSkel, cAtlas, server);
            } catch (e) {
                // An entrance set is OPTIONAL — a missing/failed "_Start" just means the
                // skin has no cinematic entrance. A main-set failure propagates.
                if (opts.mode === "entrance") return null;
                throw e;
            }
            if (aborted()) {
                spine.destroy();
                return null;
            }
            const app = appRef.current;
            if (!app) {
                spine.destroy();
                return null;
            }

            spine.autoUpdate = false;
            spine.state.timeScale = ANIMATION_SPEED;
            spine.scale.set(1);
            spine.position.set(0, 0);

            const animations = spine.spineData.animations.map((a: { name: string }) => a.name);
            const idle = animations.includes(IDLE_ANIMATION) ? IDLE_ANIMATION : (animations[0] ?? IDLE_ANIMATION);

            // A MAIN skeleton with no looping "Idle" (only a one-shot "Start" entrance)
            // has no resting frame to display — the entrance never settles into the
            // illustration (e.g. Kal'tsit "Remnant", whose pieces stay converged).
            // Silently fall back to the static charart.
            if (opts.mode === "main" && !animations.includes(IDLE_ANIMATION)) {
                spine.destroy();
                return "unsupported";
            }

            // Periodic "Special" sequences the in-game archive interleaves with the idle
            // loop — the L2D isn't a single static loop, it performs dramatic beats over
            // time (Hoshiguma the Breacher's sword flourishes / sakura "Special_Flower"
            // storm). Collect every non-idle SPECIAL clip (skip the one-shot Start
            // entrance and the tap-only Interact, which isn't auto-played in the archive).
            const specials = animations.filter((n) => /special/i.test(n) && n !== idle && !/_a\d*$/i.test(n));
            // The entrance skeleton has a single "Start" animation (the cinematic zoom-in).
            const entranceAnim = animations.includes("Start") ? "Start" : (animations[0] ?? "Start");
            let entranceEnded = false;
            const play = (playOpts?: { skipStart?: boolean }) => {
                const state = spine.state;
                if (opts.mode === "entrance") {
                    // Play the cinematic entrance ONCE; when it finishes, hand off to the
                    // main L2D (which is already built and framed identically, so the
                    // settled final frame flows straight into the idle loop).
                    state.setAnimation(0, entranceAnim, false);
                    state.addListener({
                        complete: (entry) => {
                            const name = (entry as unknown as { animation?: { name?: string } }).animation?.name;
                            if (name !== entranceAnim || entranceEnded) return;
                            entranceEnded = true;
                            opts.onEntranceEnd?.();
                        },
                    });
                    return;
                }
                // Main: loop "Idle", interleaving Specials. Cross-fade so flourishes blend
                // in/out, not snap.
                if (state.data) state.data.defaultMix = 0.35;
                // The in-game viewer opens on the character's own one-shot "Start" beat (synced
                // with the opening camera dolly), then settles into the looping idle. Play it
                // once when present, else start straight on the idle loop. When handing off FROM
                // the `_Start` cinematic (skipStart), go straight to the idle so the gala cellist
                // appears promptly as the cinematic dissolves — no slow second intro beat.
                if (!playOpts?.skipStart && animations.includes("Start") && idle !== "Start") {
                    state.setAnimation(0, "Start", false);
                    state.addAnimation(0, idle, true, 0);
                } else {
                    state.setAnimation(0, idle, true);
                }
                if (specials.length === 0) return;
                // After every couple of idle loops, play one Special, then fall back to the
                // idle loop; cycle through the variants for variety.
                let idleLoops = 0;
                let si = 0;
                const LOOPS_BETWEEN = 2;
                state.addListener({
                    complete: (entry) => {
                        const name = (entry as unknown as { animation?: { name?: string } }).animation?.name;
                        if (name !== idle) return;
                        idleLoops += 1;
                        if (idleLoops < LOOPS_BETWEEN) return;
                        idleLoops = 0;
                        const special = specials[si % specials.length];
                        si += 1;
                        state.setAnimation(0, special, false);
                        state.addAnimation(0, idle, true, 0);
                    },
                });
            };

            // Some skins keep their painted backdrop in separate mesh layers (not the
            // spine). Load them if present; they share the spine's coordinate space, so we
            // nest the spine among them and frame the whole scene to the authored camera.
            const sceneUrl = chibiAssetURL(cSkel.replace(/\.skel$/, "[scene].json"), server);
            const textureBaseUrl = chibiAssetURL(cSkel.replace(/\.skel$/, "[scene]/"), server);
            const scene = await loadSceneMeshes(sceneUrl + bust, textureBaseUrl, bust);
            if (aborted()) {
                spine.destroy();
                return null;
            }

            // Static-art backdrop: some dynamic assets omit the full painted vista
            // (sky/interior) the static illustration has. Draw the static art behind the
            // spine so the animated character overlays its own static counterpart and the
            // missing backdrop fills in. Only for the MAIN set (entrance sets always carry
            // their own scene). The frame comes from the scene data when present, else a
            // bare frame JSON (spine-only skins, e.g. Siege).
            let backdropData: ILoadedBackdrop | null = null;
            let backdropFrame: ISceneFrame | null = null;
            if (backdrop && opts.mode === "main") {
                backdropFrame = (scene && sceneFrameOf(scene.data)) || (await loadSceneFrame(sceneUrl + bust));
                if (backdropFrame) {
                    try {
                        backdropData = await loadImageTexture(backdrop);
                        if (aborted()) {
                            spine.destroy();
                            return null;
                        }
                    } catch {
                        backdropFrame = null; // backdrop failed → spine-only
                    }
                }
            }
            const hasBackdrop = !!(backdropData && backdropFrame);

            const { width, height } = app.screen;

            if (scene || hasBackdrop) {
                // Particle systems live in the same coordinate space; load and interleave
                // them: behind-character emitters just in front of the backdrop, in-front
                // emitters above the foreground layers.
                let particles: ILoadedParticles | null = null;
                if (scene) {
                    const particlesUrl = chibiAssetURL(cSkel.replace(/\.skel$/, "[particles].json"), server);
                    const particlesTexBase = chibiAssetURL(cSkel.replace(/\.skel$/, "[particles]/"), server);
                    particles = await loadParticles(particlesUrl + bust, particlesTexBase, bust);
                    if (aborted()) {
                        spine.destroy();
                        particles?.destroy();
                        return null;
                    }
                }
                spine.scale.set(1);
                spine.position.set(0, 0);
                const sceneContainer = new PIXI.Container();
                // Frame to the character's visible bounds; "authored" (fullscreen) zooms
                // OUT to reveal more of the surrounding scene, "character" (card) keeps the
                // character prominent. Measured first (renders the spine to its own
                // offscreen RT) so the static-backdrop decision can register against the
                // character's centroid. Skipped when a framingOverride is supplied (the
                // entrance reuses the main's bounds — see the orchestrator).
                const vis = opts.framingOverride ? null : (measureVisibleBounds(app.renderer, spine, idle) ?? measureAnimationBounds(spine, idle));
                const spineCentroid = (() => {
                    const vc = vis && (vis as IVisBounds).centroid;
                    return vc ? { x: vc.cx, y: vc.cy } : vis ? { x: vis.x + vis.width / 2, y: vis.y + vis.height / 2 } : { x: 0, y: 0 };
                })();

                // Decide whether the static illustration should stand in for the scene's
                // painted BACKGROUND. The static is a full opaque image, so it can only go
                // behind everything (z=0); placed there it fills wherever the dynamic is
                // transparent. That's what we want when the dynamic OMITS its backdrop (Siege:
                // floating character; Surtr/Pepe: sparse/translucent vista). But when the
                // dynamic carries its OWN painted scene meshes it is self-composed — the static
                // would re-draw the character offset from the L2D one (the "two of her"
                // duplicate: svash2's ice, Nearl "Relight"'s seated throne — the static's
                // character peeks beyond the spine silhouette in the scene's open areas, and no
                // centroid registration lands it exactly when a full scene competes). So: any
                // scene layer at all ⇒ drop the static. Only spine-only art (Siege — a BARE
                // `[scene].json`, 0 layers, or no scene) keeps it.
                const sceneLayerCount = scene ? (scene.background?.children?.length ?? 0) + (scene.foreground?.children?.length ?? 0) : 0;
                const useStatic = hasBackdrop && !!backdropData && !!backdropFrame && !!vis && sceneLayerCount === 0;

                // Draw order (bottom → top) follows the game's true sort scale: every scene
                // layer and particle system carries its Unity `m_SortingOrder` (`sort`), and
                // the character spine sits at `characterSort`. Anything with sort <
                // characterSort is BEHIND the character, anything above is IN FRONT. The
                // loaders pre-split each source into `background`/`foreground` on that pivot,
                // so we render: backdrop layers → behind-particles → SPINE → foreground
                // layers → front-particles. When the static art stands in for the background
                // we DROP the scene's background layers so the two don't double/dim.
                // An ENTRANCE composite's scene meshes must not clutter the cinematic's early
                // phases (the settled scene — Virtuosa's crystal throne — is exported identical
                // to the main scene but only materialises mid-cinematic): each entrance layer
                // carries a per-mesh `m_IsActive` window from the `_Start` clips, applied every
                // frame by the tick (see entranceFollowRef), so layers show exactly when the
                // game toggles them.
                if (scene && !useStatic) sceneContainer.addChild(scene.background);
                if (particles) sceneContainer.addChild(particles.background);
                sceneContainer.addChild(spine);
                if (scene) sceneContainer.addChild(scene.foreground);
                if (particles) sceneContainer.addChild(particles.foreground);
                // Insert the static backdrop at the very back, registered centroid-to-
                // centroid onto the character (see makeBackdropSprite).
                if (useStatic && backdropData && backdropFrame) {
                    sceneContainer.addChildAt(makeBackdropSprite(backdropData, backdropFrame, spineCentroid), 0);
                }
                // Framing. When a framingOverride is given (the entrance), reuse it verbatim
                // so the entrance renders through the SAME authored camera box as the main
                // and its settled final frame aligns exactly with the idle. Otherwise:
                // fullscreen ("authored") reproduces the game's orthographic camera — a
                // square of visible HEIGHT `2 × cameraSizePx` centred on the character's own
                // visible-bounds centre (NOT the authored `cameraOffsetPx`, the mirror axis,
                // which would crop the head). This authored-camera extent is the game's
                // ground-truth framing, consistent across normal ops and the special-entry
                // skins. Falls back to the character-bounds zoom-out for spine-only ops that
                // carry no authored camera.
                const authoredFrame = scene ? sceneFrameOf(scene.data) : backdropFrame;
                // POSE-ROBUST special-entry framing. The character is located with its SOLID body
                // span (`headTop`/`feetBottom`, alpha-mass landmarks that skip thin protrusions the
                // bbox catches — a raised rifle above the head, wispy hair/dress tails below the
                // feet — the exact failure that shoved seated Executor to the frame bottom).
                // SIZE is SCENE-based (`cameraViewPx` = the game's `_adjustes[0]` display crop),
                // so the painted scene fills the frame — the character's on-screen size then
                // follows from its size in that scene (as in-game). VERTICAL position uses the
                // robust landmarks so the character sits right for ANY pose:
                //   settled: FEET-anchored — the base sits ~90% down (a ground line), the body
                //            rises with scene headroom above. Robust to a tall standing OR a
                //            compact seated char (both put their base on the ground line).
                //   open:    HEAD-anchored — the head sits ~8% down (a close-up filling the top),
                //            feet falling off the bottom; the tight endpoint the viewer dollies
                //            OUT from. Uses the SAME landmarks, just the opposite edge.
                // Horizontal centre = the scene camera x (symmetry axis), else the char centroid.
                const vb: IVisBounds | null = vis;
                const feetY = vb ? (vb.feetBottom ?? vb.y + vb.height) : 0;
                const headY = vb ? (vb.headTop ?? vb.y) : 0;
                const frameCx = authoredFrame?.offsetPx ? authoredFrame.offsetPx[0] : vb?.centroid ? vb.centroid.cx : vb ? vb.x + vb.width / 2 : 0;
                // GAMEDATA-DRIVEN framing: the settled crop SIZE is the exact per-skin
                // `cameraViewPx` (the display controller's `_adjustes[0]` stop); the OPEN endpoint
                // is the exact `cameraViewPx2` (`_adjustes[1]`), so the dolly MAGNITUDE the viewer
                // animates is the game's own ratio (Virtuosa 1246/1929 = 0.646×) — no hardcoded
                // per-skin numbers. Combined with the "height" fit (see `fitRef`), the crop's height
                // maps to the container height, so the character fills a constant fraction of the
                // frame height on ANY container aspect (game ~67%, measured against reference
                // recordings). `RCAL` is a SINGLE global pipeline constant (not per-skin) that
                // converts the game's authored `_adjustes` px into the render/vis space the mesh +
                // spine are measured in; it was calibrated once so `cameraViewPx·RCAL` yields the
                // game's ~67% character height. The box is CENTRED on the character body — the raw
                // `_adjustes` OFFSET points ~650px BELOW the body in export space (verified), so it
                // can't be used for centring; `VBIAS` corrects the residual hair/mass drag.
                const RCAL = calibrationParam("rcal", 0.606);
                // VBIAS: shift the crop centre UP (toward the head) by this fraction of the crop
                // size `e`. The `feetBottom`/head landmarks include the character's long trailing
                // hair/tail, which drags the naive body midpoint DOWN, floating the character too
                // high in-frame with dead scene below. A small upward bias re-centres so the head
                // sits ~18% down and feet ~85% down, matching the game's roughly-centred composition.
                const VBIAS = calibrationParam("vbias", 0.151);
                const bodyCx = vb?.centroid ? vb.centroid.cx : vb ? vb.x + vb.width / 2 : frameCx;
                const bodyCy = (headY + feetY) / 2; // vertical body centre
                /** A body-centred square crop of authored extent `viewPx`, in render/vis space. */
                const bodyFrameBox = (viewPx: number): IAnimationBounds => {
                    const e = viewPx * RCAL;
                    return { x: bodyCx - e / 2, y: bodyCy - VBIAS * e - e / 2, width: e, height: e };
                };
                const authoredDisplayBounds: IAnimationBounds | null = authoredFrame?.viewPx && vis ? bodyFrameBox(authoredFrame.viewPx) : null;
                // The TIGHT open endpoint: the exact `cameraViewPx2` box (2nd `_adjustes` stop),
                // same centre — the viewer dollies OUT from here. Null unless a 2nd camera stop.
                const authoredTightBounds: IAnimationBounds | null = authoredFrame?.viewPx2 && vis ? bodyFrameBox(authoredFrame.viewPx2 as number) : null;
                let bounds: IAnimationBounds | null;
                if (opts.framingOverride) {
                    bounds = opts.framingOverride;
                } else if (framingRef.current === "authored") {
                    if (authoredFrame && vis) {
                        const e = 2 * authoredFrame.cameraSizePx;
                        const cx = vis.x + vis.width / 2;
                        const cy = vis.y + vis.height / 2;
                        bounds = { x: cx - e / 2, y: cy - e / 2, width: e, height: e };
                    } else {
                        bounds = inflateBounds(vis, SCENE_ZOOM_OUT);
                    }
                } else {
                    bounds = vis;
                }
                layoutSpine(sceneContainer, width, height, bounds, fitRef.current);
                // Snapshot every bone's REFERENCE world matrix at the SETUP (bind) pose —
                // the pose the exported particle `pos` values were baked at. Bone-following
                // emitters use this as the delta reference (D = boneNow · boneRef⁻¹);
                // anchoring it to setup makes the flame ride the sword bone from its true
                // baked spawn rather than sitting offset (the "flame off the blade" bug).
                spine.skeleton.setToSetupPose();
                spine.skeleton.updateWorldTransform();
                const boneRest = new Map<string, PIXI.Matrix>();
                for (const b of spine.skeleton.bones as unknown as { data: { name: string }; matrix: PIXI.Matrix }[]) {
                    boneRest.set(b.data.name, b.matrix.clone());
                }
                const hasShadow = useStatic && hasShadowSlots(spine);
                return {
                    spine,
                    root: sceneContainer,
                    isScene: true,
                    bounds,
                    authoredDisplayBounds,
                    authoredTightBounds,
                    entranceViewRatio: authoredFrame?.viewPx2 && authoredFrame?.viewPx ? (authoredFrame.viewPx2 as number) / authoredFrame.viewPx : null,
                    entranceDuration: scene?.data.entranceDuration ?? null,
                    entranceTransform: scene?.data.entranceTransform ?? null,
                    entranceOrthoCurve: (scene?.data.entranceOrthoCurve as [number, number][] | undefined) ?? null,
                    entranceCamCenterCurve: (scene?.data.entranceCamCenterCurve as [number, number, number][] | undefined) ?? null,
                    entranceFrameSize: (authoredFrame?.entranceViewPx as number | undefined) ?? (authoredFrame?.viewPx2 as number | undefined) ?? null,
                    sceneLayers: opts.mode === "entrance" && scene ? [scene.background, scene.foreground] : null,
                    particles,
                    boneRest,
                    hasShadow,
                    play,
                    destroy: () => {
                        particles?.destroy();
                        sceneContainer.destroy({ children: true });
                    },
                };
            }

            // Spine-only art (no separate scene / static backdrop). Frame to the
            // illustration's actual visible pixels at rest so it sits centred regardless of
            // inflated skeleton geometry or motion extremes.
            const bounds = opts.framingOverride ?? measureVisibleBounds(app.renderer, spine, idle) ?? measureAnimationBounds(spine, idle);
            layoutSpine(spine, width, height, bounds, fitRef.current);
            return {
                spine,
                root: spine as unknown as PIXI.Container,
                isScene: false,
                bounds,
                authoredDisplayBounds: null,
                authoredTightBounds: null,
                entranceViewRatio: null,
                entranceDuration: null,
                entranceTransform: null,
                entranceOrthoCurve: null,
                entranceCamCenterCurve: null,
                entranceFrameSize: null,
                sceneLayers: null,
                particles: null,
                boneRest: null,
                hasShadow: false,
                play,
                destroy: () => {
                    spine.destroy();
                },
            };
        };

        const load = async () => {
            try {
                // Build the settled main L2D first (it determines the authored framing).
                const main = await buildComposite(skelPath, atlasPath, { mode: "main" });
                if (main === "unsupported") {
                    if (!aborted()) {
                        setIsLoading(false);
                        setUnsupported(true);
                    }
                    return;
                }
                if (!main) return; // superseded/aborted during build
                if (aborted()) {
                    main.destroy();
                    return;
                }
                const app = appRef.current;
                if (!app) {
                    main.destroy();
                    return;
                }

                const composites: IComposite[] = [main];
                compositesRef.current = composites;

                const { width, height } = app.screen;
                // Bright studio environment at the BACK of the stage (see createEnvironmentBgTexture): the
                // in-game viewer frames the L2D over a lit light-grey backdrop, not black. Added
                // here (after the scene has loaded) so it appears WITH the illustration rather than
                // covering the static load placeholder, and behind every scene layer so they
                // composite over it at the game's brightness.
                const envBg = new PIXI.Sprite(createEnvironmentBgTexture());
                resizeEnvironmentBg(envBg, width, height);
                envBgRef.current = envBg;
                app.stage.addChildAt(envBg, 0);
                // HDR bloom pass: render the scene into a half-float target so additive
                // light/flame stacks don't clip to white, then tonemap to screen. Created
                // once and re-pointed at whichever composite is live. Spine-only art never
                // used HDR, so gate on the main being a scene composite. Falls back to plain
                // 8-bit compositing (add the live composite straight to the stage) when
                // float targets aren't available.
                const hdr = main.isScene ? createHDRScene(app.renderer, width, height, app.renderer.resolution) : null;
                if (hdr) {
                    hdrRef.current = hdr;
                    app.stage.addChild(hdr.mesh);
                }

                // Make `c` the live composite: point the tick loop at it, attach it (under
                // the HDR pass, or straight onto the stage), and start its playback.
                const activate = (c: IComposite, playOpts?: { skipStart?: boolean }) => {
                    spineRef.current = c.spine;
                    particlesRef.current = c.particles;
                    boneRestRef.current = c.boneRest;
                    sceneContainerRef.current = c.isScene ? c.root : null;
                    boundsRef.current = c.bounds;
                    hideShadowsRef.current = c.hasShadow;
                    if (hdr) {
                        hdrSceneRef.current = c.root;
                    } else {
                        app.stage.addChild(c.root);
                    }
                    c.play(playOpts);
                    if (c.hasShadow) hideRedundantShadowSlots(c.spine);
                };

                const gameFrame = main.authoredDisplayBounds;
                const openTight = main.authoredTightBounds; // the game's `_adjustes[1]`, else null
                if (gameFrame) main.bounds = gameFrame; // the idle settles at the game display frame

                // When we arrive from the `_Start` cinematic, the settled idle continues the
                // camera move: the game HELD the tight close-up through the whole transform
                // (seated → apple → reform) and only pulls back to the wide throne AFTERWARD.
                // So the idle OPENS on the same tight close-up the entrance ended on — the wide
                // frame scaled by the entrance camera's ratio (`entranceViewRatio`) — and dollies
                // OUT to the wide throne over the POST-reform entrance time (`duration−transform`).
                // Set by the entrance branch below; null for the no-`_Start` fallback (which opens
                // on the idle `_adjustes[1]` tight stop instead).
                let entrancePullOut: { ratio: number; dur: number } | null = null;

                // Open on the standing idle at the settled frame, then dolly-out. Used both as the
                // fallback (no `_Start`) and as the hand-off target after the `_Start` cinematic.
                const openStandingIdle = () => {
                    if (aborted() || !appRef.current || !gameFrame) return;
                    const { width: sw, height: sh } = appRef.current.screen;
                    const openFrom = entrancePullOut ? inflateBounds(gameFrame, entrancePullOut.ratio) : openTight;
                    const dur = entrancePullOut ? entrancePullOut.dur : 2.0;
                    if (openFrom) {
                        layoutSpine(main.root, sw, sh, openFrom, fitRef.current);
                        entranceZoomRef.current = { container: main.root, from: openFrom, to: gameFrame, elapsed: 0, duration: dur, delay: 0.15 };
                    } else {
                        layoutSpine(main.root, sw, sh, gameFrame, fitRef.current);
                    }
                };

                // The in-game L2D viewer PERFORMS the skin's `_Start` cinematic as its entrance
                // (verified against game recordings: seated form → transform → dissolve, then the
                // standing idle). Play it ONCE on open over the bright env-bg, then hand off to the
                // standing idle + dolly. Probe by string-replacing the skel/atlas path; skins with
                // no `_Start` fall straight through to the plain open. The hand-off is DEFERRED out
                // of the entrance spine's `complete` listener (via doSwapRef) so freeing it can't
                // corrupt the update it fired from.
                const startSkel = skelPath.replace(/\.skel$/, "_Start.skel");
                const startAtlas = atlasPath.replace(/\.atlas$/, "_Start.atlas");
                const swapToMainIdle = () => {
                    if (aborted()) return;
                    const ent = composites.find((c) => c !== main);
                    entranceZoomRef.current = null;
                    entranceFollowRef.current = null;
                    activate(main, { skipStart: true }); // straight to standing idle (no second intro beat)
                    openStandingIdle();
                    if (ent && hdr) {
                        // Crossfade: render BOTH roots in one wrapper (HDR renders it), main behind
                        // fading in, the dissolved entrance on top fading out — the character reforms
                        // instead of blinking out. The tick ramps the alphas then frees the entrance.
                        const wrapper = new PIXI.Container();
                        wrapper.addChild(main.root);
                        wrapper.addChild(ent.root);
                        main.root.alpha = 0;
                        ent.root.alpha = 1;
                        hdrSceneRef.current = wrapper;
                        crossfadeRef.current = { wrapper, mainRoot: main.root, entRoot: ent.root, ent, elapsed: 0, duration: 0.45 };
                    } else if (ent) {
                        appRef.current?.stage.removeChild(ent.root);
                        ent.destroy();
                        const i = composites.indexOf(ent);
                        if (i >= 0) composites.splice(i, 1);
                    }
                };
                const entrance = await buildComposite(startSkel, startAtlas, {
                    mode: "entrance",
                    onEntranceEnd: () => {
                        doSwapRef.current = swapToMainIdle;
                    },
                });
                if (aborted()) {
                    if (entrance && entrance !== "unsupported") entrance.destroy();
                    return; // main stays tracked in compositesRef; cleanup frees it.
                }
                const built = entrance && entrance !== "unsupported" ? entrance : null;
                if (built) {
                    // The game HOLDS the entrance shot steadily through the whole transform and only
                    // pulls back to the wide frame AFTERWARD, on the settled idle (see
                    // `entrancePullOut` → `openStandingIdle`). Frame-by-frame vs the reference
                    // recording, the entrance opens on the `_adjustes[1]` TIGHT stop (Virtuosa
                    // 1246px — a medium shot), NOT the camera's own ortho close-up (`entranceViewPx`
                    // 598px), which frames ~2× too tight: the ortho→render mapping over-tightens,
                    // while the `_adjustes` stops are RCAL-calibrated and correct.
                    composites.push(built);
                    activate(built);
                    const tight = built.authoredTightBounds ?? built.authoredDisplayBounds ?? built.bounds;
                    if (tight) {
                        layoutSpine(built.root, width, height, tight, fitRef.current);
                        const sl = built.sceneLayers ?? [];
                        for (const c of sl) c.alpha = 1;
                        // Drive the entrance camera PURELY from gamedata: the exporter-accumulated camera
                        // rig track (`entranceCamCenterCurve`, absolute mesh-px frame centre) for the
                        // pan/dolly, and the `_adjustes[1]` view extent (`entranceFrameSize`) × the ortho
                        // zoom for the frame size. No measured bounds, no per-skin tuning. Only when both
                        // gamedata inputs exist — otherwise the static `tight` framing already laid out
                        // above holds (spine-only / skins with no camera track).
                        if (built.entranceCamCenterCurve?.length && built.entranceFrameSize) {
                            entranceFollowRef.current = {
                                spine: built.spine,
                                root: built.root,
                                ortho: built.entranceOrthoCurve ?? null,
                                camCenter: built.entranceCamCenterCurve,
                                frameSize: built.entranceFrameSize,
                                sceneLayers: sl,
                            };
                        }
                    }
                    // The idle continues the shot: open on this same `_adjustes[1]` framing and dolly
                    // OUT to the wide `_adjustes[0]` throne over the POST-reform entrance window
                    // (`duration − transform` = the ~6s the game takes to pull back). All gamedata.
                    if (built.entranceViewRatio && built.entranceDuration && built.entranceTransform) {
                        entrancePullOut = { ratio: built.entranceViewRatio, dur: Math.max(0.5, built.entranceDuration - built.entranceTransform) };
                    }
                    // The "Start" clip plays STRAIGHT THROUGH to its own end — the reform, the voice
                    // beat and the settle all live inside the clip, and the live-follow camera
                    // (entranceFollowRef) keeps the reformed standing form in frame — so no early
                    // cut: the hand-off fires from the spine's own `complete` (onEntranceEnd →
                    // doSwapRef).
                } else {
                    // No `_Start` set: open straight on the standing idle + dolly.
                    activate(main);
                    openStandingIdle();
                }

                setIsLoading(false);
                onReadyRef.current?.();
            } catch (err) {
                console.error("Failed to load dynamic illustration:", err);
                if (currentLoadId === loadIdRef.current && mountedRef.current) {
                    setError("Failed to load animation");
                    setIsLoading(false);
                }
            }
        };

        load();

        const resizeObserver = new ResizeObserver(() => {
            const currentApp = appRef.current;
            if (!currentApp || !containerRef.current || !spineRef.current) return;
            const w = containerRef.current.clientWidth;
            const h = containerRef.current.clientHeight;
            if (w <= 0 || h <= 0) return;
            currentApp.renderer.resize(w, h);
            hdrRef.current?.resize(w, h, currentApp.renderer.resolution);
            if (envBgRef.current) resizeEnvironmentBg(envBgRef.current, w, h);
            const target = sceneContainerRef.current ?? spineRef.current;
            layoutSpine(target, w, h, boundsRef.current, fitRef.current);
        });
        resizeObserver.observe(container);

        return () => {
            mountedRef.current = false;
            resizeObserver.disconnect();
            cleanup();
        };
    }, [files.skel, files.atlas, files.png, server, backdrop]);

    return (
        <div className="absolute inset-0">
            <div className={cn("h-full w-full transition-opacity duration-500", isLoading || error || unsupported ? "opacity-0" : "opacity-100")} ref={containerRef} />
            {isLoading && (
                <div className="absolute right-3 bottom-3 flex items-center gap-2 rounded-md border border-white/20 bg-black/40 px-2.5 py-1.5 text-white/80 text-xs backdrop-blur-md">
                    <Spinner className="h-3.5 w-3.5" />
                    Loading animation
                </div>
            )}
            {error && <div className="absolute right-3 bottom-3 rounded-md border border-white/20 bg-black/40 px-2.5 py-1.5 text-white/80 text-xs backdrop-blur-md">{error}</div>}
        </div>
    );
}
