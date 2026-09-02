import * as PIXI from "pixi.js";
import { useEffect, useRef, useState } from "react";
import { Spinner } from "#/components/ui/spinner";
import type { IChibiSpineFiles } from "#/lib/api/chibis";
import { cn } from "#/lib/utils";
import { ANIMATION_SPEED } from "../chibi/constants";
import { chibiAssetURL, DEFAULT_SPINE_FIT, type IAnimationBounds, type ISpineFit, layoutSpine, loadSpineWithEncodedURLs, measureAnimationBounds, visibleRect } from "../chibi/helpers";
import { createHDRScene, type IHDRScene, sceneCompositeGamma } from "./hdrTonemap";
import { ensureAdditiveSpriteBoost, type FindBone, type ILoadedParticles, loadParticles, particleCensus } from "./particles";
import {
    applySceneLayerColor,
    applySceneLayerFollow,
    applySceneLayerRamScroll,
    applySceneLayerSt,
    applySceneLayerUvScroll,
    detectCurveCuts,
    type ISceneAperture,
    type ISceneData,
    type ISceneFrame,
    type ISceneLayer,
    type ISceneLayerRuntime,
    loadSceneFrame,
    loadSceneMeshes,
    orthoZoomRatio,
    sampleColorCurve,
    sampleCurveXY,
    sceneFrameOf,
} from "./sceneMesh";

/** Minimal shape of a spine attachment we can measure at the setup pose. */
interface AttachmentLike {
    name: string;
    worldVerticesLength?: number;
    computeWorldVertices?: (slot: never, start: number, count: number, out: Float32Array, offset: number, stride: number) => void;
}

interface ISceneIllustProps {
    files: IChibiSpineFiles;
    server?: "en" | "cn";
    fit?: ISpineFit;
    /**
     * How to frame a mesh-backed scene. "character" (default) frames to the
     * character's visible bounds - prominent character, good for the narrow card.
     * "authored" uses the game's `_adjustes` display frame (full-scene, square
     * viewport) - for the large fullscreen viewer.
     */
    framing?: "character" | "authored";
    /**
     * Static skin illustration URL. Some dynamic assets omit the full painted
     * backdrop (sky/interior) that the static art has - the game's archive
     * viewer composites the static illustration behind the animated spine. When
     * given, we draw it as a backdrop layer aligned to the authored camera frame
     * and frame the whole scene to that frame, so the animated spine overlays its
     * own static counterpart and the missing backdrop fills in.
     */
    backdrop?: string;
    /**
     * Which PRESENTATION SURFACE this instance is. The game shows a dynamic skin on two
     * differently shaped surfaces and behaves differently on each, so the caller states which
     * one it is rather than the renderer inferring it.
     *
     * - `"viewer"` (default): the full-screen surface. The game plays the `_Start` cinematic
     *   here, and this is what every parity reference was captured from.
     * - `"panel"`: the windowed surface with UI beside it, our operator-detail card. **The game
     *   does NOT play the entrance here.** It comes up already settled and idles.
     *
     * 🔑 Measured, because it is not obvious: across four archive-page captures the largest
     * frame-to-frame MAD after the page-open cross-dissolve is 5.054, and on the two settled
     * clips it is 1.051 and 0.566 over 37.6 s and 45.3 s. The entrance cuts in the VIEWER
     * captures run 15.2984 to 97.8794 and are preceded by a flat 253.000 span. Nothing
     * resembling a cinematic happens on the windowed surface.
     *
     * Gating on the SURFACE is deliberate. It must never be a skin id, a list of the twelve
     * entrance skins, or a flag derived from one: the game's own split is by surface, and its
     * windowed page shows no entrance/non-entrance difference at all (right-side letterbox
     * 10.9-11.8% on an entrance skin, 14.9% on a non-entrance one, 17.6% on another entrance
     * one, so the two entrance skins BRACKET the non-entrance one).
     */
    surface?: "viewer" | "panel";
    onReady?: () => void;
}

const IDLE_ANIMATION = "Idle";

/** Visible-pixel bounds of the illustration at rest, plus alpha-mass landmarks
 *  (centroid, solid head/feet rows) - see {@link measureVisibleBounds}. */
type IVisBounds = IAnimationBounds & {
    centroid?: { cx: number; cy: number };
    /** Y (container space) of the character's SOLID head top - the first row (from the
     *  top) whose alpha mass exceeds a fraction of the densest row. Unlike the bbox top,
     *  this SKIPS thin protrusions above the head (a raised rifle, wispy hair, effects),
     *  so it's a pose-robust anchor for the head. */
    headTop?: number;
    /** Y (container space) of the character's SOLID base - the last dense row (from the
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
 * quads) - that shrinks the illustration into a corner. So we render the resting
 * pose to a small offscreen texture, read back its alpha, and take the tight
 * bounding box of non-transparent pixels - the real illustration extent, centred.
 * Returns null (caller falls back to geometry bounds) if it can't be measured.
 */
function measureVisibleBounds(renderer: PIXI.IRenderer, spine: import("pixi-spine").Spine, idleAnim: string): IVisBounds | null {
    // Measure at rate 1 but RESTORE the caller's rate afterwards. Leaving it at 1 permanently
    // clobbers whatever the composite set, which silently disables any animation-rate control
    // downstream - `measureAnimationBounds` in chibi/helpers.ts already saves and restores for
    // exactly this reason. Currently metric-neutral (ANIMATION_SPEED is 1, so both paths land on
    // 1), which is why it went unnoticed; it is a correctness fix, not a measured win.
    const prevTimeScale = spine.state.timeScale;
    spine.state.timeScale = 1;
    spine.state.clearTracks();
    spine.skeleton.setToSetupPose();
    if (spine.spineData.findAnimation(idleAnim)) spine.state.setAnimation(0, idleAnim, true);
    spine.update(0);
    const geom = spine.getLocalBounds();
    if (geom.width <= 0 || geom.height <= 0) {
        spine.state.timeScale = prevTimeScale;
        return null;
    }

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
        if (x1 < x0 || y1 < y0) {
            spine.state.timeScale = prevTimeScale;
            return { x: geom.x, y: geom.y, width: geom.width, height: geom.height };
        }
        const centroid = sw > 0 ? { cx: geom.x + sx / sw / scale, cy: geom.y + sy / sw / scale } : undefined;
        // Pose-robust head/feet: the first/last row (top-down / bottom-up) whose alpha mass
        // reaches SOLID_ROW_FRAC of the densest row - skipping thin protrusions (a raised
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
        spine.state.timeScale = prevTimeScale;
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
 * so the subject reads at ~half the frame with the surrounding scene around it -
 * matching the in-game archive view. Character-adaptive: a lone character gets
 * breathing room, while an E2 whose creatures/scene ARE the visible bounds is shown
 * whole at the same factor. 1.7 frames the skin character at ~40% with the FULL
 * mirror-world scene around her (1.3 cropped the scene sides - read as "too zoomed";
 * a fixed 2× camera-height framed her too small at ~35%). */
const SCENE_ZOOM_OUT = 1.7;

/** DIAGNOSTIC (`?framebox=x,y,w,h`): pin the settled camera to an EXPLICIT box, in the same
 *  space as {@link IComposite.bounds}.
 *
 *  Exists because the two other framings are each unusable for per-layer attribution. The
 *  authored `_adjustes` camera sits INSIDE the art on most skins (Ch'en's sky quad fills
 *  99.94% of it), so anything at the art's edge is off-frame entirely; and the whole-art
 *  framing is derived from PAINTED content, so every `?abl=` ablation silently re-frames the
 *  shot and an ablated render cannot be compared with an unablated one. A fixed box is
 *  independent of both - identical across every ablation, and wide enough to include the
 *  silhouette boundary. Rendered with a `contain` fit so the whole box is visible.
 *
 *  Returns null when absent or malformed, leaving the normal framing untouched. */
/** Draw the skin's static illustration, DEFOCUSED, behind a scene that does not span the camera
 *  view - filling the bare canvas the game fills with vista. **SHIPPED, default ON; `?gapfill=0`
 *  disables.**
 *
 *  Virtuosa's entrance scene genuinely does not reach the frame (0 of 132 layers span the view,
 *  the largest reaches 0.88) and nothing is dropped by the exporter, so the uncovered margin falls
 *  through to bare canvas - a hard-edged near-black wedge where the game shows misty vista. What
 *  fell through was our own invention: a flat neutral #4d4d4e studio fill, a colour the game never
 *  shows. Replacing it with the skin's own art, blurred, does exactly what it should:
 *
 *      cel  19.422 -> 19.270     t=2 -0.678, t=5 -0.387, every other beat EXACTLY 0.000
 *      mly  17.721 -> 17.721     bit-identical
 *      ska  10.505 -> 10.505     bit-identical
 *
 *  The two beats that move are the two with the camera at its widest - exactly where the scene
 *  leaves the most frame unfilled. Everything else is untouched to the bit.
 *
 *  **Why blurred.** The static illustration contains the CHARACTER, so drawn sharp behind a
 *  translucent scene it can ghost her, misregistered. That cannot be masked away: the art sits at
 *  index 0 and the spine draws in FRONT of it, so a spine-shaped mask can only remove pixels that
 *  were already hidden - built and measured, it moved 786 px of 374400 on `lin_nian#10`, all of
 *  them antialiasing at the spine's edge. The ghost lives precisely where the spine ISN'T. But the
 *  art is only here to be a distant vista, and a vista needs its low-frequency colour and
 *  luminance, not its detail. Defocusing keeps the fill and destroys the recognisable second
 *  character.
 *
 *  The radius is a fraction of the art's own on-screen height, so it is a spatial-frequency cutoff
 *  rather than a pixel constant, and it is not a fitted one - the parity win is a broad plateau,
 *  19.270 to 19.284 over the whole range 1/48 to 1/6, and every value from 1/96 to 1/3 beats the
 *  unblurred 19.361.
 *
 *  Refuted variants are recorded in `hdrTonemap.ts` so none is retried. NOTE for the harness:
 *  `rec.js` must be given `&backdrop=<url>` or this whole path is inert - the production viewer
 *  always passes it (`SkinsContent.tsx`), the recorder does not. */
/** MEASUREMENT ARM (`?mainunder=1`, default off): keep the MAIN skeleton live UNDER the
 *  entrance rig, playing its own Start clip, and draw only the slots the rig currently holds
 *  at alpha 0 (a per-frame property read off the rig's slot colours, never a list). Read as
 *  "is the string 1", never as truthiness. */
function mainUnderOn(): boolean {
    if (typeof window === "undefined") return false;
    const v = new URLSearchParams(window.location.search).get("mainunder");
    return v === "1" || v === "2" || v === "3";
}
/** `?mainunder=3`: the admitted main slots at the RIG SPINE's own depth (a sibling right above
 *  it): under the rig's overlay and mist planes, above its scene background, which is where
 *  the game's defocused near-field crystals sit at kalts t=5. */
function mainAtRigSpine(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("mainunder") === "3";
}
/** `?mainstart=<seconds>`: how long after the cinematic's first frame the main skeleton's own
 *  Start clip begins under the rig. READ FROM IAN'S PHONE CLIP of kalts boc#6 (2026-09-03): her
 *  crystals (Mo_C, keyed on 1.267 s into the main Start) cut in 5.353 s after the cinematic's
 *  first frame, so the game starts the clip at 4.09 s; the bundle holds no field for it. Until
 *  it lands the arm keeps its old behaviour (missing parameter = 0 = start at t=0). */
function mainStartDelay(): number {
    if (typeof window === "undefined") return 0;
    const v = new URLSearchParams(window.location.search).get("mainstart");
    if (v === null) return 0;
    const n = Number(v);
    return Number.isFinite(n) && n > 0 ? n : 0;
}
/** `?mainunder=2`: the admitted main slots drawn ABOVE the rig instead of under it (the
 *  rig's opaque scene planes otherwise cover near-field content such as kalts's crystals). */
function mainAboveRig(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("mainunder") === "2";
}

function gapFillOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("gapfill") !== "0";
}

/** PANEL-ONLY (2026-09-01): present the static backdrop SHARP and never cover-toggle or retire
 *  it, so the windowed card shows the ENTIRE static illustration - the game's own archive
 *  viewer composites the static art behind the animated spine, and the card's job is the whole
 *  artwork. The blur/coverage behaviour it replaces is the VIEWER's defocused-vista rule and is
 *  untouched there (`surface === "viewer"` never reaches this). The framing needs no change:
 *  `contentBounds` is measured AFTER the backdrop joins the container, so the panel's contain
 *  already spans the full art. `?panelart=0` reverts the panel to the viewer behaviour exactly. */
function panelArtOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("panelart") !== "0";
}

/** PANEL-ONLY (2026-09-01): place the static backdrop by the EXPORT-DERIVED art-to-scene
 *  transform (`backdropScale`/`backdropOffsetPx`, computed by the exporter from the scene
 *  meshes' own texture-to-position mapping) instead of the camera-extent heuristic, which
 *  under-scales any composition wider than the authored camera (chen2_2: 2704 px of content
 *  against a 2000 px camera hid the art's outer composition under the scene). Viewer
 *  behaviour is untouched: the derived transform is consumed only where `panelArt` holds.
 *  `?bdxf=0` reverts the panel to the camera-extent scale exactly. */
function bdxfOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("bdxf") !== "0";
}

/** PANEL-ONLY (2026-09-01): clip the composite to the ILLUSTRATION'S OWN SILHOUETTE. The
 *  game's zoomed-out dynamic view keeps the painting's ragged alpha edge, MEASURED on the
 *  three phone-captured keys: the fraction of the painting's transparent pockets that the
 *  game paints is 0.2 percent (shu_nian#11), 2.5 (dusk_nian#12), 9.9 (chen2_2, weakest
 *  registration), while our unmasked card filled 89.1 percent of chen's pockets with the
 *  scene's sky plane. The mask is the static illustration itself, placed by the SAME
 *  transform as the backdrop sprite, so no new geometry is chosen.
 *
 *  DATA-GATED DEFAULT (2026-09-01, measured): the mask is ON exactly when the scene JSON
 *  carries the export-derived `backdropScale`/`backdropOffsetPx` (the character-anchored
 *  correspondence: SIFT of the illustration onto the panel render composed through the
 *  probed sceneRoot matrix; control recovered chen 1.0475 vs 1.04 and shu 1.1426 vs the
 *  1.10/1.1506 pair). With derived placement the pocket fill measures 0.0 to 0.4 percent
 *  across six keys against the game's own 0.2 to 9.9; without it the centroid-anchored
 *  camera-extent placement shatters the composition, so keys whose derivation failed its
 *  guards (17 of 73, mostly small-character chararts defaults) keep the unclipped panel
 *  rather than a broken one. `?silmask=0` forces the mask off everywhere; `?silmask=1`
 *  forces it on even without derived placement (inspection of the failure mode). */
function silMaskParam(): string | null {
    if (typeof window === "undefined") return null;
    return new URLSearchParams(window.location.search).get("silmask");
}

/** `?apscale=0` freezes the scope aperture at its baked radius, ignoring the rim transform's
 *  animated scale (diagnostic). */
function apertureScaleOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("apscale") !== "0";
}

/** `?apcenter=0` pins the scope aperture to the camera centre, ignoring the rim transform's
 *  own POSITION curve (diagnostic). */
function apertureCenterOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("apcenter") !== "0";
}

/** `?layxform=0` freezes scene-layer entrance scale and position at their baked pose. */
function layerTransformOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("layxform") !== "0";
}

/** Linear sample of a `[t, value]` keyframe list, clamped at both ends. The scene's other
 *  samplers are shaped for XY and RGBA curves; this is the scalar case. */
/** {@link sampleScalar} for the two-axis `[t, sx, sy]` curves. Kept separate rather than
 *  generalising the scalar one, because every OTHER curve in this file really is scalar and
 *  widening them all would invite pairing a y sample with the wrong x. */
function sampleScale2(curve: [number, number, number][], t: number): [number, number] {
    if (!curve.length) return [1, 1];
    if (t <= curve[0][0]) return [curve[0][1], curve[0][2]];
    const last = curve[curve.length - 1];
    if (t >= last[0]) return [last[1], last[2]];
    for (let i = 1; i < curve.length; i++) {
        if (curve[i][0] >= t) {
            const [t0, x0, y0] = curve[i - 1];
            const [t1, x1, y1] = curve[i];
            const span = t1 - t0;
            if (span <= 1e-9) return [x1, y1];
            const f = (t - t0) / span;
            return [x0 + (x1 - x0) * f, y0 + (y1 - y0) * f];
        }
    }
    return [last[1], last[2]];
}

function sampleScalar(curve: [number, number][], t: number): number {
    if (!curve.length) return 1;
    if (t <= curve[0][0]) return curve[0][1];
    const last = curve[curve.length - 1];
    if (t >= last[0]) return last[1];
    for (let i = 1; i < curve.length; i++) {
        if (curve[i][0] >= t) {
            const [t0, v0] = curve[i - 1];
            const [t1, v1] = curve[i];
            const span = t1 - t0;
            return span > 1e-9 ? v0 + ((v1 - v0) * (t - t0)) / span : v1;
        }
    }
    return last[1];
}

/** Linear sample of a `[t, dx, dy]` layer-position curve, clamped at both ends. */
function samplePosCurve(curve: [number, number, number][], t: number): [number, number] {
    const first = curve[0];
    if (t <= first[0]) return [first[1], first[2]];
    const last = curve[curve.length - 1];
    if (t >= last[0]) return [last[1], last[2]];
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i][0]) {
            const [t0, x0, y0] = curve[i - 1];
            const [t1, x1, y1] = curve[i];
            const f = t1 > t0 ? (t - t0) / (t1 - t0) : 0;
            return [x0 + (x1 - x0) * f, y0 + (y1 - y0) * f];
        }
    }
    return [last[1], last[2]];
}

/** Blur radius as a fraction of the backdrop's on-screen height. `?gapblur=<f>` sweeps it. */
const GAP_BLUR_FRACTION = 1 / 24;
function gapBlurFraction(): number {
    if (typeof window === "undefined") return GAP_BLUR_FRACTION;
    const v = parseFloat(new URLSearchParams(window.location.search).get("gapblur") ?? "");
    return Number.isFinite(v) && v > 0 ? v : GAP_BLUR_FRACTION;
}

/** `?matlead=<seconds>` - offset applied to material colour-curve sampling only. Inert (0) by
 *  default; exists to test whether those curves run on the camera's clock rather than the
 *  spine's. Parsed once per frame at most, which is why it is a plain function rather than a
 *  per-layer lookup inside the loop. */
function matLead(): number {
    if (typeof window === "undefined") return 0;
    const v = parseFloat(new URLSearchParams(window.location.search).get("matlead") ?? "");
    return Number.isFinite(v) ? v : 0;
}

function frameBoxParam(): IAnimationBounds | null {
    if (typeof window === "undefined") return null;
    const raw = new URLSearchParams(window.location.search).get("framebox");
    if (!raw) return null;
    const p = raw.split(",").map(Number);
    if (p.length !== 4 || p.some((n) => !Number.isFinite(n)) || p[2] <= 0 || p[3] <= 0) return null;
    return { x: p[0], y: p[1], width: p[2], height: p[3] };
}

/** DIAGNOSTIC (`?statcam=1`): skip the `_Start` cinematic and render the STANDING IDLE under a
 *  camera that never moves - the shot the game's OWN skin preview shows.
 *
 *  **Why this exists: it is the only uncontaminated reference we can obtain.** Every parity number
 *  to date is measured against a salvaged CRF36 recording whose encode damage is 40-57% of the
 *  score ([[dynchar-degradation-floor-real-headroom]]), and no renderer-side change can reach past
 *  that floor. The game's skin preview (FLOT Lookbook → skin → magnifier) can be captured at
 *  2340x1080 / 100 Mbps for ANY skin including unowned ones, so it is clean - but it is NOT the
 *  entrance, and pointing the entrance renderer at it would be comparing two different shots.
 *
 *  What the preview actually plays was established by measurement, not assumption:
 *
 *    - **No white flash.** `_Start` ends in a full-frame whiteout; the salvaged references peak at
 *      235-236 mean luma (1.9-2.5x their own base) at t=14.4 / 17.8 / 21.8 s. The preview captures
 *      never exceed 1.25x base and have no flash anywhere. It cannot be `_Start`.
 *    - **No camera motion.** Background-strip drift (left 15% of frame, vs. a frame shortly after
 *      open, at +2/4/6/8 s) is 2-3x smaller than the reference's on all three skins.
 *    - The character is already fully formed in the first frame, where `_Start` opens on the
 *      seated/reforming shape.
 *
 *  So the target shot is: standing idle, settled framing, camera locked. That is exactly the path
 *  the 70 dynchar skins with no `_Start` already take, which is why this flag only has to decline
 *  to build the entrance composite - `openStandingIdle()` with no `fromEntrance` and no
 *  `entrancePullOut` is already static, and is the most-exercised framing path in the file.
 *
 *  Composes with `?framebox=x,y,w,h`, which wins over the settled box (see `openStandingIdle`) -
 *  use that to search for the preview's framing if the authored `_adjustes` box turns out not to
 *  match it. Off by default; the production viewer and every existing measurement are untouched. */
function staticCamOn(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("statcam") === "1";
}

/** DIAGNOSTIC (`?statbox=<zoom>[,<dx>,<dy>]`): scale the {@link staticCamOn} framing box about its
 *  own centre by `zoom` and shift it by `dx,dy` box-space px. Inert unless `?statcam=1`.
 *
 *  Exists to PIN a measured residual to its gamedata source rather than bake it in. Against the
 *  preview captures, the authored tight box is right in kind but leaves a consistent shortfall -
 *  the render must be enlarged ~1.27x (ska 1.26, mly 1.30-1.32, cel 1.24), with a per-skin
 *  translation whose sign differs between skins (ska dx-46/dy+82, mly dx-34/dy-68, cel dx+32/
 *  dy-26, in 900x416 px). Those are real, not noise: mly and cel reproduce their offsets EXACTLY
 *  at two different times, and the phase-matched fits peak at 7.6 sigma.
 *
 *  The translation almost certainly comes from `bodyFrameBox` building the tight box on the WIDE
 *  stop's centre - the gamedata ships a separate `cameraOffsetPx2` for `_adjustes[1]` which we
 *  never read. That is the first thing to test with this knob. It is deliberately NOT applied as a
 *  default: a fitted zoom constant is exactly the sort of thing this codebase refuses to ship
 *  (see the fitted-gradient and clamp-ceiling findings), and one frame per skin cannot calibrate
 *  a constant that must hold for 82.
 *
 *  Measure with a PHASE-MATCHED pair - game time = viewer-open time + our track time (open is
 *  2.07/2.18/2.00 s for mly/cel/ska). Comparing arbitrary times silently halves the correlation
 *  and moves the fitted scale. */
/** `?tightoff=0` - under `?statcam=1`, DON'T re-seat the tight box on the tight `_adjustes[1]`
 *  stop's own centre. Exists purely so the shift can be A/B'd against the old shared-centre
 *  behaviour; default on, and inert outside statcam. */
function tightOffOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("tightoff") !== "0";
}

function staticCamBox(b: IAnimationBounds): IAnimationBounds {
    if (typeof window === "undefined") return b;
    const raw = new URLSearchParams(window.location.search).get("statbox");
    if (!raw) return b;
    const p = raw.split(",").map(Number);
    const z = p[0];
    if (!Number.isFinite(z) || z <= 0) return b;
    const dx = Number.isFinite(p[1]) ? p[1] : 0;
    const dy = Number.isFinite(p[2]) ? p[2] : 0;
    const w = b.width * z;
    const h = b.height * z;
    return { x: b.x + b.width / 2 - w / 2 + dx, y: b.y + b.height / 2 - h / 2 + dy, width: w, height: h };
}

/** Frame a no-entrance skin to everything it PAINTS rather than to its authored `_adjustes`
 *  camera. **DISABLED by default - the target is real but only ONE reference exists.**
 *
 *  The observation is solid. 69 of the 82 dynchar skins have no `_Start`, and for those the
 *  game's viewer shows the whole cut-out with margins while the authored `_adjustes[0]` box
 *  crops well inside it - content exceeds the authored view on 61 of 80 skins (Ch'en the
 *  Holungday 2690x1789 against a 1171 px view; Virtuosa 2.7x; Ch'en Wei 3.8x). Against the
 *  in-game recording of Ch'en, framing the painted content is plainly the better match.
 *
 *  It is off because ONE reference cannot calibrate 70 skins, and two concrete problems
 *  remain:
 *    • `fitRef` is `mode: "height"` for authored framing, which CROPS the width - so a
 *      content box wider than the viewport still does not show the whole art. Needs a
 *      "contain" fit, which changes layout for every skin.
 *    • `paintedLocalBounds`' alpha floor is a tuned constant. At >8/255 it catches faint
 *      haze that extends far past the visible composition: Nian's painted box comes out
 *      3037x2267 against a 1640 px authored view, dropping her art from 45% of the frame to
 *      25%. Her authored crop already showed the whole composition - but that judgement is
 *      aesthetic, not measured, because there is no recording of her to check against.
 *
 *  TO ENABLE: capture 2-3 in-game recordings of no-entrance skins with DIFFERENT content
 *  ratios (a tight one and a sprawling one), fix the fit mode to "contain", and calibrate the
 *  alpha floor against them. `?wholeart=1` turns it on now for inspection. */
function fitWholeArt(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("wholeart") !== "0";
}

/** DIAGNOSTIC (`?postfx=0`): skip the ENTRANCE post-process volume entirely (the
 *  `HGMobileBlur` / greyscale pass driven by `entrancePostFx`).
 *
 *  Added to isolate Fugue's early-cinematic error: her blur window is 1.40-7.00 s at peak
 *  weight 0.70 and her worst beats (1.8 / 2.8, MADC 49/53 against 13-16 later) sit INSIDE it.
 *  The corpus barely exercises this path - cet's beats sample NEITHER of her blur windows
 *  (`dynchar-postfx-disabled-without-fade`), so Fugue is the first real test of it. */
function postFxOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("postfx") !== "0";
}

/** DIAGNOSTIC (`?blurpx=<n>`): override the entrance blur RADIUS.
 *
 *  Fugue's game capture is visibly HAZY at t=1.0-2.8 where ours is sharp, so the radius was the
 *  obvious suspect for her early-beat error. It is NOT: swept on correctly-staged assets her
 *  measured optimum is the DATA value itself - `blurSpread 1.3 * 2^resMode 1` = **2.6 px**
 *  (2.6 -> 20.95, 5 -> 20.80, 8 -> 21.53, 12 -> 21.69, 16 -> 23.35, 24 -> 24.58 over her first
 *  three beats). Do not override it.
 *
 *  ⚠️ An earlier reading of this - that the volume exports `params: null` and falls back to 1 px -
 *  came from a STALE export. A correctly-staged export carries the params; see
 *  `dynchar-client-version-split` for why an under-staged export is so easy to believe. */
function blurPxOverride(): number | null {
    if (typeof window === "undefined") return null;
    const v = Number.parseFloat(new URLSearchParams(window.location.search).get("blurpx") ?? "");
    return Number.isFinite(v) && v >= 0 ? v : null;
}

/** Alpha floor (0-255) for {@link paintedLocalBounds}. Calibrated against the one in-game
 *  recording available (Ch'en the Holungday); `?paintalpha=<n>` sweeps it. */
const PAINTED_ALPHA_MIN = 8;
function paintedAlphaMin(): number {
    if (typeof window === "undefined") return PAINTED_ALPHA_MIN;
    const v = parseInt(new URLSearchParams(window.location.search).get("paintalpha") ?? "", 10);
    return Number.isFinite(v) && v >= 0 && v < 255 ? v : PAINTED_ALPHA_MIN;
}

/** Bounding box of the pixels a container actually PAINTS, in its own local space.
 *
 *  `getLocalBounds()` is the union of child GEOMETRY, which is not the same thing: a scene
 *  routinely carries oversized or fully-transparent quads, so the geometric box can be far
 *  larger than the visible art. Framing to it zooms a skin out into empty margin (Nian's
 *  archive art fell from 45% of the frame to 19%, while her authored crop had already shown
 *  the whole composition).
 *
 *  So render the container once into a small offscreen target and measure where the alpha
 *  actually is. Model-free and fully general - no per-skin data, no assumptions about which
 *  layers matter. Done once at load; the readback is a few tens of KB at this size.
 *
 *  Returns null when nothing is painted or the renderer cannot do the pass. */
function paintedLocalBounds(renderer: PIXI.IRenderer, target: PIXI.Container, maxSide = 192): IAnimationBounds | null {
    const lb = target.getLocalBounds();
    if (!Number.isFinite(lb.width) || lb.width <= 1 || lb.height <= 1) return null;
    const s = Math.min(1, maxSide / Math.max(lb.width, lb.height));
    const w = Math.max(1, Math.ceil(lb.width * s));
    const h = Math.max(1, Math.ceil(lb.height * s));
    let rt: PIXI.RenderTexture | null = null;
    // The container is laid out on the stage; render it from its own local space so the
    // measurement is independent of whatever camera is currently applied.
    const saved = target.transform.localTransform.clone();
    const savedX = target.x;
    const savedY = target.y;
    const savedSX = target.scale.x;
    const savedSY = target.scale.y;
    // NOTE FOR DIAGNOSTICS: this box is measured from what is currently drawable, so a `?abl=`
    // ablation shrinks it and silently RE-FRAMES the shot. An ablated render compared against
    // a full one then measures the ZOOM CHANGE, not the ablated element - which inverted a
    // real result once: Ch'en's foreground read as DARKENING the art by 11.2 when, framed
    // identically, it BRIGHTENS it by 11.4 (verified by bounding box: the content-fit pair was
    // framed differently, the authored pair identically). **Always pass `wholeart=0` on BOTH
    // sides of an ablation comparison.** Forcing children renderable here was tried and does
    // NOT fix it - the ablation is applied before this runs, through more than one flag.
    try {
        rt = PIXI.RenderTexture.create({ width: w, height: h, resolution: 1 });
        target.position.set(0, 0);
        target.scale.set(1, 1);
        const m = new PIXI.Matrix().translate(-lb.x, -lb.y).scale(s, s);
        renderer.render(target, { renderTexture: rt, clear: true, transform: m });
        const px = renderer.extract.pixels(rt);
        const aMin = paintedAlphaMin();
        let minX = w;
        let minY = h;
        let maxX = -1;
        let maxY = -1;
        for (let y = 0; y < h; y++) {
            for (let x = 0; x < w; x++) {
                if (px[(y * w + x) * 4 + 3] > aMin) {
                    if (x < minX) minX = x;
                    if (x > maxX) maxX = x;
                    if (y < minY) minY = y;
                    if (y > maxY) maxY = y;
                }
            }
        }
        if (maxX < minX || maxY < minY) return null;
        return { x: lb.x + minX / s, y: lb.y + minY / s, width: (maxX - minX + 1) / s, height: (maxY - minY + 1) / s };
    } catch {
        return null;
    } finally {
        rt?.destroy(true);
        target.position.set(savedX, savedY);
        target.scale.set(savedSX, savedSY);
        target.transform.setFromMatrix(saved);
    }
}

/** Expand a bounds box around its centre by `factor` (>1 zooms the framing out). */
function inflateBounds(bounds: IAnimationBounds | null, factor: number): IAnimationBounds | null {
    if (!bounds) return null;
    const w = bounds.width * factor;
    const h = bounds.height * factor;
    return { x: bounds.x - (w - bounds.width) / 2, y: bounds.y - (h - bounds.height) / 2, width: w, height: h };
}

/** A global calibration constant, overridable via a URL search param (e.g. `?rcal=0.6`)
 *  for on-page tuning against reference recordings. */
/** Read a calibration constant from the query string, falling back when it is ABSENT.
 *
 *  🚨 The previous form was `parseFloat(get(name) || String(fallback)) || fallback`, which
 *  carried the falsy trap this codebase documents everywhere else, TWICE in one expression:
 *  an empty `?vbias=` fell back on the first `||`, and a parsed **0 fell back on the second**,
 *  because 0 is falsy. So NO calibration knob could ever be set to zero, and asking for one
 *  silently returned the default.
 *
 *  That is not hypothetical. Sweeping `?vbias=` across the five settled clips read
 *  78.917 / 71.093 / 93.019 / 100.679 / 69.797 at BOTH 0.00 and the default 0.1815, five
 *  identical pairs, which reads exactly like "the vertical bias does not reach this region"
 *  and would have been recorded as a refutation. 0.30 and 0.45 moved, which is the only reason
 *  it was caught.
 *
 *  Absence is now tested explicitly and a non-finite parse is rejected on its own terms. The
 *  default is unchanged for every value that was previously expressible. */
function calibrationParam(name: string, fallback: number): number {
    try {
        const raw = new URLSearchParams(window.location.search).get(name);
        if (raw == null || raw === "") return fallback;
        const v = parseFloat(raw);
        return Number.isFinite(v) ? v : fallback;
    } catch {
        return fallback;
    }
}

/** A spine BACKDROP element that is a dark shadow / smoke silhouette (named as
 *  such and behind the character). */
/** Unambiguous ENGLISH markers may appear anywhere in the name. */
const DARK_BACKDROP_WORD = /^bg_.*(shadow|smoke|somke|silhou|black)/i;
/** The PINYIN markers `hei` (黑, black) and `ying` (影, shadow) are only three or four
 *  letters, so they must match a WHOLE underscore-delimited segment. As bare substrings
 *  they swallow unrelated art: Skadi the Corrupting Heart's 59 `BG_Heir_*` background
 *  figures all contain "hei" inside "**Heir**", which made `hasShadowSlots` true for her
 *  and would cull every one of them the moment she composited against a static backdrop.
 *  Corpus-wide this is the only collision (18 skins match the rule; 59 of the 108 matched
 *  attachments were hers, and every one a false positive). */
const DARK_BACKDROP_PINYIN = /^bg_(?:.*_)?(hei|ying)(?:_|\d|$)/i;
function isDarkBackdropSlot(name: string): boolean {
    return DARK_BACKDROP_WORD.test(name) || DARK_BACKDROP_PINYIN.test(name);
}

/** When a static illustration is composited behind the spine, the static art
 *  already contains every backdrop element, PROPERLY layered by the game (e.g.
 *  Pepe's black shadow-cat sits BEHIND the purple nebula in the art). But the
 *  spine re-draws those dark elements as flat, opaque slots ON TOP of the static,
 *  blotting out the nebula/wall behind them. Since the static provides them
 *  faithfully - and the animated character still renders over the static - we hide
 *  the spine's dark backdrop silhouettes so the correctly-composited static shows
 *  through. Light, well-aligned backdrop elements (doves, streamers, petals) are
 *  left animating; the character's own attached shadows aren't `BG_`-prefixed, so
 *  they're untouched. (Verified harmless on Hoshiguma-alter, whose `BG_Door_*_shadow`
 *  mirror-panel shadows are re-supplied identically by the static art.) */
function hasShadowSlots(spine: import("pixi-spine").Spine): boolean {
    return spine.skeleton.slots.some((s) => isDarkBackdropSlot((s as unknown as { data: { name: string } }).data.name));
}

/** DIAGNOSTIC (`?hideslots=<regex>` / `?keepslots=<regex>`): suppress or isolate spine
 *  slots by name so a residual can be attributed to a subset of the skeleton. Runs
 *  UNCONDITIONALLY (the shadow cull below is gated per-skin) and, like it, must run in the
 *  post-`update` pass because pixi-spine rebuilds the meshes every frame. */
function applySlotDiagnostic(spine: import("pixi-spine").Spine): void {
    if (typeof window === "undefined") return;
    const q = new URLSearchParams(window.location.search);
    const hide = q.get("hideslots");
    const keep = q.get("keepslots");
    if (!hide && !keep) return;
    const re = new RegExp((hide ?? keep) as string);
    for (const slotU of spine.skeleton.slots) {
        const slot = slotU as unknown as { data: { name: string }; currentMesh?: { renderable: boolean }; currentSprite?: { renderable: boolean }; currentGraphics?: { renderable: boolean } };
        if (re.test(slot.data.name) !== !!keep) {
            if (slot.currentMesh) slot.currentMesh.renderable = false;
            if (slot.currentSprite) slot.currentSprite.renderable = false;
            if (slot.currentGraphics) slot.currentGraphics.renderable = false;
        }
    }
}

/** Make the matched slots' per-slot display objects non-renderable. pixi-spine
 *  rebuilds the geometry from the (re-attached) attachments on every `update`, so
 *  this must run each frame AFTER `update`; flipping the already-built mesh's
 *  `renderable` flag is what actually keeps them hidden. */
function hideRedundantShadowSlots(spine: import("pixi-spine").Spine): void {
    for (const slotU of spine.skeleton.slots) {
        const slot = slotU as unknown as { data: { name: string }; currentMesh?: { renderable: boolean }; currentSprite?: { renderable: boolean }; currentGraphics?: { renderable: boolean } };
        if (!isDarkBackdropSlot(slot.data.name)) continue;
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
 * loader assemble MORE THAN ONE - a special "_Start" ENTRANCE composite and the
 * settled main L2D - and hand off between them (play the entrance once, then swap
 * to the main idle cycle) without duplicating the compositing/framing machinery.
 */
interface IComposite {
    spine: import("pixi-spine").Spine;
    /** The object to add to the stage (or point the HDR pass at) and lay out - the
     *  scene container when there are mesh layers/particles, else the bare spine. */
    root: PIXI.Container;
    /** True when `root` is a scene container (mesh layers/particles), false when it
     *  is the bare spine (spine-only art with no separate scene). */
    isScene: boolean;
    /** The scene's authored orthographic camera size in px (`cameraSize × 100`). Drives the
     *  composite transfer - see {@link sceneCompositeGamma}. Null for spine-only art. */
    cameraSizePx: number | null;
    /** Union of everything the composite actually draws (all mesh layers + the skeleton), in
     *  the same space as {@link bounds}. The authored `_adjustes` camera CROPS INTO this for
     *  most skins - content is larger than the authored view on 61 of 80 - which is right for
     *  a skin whose entrance cinematic frames a shot, and wrong for one that has none, where
     *  the game's viewer fits the whole cut-out. Null for spine-only art. */
    contentBounds: IAnimationBounds | null;
    /** True when the scene owns a DARK opaque painted backdrop (see
     *  {@link ILoadedScene.hasDarkBackdrop}) - the scene supplies its own environment, so the
     *  light-grey studio gradient must NOT be composited behind it (it would bleed through the
     *  frame's un-covered edges and wash the deep colour to grey). */
    hasDarkBackdrop: boolean;
    bounds: IAnimationBounds | null;
    /** The game's AUTHORED display frame in frontend coords (`cameraViewPx` square
     *  centred on the Y-flipped `cameraOffsetPx`) - the exact camera the in-game
     *  viewer frames these skins with. Used to frame the special-entry entrance skins
     *  (and their idle) to match the game rather than the character-bounds heuristic.
     *  Null for spine-only art with no authored camera. */
    /** The backdrop wash's seat inside this composite's spine, when the game splits the
     *  skeleton. Null when the skin ships no `separatorSlots`. */
    separatorWash: ISeparatorWash[];
    authoredDisplayBounds: IAnimationBounds | null;
    /** The game's TIGHT/zoomed-in display frame (`_adjustes[1]`, e.g. Virtuosa 1246 vs
     *  the 1929 wide) in frontend coords - the close-up the in-game viewer opens on and
     *  dollies OUT from. Same vis centre as {@link authoredDisplayBounds}, shifted by the
     *  authored offset delta (the game's small pan). Null when the skin has no tight
     *  endpoint; the opener then derives the tight box from the 0.646 game ratio. */
    authoredTightBounds: IAnimationBounds | null;
    /** The authored SHIFT from the wide `_adjustes[0]` stop to the tight `_adjustes[1]` one, in
     *  render/vis px (`(offsetPx2 − offsetPx) × RCAL`, Y flipped to screen-down).
     *
     *  `bodyFrameBox` builds BOTH boxes on the same body-measured centre, because the raw
     *  `_adjustes` offset is unusable for centring - it points ~650 px BELOW the body in export
     *  space (see the note there). Their DIFFERENCE is still meaningful in that space though, so
     *  the two stops' relative move survives even though neither absolute position does. This is
     *  the same cross-space-additive convention `centerBlend` and the hand-off delta already use.
     *
     *  Null when the skin ships no second stop. Consumed only by `?statcam=1` (see
     *  `staticCamOn`) - applying it to `authoredTightBounds` itself would move the ENTRANCE
     *  framing on the 12 skins that have one, including all three references, and silently
     *  invalidate every recorded baseline. */
    authoredTightShift: [number, number] | null;
    /** The game's SKIN-PREVIEW framing box - half the wide `_adjustes[0]` view. Consumed only by
     *  `?statcam=1`; see the derivation where it is built. Null for skins with no authored view. */
    previewBounds: IAnimationBounds | null;
    /** `[scale, dx, dy]` in OUTPUT px: the measured settled-framing correction, applied RELATIVE
     *  to whatever box the settled path produces. Null when no correction applies. */
    settleFix: [number, number, number] | null;
    /** The TIGHT `_adjustes[1]` stop as a FRACTION of the wide display view (`viewPx2/viewPx`).
     *  Lets the settled main idle open on the SAME close-up the entrance held before pulling
     *  out to the wide frame. Null when the skin ships no tight stop. */
    entranceViewRatio: number | null;
    /** ENTRANCE (`_Start`) timing from the scene gamedata (seconds): total `duration`
     *  and the `transform` reform beat. Non-null only for a "_Start" entrance composite;
     *  drives the tight→wide camera dolly length and the hand-off time. */
    entranceDuration: number | null;
    /** Where the end-of-entrance fade finishes ramping - `min(duration, clipStop)`.
     *  See {@link entranceFadeEnd}. */
    entranceFadeEnd: number | null;
    /** Straight RGBA of the director's end-of-entrance screen fade (`_params.fadeColor`), or
     *  null when the skin ships no entrance director. See {@link ENTRANCE_FADE_IN}. */
    entranceFade: [number, number, number, number] | null;
    /** The entrance camera's authored solid clear colour (rgb 0..1), or null. See
     *  {@link camClearOn}. */
    entranceClearColor: [number, number, number] | null;
    entranceTransform: number | null;
    /** The `_Start` camera dolly ZOOM curve (`[t_s, ortho]` keyframes) - the game's actual
     *  data-driven camera motion (extracted from the clip animating the Main Camera's orthographic
     *  size). Scales the entrance frame relative to its base extent as the shot pushes in/out. */
    entranceOrthoCurve: [number, number][] | null;
    /** ENTRANCE post-process volume (see {@link entrancePostFxRef}). */
    entrancePostFx: { effect: string; intensity: number; weightCurve: [number, number][]; params?: Record<string, number> | null } | null;
    /** The `_Start` camera's ABSOLUTE frame-centre track (`[t_s, cx, cy]`, mesh px) - the game's
     *  own camera rig (the animated camera-parent Transform) accumulated by the Rust exporter into
     *  a world-space centre curve. Drives the entrance pan/dolly directly; no measured bounds. */
    entranceCamCenterCurve: [number, number, number][] | null;
    entranceCamRollCurve: [number, number][] | null;
    /** The entrance frame EXTENT in mesh px - the `_Start` camera's view at its ANIMATED t=0
     *  ortho size (`2·ortho₀/skeletonScale`, falling back to the static `entranceViewPx`, then the
     *  tight `_adjustes[1]` stop) - the base frame size the live camera holds, scaled by the
     *  ortho-size ratio each tick. Same 100 px/world-unit scale as
     *  {@link entranceCamCenterCurve}, so they compose directly. */
    entranceFrameSize: number | null;
    /** The entrance's scene-mesh containers (background + foreground) - carry the per-layer
     *  `m_IsActive` gating meshes. Entrance-only. */
    sceneLayers: PIXI.Container[] | null;
    /** The circular VIEWPORT APERTURE this entrance is watched through (Executor's rifle
     *  scope), or null for the other 81 skins. Applied as a scene-space mask so the camera
     *  move carries it. See {@link ISceneAperture}. */
    aperture: ISceneAperture | null;
    /** The aperture's mask graphic, seated in the scene container so it shares the camera
     *  transform. Held so the entrance tick can lift the mask when the scope opens. */
    apertureMask: PIXI.Graphics | null;
    /** Capability A: the scene's Ram-family UV-scroll layer meshes (flat list, collected from
     *  the background/foreground/overlay containers). The always-running tick re-scrolls their
     *  UVs each frame with the continuous scene clock (idle AND entrance). Empty for scenes
     *  with no scroll layers. */
    scrollLayers: PIXI.Mesh[];
    /** The scene's RAM-MASKED layer meshes (a `_DissolveTex`/`_DisturbTex` silhouette). The
     *  always-running tick drifts their mask lookups with the same scene clock. Empty for
     *  scenes with no masked layers. */
    ramLayers: PIXI.Mesh[];
    /** The scene's BONE-FOLLOWING layer meshes (spine-unity `BoneFollower` in their Unity
     *  ancestry - Mlynar's sword flare). The always-running tick rebases them onto their
     *  live bone each frame, after the spine update. Empty for scenes with none. */
    followLayers: PIXI.Mesh[];
    /** When the authored SCENE timeline outlasts the spine's own "Start" animation
     *  (Mlynar: the spine ends at 14.33s but the white-flash plane + camera run to the
     *  clip stop 15.97s), the time (track seconds) the entrance actually ends - the
     *  spine `complete` listener defers and the tick fires {@link requestEntranceEnd}
     *  at this clock. `null` = the spine animation is the longer one (fire on
     *  `complete` as usual). Entrance-only. */
    entranceSceneEnd: number | null;
    /** Idempotent trigger for the entrance→main handoff (see entranceSceneEnd). */
    requestEntranceEnd: (() => void) | null;
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
 *  same art, so their mass centres correspond - robust to composition, unlike a
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
 *  view of the scene - it spans `2 × cameraSizePx` of scene height - so we scale
 *  it to that and centre it on the animated character's visible bounds (the focal
 *  point both the static art and the spine are composed around). This aligns the
 *  spine over its own static counterpart so the missing backdrop fills in behind. */
/** Read a fade-timing override off the query string, falling back to the shipped default. */
/** Alpha of a scene layer's colour curve at time `t` (keys are `[t, r, g, b, a]`). */
function sampleColorAlpha(curve: number[][], t: number): number {
    if (!curve.length) return 1;
    if (t <= curve[0][0]) return curve[0][4];
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i][0]) {
            const [t0, , , , a0] = curve[i - 1];
            const [t1, , , , a1] = curve[i];
            return t1 > t0 ? a0 + ((a1 - a0) * (t - t0)) / (t1 - t0) : a1;
        }
    }
    return curve[curve.length - 1][4];
}

function fadeParam(name: string, dflt: number): number {
    if (typeof window === "undefined") return dflt;
    // Guard on the RAW string: `Number(null)` is 0, which is finite and >= 0, so testing the
    // parsed number alone silently makes every absent param read as zero.
    const raw = new URLSearchParams(window.location.search).get(name);
    if (raw === null || raw === "") return dflt;
    const v = Number(raw);
    return Number.isFinite(v) && v >= 0 ? v : dflt;
}

/** Seconds spent ramping INTO the fade colour, ending at {@link entranceFadeEnd} − HOLD.
 *
 *  **0.20 is Executor's own MEASURED ramp** (0.183 s, taken off her capture at 60 fps as the
 *  10 %→90 % crossing toward white). It was 0.85, a value measured on Virtuosa alone, and the
 *  cost of shortening it used to be prohibitive - Executor went +13.3 - because her ramp END was
 *  0.4 s late. Anchoring the fade to the camera clip's stop fixed the end, and with the end right
 *  she is now **completely insensitive** to this constant (11.018 at 0.85, 0.60, 0.45, 0.30 and
 *  0.20 alike). That unblocked it.
 *
 *  With 0.20 BOTH director-fade skins land their ramp start and end against the captures:
 *
 *      exc   start 5.70 / end 5.90     measured 5.717 / 5.900
 *      wis   start 12.15 / end 12.35   measured 12.150 / (12.683)
 *
 *  Four independent numbers from two skins, which is why this is a measurement rather than a fit.
 *
 *  ⚠️ The ramps genuinely DIFFER per skin - 0.183 (exc), 0.533 (wis), 0.600 (eyja), ~0.67 (cel)
 *  - and nothing exported predicts them, so one global cannot serve all four. Virtuosa pays
 *  **+0.165** for this (her ramp is the longest); Wiš'adel gains **6.96**. Everything else is
 *  bit-identical. `?fadein=` still sweeps it. */
const ENTRANCE_FADE_IN = fadeParam("fadein", 0.2);
/** Seconds the fade holds at full before lifting (covers the idle swap). `?fadehold=`. */
/** Seconds after the TRANSFORM beat at which the settled-ground swap fires. 0 = at the transform
 *  (the behaviour this replaced).
 *
 *  🔑 ABSOLUTE, not a fraction of `transform -> duration`. The two skins with settled-ground
 *  exposure constrain this in absolute time, and a fraction cannot satisfy both because their
 *  gaps differ by 4.7x (mue 7.0s, whitw2 1.5s):
 *
 *    whitw2  needs the swap AFTER  renderer 14.05 - her ground is still dark there and ours
 *            whitened ~1.6s early, costing 41.861 -> 37.731.
 *    mue     needs it BY renderer 15.00 - measured inside the swap's own pixel mask, the game
 *            reads 203.1 there against our white ground's 203.4 and our dark ground's 110.2.
 *            Her ground RAMPS (167 at t=12, 175 at 13, 183 at 14.5, 203 at 15, 224 at 17), so it
 *            is never as dark as ours; firing late is what hurts her.
 *
 *  Window [14.05, 15.00] with both transforms at 13.0 -> lead 1.05..2.00; 1.5 sits mid-window.
 *  ⚠️ A fraction cost mue 0.759 at her t=15 for nothing; this costs her nothing.
 *  ⚠️ Both exposed skins share transform 13.0, so "transform + lead" and "absolute 14.5" are not
 *  yet distinguishable - a third exposed skin would separate them. `?settledlead=` sweeps it. */
const SETTLED_GROUND_LEAD = 1.5;

/** Seconds over which the settled ground FADES IN, ending at the lead time. 0 = switch instantly.
 *
 *  🔑 The RAMP is data-justified, not fitted: both captures show the ground ramp rather than
 *  switch. mue's ground region reads 167 @12 / 175 @13 / 183 @14.5 / 203 @15 / 224 @17, and at
 *  whitw2's t=13.5 the game reads 151.6 where our dark ground gives 99.3 and our white 200 - it
 *  is MIDWAY, which no step can produce.
 *
 *  ⚠️ The DURATION is pinned on whitw2 alone, because she is the only reference whose beats fall
 *  inside the ramp window; mue (10.756) and kalts (31.303) are BIT-IDENTICAL at every value tried,
 *  and no other skin has settled-ground exposure. Swept: 0 37.731 · 0.50 36.533 · 0.60 34.633 ·
 *  0.65 33.934 · **0.70 33.431** · 0.80 34.215 · 1.00 35.464. Interior minimum, bracketed.
 *  ⚠️ `r` declines gently across the sweep (.736 -> .717 at the optimum), so this is not the
 *  MADC-and-r-agree case a TRIM re-fit demands - it is accepted because the mechanism is measured
 *  in the captures independently of the score. `?settledramp=` sweeps it. */
const SETTLED_GROUND_RAMP = 0.7;

function settledGroundRamp(): number {
    if (typeof window === "undefined") return SETTLED_GROUND_RAMP;
    // ⚠️ Read the parameter, do not coerce. `Number(null)` is 0, which is finite and >= 0, so a
    // missing param would silently return 0 - the same trap the settled-lead reader documents,
    // and it defeated this default once already.
    const raw = new URLSearchParams(window.location.search).get("settledramp");
    if (raw == null) return SETTLED_GROUND_RAMP;
    const v = Number(raw);
    return Number.isFinite(v) && v >= 0 ? v : SETTLED_GROUND_RAMP;
}
const ENTRANCE_FADE_HOLD = fadeParam("fadehold", 0.2);
/** Seconds spent lifting the fade once the idle is live - Mlynar's capture is fully white at
 *  `duration - 0.1` and back to the idle mean by `duration + 0.3`. */
const ENTRANCE_FADE_OUT = 0.35;

/** Does the SCENE already perform the end-of-entrance fade itself, making the client-side
 *  director fade a double-count?
 *
 *  Every skin authors a white overlay for the transition, so "has a fade layer" cannot be the
 *  test - all four benchmarks have one. What separates them is whether that layer SPANS THE
 *  CAMERA VIEW, i.e. whether it can white out the frame on its own:
 *
 *    Mlynar   L17  1.73x view, alpha already 1.000 when the director ramp starts
 *    Skadi 2  L29  2.78x view, alpha 0.350 at ramp start, rising to 1.0 inside the window
 *    Virtuosa L131 0.41x view  -> cannot cover; the director fade is what whites her frame
 *    Muelsyse L66  1.05 x 0.85 -> does not cover BOTH axes; she needs the director fade too
 *
 *  Where the layer does span the view the two ramps run simultaneously and compose as
 *  `1-(1-a1)(1-a2)`, which reaches white early and steeply. Measured on Skadi, whose authored
 *  ramp and the director ramp overlap almost exactly (20.4-22.4 vs 21.28-22.13): stacking put us
 *  +32.7 luma over the capture at t=21.6 and saturated at 21.87 while the game was still at
 *  236.5 - the user-visible "the glow is instant instead of fading". Mlynar is unaffected either
 *  way (his layer is already opaque white through the whole ramp, so the director fade adds
 *  nothing to add), which is what makes this safe to apply by rule rather than per skin.
 *
 *  Colour is compared to the director's own `fadeColor` rather than assumed white. */
/** When does the end-of-entrance screen fade finish ramping?
 *
 *  The director ships a nominal `duration`, but the cinematic's authored content ends when its
 *  CAMERA CLIP stops, and the two are not the same: Executor's camera clip stops at 6.100
 *  against a duration of 6.500, and her capture saturates to white at exactly 5.900 -
 *  `clipStop − HOLD`, to a frame. Anchoring on `duration` put our ramp 0.400 s late, which a
 *  time-offset sweep rules out as a reference-trim artefact (0 is the optimum in both
 *  directions, ±0.2 s costs 8–24 MADC).
 *
 *  Only ever SHORTENS: where the clip runs past the director (9 of the 11 entrance skins that
 *  report both) the duration still wins, so this cannot stretch a fade. `?fadeanchor=0` reverts.
 *
 *  ⚠️ This is not a licence to anchor the fade RAMP on data too: the ramp lengths genuinely
 *  differ per skin (measured 0.183 s on Executor, 0.533 on Wiš'adel, 0.600 on Eyjafjalla) and
 *  nothing exported predicts them. See the note in {@link ENTRANCE_FADE_IN}. */
/** Sample a `[t, value]` curve, clamped at both ends. Used for the entrance post-process volume
 *  weight; kept local because `sampleCurveXY` is for the 3-component camera track. */
function sampleCurveAt(curve: [number, number][], t: number): number {
    if (!curve.length) return 0;
    if (t <= curve[0][0]) return curve[0][1];
    const last = curve[curve.length - 1];
    if (t >= last[0]) return last[1];
    for (let i = 1; i < curve.length; i++) {
        if (t <= curve[i][0]) {
            const [t0, v0] = curve[i - 1];
            const [t1, v1] = curve[i];
            const f = t1 > t0 ? (t - t0) / (t1 - t0) : 0;
            return v0 + (v1 - v0) * f;
        }
    }
    return last[1];
}

/** Render density for the dyn illust, in both paths.
 *
 *  The client never draws a dynamic illustration at the coarse density a browser hands us by
 *  default. Its IDLE viewer renders into a SQUARE 2048 target - `AdaptiveRenderStrategy`
 *  returns `*w = *h = 2048` unconditionally, `LegacyRenderStrategy` returns
 *  `max(platformCap, 2048)`, and a live Frida trace reports
 *  `Camera.set_orthographicSize … aspect=1.00000 pixel=2048x2048` - then rebuilds the on-screen
 *  quad from it every frame. Its ENTRANCE takes a different route (the quad rebuild rate drops to
 *  0/s for the whole cinematic while `Camera.allCamerasCount` goes 3 -> 5: the entrance prefab's
 *  own cameras render DIRECTLY) but still lands on a full-resolution surface - `Camera.main` stays
 *  `ui_camera` at **2340x1080** throughout. See `dynchar-il2cpp-render-path`.
 *
 *  Either way the client resolves far more than we did: our frames carried ~0.80x the capture's
 *  high-frequency energy on every reference skin, and it resolved ~2.8x as many distinct bright
 *  points at ~3x smaller size for the same total lit area. So drive the renderer at the density
 *  that puts 2048 device pixels across the illust's frame, for the entrance as well as the idle.
 *
 *  Applied ONCE at construction and on container resize - never switched mid-session. That is
 *  deliberate: changing `renderer.resolution` at runtime invalidates the same state a container
 *  resize does, and re-laying out to recover it overwrites whatever framing the caller was about
 *  to apply (statcam's preview box, the hand-off's standing-idle dolly).
 *
 *  `RES_CAP` bounds the buffer for small containers (a 200px-tall thumbnail would otherwise ask
 *  for 10x); the floor keeps us at no less than the display's own density. `?rt2048=0` reverts. */
const DYN_RT_SIZE = 2048;
/** The client's ENTRANCE does not go through the square RT - it renders through `ui_camera`
 *  straight onto the screen surface, measured at **2340x1080**. So its faithful target is that
 *  screen HEIGHT, not the RT's 2048. Lower than the idle's target, and correspondingly cheaper. */
const DYN_SCREEN_H = 1080;
const RES_CAP = 4;
function dynRenderResolution(cssHeight: number, path: "entrance" | "idle"): number {
    if (typeof window === "undefined") return 1;
    const dpr = window.devicePixelRatio || 1;
    if (new URLSearchParams(window.location.search).get("rt2048") === "0") return dpr;
    if (!(cssHeight > 0)) return dpr;
    const want = path === "idle" ? DYN_RT_SIZE : DYN_SCREEN_H;
    return Math.min(RES_CAP, Math.max(dpr, want / cssHeight));
}

function entranceFadeEnd(data: ISceneData | null): number | null {
    const dur = data?.entranceDuration ?? null;
    if (dur == null) return null;
    // `?fadeend=<seconds>` overrides the anchor outright. DIAGNOSTIC ONLY - it exists to price the
    // prize before anyone invests in deriving a better anchor, because nothing the game ships
    // predicts one: the director's `_params` is just { duration, charVoiceOffset, fadeColor }, with
    // no fade timing at all, so `duration − HOLD` is our own inference. It holds on Executor
    // (clipStop 6.100 − HOLD = 5.900, matching her capture to the frame) and fails badly on
    // Muelsyse, whose capture saturates at 16.767 against an anchor of 20.0 - 3.0s late, and 61%
    // of her worst beat. ⚠️ Do NOT promote a measured value here into a shipped constant; that is
    // per-skin overfitting of exactly the kind this project keeps having to retract.
    if (typeof window !== "undefined") {
        const fe = Number(new URLSearchParams(window.location.search).get("fadeend"));
        if (Number.isFinite(fe) && fe > 0) return fe;
    }
    if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("fadeanchor") === "0") return dur;
    const stop = data?.entranceClipStop ?? null;
    return stop != null && stop < dur ? stop : dur;
}

function sceneDrivesEntranceFade(data: ISceneData | null): boolean {
    const fade = data?.entranceFade;
    // `?scenefade=0` forces the DIRECTOR fade back on even when a scene layer looks like it
    // drives the ramp - the A/B for skins where the layer cannot reach the whole screen.
    if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("scenefade") === "0") return false;
    // Same anchor the ramp itself uses, so the "does the layer beat the director?" test stays
    // consistent with when the director actually finishes.
    const dur = entranceFadeEnd(data);
    if (!data || !fade || !dur || !data.cameraSizePx) return false;
    const ext = 2 * data.cameraSizePx;
    const rampStart = dur - ENTRANCE_FADE_HOLD - ENTRANCE_FADE_IN;
    // The ramp window used to end at `dur - HOLD`, an inference. A scene sheet that is scheduled
    // by the director (an effect root's `_delayTime` plus its own curve, see the exporter's
    // effect-root delay) IS the authored fade timing, and it can reach white right at `dur`:
    // whitw2's `baizhuanchang_02` under `start_04(Clone)` (13.60 s) reaches 0.995 at 14.40 and
    // 1.0 at 14.43 against her `duration` 14.5, where the game is white by scene 14.65 and holds
    // to the cut. Ending the window at HOLD before `dur` rejected it and the director sprite
    // played on top of the sheet (+15 at 14.0, +25 at 14.2 against the game). `?sheetfade=0`
    // restores the `dur - HOLD` end.
    const sheetFadeOn = typeof window === "undefined" || new URLSearchParams(window.location.search).get("sheetfade") !== "0";
    const rampEnd = sheetFadeOn ? dur : dur - ENTRANCE_FADE_HOLD;
    return data.layers.some((l: ISceneLayer) => {
        const cc = l.colorCurve;
        if (!cc?.length) return false;
        // Spans the camera view on BOTH axes - only then can it white out the frame alone.
        let x0 = Infinity;
        let x1 = -Infinity;
        let y0 = Infinity;
        let y1 = -Infinity;
        for (let i = 0; i < l.pos.length; i += 2) {
            x0 = Math.min(x0, l.pos[i]);
            x1 = Math.max(x1, l.pos[i]);
            y0 = Math.min(y0, l.pos[i + 1]);
            y1 = Math.max(y1, l.pos[i + 1]);
        }
        if (x1 - x0 < ext || y1 - y0 < ext) return false;
        // Reaches the fade colour, opaque, by the time the director ramp would finish.
        return cc.some((k: number[]) => k[0] <= rampEnd + 1e-3 && k[0] >= Math.min(rampStart, cc[cc.length - 1][0]) - 1e-3 && k[4] >= 0.99 && Math.abs(k[1] - fade[0]) < 0.06 && Math.abs(k[2] - fade[1]) < 0.06 && Math.abs(k[3] - fade[2]) < 0.06);
    });
}

/** Additive blending must contribute LIGHT, not COVERAGE.
 *
 *  PIXI's premultiplied ADD is `gl.blendFunc(ONE, ONE)`, which adds the source colour to the
 *  destination - correct - and also adds the source ALPHA to the destination alpha. On an opaque
 *  canvas that is invisible. Inside the HDR pass it is not: the scene renders into a half-float
 *  RGBA target and the tonemap composite honours that alpha, so every additive fragment OCCLUDES
 *  the environment behind it in proportion to its own alpha. A DARK additive sprite therefore
 *  DARKENS the frame, which additive blending is supposed to make impossible.
 *
 *  Measured on Civilight Eterna's `guangyun01` halo before the equivalent fix in the Ram shader:
 *  writing premultiplied black at alpha 0.8 took the frame 63.40 -> 12.43, a factor of
 *  0.196 = 1 - 0.8 - exactly a NORMAL over-blend. With `?nohdr=1` the same write was a perfect
 *  no-op, and forcing `blendMode = ADD` changed nothing, so neither the blend flag nor PIXI's
 *  blend map was at fault: only the alpha channel.
 *
 *  Fixing it in the shader only reaches the Ram path. Patching the blend FUNC reaches every
 *  additive draw - batched sprites included, whose fragments belong to PIXI's own batch shader -
 *  by switching ADD to `blendFuncSeparate(ONE, ONE, ZERO, ONE)`: colour adds, destination alpha
 *  is left alone. `?addalpha=0` reverts.
 */
function patchAdditiveBlendAlpha(app: PIXI.Application): void {
    if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("addalpha") === "0") return;
    const renderer = app.renderer as unknown as { state?: { blendModes?: number[][]; blendMode?: number }; gl?: WebGLRenderingContext };
    const st = renderer.state;
    const gl = renderer.gl;
    if (!st?.blendModes || !gl) return;
    // EXPERIMENT (`?dsta=1`): the authored Particles-L2D alpha convention instead. Every port
    // in the family writes alpha `Zero OneMinusSrcAlpha` (drawing REMOVES destination alpha,
    // see probe_rt) into a target cleared at a=0, and if the game's UI composites the RT with
    // straight alpha, additive-heavy regions dim in proportion to draw count. This arm sets
    // the carve on the ADD modes; the tonemap multiplies by the carved alpha under the same
    // param (uDstA). Additive-only on purpose: the spine draws through NORMAL and does not
    // carry the family's convention.
    const carve = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("dsta") === "1";
    for (const mode of [PIXI.BLEND_MODES.ADD, PIXI.BLEND_MODES.ADD_NPM]) {
        const cur = st.blendModes[mode];
        // Only the plain two-argument forms; anything already separate is left as authored.
        if (!cur || cur.length !== 2) continue;
        st.blendModes[mode] = carve ? [cur[0], cur[1], gl.ZERO, gl.ONE_MINUS_SRC_ALPHA] : [cur[0], cur[1], gl.ZERO, gl.ONE];
    }
    // Drop the cached mode so the new function is applied on the next state change rather
    // than being skipped by StateSystem's early-out.
    st.blendMode = -1;
}

function makeBackdropSprite(backdrop: ILoadedBackdrop, frame: ISceneFrame, spineCentroid: { x: number; y: number }, derived?: { scale: number; offset: [number, number] } | null, textureOverride?: PIXI.Texture): PIXI.Sprite {
    const { centroid } = backdrop;
    const texture = textureOverride ?? backdrop.texture;
    const sprite = new PIXI.Sprite(texture);
    if (derived) {
        // Export-derived placement (see bdxfOn): scale is scene px per art px, offset is the
        // art CENTRE in authored Y-up scene px; the container is Y-down, hence the negation.
        sprite.anchor.set(0.5, 0.5);
        sprite.scale.set(derived.scale);
        sprite.position.set(derived.offset[0], -derived.offset[1]);
        return sprite;
    }
    // Anchor at the illustration's own centroid so `position` places THAT point;
    // register it onto the spine's centroid so the two illustrations overlap by
    // mass (robust where a bounding-box centre is skewed by an arch/railing).
    sprite.anchor.set(centroid.nx, centroid.ny);
    // CONTAIN, not cover. Covering the visible rect was tried to kill the neutral fill showing
    // beyond a contained backdrop (Muelsyse's settled frame is mostly bare canvas on a 2.16:1
    // viewport) and is REFUTED: the game's surround there is a flat NEUTRAL grey/gradient, while
    // the blurred art is coloured - a cover fill puts (84,142,161) blue where the game holds
    // neutral. The surround is the viewer's own chrome, not the skin's art scaled up.
    sprite.scale.set((2 * frame.cameraSizePx) / Math.max(texture.width || 1, texture.height || 1));
    sprite.position.set(spineCentroid.x, spineCentroid.y);
    return sprite;
}

/** The in-game viewer composites the illustration over its OWN backdrop, and that backdrop
 *  is a flat NEUTRAL DARK GREY. Every skin's dynchar camera clears with `m_ClearFlags: 2`
 *  (solid colour) and `m_BackGroundColor.a = 0` - the skin renders into a TRANSPARENT
 *  target, so nothing in the scene fills the frame its art doesn't reach; whatever shows
 *  there belongs to the viewer, not to the skin. Measured on the in-game captures of three
 *  skins with three completely different worlds (Mlynar "Fields of Ruination", Virtuosa
 *  "Diversity Oneness", Skadi2 "Iteration") across ten beats: every uncovered corner reads
 *  32–39 in ALL channels - one neutral tone, the same for every skin at every time.
 *
 *  This supersedes two earlier guesses. A bright studio gradient (`#dddee2`) was ~3× too
 *  bright. Deriving the tone from the SCENE (the mean colour of its largest painted
 *  backdrop) made Mlynar's frame edges a light blue-GREY (73,81,95) - wrong in hue as well as
 *  level, because his backdrop bakes its vignette INTO the texture, so an alpha-weighted mean
 *  reports the lit painted INTERIOR. Any per-scene derivation is wrong in principle here: the
 *  tone is not the scene's to supply. `resizeEnvironmentBg` keeps it covering the viewport.
 *
 *  CORRECTED 2026-07-28: the level was `#232324` (35,35,36), taken from "every uncovered
 *  corner reads 32–39". Those corners were not uncovered. At almost every beat the art reaches
 *  the frame edge - sampled across all three captures, the margins are strongly coloured and
 *  vary 43→208 with the camera, i.e. they are painted scenery, and a dark corner of scenery
 *  reads ~35 exactly like a dark fill. The fill is genuinely exposed only at Mlynar's widest
 *  entrance framing, and there it is unambiguous: flat, neutral, and 76–83 down the FULL height
 *  of both margins and in all four corners. Sampling only the pre-flash window matters - from
 *  t≈13.2 his white transition flash ramps those same margins 79 → 125 → 202 → 255, so a
 *  sample taken a few frames later reads the flash, not the fill.
 *
 *  Measured over the frame INTERIOR this is a clean win with no cost anywhere: Mlynar 20.878 →
 *  19.733 (t=13 −5.57, t=12.4 −3.41, every other beat flat to ±0.02) and Skadi bit-identical,
 *  since his fill is never exposed. Virtuosa is untouched - she takes the `dark` branch.
 *
 *  Scored over the FULL frame it instead reads −0.12/+0.03, with apparent ~0.5 regressions at
 *  t=7 and t=11.8. That is an artifact of the metric, not of this constant: the default window
 *  includes columns 0–35 and 864–899, which are the CAPTURE'S OWN dark UI chrome (temporal SD
 *  ~1.0 vs ~70 in the interior - no renderer signal at all), so raising the fill is scored as a
 *  growing mismatch against a black border that is not ours to match. Confirmed by measuring the
 *  art's transmittance to this fill directly, from two renders that differ only in its value:
 *  it is 0 or 1 and never in between (≥99.5% of interior pixels fully opaque at t=4…11, the
 *  remainder fully uncovered), so no semi-transparent leak exists to trade against. An earlier
 *  note here claimed one and blamed thin layer alphas; that was reading the chrome. */
/*  RE-DERIVATION ATTEMPTED AND REFUTED, 2026-08-29. The exposed fill genuinely measures 83,
 *  not 77, and raising it to 83 still costs +0.512 across the corpus. Both halves of that are
 *  worth keeping, because they say the fill is NOT independently derivable.
 *
 *  The measurement. Render each skin twice differing ONLY in this fill and keep the pixels where
 *  it is UNAMBIGUOUSLY exposed: within 3 of the fill in one arm AND within 3 of black in the
 *  other. That subset is far smaller than "the two renders differ", because most differing
 *  pixels carry SEMI-TRANSPARENT art over the fill (our uncovered median is 80/83/90 sd 8 on
 *  Mlynar, 101/101/101 sd 22 on Wiš'adel), which also retracts the "0 or 1 and never in between"
 *  claim above: partial coverage is common, it just is not what this constant is measured on.
 *  Where the fill IS fully exposed two unrelated skins agree to the tenth - Wiš'adel 83.0/83.0/
 *  83.0 over 82808 px (sd 0.57), Eyjafjalla 83.0/83.0/83.0 over 2185 px (sd 0.00). So 76-83 was
 *  the right band and 83 is its value.
 *
 *  Why shipping it loses anyway. 77 -> 83 moved Mlynar 17.003 -> 16.958, exactly as the exposed
 *  pixels predict, and left ska/exc/cel/excunew bit-identical. But it cost Chen 20.230 -> 20.622
 *  and Wiš'adel 33.432 -> 33.540 (r 0.717 -> 0.713), for +0.512 net. Chen dominates because her
 *  exposed region is not fill at all: it is a black PILLARBOX (0.0/0.0/0.0 over 22148 px) where
 *  our art is narrower than the game's, 90.9% hole in the leftmost column band against ~0 through
 *  the middle four, growing 10.1% -> 23.1% -> 27.7% as her camera pulls out. Wiš'adel regresses
 *  despite 3.43% of her frame sitting at exactly 83, which says a brighter fill AMPLIFIES an
 *  alpha error in the art composited over it.
 *
 *  ⛔ Do not re-derive this constant on its own. It can only be raised to its measured 83 once
 *  the coverage and alpha errors it currently masks are fixed; until then 77 is the value that
 *  scores, not the value that is true.
 *
 *  ⚠️ `?fill=` CANNOT reproduce the refuted arm, and cannot test any brighter fill. It sets
 *  `envBg.tint`, which MULTIPLIES this texture, so `?fill=535353` renders 77x83/255 = 25, not 83
 *  (chyue 20.230 -> 32.200, r .895 -> .763 - a much darker fill, not a brighter one). Reaching 83
 *  would need a tint of 275. The arm above was produced by editing this constant, and that is the
 *  only way to raise it. */
// Fresh per-app texture (the app is destroyed with `texture: true`, so a shared/cached
// texture would be torn down under later mounts).
/** DEV diagnostic (`?assetroot=`): fetch this skin's spine, scene and particle data from an
 *  ALTERNATE export root instead of the backend, so an EXPORTER change can be scored without
 *  the deployed asset tree ever being written. The root is served by the `myrtle:alt-asset-root`
 *  vite plugin (`DYNCHAR_ALT_ROOT`) under the same relative path shape the backend uses.
 *
 *  Read as a MISSING parameter, never a falsy one, and returns undefined when absent so
 *  `chibiAssetURL` takes its normal backend branch and the default is unchanged. */
function assetRoot(): string | undefined {
    if (typeof window === "undefined") return undefined;
    return new URLSearchParams(window.location.search).get("assetroot") ?? undefined;
}

const VIEWER_BACKDROP = "#4d4d4e";

/** ENTRANCE CAMERA CLEAR COLOUR as the environment fill. The skin's entrance `Camera` authors
 *  `m_BackGroundColor` under `m_ClearFlags` 2 (SolidColor), so wherever the art does not reach
 *  during the cinematic the game shows THAT colour, not the viewer's fixed #4d4d4e. It is read
 *  from the data (`entranceClearColor`) rather than chosen: fourteen of the fifteen entrance
 *  cameras author a grey or white the fixed fill approximates (0.3382 = 86 of 255 on nine of
 *  them), and chyue authors pure black, which is exactly her pillarbox. `?camclear=0` reverts to
 *  the fixed fill; without an authored colour the fixed fill is used unchanged. */
function camClearOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("camclear") !== "0";
}

/** Draw the environment fill and settled ground INTO the HDR target, ahead of the scene, so they
 *  take the same knee and composite gamma as the art (see the tick). `?fillpass=0` reverts to
 *  the stage placement, where they reach the screen untransferred. */
function fillPassOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("fillpass") !== "0";
}

function cssHex(rgb: [number, number, number]): string {
    const h = (v: number) =>
        Math.round(Math.max(0, Math.min(1, v)) * 255)
            .toString(16)
            .padStart(2, "0");
    return `#${h(rgb[0])}${h(rgb[1])}${h(rgb[2])}`;
}
/** The ground the game shows once the entrance has passed its transform beat - a flat near-white,
 *  measured off the captures, not assumed: Muelsyse's surround settles to (252.7, 252.5, 252.3)
 *  with a per-channel std of ~10. Flat is the tell that it is a ground and not content, and its
 *  boundary follows the art's own silhouette, so it shows through the illustration's ALPHA rather
 *  than being the render target's edge. See {@link settledGroundOn}. */
const SETTLED_GROUND = "#fcfcfc";

/** Past the entrance's transform beat the game paints a flat near-white ground and stops showing
 *  the static art's vista behind the scene - so the gap fill must stop there too.
 *
 *  The two are ONE defect and neither half works alone: on Muelsyse's t=18, dropping the gap fill
 *  alone is WORSE (49.142 -> 52.632) because the region it vacates falls through to the studio
 *  grey, and whitening the ground alone leaves blurred art over 18.5% of frame. Together:
 *  **49.142 -> 26.910**.
 *
 *  ⚠️ Anchored on `entranceTransform`, NOT on the end-fade ramp and NOT on the hand-off - both are
 *  provably out of reach. `entranceFadeEnd` is min(duration, clipStop) = 20.0 for her, so with
 *  FADE_IN/FADE_HOLD at 0.2 the ramp runs 19.6-19.8 and the hand-off fires at 20.0, while the
 *  scored beat is t=18 and the capture's ground finishes at 16.767. The fade anchor IS the value
 *  that is ~3s late; anchoring to it inherits the error. `entranceTransform` (13.0) leads the
 *  capture's ramp by ~1.4s instead.
 *
 *  🔑 Leading is safe, and this was measured rather than assumed. The risk was saturating to white
 *  while the capture is still mid-ramp, so t=15 (corner ~104) was priced: 32.596 -> 31.357, it
 *  IMPROVES. The reason is that the studio fill is only 2.6% of frame there against 10.3% at t=18 -
 *  an early switch has almost no surface to act on while the scene still covers the frame.
 *
 *  ⚠️ Gap fill is worth real parity DURING the entrance (cello t=2 -0.678), so this gates rather
 *  than disabling it. `?settledground=0` reverts. */
function settledGroundOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("settledground") !== "0";
}

function createEnvironmentBgTexture(dark = false, fill: string = VIEWER_BACKDROP): PIXI.Texture {
    const S = 512;
    const cvs = document.createElement("canvas");
    cvs.width = S;
    cvs.height = S;
    const ctx = cvs.getContext("2d");
    if (ctx) {
        if (dark) {
            // A scene that owns a self-lit DARK painted world (`hasDarkBackdrop`, Virtuosa's
            // mirror-world) composites that world SEMI-TRANSPARENTLY over this fill, so here the
            // fill is not only the frame surround but also a light LEAK through the art - and our
            // layer alphas run thinner than the game's. Its measured optimum therefore sits below
            // the viewer's own grey (Virtuosa MAD 31.89 with this dark gradient vs 32.01 with the
            // viewer grey), so that family keeps the fill as measured.
            const g = ctx.createRadialGradient(S / 2, S * 0.3, S * 0.08, S / 2, S * 0.5, S * 0.78);
            g.addColorStop(0, "#1a1b22");
            g.addColorStop(1, "#0c0d12");
            ctx.fillStyle = g;
        } else {
            ctx.fillStyle = fill;
        }
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
 * skeleton already contains the ENTIRE scene - painted background, effect/glass
 * layers, sparkles - as Spine slots (with their blend modes baked in) and mesh
 * attachments, so we simply render the skeleton and frame it to its own animation
 * bounds (the same approach as the reference aklive2d renderer). pixi-spine
 * honours each slot's blend mode, reproducing the game's compositing.
 */
/** The backdrop wash re-parented into a spine, and the slot it sits in front of. */
interface ISeparatorWash {
    wash: PIXI.Container;
    slotIndex: number;
}

/** Put the `isBackdropParticle` wash between the spine's SEPARATOR parts.
 *
 *  Virtuosa's `bg_*` haze sheets are authored ABOVE `characterSort`, so Unity draws them in
 *  front of the whole skeleton - but the game does not wash her body, because the game SPLITS
 *  the skeleton: `SkeletonRenderer.separatorSlotNames` + `SkeletonRenderSeparator` render it as
 *  two submeshes with the sheet interleaved. Demoting the sheet behind the entire spine (what
 *  `isBackdropParticle` does) therefore fixes the character and breaks the background: her
 *  seven `*_Door_*` architecture slots, which sit BEFORE the separator at draw index 7, lose
 *  the wash the game gives them. Measured at t=2: those pixels read 114 against the game's
 *  193.8, and simply undemoting lands them at 157.3 but ghosts her darks to 145.7 (game 22.2).
 *
 *  `spine.update()` rebuilds `children` from `skeleton.drawOrder` each frame, so the wash is
 *  re-seated every tick rather than parented once.
 *
 *  SHIPPED (`?sepwash=0` disables). The per-frame re-seat once LEAKED - `spine.update()` re-appends slot
 *  containers each frame, so re-seating grows `spine.children` without bound: cello reaches
 *  849 children by t=17 against ~329 slots, one leaked per frame. Fix the leak before
 *  re-enabling; the 15.190 measurement below was taken WITH the leak active.
 *
 *  Previously shipped (`?sepwash=0` disables). At cello t=2 the deficit region goes 114.0 -> 173.8
 *  against the game's 193.8 with NO ghosting (her darks 26.8 vs the game's 25.5; undemoting
 *  gives 151.9). Seating all four sheets regressed the frame (21.969 -> 24.671) - the fix was
 *  to bind each sheet by SORT to the parts' own depths (see `sheetTarget` in particles.ts),
 *  which puts bg_ref/bg_tint_01/air_01 in the gap and leaves bg_rain_01 genuinely in front.
 *  cel 18.844 -> 17.167, ska 10.500 -> 10.465, mly bit-identical (ships no separator).
 */
/** `?actwin=0` disables the per-layer `m_IsActive` window gating (diagnostic). */
function activeWindowsOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("actwin") !== "0";
}

/** `?statictail=1`: an EXPIRED layer reverts to its static idle tint rather than hiding
 *  (models a mid-cinematic cut from the `_Start` scene to the idle scene). */
function staticTailOn(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("statictail") === "1";
}

function reseatSeparatorWash(spine: unknown, seps: ISeparatorWash[]): void {
    if (!seps.length) return;
    const sp = spine as unknown as { children: PIXI.DisplayObject[]; addChildAt(c: PIXI.DisplayObject, i: number): unknown; slotContainers?: PIXI.DisplayObject[] };
    const conts = sp.slotContainers;
    if (!conts) return;
    // pixi-spine re-adds every slot container each `update()` to restore `skeleton.drawOrder`.
    // With a FOREIGN child spliced in, its index bookkeeping stops matching and containers get
    // appended instead of moved, so `children` grows one per frame - cello reached 849 against
    // 328 slots by t=17 (a leak plus a per-frame traversal cost on every render).
    //
    // Dedupe in place before re-seating, keeping the LAST occurrence of each child: the last
    // pass is pixi-spine's most recent ordering, so that is the authoritative one. Cheap (one
    // O(n) sweep) and it bounds `children` at slots + foreign for good.
    {
        const kids = sp.children;
        const seen = new Set<PIXI.DisplayObject>();
        let w = kids.length;
        for (let i = kids.length - 1; i >= 0; i--) {
            const c = kids[i];
            if (seen.has(c)) continue;
            seen.add(c);
            kids[--w] = c;
        }
        if (w > 0) kids.splice(0, w);
    }
    // `seps` arrives sorted deepest-slot-first, and each insert goes immediately BEFORE its
    // slot container, so walking forward preserves both the slot order and the order within
    // a gap. The index is re-read every time rather than cached, because each insert shifts it.
    for (let k = 0; k < seps.length; k++) {
        const sep = seps[k];
        if (sep.slotIndex <= 0 || sep.slotIndex >= conts.length) continue;
        const at = sp.children.indexOf(conts[sep.slotIndex]);
        if (at < 0) continue;
        if (sp.children[at - 1] === sep.wash) continue; // already seated
        sp.addChildAt(sep.wash, at);
        if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("sepdbg") === "1") {
            {
                const b = (sep.wash as unknown as { getBounds(): { x: number; y: number; width: number; height: number } }).getBounds();
                console.log(`DBGBG seat k=${k} slotIndex=${sep.slotIndex} at=${at} washKids=${sep.wash.children.length} box=${Math.round(b.x)},${Math.round(b.y)},${Math.round(b.width)}x${Math.round(b.height)} spineKids=${sp.children.length}`);
            }
        }
    }
}

export function SceneIllust({ files, server, fit = DEFAULT_SPINE_FIT, framing = "character", backdrop, surface = "viewer", onReady }: ISceneIllustProps) {
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
    const attachRestRef = useRef<Map<string, { x: number; y: number }> | null>(null);
    // HDR bloom pass: when float render targets are available, the scene is drawn
    // into `hdr.target` (half-float, additive stacks don't clip) and tonemapped to
    // screen via `hdr.mesh`. `hdrSceneRef` is the container rendered into it.
    const hdrRef = useRef<IHDRScene | null>(null);
    /** The gap-fill backdrop sprite, when it is composited by the tonemap threshold rather than
     *  drawn inline - rendered alone into `hdr.bdTarget` each frame (see the tick). */
    const hdrSceneRef = useRef<PIXI.Container | null>(null);
    /** The backdrop wash re-parented inside the spine (see {@link reseatSeparatorWash}). */
    const separatorWashRef = useRef<ISeparatorWash[]>([]);
    const envBgRef = useRef<PIXI.Sprite | null>(null);
    /** Whether the live environment fill is the DARK-backdrop variant. The settled swap skips
     *  those: that fill is not only the surround but a measured light LEAK through a
     *  semi-transparent painted world (see createEnvironmentBgTexture), so whitening it would
     *  change the art itself, and no reference scores a dark-backdrop skin this late. */
    const envBgDarkRef = useRef(false);
    /** The SETTLED ground, held above the viewer backdrop and faded in. Both captures show the
     *  ground RAMP rather than switch, so a step cannot match either: at whitw2's t=13.5 the game
     *  reads 151.6 where our dark ground gives 99.3 and our white 200 - it is midway. */
    const settledBgRef = useRef<PIXI.Sprite | null>(null);
    /** Latched once the entrance clock passes `entranceTransform`. A LATCH, not a one-shot sweep:
     *  the settled composite is BUILT after the beat and constructs its own gap-fill sprite, so
     *  anything built from here on must consult this. See {@link settledGroundOn}. */
    const settledRef = useRef(false);
    // Every composite built for the current skin (usually one; two while a "_Start"
    // entrance is playing before it hands off to the settled main L2D). Tracked so
    // cleanup frees them all - under the HDR pass their containers live OUTSIDE the
    // stage, so app.destroy() can't reach them.
    const compositesRef = useRef<IComposite[]>([]);
    // Set when the entrance's "Start" completes; the tick runs it AFTER the spine
    // update returns (the completion fires mid-update, so freeing the entrance spine
    // there would corrupt the in-progress update).
    const doSwapRef = useRef<(() => void) | null>(null);
    // Camera dolly ZOOM, driven by GAMEDATA: the `_Start` clip animates the Main Camera's
    // orthographic size (verified by extracting the curve - Virtuosa holds 1.87, zooms in to 1.50
    // on the transform at ~8.6s, out to 1.91 for the standing reveal). The camera itself has NO
    // positional curve - the pan lives in the animated camera-PARENT rig (`camCenter`). Each
    // frame we sample the curve at the clip's track time and scale the entrance frame by the ratio
    // to its t=0 value (so no world↔authored unit conversion), zooming around the frame centre.
    /** ENTRANCE LAYER SEQUENCING - the `_Start` clip's per-layer `m_IsActive` windows,
     *  material-colour curves and UV-ST curves. Kept SEPARATE from `entranceFollowRef`
     *  because that ref only exists when the skin ships a camera-centre curve, and layer
     *  sequencing has nothing to do with the camera: gating it on the camera track left
     *  three skins (Kalt'sits "boc#6", Cetsyr, Chongyue "epoque#7") replaying their entrance
     *  with EVERY layer at its static tint. For Kalt'sits that means the closing full-frame
     *  white flash - authored white at full alpha - paints the whole cinematic pure white. */
    const entranceSeqRef = useRef<{
        spine: import("pixi-spine").Spine;
        sceneLayers: PIXI.Container[];
        endAt: number | null;
        fireEnd: (() => void) | null;
        aperture: ISceneAperture | null;
        apertureMask: PIXI.Graphics | null;
    } | null>(null);
    const entranceFollowRef = useRef<{
        spine: import("pixi-spine").Spine;
        root: PIXI.Container;
        ortho: [number, number][] | null;
        camCenter: [number, number, number][];
        /** Authored camera roll, degrees. Null for every rig whose basis is axis-aligned. */
        camRoll: [number, number][] | null;
        /** Segment end-indices in `camCenter` that are HARD CUTS (Skadi the Corrupting Heart's
         *  rig repositions) - the sampler STEPS through these instead of interpolating, so an
         *  instant reposition isn't smeared into a visible pan. Empty for cut-free curves. */
        camCuts: Set<number>;
        frameSize: number;
        /** Steady framing (see the tick): for skins with NO authored transform beat (Mlynar), the
         *  baked rig-centre curve swings the hero into the right third (its X excursion) and the
         *  game instead holds him CENTRED, letting only the ortho zoom widen the shot. We HOLD the
         *  horizontal centre at the settle-open box centre (`cx0`), KEEP the rig's vertical motion
         *  re-based so it lands on `cy0` at the rig's end (`cy0 + (rigY − rigEndY)` - needed so the
         *  tight t=0 frame catches the reforming head), and let the ortho zoom pull out to full body
         *  uncapped (the wide→tight idle handoff hides behind the white flash - same settle centre).
         *  Null = pure rig camera (cello, skadi2). */
        centerBlend: { cx0: number; cy0: number; rigEndY: number } | null;
        /** The live rig camera's `camCenter` sample at track-time 0, captured once at build
         *  time - the reference point for `lastLiveCenter` below, so `swapToMainIdle` can
         *  compute how far the camera panned by hand-off (see Sf handoff-continuity fix in
         *  `openStandingIdle`). */
        startCenter: [number, number];
        /** The most recently sampled live camera centre (refreshed every tick, same value
         *  used to build `liveDisplayBox`). Read by `swapToMainIdle` at the exact hand-off
         *  instant - BEFORE this ref is nulled - so the post-handoff dolly can start from
         *  where the entrance camera actually ended rather than a value re-derived
         *  independently from the idle skeleton's own static bounds. Null until the first
         *  tick runs. */
        lastLiveCenter: [number, number] | null;
        /** The RAW entrance camera centre (mesh px) sampled the same frame as
         *  {@link lastLiveCenter}. The aperture needs it to express the scope rim's motion
         *  RELATIVE to the camera; `lastLiveCenter` alone has skin-specific re-basing folded in. */
        lastCamRaw: [number, number] | null;
        sceneLayers: PIXI.Container[];
        /** Deferred entrance end (see IComposite.entranceSceneEnd): fire `fireEnd`
         *  when the track clock reaches `endAt`. Null when the spine's own
         *  `complete` handles the handoff. */
        endAt: number | null;
        fireEnd: (() => void) | null;
    } | null>(null);
    // Crossfade the `_Start` cinematic OUT (frozen at its dissolved final frame) while the main
    // gala idle fades IN - so the character is never absent during the transformation (the game
    // overlaps the reform with the dissolve). Both roots are wrapped in one container the HDR
    // pass renders; the tick ramps their alphas, then detaches the main and frees the entrance.
    // END-OF-ENTRANCE SCREEN FADE. The director carries the COLOUR (`_params.fadeColor`) and the
    // END TIME (`_params.duration`) as data; only the ramp lengths are client behaviour, and they
    // are measured, not guessed. Virtuosa is the one reference where the fade is unconfounded -
    // Mlynar's own white-transition plane already holds the frame white from t=15.0, and Skadi
    // reaches white through her authored fade LAYERS - and her recording runs
    // 134 → 146 → 178 → 211 → 240 → 254 over `duration - 1.0s` → `duration - 0.2s`.
    /** Gap-fill sprite + the layers CAPABLE of covering the frame. Whether one actually covers is
     *  re-evaluated every entrance frame from its clip window and animated alpha, so a transition
     *  flash cannot suppress the backdrop for a whole cinematic. */
    const gapFillRef = useRef<{ sprite: PIXI.Sprite; covers: ISceneLayer[] } | null>(null);
    /** ENTRANCE POST-PROCESS. Every dyn-illust entrance ships a `pp` PostProcessVolume whose
     *  WEIGHT the `_Start` clip animates, and we reproduced none of it. Whislash the Decadenza's
     *  profile is `HGGreyScale` at intensity 1.0: her capture is objectively monochrome early on
     *  (mean saturation 0.000 at t=2, 0.006 at t=4) while we rendered full colour, which is also
     *  why her CHROMA error dwarfed every other skin's (15.5 vs ~2).
     *
     *  Driven off the entrance clock like the fade, and gated on the exported data, so a skin
     *  with no volume - or one whose volume is never opened - is untouched. Civilight Eterna
     *  ships an `HGMobileBlur` profile whose weight measures 0 at every scored beat, and she is
     *  bit-identical with this in place. */
    const entrancePostFxRef = useRef<{
        filter: PIXI.ColorMatrixFilter | PIXI.BlurFilter;
        kind: "saturation" | "blur";
        /** Full-strength blur radius in px at weight 1, derived from the profile. */
        blurPx: number;
        curve: [number, number][];
        intensity: number;
        target: PIXI.Container;
        /** Its OWN entrance clock. This used to piggy-back on the screen fade's `elapsed`, which
         *  silently disabled the whole subsystem for any skin whose director authors no fade
         *  colour - Civilight Eterna ships an `HGMobileBlur` volume and `entranceFade` null, so
         *  her blur never ran. */
        elapsed: number;
    } | null>(null);
    const entranceFadeRef = useRef<{ sprite: PIXI.Sprite; elapsed: number; duration: number; out: number | null; transform: number | null } | null>(null);
    /** Gap-fill vista sprites in the tree, so the transform beat can retire them. A list: while the
     *  entrance hands off, TWO composites are alive and each builds its own. */
    const gapFillSpritesRef = useRef<PIXI.Sprite[]>([]);
    const crossfadeRef = useRef<{ wrapper: PIXI.Container; mainRoot: PIXI.Container; entRoot: PIXI.Container; ent: IComposite; elapsed: number; duration: number } | null>(null);
    /** `?mainunder=1` state: the main composite drawn under the rig, where its spine was
     *  taken from (to put back at the hand-off), and how many main slots had no same-name
     *  rig slot (excluded by the guard, counted for the report). */
    const mainUnderRef = useRef<{ main: IComposite; ent: IComposite; underRoot: PIXI.Container; spineParent: PIXI.Container; spineIndex: number; noCounterpart: number; rigNever: Set<string> } | null>(null);
    // The opening zoom: the in-game viewer opens on a tight close-up of the character
    // and zooms OUT to the steady framing over a fraction of a second, then holds. This
    // interpolates the entrance's framing from `from` (close) to `to` (steady) over
    // `duration` seconds of REAL time (not animation progress - the zoom is a fixed
    // ~0.7s regardless of the 17s entrance), then clears itself so the framing holds.
    const entranceZoomRef = useRef<{ container: PIXI.Container; from: IAnimationBounds; to: IAnimationBounds; elapsed: number; duration: number; delay: number; fit?: ISpineFit } | null>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const mountedRef = useRef(true);
    const loadIdRef = useRef(0);
    const onReadyRef = useRef(onReady);
    onReadyRef.current = onReady;
    // "authored" (the in-game viewer) scales the crop to the container HEIGHT - the character
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

        /** Step to the idle target when the idle path takes over: the hand-off, or load completion
         *  for a skin that plays no cinematic. Resizes the density-dependent targets ONLY -
         *  re-laying out here would overwrite the framing the caller is about to apply (statcam's
         *  preview box, the hand-off's standing-idle dolly). Idempotent. */
        const raiseToIdleResolution = () => {
            const a = appRef.current;
            const cw = containerRef.current?.clientWidth ?? 0;
            const ch = containerRef.current?.clientHeight ?? 0;
            if (!a || cw <= 0 || ch <= 0) return;
            const want = dynRenderResolution(ch, "idle");
            if (Math.abs(a.renderer.resolution - want) <= 0.01) return;
            a.renderer.resolution = want;
            a.renderer.resize(cw, ch);
            hdrRef.current?.resize(cw, ch, want);
            if (envBgRef.current) resizeEnvironmentBg(envBgRef.current, cw, ch);
            if (settledBgRef.current) resizeEnvironmentBg(settledBgRef.current, cw, ch);
        };

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
            settledBgRef.current = null;
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

        ensureAdditiveSpriteBoost(); // MUST precede Renderer construction (plugins bind at build)
        const app = new PIXI.Application({
            width: container.clientWidth || 600,
            height: container.clientHeight || 450,
            backgroundAlpha: 0,
            antialias: true,
            // Opens at the ENTRANCE target (the client's screen surface). `raiseToIdleResolution`
            // steps up to the square-2048 RT target when the idle path takes over - which is
            // exactly where the client itself switches between the two.
            resolution: dynRenderResolution(container.clientHeight || 450, staticCamOn() ? "idle" : "entrance"),
            autoDensity: true,
            // The tick below drives EVERYTHING - clock, spine, particles, and both render
            // passes - so PIXI's own ticker must not render as well. Left on it did two
            // things: it drew `app.stage` a second time every frame (double the GPU work for
            // the whole viewer), and it could draw the stage BEFORE the frame's scene pass
            // had written the HDR target, so the tonemap quad sampled a render target that
            // nothing had filled in yet.
            autoStart: false,
        });
        app.ticker.stop();
        patchAdditiveBlendAlpha(app);
        appRef.current = app;
        container.appendChild(app.view as HTMLCanvasElement);

        let lastTick = performance.now();
        let sceneClock = 0;
        const tick = (now: number) => {
            if (!mountedRef.current) return;
            const dt = Math.min((now - lastTick) / 1000, 0.1);
            lastTick = now;
            // Capability A - continuous shader UV-scroll. The Ram-family scene layers scroll
            // their `_MainTex` UVs against Unity `_Time` (seconds since load); replay that with
            // a monotonic scene clock for BOTH idle and entrance composites. Cheap CPU rewrite.
            sceneClock += dt;
            for (const comp of compositesRef.current) {
                for (const m of comp.scrollLayers) applySceneLayerUvScroll(m, sceneClock);
                for (const m of comp.ramLayers) applySceneLayerRamScroll(m, sceneClock);
            }
            if (spineRef.current) {
                spineRef.current.update(dt);
                // Re-seat the backdrop wash BETWEEN the spine's parts. `spine.update()` rebuilds
                // `children` from `skeleton.drawOrder` every frame, so a one-time `addChildAt`
                // does not stick - this has to run after each update.
                reseatSeparatorWash(spineRef.current, separatorWashRef.current);
                if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("sepdbg") === "1") {
                    // biome-ignore lint/suspicious/noExplicitAny: ad hoc ?sepdbg=1 introspection of pixi-spine internals (slotContainers) not present in its public type
                    const sk: any = spineRef.current as any;
                    const kids = sk.children.length;
                    const conts = new Set(sk.slotContainers ?? []);
                    let foreign = 0;
                    for (const c of sk.children) if (!conts.has(c)) foreign++;
                    console.log(`DBGBG kids=${kids} slotConts=${(sk.slotContainers ?? []).length} foreign=${foreign}`);
                }
                // `update` rebuilds the dark shadow slots' meshes each frame; flip them
                // back to non-renderable so the static backdrop's version shows instead.
                if (hideShadowsRef.current) hideRedundantShadowSlots(spineRef.current);
                const mu = mainUnderRef.current;
                if (mu && spineRef.current === mu.ent.spine) {
                    // The main skeleton under the rig (`?mainunder=1`): tick it, mirror the rig's
                    // transforms so both skeletons share pixels (same scene space), then the GUARD:
                    // a main slot renders only while the rig's same-name slot is at alpha 0. The
                    // symmetric idle-under-entrance arm cost -9.53, so nothing the rig draws itself
                    // is admitted, and a slot with no rig counterpart is excluded and counted.
                    mu.main.spine.update(dt);
                    // The rig's framing lives on a nested scene root, not on `ent.root` (copying
                    // that root's transform put the main skeleton unframed at the canvas origin), so
                    // take the rig SPINE's world matrix as of the last render and hold the main
                    // spine at identity under it.
                    const es = mu.ent.spine;
                    const ms = mu.main.spine;
                    mu.underRoot.transform.setFromMatrix(mainAtRigSpine() ? es.transform.localTransform : es.worldTransform);
                    ms.position.set(0, 0);
                    ms.scale.set(1, 1);
                    ms.rotation = 0;
                    const rigSk = es.skeleton as unknown as { findSlot: (n: string) => { color: { a: number } } | null };
                    // Before a delayed Start begins, the current track entry is the empty hold.
                    const cur = (ms.state as unknown as { getCurrent: (i: number) => { animation?: { name?: string } } | null }).getCurrent(0);
                    const holding = !cur?.animation?.name || cur.animation.name === "<empty>";
                    let none = 0;
                    type GuardSlot = { data: { name: string }; currentMesh?: PIXI.DisplayObject; currentSprite?: PIXI.DisplayObject };
                    for (const sl of (ms.skeleton as unknown as { slots: GuardSlot[] }).slots) {
                        const rs = rigSk.findSlot(sl.data.name);
                        const show = !holding && !!rs && rs.color.a <= 0.001 && mu.rigNever.has(sl.data.name);
                        if (!rs) none += 1;
                        const disp = sl.currentMesh ?? sl.currentSprite;
                        if (disp) disp.renderable = show;
                    }
                    mu.noCounterpart = none;
                }
                applySlotDiagnostic(spineRef.current);
                // BONE-FOLLOWING scene layers: a spine-unity `BoneFollower` snaps these
                // quads onto a bone at runtime, so their baked geometry is only an editor
                // pose (Mlynar's sword flare bakes as a streak in the lower-left instead
                // of a halo riding the blade). Rebase them onto the live bone now that
                // the skeleton's world transforms are current for this frame. Only the
                // composite currently being ticked has fresh bones.
                const followSpine = spineRef.current;
                for (const comp of compositesRef.current) {
                    if (comp.spine !== followSpine || !comp.followLayers.length) continue;
                    const find = (name: string) => (followSpine.skeleton.findBone(name) as unknown as { matrix: PIXI.Matrix } | null)?.matrix ?? null;
                    for (const m of comp.followLayers) applySceneLayerFollow(m, find);
                }
            }
            // ENTRANCE camera = the game's OWN camera rig, replayed straight from gamedata.
            // The `_Start` prefab animates the camera's parent Transform (a position curve) as the
            // seated form reforms into the standing form ~3300px higher; the Rust exporter accumulates
            // that camera chain into an ABSOLUTE frame-centre curve (`entranceCamCenterCurve`, mesh px)
            // and pairs it with the authored ortho-size zoom. We sample the centre and scale the frame
            // by the ortho ratio - no measured bounds, no easing, no tuning: purely the rig's motion.
            // Live authored display box for particle edge-clip culling this frame - the
            // entrance camera's live zoom/pan box when driving, the opening zoom-out dolly's
            // box when THAT'S driving, else the active composite's own settled `bounds`. Same
            // coordinate space `layoutSpine` already frames the scene with.
            let liveDisplayBox: IAnimationBounds | null = null;
            const ef = entranceFollowRef.current;
            if (ef && appRef.current && spineRef.current === ef.spine) {
                const tr = ef.spine.state.tracks[0] as unknown as { trackTime?: number } | null;
                const tt = tr?.trackTime ?? 0;
                const { width: sw, height: sh } = appRef.current.screen;
                // Camera CENTRE: sample the accumulated gamedata camera track at the clip time. Its
                // keyframe timing IS the game's dolly/pan - replay it directly.
                // DIAGNOSTIC (`?camlead=<seconds>`): sample the camera CENTRE at `tt + lead`
                // while leaving the spine, scene layers and particles on `tt`. This isolates a
                // CAMERA-vs-CONTENT timing skew from a global clock error - a global error moves
                // both and is already ruled out for cello by a trim sweep that minimises sharply
                // at her shipped offset. Inert at 0. Twin of `?ortholead=` for the zoom.
                const camLead = typeof window !== "undefined" ? parseFloat(new URLSearchParams(window.location.search).get("camlead") ?? "0") || 0 : 0;
                const c = sampleCurveXY(ef.camCenter, tt + camLead, ef.camCuts) ?? [0, 0];
                // Camera SIZE: the gamedata frame extent (`_adjustes` view px) × the ortho-size ratio,
                // so the character grows into the frame exactly as the authored zoom dictates.
                // DIAGNOSTIC (`?ortholead=<seconds>`): sample the ZOOM curve at `tt + lead` only,
                // leaving the pan/centre on `tt`. Isolates a zoom-specific timing error from a
                // global clock error (which would move the centre too). Inert at 0.
                const orthoLead = typeof window !== "undefined" ? parseFloat(new URLSearchParams(window.location.search).get("ortholead") ?? "0") || 0 : 0;
                let size = ef.frameSize * orthoZoomRatio(ef.ortho, tt + orthoLead);
                // DIAGNOSTIC (`?camscale=`): multiply the entrance frame extent. Separates a
                // wrong-ZOOM error from the wrong-CENTRE error `?camdx=`/`?camdy=` probe.
                if (typeof window !== "undefined") {
                    const cs = parseFloat(new URLSearchParams(window.location.search).get("camscale") ?? "");
                    if (Number.isFinite(cs) && cs > 0) size *= cs;
                }
                // STEADY FRAMING (skins with no authored `_transform` beat, e.g. Mlynar "Fields of
                // Ruination"): measured against the game recording, the baked rig-centre curve swings
                // the character HORIZONTALLY into the right third (its X excursion reaches ~-355 off
                // the settle centre at t≈7) and its Y curve parks the reforming form low. The in-game
                // operator viewer instead holds the hero CENTRED the whole entrance and lets only the
                // ortho zoom widen the shot - pulling out to a near-FULL-BODY frame (feet + campfire)
                // by t≈12-13, then the post-flash idle settles back tighter. So we:
                //  - FOLLOW the rig's horizontal curve RAW (`cx = rigX`). Measured against the
                //    recording at nine beats spanning t=7→14, the game's horizontal framing tracks
                //    the baked curve exactly: converting each frame's best-fit shift back into mesh
                //    px reproduces `rigX` to within ~5 % over offsets from 441 px down to 39 px
                //    (t=7 predicts 441, measures 450; t=9.5 predicts 89, measures 85; t=13 predicts
                //    39, measures 40). Two earlier readings were wrong in opposite directions: the
                //    RE-BASED form `cx0 + (rigX − rigEndX)` added a constant ~+81 px (`cx0` sits
                //    that far from `rigEndX`) and threw the hero right, and the reaction to that -
                //    holding `cx0` - discarded a real authored pan. The raw curve is the one the
                //    game replays;
                //  - KEEP the rig's VERTICAL motion re-based onto the settle centre (`cy = cy0 +
                //    (rigY − rigEndY)`) - required, or the tight t=0 frame clips the reforming head
                //    (the rig lifts the frame to catch the pose, which sits higher than the idle rest).
                //    Y is NOT raw: raw `rigY` mispredicts by ~250 px where the re-based form is
                //    within ~20;
                //  - follow the ortho zoom-out to full body (NO size cap) - the wide→tight step at the
                //    idle handoff lands behind the white flash (both frames share the settle centre, so
                //    it is a pure scale change) + the 0.45s crossfade, matching the game.
                // All gamedata-derived, no magic constant. Skins WITH an authored transform beat (cello,
                // skadi2) keep the pure rig camera - `centerBlend` is null for them (verified parity).
                let cxShift = 0;
                let cy = c[1];
                // ⛔ RETIRED (`?centerblend=1` restores it). This re-based the rig's VERTICAL curve
                // onto the settle-open centre, `cy0 + (rigY − rigEndY)`, because Mlynar's absolute
                // rig centre could not be trusted. It cannot be trusted no longer: his rig was off
                // by a CONSTANT +273 authored px because his `_Start` pins `Dummy002`'s SCALE to
                // (1, 0, 1) - collapsing the axis that carries his Main Camera's local +2.9947 Y -
                // and `entrance_camera_track` never sampled clip SCALE curves at all. With that
                // fixed the re-base is a measured NO-OP for him (17.192 → 17.191).
                //
                // Keeping it was NOT free: because it depends only on the curve's SHAPE relative
                // to its end, it CANCELS any constant correction to the exported centre - which is
                // exactly what hid Chongyue's remaining vertical error. Retiring it is worth
                // −28.7 on her (78.825 → 50.158, r −0.019 → 0.606) and 0.000 everywhere else.
                const cbOn = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("centerblend") === "1";
                const cb = cbOn ? ef.centerBlend : null;
                if (cb) {
                    cy = cb.cy0 + (c[1] - cb.rigEndY);
                }
                // DIAGNOSTIC (`?camdx=` / `?camdy=`, authored px): shift the entrance camera centre
                // by a constant. A constant framing offset is invisible to BOTH of the obvious
                // tests - a pan-delta comparison sees only differences, and a scale sweep sees only
                // zoom - so it needs its own probe. This is what localised Muelsyse's defect: a
                // sharp optimum at +300 px in Y and exactly 0 in X (MADC 87.168 -> 23.699), which
                // then matched her rig's Main Camera local Y of +2.9947 units to within the
                // sampling grid and led to the zero-scale bug in `entrance_camera_track`.
                if (typeof window !== "undefined") {
                    const q = new URLSearchParams(window.location.search);
                    const ddx = parseFloat(q.get("camdx") ?? "");
                    const ddy = parseFloat(q.get("camdy") ?? "");
                    if (Number.isFinite(ddy)) cy += ddy;
                    if (Number.isFinite(ddx)) cxShift = ddx;
                }
                const cx = c[0] + cxShift;
                liveDisplayBox = { x: cx - size / 2, y: cy - size / 2, width: size, height: size };
                ef.lastLiveCenter = [cx, cy];
                ef.lastCamRaw = [c[0], c[1]];
                layoutSpine(ef.root, sw, sh, { x: cx - size / 2, y: cy - size / 2, width: size, height: size }, fitRef.current);
                // CAMERA-RIDING OVERLAYS. A quad parented to the Main Camera keeps a CONSTANT
                // screen position and size no matter how far the shot dollies - a film-strip
                // border, a lens sheet, a full-frame haze. Its exported vertices are the rest
                // pose, i.e. only correct at t=0, so re-place it against the live frame: the box
                // is centred on `c` with side `size`, so mapping p -> c + (p - c0)·k with
                // k = size/size0 holds it exactly still on screen.
                //
                // whitw2 is the case that exposed it: her 8 `biankuan` (边框, "border") quads are
                // children of `Main Camera/char_01/...`, and her shot dollies 1.2 -> 13 world
                // units, so the baked border flies off frame within a second and her whole
                // cinematic renders without the film strip the game draws over it.
                if (ef.sceneLayers && ef.frameSize > 0) {
                    const [c0x, c0y] = ef.startCenter;
                    for (const cont of ef.sceneLayers) {
                        for (const m of cont.children) {
                            const rt = m as unknown as ISceneLayerRuntime;
                            if (!rt.__camLocked) continue;
                            // Size against the frustum at THIS overlay's own distance rather than the
                            // camera's focal-plane extent (`ISceneLayer.camLockViewPx`).
                            //
                            // Derived from Whislash-alter's prefab chain: her sprocket bars hang at
                            // d=4.36 under a fov-60 rig, so their extent is 503.45px, while
                            // `entranceViewPx` is 346.41 - the frustum at the dolly's d0=3.0. The
                            // 1.4535 ratio was exactly the size-and-position error. With this the
                            // bars land at y 0..36 and 380..416 against 0..35 and 380..415 measured
                            // in her capture.
                            //
                            // ⚠️ This measured WORSE (46.975 -> 57.185) until the reveal timeline
                            // learned to read her cinematic clip, because the game switches the
                            // whole camera-locked group OFF at t=8.13 and we drew it throughout -
                            // correctly-placed bars on beats that should have none. With the gate
                            // in place it pays: 42.903 -> 41.861. `?camlockview=0` reverts.
                            const cvOff = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("camlockview") === "0";
                            const ref = !cvOff && rt.__camLockView && rt.__camLockView > 0 ? rt.__camLockView : ef.frameSize;
                            const k = size / ref;
                            m.scale.set(k);
                            m.position.set(cx - k * c0x, cy - k * c0y);
                        }
                    }
                }
                // DIAGNOSTIC (`?camroll=<deg>`): ROLL the frame about its centre. Wiš'adel's
                // `_Start` clip animates her camera parent's euler Z from 11.34° to 29.56° over
                // the first 2.4s, and `entrance_camera_track` only samples POSITION curves - so
                // the authored roll is frozen at its t=0 value. A roll cannot be approximated by
                // the `?camdx=`/`?camdy=` probe, which is why that one found a wrong-signed
                // optimum. Position is re-solved so the camera centre stays centred.
                {
                    const override = typeof window === "undefined" ? Number.NaN : parseFloat(new URLSearchParams(window.location.search).get("camroll") ?? "");
                    const authored = ef.camRoll?.length ? sampleScalar(ef.camRoll, tt) : 0;
                    const rollDeg = Number.isFinite(override) ? override : authored;
                    if (Number.isFinite(rollDeg) && rollDeg !== 0) {
                        const r = (rollDeg * Math.PI) / 180;
                        ef.root.rotation = r;
                        const p0x = ef.root.position.x;
                        const p0y = ef.root.position.y;
                        const ddx = sw / 2 - p0x;
                        const ddy = sh / 2 - p0y;
                        const co = Math.cos(r);
                        const si = Math.sin(r);
                        ef.root.position.set(sw / 2 - (co * ddx - si * ddy), sh / 2 - (si * ddx + co * ddy));
                    }
                }
                // DIAGNOSTIC: expose the live entrance camera so a probe can read the ACTUAL
                // world-units-per-screen-pixel, instead of inferring the zoom from pixels.
                if (typeof window !== "undefined" && new URLSearchParams(window.location.search).has("cambox")) {
                    (window as unknown as { __camBox?: unknown }).__camBox = {
                        tt,
                        // The spine track's OWN animation length, next to the director clock it is
                        // being driven against. A skeleton whose `Start` is not `entranceDuration`
                        // long drifts in the middle while still lining up at both ends - exactly
                        // Whislash the Decadenza's signature.
                        animDur: (ef.spine.state.tracks[0] as unknown as { animation?: { duration?: number } } | null)?.animation?.duration ?? null,
                        animName: (ef.spine.state.tracks[0] as unknown as { animation?: { name?: string } } | null)?.animation?.name ?? null,
                        timeScale: ef.spine.state.timeScale,
                        cx,
                        cy,
                        size,
                        sw,
                        sh,
                        fit: fitRef.current,
                        rootScaleX: ef.root.scale.x,
                        rootScaleY: ef.root.scale.y,
                        viewWorldH: sh / ef.root.scale.y,
                        viewWorldW: sw / ef.root.scale.x,
                        usingBlend: !!ef.centerBlend,
                        rawCy: c[1],
                    };
                }
            }
            // ENTRANCE LAYER SEQUENCING. Runs for ANY live entrance composite, independent of
            // whether the skin ships a camera track - see `entranceSeqRef`.
            const eseq = entranceSeqRef.current;
            if (eseq && spineRef.current === eseq.spine) {
                const st = eseq.spine.state.tracks[0] as unknown as { trackTime?: number } | null;
                const tt = st?.trackTime ?? 0;
                // GAP-FILL, per frame: hide the defocused backdrop only while something opaque is
                // actually in front of it. Decided at build time this suppressed the fill for a
                // whole cinematic on the strength of a flash lasting under a second.
                const gf = gapFillRef.current;
                if (gf) {
                    // DIAGNOSTIC (`?gapcover=0`): keep the vista visible regardless of the
                    // coverage test. The test asks whether any layer in `covers` is at alpha
                    // >= 0.99, which is a claim about OPACITY, not about whether that layer
                    // actually reaches the pixels the vista is filling - so a spanning-but-
                    // elsewhere layer can suppress the fill over a region it never paints.
                    let covered = false;
                    for (const l of gf.covers) {
                        if (l.activeFrom != null && tt < l.activeFrom) continue;
                        if (l.activeUntil != null && tt > l.activeUntil) continue;
                        let a = l.tint?.[3] ?? 1;
                        if (l.colorCurve?.length) a = sampleColorAlpha(l.colorCurve, tt);
                        if (a >= 0.99) {
                            covered = true;
                            break;
                        }
                    }
                    const gapCoverOn = typeof window === "undefined" || new URLSearchParams(window.location.search).get("gapcover") !== "0";
                    gf.sprite.renderable = gapCoverOn ? !covered : true;
                }
                // Executor: 5.2 s, where the scene's second root activates and the capture goes
                // 100% lit within one frame.
                const ap = eseq.aperture;
                if (ap && eseq.apertureMask) {
                    const live = entranceFollowRef.current?.lastLiveCenter;
                    if (live) {
                        // APERTURE CENTRE. The mask is pinned to the camera centre, which assumes
                        // the scope rim rides the camera exactly. It very nearly does - Executor's
                        // rim tracks the pan at correlation 0.9996 - but it keeps a residual of its
                        // own, and that residual IS the aperture's drift off frame centre. The
                        // exporter decoded the rim's POSITION curve all along and nothing read it
                        // (`entrance_transform_curves` even paired it with the wrong transform).
                        //
                        // Predicted from the curve with no fitting: `restCenter + posCurve(t) −
                        // camCentre(t)` = −23.9 mesh px at her t=5 beat, which is −21.0 screen px
                        // at that frame's zoom. Measured from the capture: the game's aperture
                        // centre sits at x=428 against our 449 - **−21 px**. `?apcenter=0` reverts.
                        //
                        // X ONLY. The camera curve's Y is expressed in a different frame from the
                        // layer `pos` (they disagree by ~400 mesh px, where our vertical aperture
                        // error is just −6 px), and the only rim curve in the corpus has dy ≡ 0, so
                        // there is nothing to validate a Y correction against. Left pinned.
                        const camRaw = entranceFollowRef.current?.lastCamRaw;
                        let apX = live[0];
                        if (ap.posCurve?.length && camRaw && apertureCenterOn()) {
                            const pc = ap.posCurve;
                            let dx = pc[0][1];
                            if (tt <= pc[0][0]) dx = pc[0][1];
                            else if (tt >= pc[pc.length - 1][0]) dx = pc[pc.length - 1][1];
                            else
                                for (let i = 1; i < pc.length; i++) {
                                    if (tt <= pc[i][0]) {
                                        const [t0, x0] = pc[i - 1];
                                        const [t1, x1] = pc[i];
                                        const f = t1 === t0 ? 0 : (tt - t0) / (t1 - t0);
                                        dx = x0 + (x1 - x0) * f;
                                        break;
                                    }
                                }
                            const off = ap.restCenter[0] + dx - camRaw[0];
                            if (Number.isFinite(off)) apX += off;
                        }
                        eseq.apertureMask.position.set(apX, live[1]);
                    }
                    // The rim's own transform is ANIMATED, and the aperture is its inner circle,
                    // so the radius has to follow. Scaling the whole plate is safe: the hole is
                    // centred on the graphic's origin, and the 50000-wide plate still covers the
                    // frame at every multiplier. Decoded from the entrance clip and validated
                    // against the capture - at t=4.0 the multiplier is 0.669 where the measured
                    // width/camera-scale ratio is 0.667. `?apscale=0` restores the frozen radius.
                    if (ap.scaleCurve?.length && apertureScaleOn()) {
                        // Per axis now that the curve carries both. The rim is a circle whose x and
                        // y were previously assumed equal; where they are, this is bit-identical.
                        const [mx, my] = sampleScale2(ap.scaleCurve, tt);
                        if (Number.isFinite(mx) && mx > 0 && Number.isFinite(my) && my > 0) eseq.apertureMask.scale.set(mx, my);
                    }
                    // The rim's own curve carries the whole beat - black hold, then the WHITE
                    // reveal flash, then alpha 0 - so replay it rather than inferring an end.
                    if (ap.curve?.length) {
                        const [r, g, b, a] = sampleColorCurve(ap.curve, tt);
                        const q = (v: number) => Math.max(0, Math.min(255, Math.round(v * 255)));
                        eseq.apertureMask.tint = (q(r) << 16) | (q(g) << 8) | q(b);
                        eseq.apertureMask.alpha = Math.max(0, Math.min(1, a));
                        eseq.apertureMask.visible = a > 0.002;
                    } else if (ap.until != null) {
                        eseq.apertureMask.visible = tt < ap.until;
                    }
                }
                // Per-layer `m_IsActive` window from the `_Start` clips (gamedata): a layer with
                // `activeFrom`/`activeUntil` renders only while `activeFrom <= t < activeUntil`.
                // Absent = always visible (Virtuosa's backdrop is entirely always-on).
                for (const c of eseq.sceneLayers) {
                    for (const m of c.children) {
                        const mm = m as unknown as ISceneLayerRuntime;
                        const af = mm.__activeFrom;
                        const au = mm.__activeUntil;
                        // DIAGNOSTIC `?actwin=0`: ignore the exported windows and leave every
                        // layer on. Virtuosa hides 74 of 132 layers at t=2/5/8 this way -
                        // including 28 flank layers (authored x -770..-332 and +1039..+1208)
                        // that sit exactly where her background is ~80 luma too dark.
                        // A multi-window schedule supersedes the `af`/`au` pair (which carries only
                        // its first window): the layer draws inside ANY of its windows.
                        const aws = mm.__activeWindows;
                        if (aws?.length && activeWindowsOn()) {
                            m.renderable = aws.some(([f, u]) => (f == null || tt >= f) && (u == null || tt < u));
                        } else if ((af != null || au != null) && activeWindowsOn()) {
                            m.renderable = (af == null || tt >= af) && (au == null || tt < au);
                        }
                        // DIAGNOSTIC `?statictail=1`: once a layer's window has EXPIRED, keep it
                        // drawn at its STATIC authored tint instead of hiding it. This models the
                        // hypothesis that the game cuts from the `_Start` scene to the IDLE scene
                        // mid-cinematic: the idle copies of Civilight Eterna's three lavender
                        // fields carry no window and no colour curve, so after her t=12.0 cut they
                        // would come back at full static tint. `?nowin=1` cannot test this - it
                        // leaves the colour curve driving, and these curves end at alpha 0.
                        if (staticTailOn() && !aws?.length && au != null && tt >= au && mm.__staticTint) {
                            m.renderable = true;
                            applySceneLayerColor(m, mm.__staticTint);
                            continue;
                        }
                        // Scene geometry is baked in absolute coordinates: `buildLayerMesh` copies
                        // X and negates Y from `layer.pos`, so replay the exported Y-up pivot and
                        // offset in that same Y-down mesh space. Set absolute transforms each tick
                        // so held endpoint samples neither drift nor retain a previous frame.
                        //
                        // The SCALE term needs its fixed point and has no defensible default: a
                        // uniform scale about the wrong pivot slides the layer across the frame
                        // instead of resizing it in place, and the world origin is a guess, not a
                        // derivation. So it applies ONLY where the exporter supplied `scalePivot`.
                        // No deployed scene JSON carries that field yet (it is emitted by the
                        // exporter change in this same commit and needs a FULL corpus re-export to
                        // appear - exporting a bundle in isolation drops its `ram`, `uvScroll` and
                        // `followBone` data, so it cannot be used to validate this), which is why
                        // every current skin is bit-identical on this arm.
                        //
                        // `posCurve` needs no pivot. The exporter documents it as "authored-px
                        // offsets the frontend ADDS to this layer's rest pose", so it is complete
                        // on its own and applies now.
                        if (mm.__scaleCurve || mm.__posCurve) {
                            if (layerTransformOn()) {
                                const piv = mm.__scalePivot;
                                const [sx, sy] = mm.__scaleCurve && piv ? sampleScale2(mm.__scaleCurve, tt) : [1, 1];
                                const [px, py] = piv ?? [0, 0];
                                const [dx, dy]: [number, number] = mm.__posCurve ? samplePosCurve(mm.__posCurve, tt) : [0, 0];
                                if (Number.isFinite(sx) && sx > 0 && Number.isFinite(sy) && sy > 0 && Number.isFinite(px) && Number.isFinite(py) && Number.isFinite(dx) && Number.isFinite(dy)) {
                                    // Each axis compensates about the pivot with ITS OWN factor, or
                                    // a non-uniform scale would slide the layer as well as resize it.
                                    m.scale.set(sx, sy);
                                    m.position.set(px * (1 - sx) + dx, -py * (1 - sy) - dy);
                                }
                            } else {
                                m.scale.set(1);
                                m.position.set(0, 0);
                            }
                        }
                        // Material-colour replay: the `_Start` clip animates some layers'
                        // material colour (Mlynar's white flash alpha ramps 0→0.671 over
                        // 13→15s); sample the exported curve at the track time and re-tint.
                        // DIAGNOSTIC (?matlead=<s>): sample the material curves at `tt + s`
                        // instead of `tt`. These curves are authored on the CAMERA's clip
                        // (`probe_bind`: the entrance clip's 28 bindings are 20 MeshRenderer
                        // material properties alongside the 4 Transform and 1 Camera ones), and
                        // that clip is the one measured to run ~0.165 s out of step with the
                        // spine. We nevertheless sample it at the SPINE's track time. If the
                        // curves are genuinely on the camera's clock, their error should
                        // minimise away from 0 - and if it minimises AT 0, that is a clean kill.
                        if (mm.__colorCurve) applySceneLayerColor(m, sampleColorCurve(mm.__colorCurve, tt + matLead()));
                        if (mm.__stCurve) applySceneLayerSt(m, tt, sceneClock);
                    }
                }
                // Deferred entrance end: the spine's own "Start" animation ended before the
                // authored scene timeline, so its `complete` listener held off and the handoff
                // waits on the track clock instead - a hair EARLY so the dissolve starts on the
                // flash's final HELD frame rather than a one-frame scene pop after it closes.
                //
                // This MUST live here rather than in the camera-follow tick. Gated on the camera
                // it never fired for a skin that defers its end AND ships no camera track:
                // Wiš'adel "sale#15" held PURE WHITE from 14.5s to past 18s, because the
                // `complete` listener returns early when a deferral is set and nothing else was
                // left to fire it. Same coupling that stranded Kalt'sits's layer sequencing.
                if (eseq.fireEnd && eseq.endAt != null && tt >= eseq.endAt - 0.1) {
                    const fire = eseq.fireEnd;
                    eseq.fireEnd = null;
                    fire();
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
            // End-of-entrance screen fade: ramp to the director's `fadeColor` so the last second
            // of the cinematic whites out exactly as the game's does, hold through the idle swap,
            // then lift. `out` is set by the hand-off; until then the ramp is driven purely by the
            // entrance clock against the authored `duration`.
            // ENTRANCE POST-PROCESS: sample the authored volume weight at the entrance clock and
            // apply the effect. Saturation is 1 - weight*intensity, so weight 0 is a true no-op.
            const pfx = entrancePostFxRef.current;
            if (pfx) {
                // Its OWN clock - see the setup. Driving this off the screen fade's `elapsed`
                // disabled the whole subsystem for every skin with no authored fade colour.
                // Sample BEFORE advancing: the previous form read the screen fade's `elapsed`
                // at this point in the tick, which the fade only increments further down. Keeping
                // that phase makes the greyscale path bit-identical to what it was validated on.
                const w = Math.max(0, Math.min(1, sampleCurveAt(pfx.curve, pfx.elapsed) * pfx.intensity));
                pfx.elapsed += dt;
                if (pfx.kind === "blur") {
                    // The volume's weight fades the effect IN, so it scales the radius.
                    (pfx.filter as PIXI.BlurFilter).blur = pfx.blurPx * w;
                } else {
                    const sat = 1 - w;
                    // Rec.601 luma-preserving saturation matrix (PIXI's ColorMatrixFilter layout).
                    const lr = 0.299 * (1 - sat);
                    const lg = 0.587 * (1 - sat);
                    const lb = 0.114 * (1 - sat);
                    (pfx.filter as PIXI.ColorMatrixFilter).matrix = [lr + sat, lg, lb, 0, 0, lr, lg + sat, lb, 0, 0, lr, lg, lb + sat, 0, 0, 0, 0, 0, 1, 0];
                }
                pfx.target.filters = w > 0.001 ? [pfx.filter] : null;
            }
            const efd = entranceFadeRef.current;
            if (efd) {
                efd.elapsed += dt;
                // SETTLED GROUND, latched off the entrance clock (see settledGroundOn). Driven from
                // the clock rather than the hand-off event because the hand-off fires at
                // `duration`, ~3s after the capture's ground has already finished.
                // DIAGNOSTIC (`?settledfrac=<f>`): move the swap from the TRANSFORM beat to
                // `transform + f*(duration - transform)`, i.e. a fraction of the way from the
                // transform to the authored end. Default 0 = fire at the transform, the shipped
                // behaviour. It exists because the transform anchor was calibrated on ONE skin
                // (Muelsyse) and demonstrably overshoots another: Whislash-alter's ground goes
                // white ~1.6s before the capture's, costing her 41.861 -> 37.731. The captures
                // put mue at ~54% of that gap and whitw2 at ~70%, so a single fraction looked like
                // it might serve both where a single instant cannot.
                //
                // ⛔ PRICED, and the two skins want OPPOSITE things - there is no free fraction:
                //     f      whitw2              mue
                //     0      41.861 (shipped)    10.756 (shipped, best)
                //     0.5    41.861              11.515
                //     0.6    41.861              11.515
                //     0.7    37.731 (best)       11.515
                //     off    37.731              16.372
                // whitw2 needs f>=0.7 (her transform->duration gap is only 1.5s, so anything less
                // still fires before her scored beat); mue is best at f=0 and pays 0.759 for any
                // move. Net at 0.7 is -3.371 across the 12 references, but the corpus-8 MEAN
                // REGRESSES 11.616 -> 11.711 because whitw2 is not in that eight. No other skin is
                // affected either way. Default stays 0 - shipping a 1-parameter fit across two
                // skins with opposed preferences is the overfitting this project keeps retracting.
                // ⚠️ Read the parameter, do not coerce: `?settledfrac=0` is a MEANINGFUL value
                // (fire at the transform, the pre-2026-08-24 behaviour) and a `|| 0` fallback
                // would silently turn it into the default.
                const sgParam = typeof window === "undefined" ? null : new URLSearchParams(window.location.search).get("settledlead");
                const sgRaw = sgParam == null ? SETTLED_GROUND_LEAD : Number(sgParam);
                const sgLead = Number.isFinite(sgRaw) && sgRaw >= 0 ? sgRaw : SETTLED_GROUND_LEAD;
                const sgAt = efd.transform != null ? efd.transform + Math.min(sgLead, Math.max(0, (efd.duration ?? Infinity) - efd.transform)) : efd.transform;
                const sgRamp = settledGroundRamp();
                if (settledGroundOn() && sgAt != null && settledBgRef.current) {
                    settledBgRef.current.alpha = sgRamp > 0 ? Math.max(0, Math.min(1, (efd.elapsed - (sgAt - sgRamp)) / sgRamp)) : efd.elapsed >= sgAt ? 1 : 0;
                }
                // Gap fill is retired AT the lead time regardless of the ramp: the two are
                // coupled (mue with both off is 18.166, worse than either alone), so the region it
                // vacates must already be receiving ground.
                if (settledGroundOn() && !settledRef.current && sgAt != null && efd.elapsed >= sgAt) {
                    settledRef.current = true;
                    for (const sp of gapFillSpritesRef.current) sp.renderable = false;
                    gapFillSpritesRef.current = [];
                }
                let a = 0;
                if (efd.out !== null) {
                    efd.out += dt;
                    a = efd.out <= ENTRANCE_FADE_HOLD ? 1 : Math.max(0, 1 - (efd.out - ENTRANCE_FADE_HOLD) / ENTRANCE_FADE_OUT);
                    if (a <= 0) {
                        efd.sprite.parent?.removeChild(efd.sprite);
                        efd.sprite.destroy();
                        entrancePostFxRef.current = null;
                        entranceFadeRef.current = null;
                    }
                } else {
                    // The ramp COMPLETES at `duration - HOLD`, not at `duration`: the capture is
                    // already pure white 0.2s before the authored end and holds there.
                    const end = efd.duration - ENTRANCE_FADE_HOLD;
                    a = Math.max(0, Math.min(1, (efd.elapsed - (end - ENTRANCE_FADE_IN)) / ENTRANCE_FADE_IN));
                }
                // DIAGNOSTIC (`?fadesprite=0`): suppress the INFERRED global screen fade while
                // keeping this ref alive. `?entfade=0` cannot answer this question because the ref
                // is a COMPOUND switch - it carries the fade sprite AND the clock that drives the
                // settled-ground swap and the gap-fill retirement - so disabling it moves two
                // things at once. Nine of the twelve references author their own fade PLANE with
                // an explicit window and alpha curve (Kal'tsit: sort 100, 13.800..14.667, alpha
                // 0 -> 1 across 28 keys), and instrumented at 14.30 the plane ALONE leaves the
                // frame 1.67% saturated where the plane plus this sprite is 100%. This isolates
                // that. Read the param explicitly, never coerced.
                const spriteOff = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("fadesprite") === "0";
                efd.sprite.alpha = spriteOff ? 0 : a;
            }
            const ez = entranceZoomRef.current;
            if (ez && appRef.current) {
                ez.elapsed += dt;
                const active = Math.max(0, ez.elapsed - ez.delay); // hold at `from` during the delay
                const t = Math.min(1, active / ez.duration);
                const e = t * t * (3 - 2 * t); // smoothstep - symmetric ease for the dolly-out
                const from = ez.from;
                const to = ez.to;
                const b = {
                    x: from.x + (to.x - from.x) * e,
                    y: from.y + (to.y - from.y) * e,
                    width: from.width + (to.width - from.width) * e,
                    height: from.height + (to.height - from.height) * e,
                };
                const { width: sw, height: sh } = appRef.current.screen;
                liveDisplayBox = b;
                layoutSpine(ez.container, sw, sh, b, ez.fit ?? fitRef.current);
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
                    // EXPERIMENT (`?gammadomain=entrance`): confine the fitted composite gamma
                    // to the entrance domain it was calibrated on by restoring exponent 1.0 at
                    // the handoff. Every point in GAMMA_CAL is an entrance-capture sweep minimum
                    // and the table nonetheless applies at settle, where it is uncalibrated
                    // (measured: it currently masks ~7.5 luma of mly's settled deficit). Missing
                    // param = unchanged behaviour.
                    if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("gammadomain") === "entrance") {
                        hdrRef.current?.setGamma(1.0);
                    }
                    cf.ent.destroy(); // frees the entrance root (detaches it from the wrapper)
                    const i = compositesRef.current.indexOf(cf.ent);
                    if (i >= 0) compositesRef.current.splice(i, 1);
                    cf.wrapper.destroy({ children: false });
                }
            }
            if (!liveDisplayBox) {
                const activeComposite = compositesRef.current.find((c) => c.spine === spineRef.current);
                liveDisplayBox = activeComposite?.bounds ?? null;
            }
            // The particle off-screen cull must test what is actually VISIBLE, not the FRAMING
            // box. `layoutSpine` fits that box by its smaller axis, so the 2.16:1 viewport shows
            // ~2.16x its width - culling against the box itself throws away particles sitting
            // plainly on screen. Skadi the Corrupting Heart's crown fish are the proof: 30 of her
            // 36 fish systems never spawned a SINGLE particle across the whole entrance, because
            // the square framing box rejected their spawn->death segment before it was created.
            if (liveDisplayBox && appRef.current) {
                const scr = appRef.current.screen;
                liveDisplayBox = visibleRect(liveDisplayBox, scr.width, scr.height, fitRef.current);
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
                particlesRef.current.update(dt, findBone, boneRestRef.current ?? undefined, liveDisplayBox, attachRestRef.current ?? undefined);
            }
            const currentApp = appRef.current;
            if (currentApp?.renderer) {
                // HDR path: draw the scene into the half-float target (additive stacks
                // accumulate past 1 without clipping), then let the stage's tonemap quad
                // blit it to screen. Otherwise render the stage straight (8-bit).
                if (hdrRef.current && hdrSceneRef.current) {
                    // FILL THROUGH THE PASS (`?fillpass=0` reverts). The environment fill and the
                    // settled ground are stage sprites under the tonemap quad, so they reached the
                    // screen raw while every layer of art went through the knee and the composite
                    // gamma. The game's clear colour is the first thing in its framebuffer and takes
                    // the same transfer as its art, so draw them into the target first, in stage
                    // order, and let the scene composite over them. The quad's alpha then reads 1
                    // wherever a fill was drawn, which is exactly where the stage copy showed
                    // through before. Predicted on the 0.3382 cameras: 86.25 -> 84.4 at gamma 1.02.
                    let clear = true;
                    if (fillPassOn()) {
                        for (const s of [envBgRef.current, settledBgRef.current]) {
                            if (!s || !s.renderable) continue;
                            currentApp.renderer.render(s, { renderTexture: hdrRef.current.target, clear });
                            clear = false;
                        }
                    }
                    currentApp.renderer.render(hdrSceneRef.current, { renderTexture: hdrRef.current.target, clear });
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
        // but NOT yet attached to the stage or started - the orchestrator below decides
        // attach + playback so a special "_Start" ENTRANCE composite can play first and
        // hand off to the settled main L2D. Returns "unsupported" when the MAIN set has
        // no resting frame (entrance-only skeleton → keep the static charart), or null
        // when the load is superseded/aborted or an optional entrance set is absent.
        const buildComposite = async (cSkel: string, cAtlas: string, opts: { mode: "main" | "entrance"; framingOverride?: IAnimationBounds | null; onEntranceEnd?: () => void }): Promise<IComposite | "unsupported" | null> => {
            let spine: import("pixi-spine").Spine;
            try {
                spine = await loadSpineWithEncodedURLs(cSkel, cAtlas, server, assetRoot());
            } catch (e) {
                // An entrance set is OPTIONAL - a missing/failed "_Start" just means the
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
            // DIAGNOSTIC (`?animspeed=<f>`): scale the spine track rate. A `_Start` skeleton whose
            // animation is LONGER than the director's `entranceDuration` must be fitted into the
            // cinematic somehow - compressed or truncated - and the two look identical at both
            // ends while differing through the middle. Whislash the Decadenza's Start runs 18.667s
            // against a 14.5s entrance (ratio 1.2874); every other reference is within 2.4%.
            const animSpeedQ = typeof window !== "undefined" ? parseFloat(new URLSearchParams(window.location.search).get("animspeed") ?? "") : Number.NaN;
            spine.state.timeScale = ANIMATION_SPEED * (Number.isFinite(animSpeedQ) && animSpeedQ > 0 ? animSpeedQ : 1);
            spine.scale.set(1);
            spine.position.set(0, 0);

            const animations = spine.spineData.animations.map((a: { name: string }) => a.name);
            const idle = animations.includes(IDLE_ANIMATION) ? IDLE_ANIMATION : (animations[0] ?? IDLE_ANIMATION);
            // The clip actually PLAYED at settle. The game binds the skeleton's serialized
            // `_animationName` (exported as `settleAnimation`), which is "Idle" on 86 of 104
            // dynchar bindings but AUTHORED per skin: cel settles into "Interact" and
            // rosmon_2 into "Special", and hardcoding "Idle" plays a different animation
            // than the game on exactly those settled surfaces. Assigned after the scene
            // JSON loads (play() runs later); `?settleclip=0` restores the "Idle" binding.
            let settleClip = idle;

            // A MAIN skeleton with no looping "Idle" (only a one-shot "Start" entrance)
            // has no resting frame to display - the entrance never settles into the
            // illustration (e.g. Kal'tsit "Remnant", whose pieces stay converged).
            // Silently fall back to the static charart.
            if (opts.mode === "main" && !animations.includes(IDLE_ANIMATION)) {
                spine.destroy();
                return "unsupported";
            }

            // Periodic "Special" sequences the in-game archive interleaves with the idle
            // loop - the L2D isn't a single static loop, it performs dramatic beats over
            // time (Hoshiguma the Breacher's sword flourishes / sakura "Special_Flower"
            // storm). Collect every non-idle SPECIAL clip (skip the one-shot Start
            // entrance and the tap-only Interact, which isn't auto-played in the archive).
            const specials = animations.filter((n) => /special/i.test(n) && n !== idle && !/_a\d*$/i.test(n));
            // The entrance skeleton has a single "Start" animation (the cinematic zoom-in).
            const entranceAnim = animations.includes("Start") ? "Start" : (animations[0] ?? "Start");
            let entranceEnded = false;
            // Set (below, once the scene loads) when the authored scene timeline outlasts
            // the spine's own "Start" animation: the `complete` listener then DEFERS the
            // handoff and the tick fires it at this track time instead (Mlynar's spine
            // anim ends 14.33s while the white-flash plane runs to the clip stop 15.97s -
            // handing off on `complete` cut the white-out ~1.6s early).
            let deferEndUntil: number | null = null;
            const fireEntranceEnd = () => {
                if (entranceEnded) return;
                entranceEnded = true;
                opts.onEntranceEnd?.();
            };
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
                            if (deferEndUntil != null) return; // the tick fires at the scene end
                            fireEntranceEnd();
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
                // appears promptly as the cinematic dissolves - no slow second intro beat.
                if (!playOpts?.skipStart && animations.includes("Start") && settleClip !== "Start") {
                    state.setAnimation(0, "Start", false);
                    state.addAnimation(0, settleClip, true, 0);
                } else {
                    state.setAnimation(0, settleClip, true);
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
                        if (name !== settleClip) return;
                        idleLoops += 1;
                        if (idleLoops < LOOPS_BETWEEN) return;
                        idleLoops = 0;
                        const special = specials[si % specials.length];
                        si += 1;
                        state.setAnimation(0, special, false);
                        state.addAnimation(0, settleClip, true, 0);
                    },
                });
            };

            // Some skins keep their painted backdrop in separate mesh layers (not the
            // spine). Load them if present; they share the spine's coordinate space, so we
            // nest the spine among them and frame the whole scene to the authored camera.
            const sceneURL = chibiAssetURL(cSkel.replace(/\.skel$/, "[scene].json"), server, assetRoot());
            const textureBaseURL = chibiAssetURL(cSkel.replace(/\.skel$/, "[scene]/"), server, assetRoot());
            const scene = await loadSceneMeshes(sceneURL + bust, textureBaseURL, bust);
            if (aborted()) {
                spine.destroy();
                return null;
            }
            // EXPERIMENT (`?settleclip=1`) - MEASURED AND REFUTED AS A DEFAULT. The exporter
            // now emits the SkeletonMecanim's serialized `_animationName` (settleAnimation:
            // "Interact" on cel, "Special" on rosmon_2, null elsewhere), and the hypothesis
            // was that the game binds it at settle. The measurement says otherwise: under
            // "Interact" cel's settled mean moves AWAY from the game (133.1 -> 141..143 vs
            // the game's flat 133.8, which matches our Idle plateau to 0.7 luma) and the
            // control-validated edge instrument drops 0.101 -> 0.022 across all 105 phases.
            // So the serialized name is the inspector's initial value and the runtime action
            // system (DynIllust.ChangeAction/_ApplyAnimation in the IL2CPP dump) plays Idle
            // at rest; "Interact"-class clips are action/touch responses. Default stays the
            // Idle binding, bit-exact with the previous behaviour; the opt-in arm remains
            // for action-response work.
            {
                const authored = (scene?.data as { settleAnimation?: string | null } | undefined)?.settleAnimation;
                const on = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("settleclip") === "1";
                if (on && authored && animations.includes(authored)) settleClip = authored;
            }

            // Static-art backdrop: some dynamic assets omit the full painted vista
            // (sky/interior) the static illustration has. Draw the static art behind the
            // spine so the animated character overlays its own static counterpart and the
            // missing backdrop fills in. Only for the MAIN set (entrance sets always carry
            // their own scene). The frame comes from the scene data when present, else a
            // bare frame JSON (spine-only skins, e.g. Siege).
            let backdropData: ILoadedBackdrop | null = null;
            let backdropFrame: ISceneFrame | null = null;
            // GAP FILL (?gapfill=1): also load it for the ENTRANCE, where a scene that does
            // not span the camera view leaves bare canvas the game fills with vista.
            if (backdrop && (opts.mode === "main" || gapFillOn())) {
                backdropFrame = (scene && sceneFrameOf(scene.data)) || (await loadSceneFrame(sceneURL + bust));
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
                    const particlesURL = chibiAssetURL(cSkel.replace(/\.skel$/, "[particles].json"), server, assetRoot());
                    const particlesTexBase = chibiAssetURL(cSkel.replace(/\.skel$/, "[particles]/"), server, assetRoot());
                    // Union of the character's own geometry bounds across its FULL played
                    // animation (idle loop, or the "Start" entrance clip) - used by particles.ts
                    // to detect a world-space background particle system whose static spawn disc
                    // sits on the character's own body (Mlynar's dominant rain system) so it can
                    // be un-occluded. A full-clip union (not a single-frame snapshot) is needed
                    // because the character's OWN pose/position moves substantially during the
                    // "Start" reform - a single frame would miss most of the overlap.
                    const characterBounds = measureAnimationBounds(spine, opts.mode === "entrance" ? entranceAnim : idle);
                    particles = await loadParticles(particlesURL + bust, particlesTexBase, bust, characterBounds, scene.hasDarkBackdrop);
                    if (aborted()) {
                        spine.destroy();
                        particles?.destroy();
                        return null;
                    }
                }
                spine.scale.set(1);
                spine.position.set(0, 0);
                const sceneContainer = new PIXI.Container();
                // Scene-fg layers hoisted ABOVE the particle container (sort-driven, see
                // below); tracked so the entrance tick still replays their windows/colours.
                let sceneOverlay: PIXI.Container | null = null;
                // Frame to the character's visible bounds; "authored" (fullscreen) zooms
                // OUT to reveal more of the surrounding scene, "character" (card) keeps the
                // character prominent. Measured first (renders the spine to its own
                // offscreen RT) so the static-backdrop decision can register against the
                // character's centroid. Skipped when a framingOverride is supplied (the
                // entrance reuses the main's bounds - see the orchestrator).
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
                // dynamic carries its OWN painted scene meshes it is self-composed - the static
                // would re-draw the character offset from the L2D one (the "two of her"
                // duplicate: svash2's ice, Nearl "Relight"'s seated throne - the static's
                // character peeks beyond the spine silhouette in the scene's open areas, and no
                // centroid registration lands it exactly when a full scene competes). So: any
                // scene layer at all ⇒ drop the static. Only spine-only art (Siege - a BARE
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
                // phases (the settled scene - Virtuosa's crystal throne - is exported identical
                // to the main scene but only materialises mid-cinematic): each entrance layer
                // carries a per-mesh `m_IsActive` window from the `_Start` clips, applied every
                // frame by the tick (see entranceFollowRef), so layers show exactly when the
                // game toggles them.
                if (scene && !useStatic) sceneContainer.addChild(scene.background);
                // Fallback position when the skin has no separator (or the seat is off): the
                // gap containers are empty then, so this is a no-op.
                if (scene && !useStatic) for (const g of scene.gaps) sceneContainer.addChild(g);
                if (particles) sceneContainer.addChild(particles.background);
                // Sibling by default (drawn exactly where it used to be, inside `background`);
                // moved INSIDE the spine below when the game splits the skeleton.
                if (particles) for (const w of particles.backdropWashes) sceneContainer.addChild(w);
                sceneContainer.addChild(spine);
                // The GAME splits this skeleton and interleaves the backdrop wash between the
                // parts (`separatorSlots`, from Spine-Unity's `separatorSlotNames`). Resolve the
                // split slot to a draw index here; `reseatSeparatorWash` does the per-frame
                // move. Null when the skin ships no separator, which keeps the wash a plain
                // sibling and the render byte-identical.
                let separatorWash: ISeparatorWash[] = [];
                {
                    const sepNames = scene?.data.separatorSlots;
                    // ON by default; `?sepwash=0` disables. Data-gated - a skin with no
                    // `separatorSlots` is untouched (Mlynar ships none and is bit-identical).
                    const on = typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("sepwash") !== "0" : true;
                    if (on && particles && sepNames && sepNames.length > 0) {
                        const slots = (spine as unknown as { skeleton: { slots: { data: { name: string } }[] } }).skeleton.slots;
                        // Draw indices of the split slots, ASCENDING - gap k sits at the k-th
                        // separator in draw order, which is the same ordering as the ascending
                        // `separatorPartSorts` (a lower sort draws earlier). Index 0 would put a
                        // wash behind everything, which the plain demotion already does.
                        const idx = sepNames
                            .map((n) => slots.findIndex((sl) => sl.data.name === n))
                            .filter((i) => i > 0)
                            .sort((a, b) => a - b);
                        // Within one gap the scene layer sits UNDER the particle sheet, matching
                        // the sibling order they have when nothing is seated.
                        const seats: ISeparatorWash[] = [];
                        idx.forEach((slotIndex, k) => {
                            // Skip EMPTY containers: with `?gaplayers` off the scene gaps hold
                            // nothing, and seating them would re-parent a no-op every frame.
                            const sceneGap = scene?.gaps[k];
                            if (sceneGap?.children.length) seats.push({ wash: sceneGap, slotIndex });
                            const w = particles.backdropWashes[k];
                            if (w?.children.length) seats.push({ wash: w, slotIndex });
                        });
                        // Deepest slot FIRST: inserting before a deeper slot cannot move a
                        // shallower one, and within a slot the array order is preserved.
                        seats.sort((a, b) => b.slotIndex - a.slotIndex);
                        separatorWash = seats;
                    }
                }
                if (scene) sceneContainer.addChild(scene.foreground);
                if (particles) sceneContainer.addChild(particles.foreground);
                // The container split draws ALL front-particles above ALL foreground scene
                // layers, but Unity orders both by the same `m_SortingOrder` scale: a scene
                // layer authored ABOVE every particle system must cover the particles too -
                // Mlynar's white transition flash is a sort-100 plane that whites out the
                // whole frame, crystals and sparks included, while his particle systems top
                // out at sort 2. Hoist such above-every-emitter layers over the particle
                // container. Purely sort-data-driven; layers below any emitter stay put.
                if (scene && particles && particles.data.systems.length) {
                    const maxEmitterSort = Math.max(...particles.data.systems.map((s) => s.sort));
                    const hoisted = scene.foreground.children.filter((m) => ((m as unknown as ISceneLayerRuntime).__sort ?? Number.NEGATIVE_INFINITY) > maxEmitterSort);
                    if (hoisted.length) {
                        // Keep them in the SAME scene-fg container tree the entrance tick
                        // walks (window + colour replay) by moving the container itself:
                        // split the fg into [below-particles] + [above-particles overlay].
                        const overlay = new PIXI.Container();
                        for (const m of hoisted) overlay.addChild(m);
                        // NOTE: a container-alpha knob here does NOT work, and the reason is worth
                        // recording. These hoisted layers draw through a CUSTOM shader whose
                        // `uColor` is PREMULTIPLIED (Mlynar's blue sheet dumps
                        // `uColor=[0.121, 0.137, 0.294]`, `tint=None` - i.e. tint x alpha), and
                        // that shader never reads `worldAlpha`. Setting `overlay.alpha` changed
                        // exactly 0 pixels while ablating the same container changed 511,028.
                        // Scaling one of these layers means scaling its `uColor`, not any alpha.
                        sceneContainer.addChild(overlay);
                        sceneOverlay = overlay;
                    }
                }
                // DIAGNOSTIC (`?abl=`): drop whole draw groups to attribute a residual to one
                // of them. `renderable` (not `visible`) because the entrance tick rewrites
                // per-mesh `visible` every frame from the clip's `m_IsActive` window.
                {
                    const abl = typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("abl") : null;
                    if (abl) {
                        const off = new Set(abl.split(","));
                        if (scene && off.has("scenebg")) scene.background.renderable = false;
                        if (scene && off.has("scenefg")) scene.foreground.renderable = false;
                        if (particles && off.has("partbg")) particles.background.renderable = false;
                        if (particles && off.has("partfg")) particles.foreground.renderable = false;
                        // BACKDROP particles are routed to neither container: `loadParticles`
                        // sends an `isBackdropParticle` system to `sheetTarget()`, one of the
                        // `backdropWashes` seated inside the spine's separator gaps. Ablating
                        // only background+foreground therefore left them DRAWN, which read as
                        // "particles are net harmful" on Virtuosa's early beats when they are in
                        // fact worth ~12 MADC there (a true `?psoff=0,…,N` gives t=2 27.326 /
                        // t=5 26.082 against 14.757 / 13.880 for the partial ablation).
                        if (particles && off.has("partbg")) {
                            for (const w of particles.backdropWashes) w.renderable = false;
                        }
                        if (off.has("spine")) spine.renderable = false;
                        // `bg:<i>` drops one background layer, `bgonly:<i>` keeps only that one;
                        // `fg:` / `fgonly:` do the same for the foreground. Ranges allowed
                        // (`bgonly:4-7`). Per-layer isolation is the only way to attribute a
                        // symptom to ONE layer - see the Ch'en silhouette hunt.
                        for (const tok of off) {
                            const m = /^(bg|fg)(only)?:(\d+)(?:-(\d+))?$/.exec(tok);
                            if (!m || !scene) continue;
                            const side = m[1] === "bg" ? scene.background : scene.foreground;
                            if (!side) continue;
                            const lo = Number(m[3]);
                            const hi = m[4] ? Number(m[4]) : lo;
                            side.children.forEach((c, i) => {
                                const inRange = i >= lo && i <= hi;
                                if (m[2]) c.renderable = inRange;
                                else if (inRange) c.renderable = false;
                                // The entrance tick REWRITES `renderable` every frame for any
                                // layer carrying an `m_IsActive` window, which silently undid this
                                // ablation for exactly those layers - so a windowed layer always
                                // measured as "contributes 0 px" no matter what it drew, while
                                // window-less siblings ablated fine. Clear the window on the
                                // layers this token touches so the tick leaves them alone.
                                if (m[2] ? true : inRange) {
                                    const rt = c as unknown as ISceneLayerRuntime;
                                    rt.__activeFrom = undefined;
                                    rt.__activeUntil = undefined;
                                    // The multi-window schedule SUPERSEDES the `af`/`au` pair, and
                                    // clearing only the pair left the tick re-deriving `renderable`
                                    // from the windows every frame - so a windowed layer measured
                                    // as "contributes 0 px" no matter what it drew. Civilight
                                    // Eterna's whole foreground reads that way: every `fg:<i>`
                                    // moved the frame by 0.00 while ablating the container moved
                                    // it by 114 luma.
                                    rt.__activeWindows = undefined;
                                }
                            });
                        }
                    }
                }
                // DIAGNOSTIC (`?partalpha=fg:0.5,bg:0.25`): scale a PARTICLE container's alpha.
                //
                // The scene-layer twin of this is `?layalpha=` below, and it exists for the same
                // reason: `?abl=partfg` answers "does this group contribute error" and cannot
                // answer "should it be here at all, or just weaker". whitw2's particle foreground
                // is worth dMADC -11.577 when ablated outright, which is large enough that the
                // difference between a cull and a blend correction matters a great deal.
                //
                // Container-level rather than per-system, deliberately: it classifies the DEFECT
                // before anyone spends a per-system sweep on it. Nothing in the frame loop writes
                // a particle container's alpha, so this survives.
                {
                    const raw = typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("partalpha") : null;
                    for (const tok of raw ? raw.split(",") : []) {
                        // `fgkids`/`bgkids` scale each EMITTER's own alpha instead of the
                        // container's. The two are different questions and the container form
                        // could not tell them apart: `__partProbe` reads the containers at a flat
                        // alpha 1 / worldAlpha 1 on whitw2 while her emitter children carry
                        // authored 0.1333, 0.2, 0.2667, 0.4 and 0.5333. So a container knob can
                        // only ever test propagation, never whether the authored per-emitter
                        // value reaches the draw.
                        const m = /^(bg|fg|fgkids|bgkids):([0-9.]+)$/.exec(tok);
                        if (!m || !particles) continue;
                        const a = Number(m[2]);
                        if (!Number.isFinite(a)) continue;
                        const kids = m[1].endsWith("kids");
                        const c = m[1].startsWith("bg") ? particles.background : particles.foreground;
                        if (!c) continue;
                        if (kids) for (const ch of c.children) ch.alpha *= a;
                        else c.alpha = a;
                    }
                }
                // DIAGNOSTIC (`__srcProbe`, DEV only): which scene-JSON layer indices actually became
                // meshes, and their live state. `?abl=src:` and `?abl=srconly:` address the SCENE
                // CONTAINERS by `__srcIndex`, so an index that never became a mesh is INVISIBLE to
                // both, and "ablating it changes nothing" then means "it is not there" rather than
                // "it paints nothing". Excu2's aperture set that trap once and kalts's 47/48 look
                // like it again, so this enumerates the built set instead of inferring it.
                if (import.meta.env?.DEV && typeof window !== "undefined") {
                    const wS2 = window as unknown as { __srcProbe?: () => unknown };
                    wS2.__srcProbe = () => {
                        const rows: unknown[] = [];
                        // EVERY container the builder returns, not just two of them. The builder
                        // hands back `{ aperture, background, foreground, gaps }` and a layer can be
                        // routed into a separator GAP instead of either main container, so counting
                        // only background+foreground reproduces the exact blind spot this probe was
                        // written to close.
                        const conts: (PIXI.Container | null | undefined)[] = [scene?.background, scene?.foreground];
                        for (const g of scene?.gaps ?? []) conts.push(g);
                        for (const c of conts) {
                            if (!c) continue;
                            for (const m of c.children) {
                                const rt = m as unknown as ISceneLayerRuntime;
                                rows.push({
                                    src: rt.__srcIndex,
                                    vis: m.visible,
                                    rend: m.renderable,
                                    a: Number(m.alpha.toFixed(3)),
                                    camLocked: !!rt.__camLocked,
                                });
                            }
                        }
                        const built = rows.map((r) => (r as { src?: number }).src).filter((v) => v !== undefined);
                        return {
                            // STAGE COUNTS. The built set is CONTIGUOUS (0..46 of 51), and
                            // contiguous means truncation rather than per-layer rejection, so the
                            // question is where the count drops rather than which layer is refused.
                            loop: (window as unknown as { __sceneCounts?: unknown }).__sceneCounts ?? null,
                            jsonLayers: scene?.data?.layers?.length ?? -1,
                            textureCount: scene?.data?.textureCount ?? -1,
                            count: rows.length,
                            built,
                            has47: built.includes(47),
                            has48: built.includes(48),
                            gapCount: scene?.gaps?.length ?? 0,
                            gapChildren: (scene?.gaps ?? []).map((g) => g.children.length),
                            hasAperture: !!scene?.aperture,
                            rows: rows.slice(0, 60),
                        };
                    };
                }
                // DIAGNOSTIC (`__partProbe`, DEV only, read via `PROBE=__partProbe` in rec.js):
                // the particle containers' OWN alpha and their `worldAlpha`, plus the same for the
                // scene containers as a control.
                //
                // Why it is worth a hook rather than a guess. `?partalpha=` turned out to be a
                // SWITCH on whitw2 rather than an intensity: 1.00/0.25/0.10/0.05/0.01 all render
                // the same frame and only exactly 0 removes anything. That is consistent with the
                // custom Disturb/Ram ports never reading world alpha, and it only MATTERS if some
                // container is actually below 1 during the cinematic. A grep says nothing writes
                // these, but a container inherits its ancestors' alpha through `worldAlpha`, so
                // the grep is a claim and this is the measurement.
                if (import.meta.env?.DEV && typeof window !== "undefined") {
                    const wP = window as unknown as { __partProbe?: () => unknown };
                    wP.__partProbe = () => ({
                        partFg: particles ? { alpha: particles.foreground.alpha, world: particles.foreground.worldAlpha, n: particles.foreground.children.length } : null,
                        partBg: particles ? { alpha: particles.background.alpha, world: particles.background.worldAlpha, n: particles.background.children.length } : null,
                        sceneFg: scene ? { alpha: scene.foreground.alpha, world: scene.foreground.worldAlpha, n: scene.foreground.children.length } : null,
                        sceneBg: scene ? { alpha: scene.background.alpha, world: scene.background.worldAlpha, n: scene.background.children.length } : null,
                        // Per-child alpha inside the particle foreground, which is where a fade
                        // would live if it is authored per emitter rather than on the container.
                        fgChildren: particles ? particles.foreground.children.slice(0, 40).map((c) => Number(c.alpha.toFixed(4))) : null,
                    });
                }
                // DIAGNOSTIC (`?layalpha=fg:20-25:0.5,bg:3:0`): scale one layer range's ALPHA.
                //
                // WHY THIS EXISTS AND `?abl=` DOES NOT REPLACE IT. Removing a layer tells you it
                // contributes error; it cannot tell you WHICH error. The three cases need different
                // fixes and only one of them is a visibility bug:
                //
                //   best alpha 0             the content should not be drawn at all
                //   best alpha intermediate  it belongs but we draw it too strongly, an opacity or
                //                            blend-mode question, and CULLING would be the wrong
                //                            change shipped off the right measurement
                //   best alpha 1             it belongs at full strength and the error is elsewhere,
                //                            placement or timing
                //
                // 🚨 Removing content improves MADC whenever the content is imperfect, whether or
                // not it belongs. This sweep is what separates "wrong" from merely "imperfect".
                //
                // Per-MESH alpha is safe to set here: the entrance tick rewrites `renderable` from
                // the `m_IsActive` windows and sets the CONTAINER alpha to 1, but never touches an
                // individual mesh's alpha, so this survives the frame loop.
                {
                    const raw = typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("layalpha") : null;
                    for (const tok of raw ? raw.split(",") : []) {
                        const m = /^(bg|fg):(\d+)(?:-(\d+))?:([0-9.]+)$/.exec(tok);
                        if (!m || !scene) continue;
                        const side = m[1] === "bg" ? scene.background : scene.foreground;
                        if (!side) continue;
                        const lo = Number(m[2]);
                        const hi = m[3] ? Number(m[3]) : lo;
                        const a = Number(m[4]);
                        if (!Number.isFinite(a)) continue;
                        side.children.forEach((c, i) => {
                            if (i >= lo && i <= hi) c.alpha = a;
                        });
                    }
                }
                {
                    const abl2 = typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("abl") : null;
                    const off = new Set(abl2 ? abl2.split(",") : []);
                    if (off.size) {
                        // `src:<i>` drops the layer whose SCENE-JSON index is `i`; `srconly:<i>`
                        // keeps only that one. Ranges allowed (`src:40-47`). Both sides are
                        // searched, so the caller does not need to know which container a sort
                        // landed in.
                        //
                        // 🔑 Why this exists alongside `bg:`/`fg:`. Those index into the CONTAINER,
                        // and the container order is not the JSON order: `background` receives
                        // `backdropMeshes` before `otherBg`, and the bg/fg split is
                        // `layer.sort >= characterSort` with a `backdropMisSorted` exception. So a
                        // positional token cannot be quoted against the scene JSON without first
                        // dumping, and `?dumplayers=1` returns `[]` whenever the hook closed over a
                        // composite whose `scene` is null, which is exactly what it did on
                        // Executor's settled composite. Matching `__srcIndex` removes the mapping
                        // step and makes an ablation quotable as "layer 40 of the JSON".
                        //
                        // Same window-clearing as the positional tokens: the entrance tick rewrites
                        // `renderable` every frame for any layer carrying an `m_IsActive` window, so
                        // without this a windowed layer measures as contributing nothing whatever it
                        // draws.
                        {
                            const wanted: Array<[number, number, boolean]> = [];
                            for (const tok of off) {
                                const m = /^src(only)?:(\d+)(?:-(\d+))?$/.exec(tok);
                                if (m) wanted.push([Number(m[2]), m[3] ? Number(m[3]) : Number(m[2]), !!m[1]]);
                            }
                            if (wanted.length && scene) {
                                // 🚨 `sceneOverlay` MUST be here. Layers whose `__sort` exceeds every
                                // emitter's are HOISTED out of `scene.foreground` into it (see the
                                // hoist above), so a token that searched only background+foreground
                                // silently matched nothing for them and "ablating it changes nothing"
                                // read identically to "it does not exist". That cost several turns on
                                // Kal'tsit's 47..50, which are hoisted and were reported as unbuilt.
                                let matched = 0;
                                for (const side of [scene.background, scene.foreground, sceneOverlay]) {
                                    if (!side) continue;
                                    for (const c of side.children) {
                                        const rt = c as unknown as ISceneLayerRuntime;
                                        const si = rt.__srcIndex;
                                        if (si == null) continue;
                                        const hit = wanted.some(([lo, hi]) => si >= lo && si <= hi);
                                        if (hit) matched++;
                                        const anyOnly = wanted.some(([, , only]) => only);
                                        if (anyOnly) c.renderable = hit;
                                        else if (hit) c.renderable = false;
                                        // Clear windows ONLY on the layers the token KEEPS. The
                                        // first form cleared them on every layer under `srconly`,
                                        // which is backwards: the entrance tick recomputes
                                        // `renderable` each frame as `(af == null || tt >= af) &&
                                        // (au == null || tt < au)`, so a window-less layer is
                                        // unconditionally turned back ON and the isolation leaked
                                        // the whole scene back in on the very next frame.
                                        if (hit) {
                                            rt.__activeFrom = undefined;
                                            rt.__activeUntil = undefined;
                                            rt.__activeWindows = undefined;
                                        }
                                    }
                                }
                                // LOUD when a requested index matches nothing. Silence here is what
                                // made "it contributes nothing" and "it is not in this container"
                                // indistinguishable; now the null result names itself.
                                if (matched === 0) {
                                    const names = wanted.map(([lo, hi]) => (lo === hi ? `${lo}` : `${lo}-${hi}`)).join(",");
                                    console.warn(`[abl] src token matched NO mesh: ${names}. The index may not exist, or its layer may live outside background/foreground/overlay.`);
                                    (window as unknown as { __ablUnmatched?: string }).__ablUnmatched = names;
                                }
                            }
                        }
                        // `on:bg:<i>` / `on:fg:<i>` clears a layer's `m_IsActive` window so it
                        // stays drawn - tests whether missing content is a mis-timed window.
                        // NOTE THE SECOND COLON: the regex is `on:(bg|fg):(\d+)`. Written
                        // `on:bg12` the token silently does not match, the ablation is a no-op,
                        // and the run reads as "this layer contributes nothing".
                        for (const tok of off) {
                            const m = /^on:(bg|fg):(\d+)$/.exec(tok);
                            if (!m || !scene) continue;
                            const c = (m[1] === "bg" ? scene.background : scene.foreground).children[Number(m[2])];
                            if (!c) continue;
                            const mm = c as unknown as ISceneLayerRuntime;
                            mm.__activeFrom = undefined;
                            mm.__activeUntil = undefined;
                            c.renderable = true;
                        }
                        if (off.has("bgcount") && scene) console.log(`[abl] bgchildren=${scene.background.children.length} fgchildren=${scene.foreground.children.length}`);
                        if (sceneOverlay && (off.has("scenefg") || off.has("overlay"))) sceneOverlay.renderable = false;
                    }
                }
                // DIAGNOSTIC (`?dumplayers=1`): expose live per-layer draw state so a
                // residual can be matched against what each scene layer actually contributes.
                if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("dumplayers")) {
                    (window as unknown as { __particleCensus?: () => unknown }).__particleCensus = () => particleCensus();
                    (window as unknown as { __dumpEmitters?: () => unknown }).__dumpEmitters = () => {
                        const lp = particlesRef.current;
                        const r = appRef.current?.renderer;
                        return lp ? lp.probe(r ? r.width : 0, r ? r.height : 0) : null;
                    };
                    // Per-slot MESH geometry in CANVAS space + atlas UVs, so a rendered pixel can
                    // be reconstructed analytically from the shipped atlas and compared against
                    // both our render and the game.
                    (window as unknown as { __dumpSkins?: () => unknown }).__dumpSkins = () => {
                        const sp = spine as unknown as {
                            skeleton: { data: { skins: { name: string }[] }; skin?: { name: string } | null };
                            state: { tracks: ({ animation?: { name: string; duration: number }; trackTime: number; alpha: number; mixDuration?: number } | null)[] };
                        };
                        return {
                            skins: sp.skeleton.data.skins.map((k) => k.name),
                            active: sp.skeleton.skin ? sp.skeleton.skin.name : null,
                            tracks: (sp.state.tracks || []).filter(Boolean).map((t) => ({
                                anim: t?.animation?.name ?? null,
                                dur: t?.animation?.duration ?? null,
                                time: Number((t?.trackTime ?? 0).toFixed(3)),
                                alpha: t?.alpha ?? null,
                            })),
                        };
                    };
                    (window as unknown as { __dumpMesh?: (n: string) => unknown }).__dumpMesh = (name: string) => {
                        const sk = (spine as unknown as { skeleton: { slots: unknown[] } }).skeleton;
                        for (const slotU of sk.slots) {
                            const sl = slotU as { data: { name: string }; color: { r: number; g: number; b: number; a: number }; currentMesh?: PIXI.Mesh; currentSprite?: PIXI.Sprite };
                            if (sl.data.name !== name) continue;
                            const m = sl.currentMesh;
                            if (!m) return { name, err: "no currentMesh" };
                            const g = m.geometry;
                            const pos = g.getBuffer("aVertexPosition").data as unknown as Float32Array;
                            const uv = g.getBuffer("aTextureCoord").data as unknown as Float32Array;
                            const idx = Array.from(g.getIndex().data as unknown as Uint16Array);
                            const wt = m.worldTransform;
                            const gp: number[] = [];
                            for (let i = 0; i < pos.length; i += 2) {
                                gp.push(wt.a * pos[i] + wt.c * pos[i + 1] + wt.tx, wt.b * pos[i] + wt.d * pos[i + 1] + wt.ty);
                            }
                            const tex = m.texture;
                            return {
                                name,
                                verts: gp,
                                uvs: Array.from(uv),
                                idx,
                                tint: m.tint,
                                alpha: m.worldAlpha,
                                blend: m.blendMode,
                                renderable: m.renderable,
                                visible: m.visible,
                                slotColor: [sl.color.r, sl.color.g, sl.color.b, sl.color.a],
                                baseSize: [tex.baseTexture.realWidth, tex.baseTexture.realHeight],
                                frame: [tex.frame.x, tex.frame.y, tex.frame.width, tex.frame.height],
                                orig: [tex.orig.x, tex.orig.y, tex.orig.width, tex.orig.height],
                                rotate: (tex as unknown as { rotate?: number }).rotate ?? 0,
                                uvMatrix: (() => {
                                    const um = (m.shader as unknown as { uvMatrix?: { mapCoord?: PIXI.Matrix } }).uvMatrix?.mapCoord;
                                    return um ? [um.a, um.b, um.c, um.d, um.tx, um.ty] : null;
                                })(),
                            };
                        }
                        return { name, err: "slot not found" };
                    };
                    // DIAGNOSTIC (`__dumpColorTracks`, DEV only, a PROBE hook): per animation, the
                    // slots whose alpha a colour timeline keys and the alpha range it spans, each
                    // slot's SETUP alpha, and the tracks currently playing. Built for the never-keyed
                    // slot census (kalts Mo_A..E held at alpha 0 at every beat, 2026-09-02): it says
                    // which clip keys a slot on and whether that clip is the one bound.
                    // Registered under the plain name AND per composite mode
                    // (`__dumpColorTracks_main` / `__dumpColorTracks_entrance`): the plain name is
                    // whichever composite was built last, and once the entrance composite is gone
                    // its hook returns null, which is how kalts's MAIN skeleton went unread.
                    const dumpColorTracks = () => {
                        const sp = spine as unknown as {
                            skeleton: { data: { slots: { name: string; color: { a: number } }[]; animations: { name: string; duration: number; timelines: unknown[] }[] } };
                            state?: { tracks?: ({ animation?: { name?: string }; trackIndex?: number; loop?: boolean } | null)[] };
                        };
                        const slots = sp.skeleton.data.slots;
                        // `?ctracks=<regex>`: for the matching SLOTS, every colour key as [time, alpha]
                        // per animation (kalts Mo_A..E: when the main Start clip fades them in).
                        const ctq = new URLSearchParams(window.location.search).get("ctracks");
                        const ctre = ctq ? new RegExp(ctq) : null;
                        const anims = sp.skeleton.data.animations.map((an) => {
                            const keyed: Record<string, [number, number, number]> = {};
                            const ckeys: Record<string, number[][]> = {};
                            for (const tlU of an.timelines) {
                                const tl = tlU as { slotIndex?: number; frames?: ArrayLike<number>; getFrameEntries?: () => number; constructor: { name: string } };
                                if (typeof tl.slotIndex !== "number" || !tl.frames) continue;
                                const cn = tl.constructor.name;
                                if (!/Color|RGBA|Alpha/.test(cn)) continue;
                                const stride = typeof tl.getFrameEntries === "function" ? tl.getFrameEntries() : cn.includes("Two") ? 8 : 5;
                                const fr = tl.frames;
                                let lo = Number.POSITIVE_INFINITY;
                                let hi = Number.NEGATIVE_INFINITY;
                                for (let i = 0; i + stride - 1 < fr.length; i += stride) {
                                    const a = /^Alpha/.test(cn) ? fr[i + 1] : fr[i + 4];
                                    lo = Math.min(lo, a);
                                    hi = Math.max(hi, a);
                                }
                                const sn = slots[tl.slotIndex]?.name ?? String(tl.slotIndex);
                                keyed[sn] = [Number(lo.toFixed(3)), Number(hi.toFixed(3)), fr.length / stride];
                                if (ctre?.test(sn)) {
                                    const pairs: number[][] = [];
                                    for (let i = 0; i + stride - 1 < fr.length; i += stride) pairs.push([Number(fr[i].toFixed(3)), Number((/^Alpha/.test(cn) ? fr[i + 1] : fr[i + 4]).toFixed(3))]);
                                    ckeys[`${sn}:${cn}`] = pairs;
                                }
                            }
                            return { name: an.name, duration: Number(an.duration.toFixed(3)), keyed, ckeys };
                        });
                        // `?tracks=<regex>`: per animation, the translate/rotate/scale timelines of the
                        // matching BONES with their frame ranges (chen2_2's ship bobbing question).
                        const trq = new URLSearchParams(window.location.search).get("tracks");
                        const bones: Record<string, Record<string, unknown>> = {};
                        if (trq) {
                            const re = new RegExp(trq);
                            const bd = (sp.skeleton.data as unknown as { bones: { name: string; x: number; y: number }[] }).bones;
                            for (const an of sp.skeleton.data.animations) {
                                for (const tlU of an.timelines) {
                                    const tl = tlU as { boneIndex?: number; frames?: ArrayLike<number>; getFrameEntries?: () => number; constructor: { name: string } };
                                    if (typeof tl.boneIndex !== "number" || !tl.frames) continue;
                                    const bn = bd[tl.boneIndex]?.name;
                                    if (!bn || !re.test(bn)) continue;
                                    const cn = tl.constructor.name;
                                    const stride = typeof tl.getFrameEntries === "function" ? tl.getFrameEntries() : /Rotate/.test(cn) ? 2 : 3;
                                    const fr = tl.frames;
                                    const cols: number[][] = [];
                                    for (let i = 0; i + stride - 1 < fr.length; i += stride) cols.push(Array.from({ length: stride }, (_, j) => fr[i + j]));
                                    const rng = cols.length ? cols[0].map((_, j) => [Math.min(...cols.map((c) => c[j])), Math.max(...cols.map((c) => c[j]))].map((v) => Number(v.toFixed(2)))) : [];
                                    if (!bones[bn]) bones[bn] = {};
                                    bones[bn][`${an.name}:${cn}`] = { keys: cols.length, range: rng };
                                }
                            }
                        }
                        return {
                            setup: Object.fromEntries(slots.map((s) => [s.name, Number(s.color.a.toFixed(3))])),
                            playing: (sp.state?.tracks ?? []).filter(Boolean).map((t) => ({ track: t?.trackIndex, anim: t?.animation?.name, loop: t?.loop })),
                            anims,
                            bones,
                        };
                    };
                    const hookHost = window as unknown as Record<string, () => unknown>;
                    hookHost.__dumpColorTracks = dumpColorTracks;
                    hookHost[`__dumpColorTracks_${opts.mode}`] = dumpColorTracks;
                    (window as unknown as { __dumpSlots?: () => unknown }).__dumpSlots = () => {
                        const sk = (spine as unknown as { skeleton: { slots: unknown[] } }).skeleton;
                        // Children of the Spine container that are NOT slot containers (separator
                        // wash, seated layers, anything else parented in): reported after the slots
                        // as `__extra:<ctor>` rows with bounds, for the reed2 painter question.
                        const spc = spine as unknown as { children: PIXI.DisplayObject[]; slotContainers?: PIXI.DisplayObject[] };
                        const slotSet = new Set(spc.slotContainers ?? []);
                        const extras = spc.children
                            .filter((c) => !slotSet.has(c))
                            .map((c) => {
                                const b = c.getBounds();
                                const cc = c as unknown as { renderable?: boolean; visible?: boolean; worldAlpha?: number; blendMode?: number; children?: PIXI.DisplayObject[]; name?: string };
                                // Name the kids (constructor, the builder's `__srcIndex`, texture id,
                                // blend, bounds) so a seated container's content can be matched back
                                // to the scene JSON.
                                const kids = (cc.children ?? []).slice(0, 24).map((k) => {
                                    const kk = k as unknown as { __srcIndex?: number; texture?: { baseTexture?: { uid?: number } }; blendMode?: number; renderable?: boolean; visible?: boolean; worldAlpha?: number; name?: string };
                                    const kb = k.getBounds();
                                    // One level deeper: a slot container's own children (the drawn
                                    // sprite or mesh, or whatever else was parented in), with texture ids.
                                    const grand = ((k as unknown as { children?: PIXI.DisplayObject[] }).children ?? []).slice(0, 6).map((g) => {
                                        const gg = g as unknown as { texture?: { baseTexture?: { uid?: number; resource?: { url?: string } } }; blendMode?: number; renderable?: boolean; visible?: boolean; region?: { name?: string }; name?: string };
                                        const gb = g.getBounds();
                                        return `${g.constructor.name}${gg.region?.name ? `:${gg.region.name}` : gg.name ? `:${gg.name}` : ""} tex ${gg.texture?.baseTexture?.uid ?? "-"} ${(gg.texture?.baseTexture?.resource?.url ?? "").split("/").pop() ?? ""} blend ${gg.blendMode ?? "-"} r${gg.renderable ? 1 : 0}v${gg.visible ? 1 : 0} [${Math.round(gb.x)},${Math.round(gb.y)},${Math.round(gb.width)},${Math.round(gb.height)}]`;
                                    });
                                    const owner = (sk.slots as { data: { name: string }; currentSprite?: unknown; currentMesh?: unknown; getAttachment?: () => { name?: string } | null }[]).find(
                                        (sl) => (k as unknown as { children?: unknown[] }).children?.includes(sl.currentSprite) || (k as unknown as { children?: unknown[] }).children?.includes(sl.currentMesh),
                                    );
                                    return `${k.constructor.name}${kk.name ? `:${kk.name}` : ""}${owner ? ` slot ${owner.data.name} att ${owner.getAttachment?.()?.name ?? "-"}` : ""} src ${kk.__srcIndex ?? "-"} tex ${kk.texture?.baseTexture?.uid ?? "-"} blend ${kk.blendMode ?? "-"} r${kk.renderable ? 1 : 0}v${kk.visible ? 1 : 0} wa ${typeof kk.worldAlpha === "number" ? kk.worldAlpha.toFixed(2) : "-"} box [${Math.round(kb.x)},${Math.round(kb.y)},${Math.round(kb.width)},${Math.round(kb.height)}] {${grand.join(" ; ")}}`;
                                });
                                return {
                                    name: `__extra:${c.constructor.name}${cc.name ? `:${cc.name}` : ""}`,
                                    bone: null,
                                    root: kids.join(" | "),
                                    dataBlend: null,
                                    drawnBlend: cc.blendMode ?? null,
                                    col: null,
                                    dark: null,
                                    att: cc.children ? `kids:${cc.children.length}` : null,
                                    rend: cc.renderable ?? null,
                                    vis: cc.visible ?? null,
                                    wa: typeof cc.worldAlpha === "number" ? Number(cc.worldAlpha.toFixed(3)) : null,
                                    box: [Math.round(b.x), Math.round(b.y), Math.round(b.width), Math.round(b.height)],
                                };
                            });
                        return sk.slots
                            .map((slotU) => {
                                const sl = slotU as {
                                    data: { name: string; blendMode: number; darkColor?: unknown };
                                    color: { r: number; g: number; b: number; a: number };
                                    darkColor?: { r: number; g: number; b: number };
                                    getAttachment?: () => { name?: string } | null;
                                    currentMesh?: { blendMode: number; alpha: number; renderable: boolean; visible: boolean; worldAlpha: number };
                                    currentSprite?: { blendMode: number; alpha: number; renderable: boolean; visible: boolean; worldAlpha: number };
                                };
                                const disp = sl.currentMesh ?? sl.currentSprite;
                                const d2 = disp as unknown as { renderable?: boolean; visible?: boolean; worldAlpha?: number } | undefined;
                                return {
                                    name: sl.data.name,
                                    // Bone + its ROOT ancestor: the only structural handle for
                                    // splitting scenery slots from character slots.
                                    bone: (sl as unknown as { bone?: { data?: { name?: string } } }).bone?.data?.name ?? null,
                                    root: (() => {
                                        let b = (sl as unknown as { bone?: { parent?: unknown; data?: { name?: string } } }).bone as { parent?: { parent?: unknown; data?: { name?: string } } | null; data?: { name?: string } } | undefined;
                                        let guard = 0;
                                        while (b?.parent && guard++ < 64) b = b.parent as typeof b;
                                        return b?.data?.name ?? null;
                                    })(),
                                    dataBlend: sl.data.blendMode,
                                    drawnBlend: disp ? disp.blendMode : null,
                                    col: [sl.color.r, sl.color.g, sl.color.b, sl.color.a].map((x) => Number(x.toFixed(3))),
                                    dark: sl.darkColor ? [sl.darkColor.r, sl.darkColor.g, sl.darkColor.b].map((x) => Number(x.toFixed(3))) : null,
                                    att: sl.getAttachment ? (sl.getAttachment()?.name ?? null) : null,
                                    rend: d2 ? !!d2.renderable : null,
                                    vis: d2 ? !!d2.visible : null,
                                    wa: d2 && typeof d2.worldAlpha === "number" ? Number(d2.worldAlpha.toFixed(3)) : null,
                                    // Canvas-space bounds, so a slot can be matched to a REGION of
                                    // the residual instead of guessed at from its name.
                                    box: (() => {
                                        const b = (disp as unknown as { getBounds?: () => { x: number; y: number; width: number; height: number } } | undefined)?.getBounds?.();
                                        return b ? [Math.round(b.x), Math.round(b.y), Math.round(b.width), Math.round(b.height)] : null;
                                    })(),
                                };
                            })
                            .concat(extras as never[]);
                    };
                    (window as unknown as { __dumpTex?: () => unknown }).__dumpTex = () => {
                        const cache = (PIXI.utils as unknown as { BaseTextureCache: Record<string, PIXI.BaseTexture> }).BaseTextureCache;
                        // `b.mipmap` is only the REQUESTED mode. PIXI silently falls back to no
                        // mipmap chain where GL cannot build one (an NPOT page on WebGL1), so
                        // also report what actually reached the GPU and the context version -
                        // an unmipmapped 2280px atlas minified ~3x at the widest camera aliases
                        // thin line art into over-dark pixels.
                        const r = appRef.current?.renderer as unknown as {
                            CONTEXT_UID?: number;
                            context?: { webGLVersion?: number };
                        } | null;
                        const uid = r?.CONTEXT_UID ?? -1;
                        return {
                            webGLVersion: r?.context?.webGLVersion ?? null,
                            textures: Object.entries(cache).map(([k, b]) => {
                                const gl = (b as unknown as { _glTextures?: Record<number, { mipmap?: boolean }> })._glTextures?.[uid];
                                return {
                                    url: k.slice(-58),
                                    alphaMode: b.alphaMode,
                                    mipmapRequested: b.mipmap,
                                    mipmapUploaded: gl ? !!gl.mipmap : null,
                                    size: [b.realWidth, b.realHeight],
                                };
                            }),
                        };
                    };
                    // LIVE STAGE WALK. `__dumpLayers` below closes over ONE composite's
                    // containers, and the entrance composite is destroyed at the hand-off
                    // (`sceneContainer.destroy({children: true})`) - so past the hand-off it keeps
                    // answering from an emptied container and reports 0 rows, which reads exactly
                    // like "the settled idle draws no scene layers". It cost a wrong diagnosis
                    // once. This one walks the actual stage, so it is correct at any time.
                    (window as unknown as { __dumpStage?: () => unknown }).__dumpStage = () => {
                        const rows: { path: string; kind: string; vis: boolean; rend: boolean; wAlpha: number; box: number[] }[] = [];
                        const walk = (o: PIXI.DisplayObject, path: string, depth: number) => {
                            if (depth > 24) return;
                            const c = o as PIXI.Container;
                            const kids = c.children as PIXI.DisplayObject[] | undefined;
                            const name = (o as unknown as { name?: string }).name ?? o.constructor?.name ?? "?";
                            const here = `${path}/${name}`;
                            if (!kids || kids.length === 0) {
                                const b = o.getBounds();
                                rows.push({
                                    path: here,
                                    kind: o.constructor?.name ?? "?",
                                    vis: o.visible,
                                    rend: o.renderable,
                                    wAlpha: Number(((o as unknown as { worldAlpha: number }).worldAlpha ?? -1).toFixed(3)),
                                    box: [Math.round(b.x), Math.round(b.y), Math.round(b.width), Math.round(b.height)],
                                });
                                return;
                            }
                            for (const k of kids) walk(k, here, depth + 1);
                        };
                        // Walk every LIVE composite root, not the stage: the scene is rendered
                        // into a render texture from a container that is not a stage child, so a
                        // stage walk sees almost nothing.
                        const comps = compositesRef.current ?? [];
                        comps.forEach((cp, i) => {
                            const root = (cp as unknown as { root?: PIXI.Container }).root;
                            if (root) walk(root, `#${i}${root.parent ? "" : "(DETACHED)"}`, 0);
                        });
                        return rows;
                    };
                    (window as unknown as { __dumpLayers?: () => unknown }).__dumpLayers = () => {
                        const rows: unknown[] = [];
                        const walk = (c: PIXI.Container | null, side: string) => {
                            if (!c) return;
                            c.children.forEach((m, i) => {
                                const r = m as unknown as ISceneLayerRuntime & { alpha: number; visible: boolean; renderable: boolean; shader?: { uniforms?: Record<string, unknown> } };
                                const b = m.getBounds();
                                rows.push({
                                    side,
                                    i,
                                    sort: r.__sort,
                                    vis: r.visible,
                                    rend: r.renderable,
                                    alpha: Number(r.alpha.toFixed(3)),
                                    tint: (m as unknown as { tint?: number }).tint,
                                    blend: (m as unknown as { blendMode?: number }).blendMode,
                                    box: [Math.round(b.x), Math.round(b.y), Math.round(b.width), Math.round(b.height)],
                                    dbg: (m as unknown as { __blendDbg?: unknown }).__blendDbg,
                                    // An OVERBRIGHT / vertex-colour layer carries no MeshMaterial
                                    // tint - its whole colour lives in the `uColor` uniform, so a
                                    // dump without it cannot tell "drawn dim" from "not drawn".
                                    uColor: r.shader?.uniforms?.uColor,
                                    hasVCol: !!r.shader?.uniforms?.uColor,
                                    // Display-list state: a mesh can be visible+renderable with a
                                    // correct uniform and still never reach the framebuffer if its
                                    // PARENT chain is off or its worldAlpha collapsed.
                                    wAlpha: Number(((m as unknown as { worldAlpha: number }).worldAlpha ?? -1).toFixed(3)),
                                    pRend: (m.parent as unknown as { renderable?: boolean } | null)?.renderable,
                                    pVis: (m.parent as unknown as { visible?: boolean } | null)?.visible,
                                    pAlpha: Number(((m.parent as unknown as { alpha?: number } | null)?.alpha ?? -1).toFixed(3)),
                                    inList: !!m.parent,
                                    // The layer's SOURCE identity. Without it a dump row can be
                                    // matched back to the scene JSON only by guessing at its
                                    // screen box, which does not survive the camera transform -
                                    // and attributing an error to the wrong layer wastes a round.
                                    tex: r.__texIndex,
                                    src: r.__srcIndex,
                                });
                            });
                        };
                        walk(scene ? scene.background : null, "bg");
                        walk(scene ? scene.foreground : null, "fg");
                        walk(sceneOverlay, "ov");
                        return rows;
                    };
                }
                // Insert the static backdrop at the very back, registered centroid-to-
                // centroid onto the character (see makeBackdropSprite).
                // GAP FILL (?gapfill=1). Distinct from `useStatic`, which REPLACES a missing
                // scene; this keeps the scene and puts the static art BEHIND it purely to fill
                // the frame where the scene does not reach. Virtuosa's entrance is the case:
                // NO layer spans her camera view (largest reaches 0.88 of it, 0 of 132 cover
                // both axes), so the uncovered margin and the gaps between art pieces fall
                // through to bare canvas - a hard-edged near-black wedge where the game shows
                // misty vista. Gated on the geometry, not on a skin: only when no layer spans
                // the view, so a scene that already fills the frame is untouched.
                const viewExt = 2 * (scene?.data.cameraSizePx ?? 0);
                // Layers CAPABLE of covering the frame: big enough, actually able to occlude,
                // and not additive. Whether one covers at a given moment is decided per frame
                // below, from its clip window and animated alpha.
                const coverLayers =
                    !scene || viewExt <= 0
                        ? []
                        : scene.data.layers.filter((l) => {
                              let x0 = Infinity;
                              let x1 = -Infinity;
                              let y0 = Infinity;
                              let y1 = -Infinity;
                              for (let i2 = 0; i2 < l.pos.length; i2 += 2) {
                                  x0 = Math.min(x0, l.pos[i2]);
                                  x1 = Math.max(x1, l.pos[i2]);
                                  y0 = Math.min(y0, l.pos[i2 + 1]);
                                  y1 = Math.max(y1, l.pos[i2 + 1]);
                              }
                              if (!(x1 - x0 >= viewExt && y1 - y0 >= viewExt)) return false;
                              // A BOUNDING BOX IS NOT COVERAGE. Wiš'adel's backdrop is a 73-vertex mesh whose
                              // box is 2047² against a 2000 view - so it passed - but its silhouette is cut
                              // off diagonally and holds only 70% of the view's area. It suppressed gap-fill
                              // and left a hard-edged grey wedge across a third of her frame. Require the mesh
                              // to actually CONTAIN the view's worth of area. The 0.95 is slack for a tight
                              // quad whose box slightly exceeds the view, not a tuned constant: real covers
                              // sit at 2.4–6.1 and concave ones at 0.70–0.73.
                              let meshArea = 0;
                              for (let k = 0; k + 2 < l.idx.length; k += 3) {
                                  const a = l.idx[k] * 2;
                                  const b = l.idx[k + 1] * 2;
                                  const c = l.idx[k + 2] * 2;
                                  if (Math.max(a, b, c) + 1 >= l.pos.length) continue;
                                  meshArea += Math.abs((l.pos[b] - l.pos[a]) * (l.pos[c + 1] - l.pos[a + 1]) - (l.pos[c] - l.pos[a]) * (l.pos[b + 1] - l.pos[a + 1])) / 2;
                              }
                              if (meshArea < 0.95 * viewExt * viewExt) return false;
                              // AND IT HAS TO BE ABLE TO OCCLUDE. Geometry alone said "covered" for layers
                              // that never become opaque - Mlynar's layer 17 peaks at alpha 0.275, Civilight
                              // Eterna's layer 21 at 0.298 - and for ADDITIVE layers, which cannot hide
                              // anything by construction.
                              if (l.additive) return false;
                              let maxAlpha = l.tint?.[3] ?? 1;
                              if (l.colorCurve?.length) {
                                  maxAlpha = 0;
                                  for (const k of l.colorCurve) maxAlpha = Math.max(maxAlpha, k[4]);
                              }
                              return maxAlpha >= 0.99;
                          });
                const sceneCoversFrame = coverLayers.length > 0;
                // `settledRef` is why this is not just `gapFillOn()`: past the transform beat the
                // game shows a flat ground where this vista paints, and the settled composite is
                // built AFTER the latch is set.
                const gapFill = !useStatic && gapFillOn() && !(settledGroundOn() && settledRef.current) && !!backdropData && !!backdropFrame;
                if ((useStatic || gapFill) && backdropData && backdropFrame) {
                    // The card surface presents the artwork itself: sharp, never toggled (see panelArtOn).
                    const panelArt = surface === "panel" && panelArtOn();
                    // Read straight from scene data, NOT via ISceneFrame: see the sceneMesh note
                    // (frame-object keys re-phase the seeded particles). Spine-only skins have no
                    // scene and never need the derived placement (their backdrop IS the artwork).
                    const bdd = scene?.data;
                    const bdDerived = panelArt && bdxfOn() && typeof bdd?.backdropScale === "number" && bdd.backdropOffsetPx ? { scale: bdd.backdropScale, offset: bdd.backdropOffsetPx } : null;
                    const bd = makeBackdropSprite(backdropData, backdropFrame, spineCentroid, bdDerived);
                    const bdAblated = typeof window !== "undefined" && (new URLSearchParams(window.location.search).get("abl") || "").split(",").includes("backdrop");
                    if (bdAblated) bd.renderable = false;
                    // Silhouette clip (see silMaskParam): the illustration's COVERAGE, placed like
                    // `bd`, as the composite's alpha mask. NOT the illustration itself: a Pixi
                    // sprite mask MULTIPLIES by the mask sample (red x alpha), so masking with the
                    // painting dimmed every interior pixel by its own partial alpha and colour
                    // (measured on chen: interior luma -20.39 mean, -39.99 in blue regions, 12.6
                    // percent of her covered pixels are partial). The mask is therefore a WHITE
                    // texture whose alpha is the painting's coverage (alpha > 0, a definition, not
                    // a threshold): outside becomes transparent, the interior passes untouched,
                    // and the boundary keeps the source feather only where alpha is genuinely
                    // fractional at the edge. Canvas failure skips the mask rather than fading.
                    // Implementation is an ERASE-blend cutout, NOT a Pixi mask: a sprite mask
                    // renders the composite through a SpriteMaskFilter RT and that recomposition
                    // alone still cost the interior -12.23 luma (blue -24.25) even with a pure
                    // white full-alpha mask texture. The cutout is the INVERSE coverage (opaque
                    // exactly where the painting has zero alpha) drawn last with ERASE, which
                    // clears the composite's alpha in the same render pass with no intermediate
                    // target and leaves every interior pixel byte-untouched.
                    const silOn = silMaskParam() === "1" || (silMaskParam() !== "0" && bdDerived != null);
                    if (panelArt && silOn) {
                        let silhouette: PIXI.Sprite | null = null;
                        try {
                            const src = backdropData.texture.baseTexture.resource as unknown as { source?: CanvasImageSource & { width: number; height: number } };
                            const im = src?.source;
                            if (im && im.width > 0 && bdDerived) {
                                // PAD the cutout so it also erases scene content drawn BEYOND the
                                // art rect (chen's sky plane is wider than her painting; a finite
                                // sprite erases nothing outside itself and left sky bars at the
                                // frame edges). The pad is DERIVED per key: the scene's own local
                                // bounds, converted to art px through the derived scale, measured
                                // against the art rect; symmetric, so the padded canvas centre
                                // stays the art centre and the derived anchor/scale/offset hold.
                                const lb = sceneContainer.getLocalBounds();
                                const cxs = bdDerived.offset[0];
                                const cys = -bdDerived.offset[1];
                                const halfW = (im.width / 2) * bdDerived.scale;
                                const halfH = (im.height / 2) * bdDerived.scale;
                                const overhang = Math.max(0, lb.x + lb.width - (cxs + halfW), cxs - halfW - lb.x, lb.y + lb.height - (cys + halfH), cys - halfH - lb.y);
                                const padArt = Math.min(im.width * 4, Math.ceil(overhang / bdDerived.scale));
                                const cv = document.createElement("canvas");
                                cv.width = im.width + 2 * padArt;
                                cv.height = im.height + 2 * padArt;
                                const cx = cv.getContext("2d", { willReadFrequently: true });
                                if (cx) {
                                    cx.drawImage(im, padArt, padArt);
                                    const id = cx.getImageData(0, 0, cv.width, cv.height);
                                    const px = id.data;
                                    for (let p = 0; p < px.length; p += 4) {
                                        const a = px[p + 3];
                                        px[p] = 255;
                                        px[p + 1] = 255;
                                        px[p + 2] = 255;
                                        px[p + 3] = a > 0 ? 0 : 255;
                                    }
                                    cx.putImageData(id, 0, 0);
                                    silhouette = new PIXI.Sprite(PIXI.Texture.from(cv));
                                    silhouette.anchor.set(0.5, 0.5);
                                    silhouette.scale.set(bdDerived.scale);
                                    silhouette.position.set(cxs, cys);
                                }
                            }
                        } catch {
                            silhouette = null;
                        }
                        if (silhouette) {
                            silhouette.blendMode = PIXI.BLEND_MODES.ERASE;
                            sceneContainer.addChild(silhouette);
                        }
                    }
                    if (gapFill && !panelArt) {
                        // Defocused vista fill. Radius follows the art's own height so the cutoff
                        // is a spatial frequency, not a pixel count (see gapFillOn).
                        const blur = new PIXI.BlurFilter();
                        blur.blur = Math.max(1, bd.height * gapBlurFraction());
                        // The art is drawn far outside the camera box; without padding the filter
                        // crops it to its own bounds and leaves a hard seam at the frame edge.
                        blur.padding = blur.blur * 2;
                        bd.filters = [blur];
                    }
                    sceneContainer.addChildAt(bd, 0);
                    // Tracked so the transform beat can retire it. Only the GAP FILL: a `useStatic`
                    // backdrop IS the artwork, and retiring that would blank the view.
                    if (gapFill && !panelArt) gapFillSpritesRef.current.push(bd);
                    // ⚠️ Do NOT register the per-frame coverage toggle when this sprite has been
                    // ABLATED. The toggle below assigns `renderable` every frame, so it silently
                    // overwrote `?abl=backdrop` and the token appeared to do nothing - which is
                    // exactly how a gap-fill wash got mis-attributed as an unexplained base layer
                    // (`?gapfill=0` removed it; `?abl=backdrop` did not). Same failure as the
                    // `?abl=fg:` tokens that never cleared `__activeWindows`.
                    if (gapFill && sceneCoversFrame && !bdAblated && !panelArt) gapFillRef.current = { sprite: bd, covers: coverLayers };
                }
                // Framing. When a framingOverride is given (the entrance), reuse it verbatim
                // so the entrance renders through the SAME authored camera box as the main
                // and its settled final frame aligns exactly with the idle. Otherwise:
                // fullscreen ("authored") reproduces the game's orthographic camera - a
                // square of visible HEIGHT `2 × cameraSizePx` centred on the character's own
                // visible-bounds centre (NOT the authored `cameraOffsetPx`, the mirror axis,
                // which would crop the head). This authored-camera extent is the game's
                // ground-truth framing, consistent across normal ops and the special-entry
                // skins. Falls back to the character-bounds zoom-out for spine-only ops that
                // carry no authored camera.
                const authoredFrame = scene ? sceneFrameOf(scene.data) : backdropFrame;
                // POSE-ROBUST special-entry framing. The character is located with its SOLID body
                // span (`headTop`/`feetBottom`, alpha-mass landmarks that skip thin protrusions the
                // bbox catches - a raised rifle above the head, wispy hair/dress tails below the
                // feet - the exact failure that shoved seated Executor to the frame bottom).
                // SIZE is SCENE-based (`cameraViewPx` = the game's `_adjustes[0]` display crop),
                // so the painted scene fills the frame - the character's on-screen size then
                // follows from its size in that scene (as in-game). VERTICAL position uses the
                // robust landmarks so the character sits right for ANY pose:
                //   settled: FEET-anchored - the base sits ~90% down (a ground line), the body
                //            rises with scene headroom above. Robust to a tall standing OR a
                //            compact seated char (both put their base on the ground line).
                //   open:    HEAD-anchored - the head sits ~8% down (a close-up filling the top),
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
                // animates is the game's own ratio (Virtuosa 1246/1929 = 0.646×) - no hardcoded
                // per-skin numbers. Skins with NO pull-out (Mlynar) hold the tight `_adjustes[1]`
                // stop as their steady frame; skins with a pull-out (Virtuosa/Skadi2) settle on the
                // wide `_adjustes[0]` stop - matching the game, whose steady camera frames each
                // character at a DIFFERENT size within its own scene (Mlynar knees-up and large,
                // Virtuosa smaller and lower, Skadi2 near-full-body), NOT at a constant character
                // height. `RCAL` is a SINGLE global pipeline constant (not per-skin) that converts
                // the game's authored `_adjustes` px into the render/vis space the mesh + spine are
                // measured in. It was RE-CALIBRATED (0.606 → 0.78) against the Mlynar/Virtuosa/Skadi2
                // settle recordings: the old value framed ~1.3× too tight (Mlynar cropped to the
                // waist, Virtuosa waist-up), so the character now reads at the game's wider,
                // pillar-boxed size (Mlynar knees-up, Virtuosa full-ish body low in the frame, the
                // scene filling around them). Combined with the "height" fit (see `fitRef`), the
                // crop height maps to the container height, so the finite scene art leaves the
                // game's side pillar-box slack - no stretch, no distortion. The box is CENTRED on the
                // character body - the raw `_adjustes` OFFSET points BELOW the body in export space,
                // so it is not used for centring; `VBIAS` corrects the residual hair/mass drag.
                //
                // ⚠️ CORRECTED 2026-08-26. This said "~650px BELOW the body (verified)". Measured
                // with `__frameProbe` on the five skins that have settled captures, the gap is
                // 802.57 / 984.07 / 1007.12 / 1046.07 / 1213.85 authored px: mean 1010.74, spread
                // **411.28**. The direction is right and the magnitude is 1.55x the figure quoted,
                // but the important part is that it is NOT A CONSTANT, and that spread is exactly
                // the per-skin signal `VBIAS` throws away.
                //
                // ⛔ Three attempts to turn it into a derived per-skin term all failed, see
                // `dynchar-cameraoffsetpxy-refuted`. The closest is treating the authored camera as
                // the crop centre outright, which gets the SLOPE right (1.0171, i.e. it captures
                // the per-skin variation almost exactly) but still needs a global +254.38 px and
                // lands at R2 0.6992, worse than a naive line through the raw offsets. Nothing
                // ships until a form without a fitted constant is found.
                //
                // Re-swept by the same method as `VBIAS` after that constant was corrected (the
                // two interact: `cy0 = bodyCy - VBIAS*e` with `e = viewPx*RCAL`). Mlynar is again
                // the only surface that moves - Virtuosa and Skadi2 are BIT-IDENTICAL at every
                // value tried, since they keep the pure rig camera:
                //
                //     rcal   0.74    0.76    0.772   0.776   0.780   0.781   0.782   0.784   0.80
                //     MADC   29.697  25.814  22.085  20.534  19.352  19.278  19.364  19.820  24.999
                //
                // 0.78 was already right to ~0.1%; 0.781 is the reproducible optimum and is worth
                // only 0.074 MADC - kept because it IS the measured minimum, but do not read it as
                // a meaningful parity gain the way the VBIAS correction was (-2.13).
                const RCAL = calibrationParam("rcal", 0.781);
                // VBIAS: shift the crop centre UP (toward the head) by this fraction of the crop
                // size `e`. The `feetBottom`/head landmarks include the character's long trailing
                // hair/tail, which drags the naive body midpoint DOWN, floating the character too
                // high in-frame with dead scene below. A small upward bias re-centres so the head
                // sits ~10% down and the body fills toward the bottom, matching the game's
                // roughly-centred (Mlynar) to low-centred (Virtuosa) composition.
                //
                // MEASURED, not tuned by eye. Mlynar's entrance is the only phase-locked surface
                // that exposes this constant (his `centerBlend.cy0` is the settle-open box centre,
                // so a wrong VBIAS offsets the whole late entrance): sweep it against the recording
                // and take the value where the residual best-fit vertical shift crosses zero.
                //
                // History: 0.151 (calibrated when `RCAL` was 0.606) left +26 px once the frame
                // widened to 0.78; 0.180 cut that to +1 px and was kept. That +1 px was a real
                // residual, not noise - it held at +1.1 px on EVERY scored beat and cost ~16% of
                // his interior luma error. Re-swept at finer resolution against the current
                // pipeline (frames decoded and INDEXED, never `ffmpeg -ss`, which is a frame late):
                //
                //     vbias    0.1805   0.1810   0.1815   0.1820   0.1825   0.184
                //     MADC     20.649   19.863   19.352   19.387   19.902   22.226
                //
                // 0.1815 is the optimum and drops the residual shift +1.08 px → +0.18 px
                // (per-beat +1.2,+1.3,+1.1,+1.1,+1.1,+0.7 → +0.2,+0.2,+0.2,+0.2,+0.2,+0.1).
                // Virtuosa and Skadi2 are BIT-IDENTICAL across the change (29.904 / 18.671 at both
                // values) - they carry an authored transform beat, so they keep the pure rig camera
                // and never enter the `centerBlend` branch this constant feeds.
                const VBIAS = calibrationParam("vbias", 0.1815);
                const bodyCx = vb?.centroid ? vb.centroid.cx : vb ? vb.x + vb.width / 2 : frameCx;
                const bodyCy = (headY + feetY) / 2; // vertical body centre
                /** A body-centred square crop of authored extent `viewPx`, in render/vis space. */
                const bodyFrameBox = (viewPx: number): IAnimationBounds => {
                    const e = viewPx * RCAL;
                    return { x: bodyCx - e / 2, y: bodyCy - VBIAS * e - e / 2, width: e, height: e };
                };
                // DIAGNOSTIC (`__frameProbe`, DEV only): the inputs `bodyFrameBox` uses and the
                // authored camera it declines to use, in one place and in comparable units.
                //
                // It exists to CHECK a load-bearing claim rather than argue about it. The comment
                // above says the raw `_adjustes` offset "points ~650 px BELOW the body in export
                // space (verified)", which is why the vertical centre is measured off the body and
                // corrected by a single global `VBIAS`. Nothing exposed `bodyCy`, so that number
                // could not be re-derived, and a claim that cannot be re-derived is the shape this
                // project has had to retract four times.
                //
                // `offsetBelowBody` is the quantity in question, expressed in AUTHORED px so it can
                // be read against `cameraOffsetPxY` directly: positive means the authored camera
                // centre sits below the measured body centre.
                if (import.meta.env.DEV && typeof window !== "undefined") {
                    (window as unknown as { __frameProbe?: () => unknown }).__frameProbe = () => ({
                        bodyCx,
                        bodyCy,
                        headY,
                        feetY,
                        centroid: vb?.centroid ?? null,
                        visBox: vb ? { x: vb.x, y: vb.y, w: vb.width, h: vb.height } : null,
                        RCAL,
                        VBIAS,
                        offsetPx: authoredFrame?.offsetPx ?? null,
                        offsetPx2: authoredFrame?.offsetPx2 ?? null,
                        viewPx: authoredFrame?.viewPx ?? null,
                        viewPx2: authoredFrame?.viewPx2 ?? null,
                        cameraSizePx: authoredFrame?.cameraSizePx ?? null,
                        // What the shipped formula puts the crop centre at, and what the authored
                        // camera would put it at, both in render/vis px.
                        shippedCy: bodyCy - VBIAS * ((authoredFrame?.viewPx2 ?? authoredFrame?.viewPx ?? 0) * RCAL),
                        offsetBelowBody: authoredFrame?.offsetPx ? (bodyCy - authoredFrame.offsetPx[1] * RCAL) / RCAL : null,
                    });
                }
                /** An authored `_adjustes` extent is only usable if it is a POSITIVE FINITE
                 *  number. Unity writes an uninitialised stop as `-FLT_MAX`
                 *  (-3.4028235e38), which the exporter carries through verbatim - and a
                 *  plain truthiness test accepts it, because it is non-zero. `bodyFrameBox`
                 *  then builds a crop 3e38 px wide, `tight` prefers it over the sane
                 *  `cameraSizePx` fallback, and the skin renders as an EMPTY frame.
                 *  Nearl the Radiant Knight "Epoque" is the one such skin in the corpus
                 *  (both of her stops are -FLT_MAX); she rendered blank. Property-driven -
                 *  a no-op for every skin whose stops are real numbers. */
                const usableExtent = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v) && v > 0;
                /** THE SETTLED FRAMING CORRECTION (`?settlecam=0` reverts), as the
                 *  three registration parameters measured against the captures.
                 *
                 *  Recovered by registering our settled frames onto the game on EDGE magnitude (luma
                 *  correlation cannot do it: one subject's backdrop is dark where the game's is
                 *  white, and that flat anti-correlated area dominates at every alignment). Each
                 *  parameter tracks a DIFFERENT authored field:
                 *
                 *      scale = cameraViewPx / 1935.2        r +0.9612, R2 0.9999 over four of five
                 *      dx    = 0.399 * offsetPx.x + 16.26   r +0.9709
                 *      dy    = -0.754 * offsetPx.y          r -0.9717, origin-through
                 *
                 *  These are how much our OUTPUT must move to land on the game, in output pixels of
                 *  the 900x416 frame, so they are applied RELATIVE to whatever box the settled path
                 *  already produced rather than replacing it.
                 *
                 *  🚨 An earlier version built an ABSOLUTE box from the authored fields instead, and
                 *  that is what made the camera disagree with the warp (fugue +22.252, kalts
                 *  +56.356) while the same numbers as an image warp were worth -84.947. The rule was
                 *  never the problem; discarding the existing box was.
                 *
                 *  ⚠️ Worth -84.947 across the five settled subjects as a warp, every one improving
                 *  on BOTH MADC and r. Scale ALONE costs +5.759, so it must never be applied
                 *  partially. */
                // DEFAULT ON. `?settlecam=0` reverts, tested for the STRING "0" rather than for
                // falsiness: `Number(null)` is 0 and finite, and that has silently defeated a
                // default in this file before.
                const settleCamOn = typeof window === "undefined" || new URLSearchParams(window.location.search).get("settlecam") !== "0";
                const SETTLE_VIEW_PX = calibrationParam("settleview", 1935.2);
                /** `[scale, dx, dy]` in OUTPUT pixels, or null when no correction applies. */
                const settleFix = ((): [number, number, number] | null => {
                    // `?settledxy=s,dx,dy` feeds the parameters directly, which is how the camera
                    // path is checked against the warp that measured them: same numbers, two
                    // implementations, and they must agree.
                    const raw = typeof window === "undefined" ? null : new URLSearchParams(window.location.search).get("settledxy");
                    if (raw) {
                        const v = raw.split(",").map(Number);
                        if (v.length === 3 && v.every((n) => Number.isFinite(n)) && v[0] > 0) return [v[0], v[1], v[2]];
                    }
                    if (!settleCamOn) return null;
                    const off = authoredFrame?.offsetPx;
                    const vpx = authoredFrame?.viewPx;
                    if (!off || !usableExtent(vpx)) return null;
                    return [(vpx as number) / SETTLE_VIEW_PX, 0.399 * off[0] + 16.26, -0.754 * off[1]];
                })();
                const authoredDisplayBounds: IAnimationBounds | null = usableExtent(authoredFrame?.viewPx) && vis ? bodyFrameBox(authoredFrame.viewPx as number) : null;
                // The TIGHT open endpoint: the exact `cameraViewPx2` box (2nd `_adjustes` stop),
                // same centre - the viewer dollies OUT from here. Null unless a 2nd camera stop.
                const authoredTightBounds: IAnimationBounds | null = usableExtent(authoredFrame?.viewPx2) && vis ? bodyFrameBox(authoredFrame.viewPx2 as number) : null;
                // The game's SKIN-PREVIEW framing (`?statcam=1`): the CAMERA's own orthographic
                // size, not either `_adjustes` display stop. Built through `bodyFrameBox` so the
                // `VBIAS` term rescales with the extent rather than being carried over from a box
                // of a different size.
                //
                // MEASURED against the clean 2340x1080 captures. Method: the RENDERER generates each
                // candidate framing (so no numpy resampling artefact enters the fit), each is scored
                // against a 10-frame time-average of the settled idle (t>=9, so the character and
                // particles wash out and only static scenery drives the match), and translation is
                // searched exhaustively to +/-140 px - the last of which matters, since a shift
                // clipped at the search boundary silently depresses EVERY scale (it hid Mlynar's
                // true optimum and cost him a factor of 3.5 in correlation). Re-expressing each
                // skin's fitted extent against the candidate bases:
                //
                //     basis            ska      cel      mly     spread
                //     viewPx          0.4944   0.5206   0.5180    5.1%
                //     viewPx2         0.7899   0.8060   0.7687    4.6%
                //     cameraSizePx    0.9565   0.9565   0.9278    3.0%   <- THIS
                //
                // `_adjustes` are the in-game VIEWER's display crops; the preview is a plain camera
                // render, so the camera's own size is what it should track. ska and cel land on
                // 0.9565 EXACTLY - both peak at zoom 1.000 with sharp curves (NCC 0.715 and 0.506
                // against ~0.3 at +/-3%) despite unrelated `_adjustes` values, which is what makes
                // this the basis rather than a coincidence.
                //
                // How exact: inverting the fits to the ortho each skin's framing implies gives
                // **10.0000** against an authored 10.000 (ska) and **10.4997** against 10.500 (cel)
                // - five significant figures on two independent skins. Mlynar implies 10.7768
                // against 11.110.
                //
                // ⚠️ VALIDATED ON TWO SKINS. Mlynar is a genuine 3% outlier - he peaks at
                // 0.965-0.970, also sharply (0.677 against 0.336 at 1.000), so it is not scatter.
                // Do NOT "fix" it by averaging: that would make all three wrong instead of one.
                // Excluded as causes: `skeletonScale` (identical 0.01 on all three), idle-vs-`_Start`
                // camera size (identical), content-limiting (the scene exceeds the view on all
                // three), a second display controller (each bundle ships exactly one), `_maxSize`
                // (2048x2048 everywhere), and every other numeric field in the scene JSON - none has
                // the required signature of "equal for ska and cel, 0.970x for mly".
                //
                // The one structural difference found: Mlynar ships **three** `_adjustes` stops
                // (1990 / 1341 / 2808) where ska and cel ship two, and the exporter only ever reads
                // the first two. That splits these three samples exactly along the observed outlier
                // - but it is a class covering **46 of 82** skins, so with n=3 it is a correlation,
                // not a cause, and the magnitude is unexplained (3-stop skins' stop0/cameraSizePx
                // does differ, 1.857 vs 1.631, but by well under one SD). The decisive test is a
                // fresh capture of one more 2-stop and one more 3-stop skin. Until then, treat this
                // framing as unvalidated on the 3-stop half of the corpus.
                //
                // On the 0.9565: if the preview shows exactly `cameraSizePx` of authored height,
                // then the true authored->render conversion for this shot is 0.9565 x RCAL = 0.747,
                // against the 0.781 `RCAL` carries. So this constant is really the gap between the
                // preview's conversion and RCAL's - and RCAL is the suspect one: it was fitted on
                // ONE skin's entrance against CRF36 footage. If RCAL is ever recalibrated against
                // clean data, this must move with it (ideally to 1.0, deleting itself).
                //
                // Also more robust than the `_adjustes` bases: `cameraSizePx` is required by
                // `sceneFrameOf`, whereas `viewPx` can be an uninitialised -FLT_MAX (Nearl Epoque).
                // `?statfrac=<f>` sweeps it.
                const PREVIEW_CAM_FRAC = calibrationParam("statfrac", 0.9565);
                const previewBounds: IAnimationBounds | null = authoredFrame?.cameraSizePx && vis ? bodyFrameBox(authoredFrame.cameraSizePx * PREVIEW_CAM_FRAC) : null;
                // Relative move between the two authored stops (see `IComposite.authoredTightShift`).
                // Y is flipped because the authored offsets are spine-authored Y-UP and the framing
                // boxes are screen-down - the same flip `authoredDisplayBounds` documents.
                const o0 = authoredFrame?.offsetPx;
                const o2 = authoredFrame?.offsetPx2;
                const authoredTightShift: [number, number] | null = o0 && o2 ? [(o2[0] - o0[0]) * RCAL, -(o2[1] - o0[1]) * RCAL] : null;
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
                // Snapshot every bone's REFERENCE world matrix at the SETUP (bind) pose -
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
                // Setup-pose CENTRE of every attachment, keyed by attachment name. A
                // `BoneFollower` rig that CONTINUES a spine prop (Virtuosa's falling apple:
                // bone `L_C_Apple_F` carries the attachment of the same name, which the
                // entrance fades out exactly as the particle copy spawns) must hand off from
                // where the ART is, not from the bone ORIGIN - the authored `followOffset` is
                // measured to the origin and lands the particle low by the art's own offset.
                // Same space as `boneRest` (skeleton world, Y-down).
                const attachRest = new Map<string, { x: number; y: number }>();
                for (const slot of spine.skeleton.slots as unknown as { attachment: AttachmentLike | null }[]) {
                    const att = slot.attachment;
                    if (!att || typeof att.computeWorldVertices !== "function" || !att.worldVerticesLength) continue;
                    const n = att.worldVerticesLength;
                    const w = new Float32Array(n);
                    try {
                        att.computeWorldVertices(slot as never, 0, n, w, 0, 2);
                    } catch {
                        continue;
                    }
                    let mnx = Infinity;
                    let mxx = -Infinity;
                    let mny = Infinity;
                    let mxy = -Infinity;
                    for (let i = 0; i < n; i += 2) {
                        mnx = Math.min(mnx, w[i]);
                        mxx = Math.max(mxx, w[i]);
                        mny = Math.min(mny, w[i + 1]);
                        mxy = Math.max(mxy, w[i + 1]);
                    }
                    if (Number.isFinite(mnx)) attachRest.set(att.name, { x: (mnx + mxx) / 2, y: (mny + mxy) / 2 });
                }
                attachRestRef.current = attachRest;
                const hasShadow = useStatic && hasShadowSlots(spine);
                // ENTRANCE frame extent (authored px): the `_Start` camera's view at its ANIMATED
                // t=0 ortho size (`2·ortho₀/skeletonScale`). The exporter's `entranceViewPx` is
                // derived from the camera's STATIC serialized ortho, which the ortho curve overrides
                // from frame 0 - for Virtuosa that's 598 px (ortho 2.99) vs the true animated 374 px
                // (ortho 1.87), framing the whole entrance 1.6× too wide. Measured against the game
                // recording, the animated view matches the hold beats to <1% (the game's seated shot
                // is a tight head-and-shoulders close-up). Fall back to the static `entranceViewPx`
                // (then the tight `_adjustes[1]` stop) only when no ortho curve/scale shipped.
                const entranceOrtho0 = scene?.data.entranceOrthoCurve?.[0]?.[1];
                const entranceSkelScale = scene?.data.skeletonScale;
                // ...EXCEPT on a PERSPECTIVE rig, where that curve is the camera DOLLY and its
                // keyframes are DISTANCES. `2·d₀/skeletonScale` is then not a view extent at all:
                // whitw2 reads 600 px against her true frustum height of 346.41
                // (`2·d₀·tan(fov/2)/skeletonScale`), framing her whole entrance √3 = 1.732× too
                // wide. The exporter already puts that frustum height in `entranceViewPx` and it
                // is derived from the dolly's FIRST keyframe, so it is the animated t=0 extent the
                // comment above asks for - not the static ortho this fallback normally means.
                // Measured: her best cross-correlation alignment needs a 1.6-1.8× zoom-in at EVERY
                // beat while the ortho ratio (which varies 10× over the shot) is already correct.
                const entrancePersp = scene?.data.entrancePerspective === true;
                const entranceFrameSize =
                    entrancePersp && usableExtent(authoredFrame?.entranceViewPx)
                        ? (authoredFrame?.entranceViewPx as number)
                        : entranceOrtho0 && entranceSkelScale
                          ? (2 * entranceOrtho0) / entranceSkelScale
                          : ((authoredFrame?.entranceViewPx as number | undefined) ?? (authoredFrame?.viewPx2 as number | undefined) ?? null);
                // Authored SCENE-timeline end: when the entrance→idle handoff (pose swap +
                // pull-out dolly) fires, relative to the entrance track clock.
                //
                // Skins with an authored `entranceTransform` beat (the exported reform/
                // pose-transition timestamp, e.g. cello 12.0, skadi2 9.5) use it as the base
                // timing candidate INSTEAD OF the raw camera-curve tail: the ortho/cam-center
                // curves keep being keyed almost to the end of the cinematic (baked idle-hold/
                // sway data past the real transition), so their literal last keyframe over-
                // defers the handoff by several seconds (cello: curve tail 19.73 vs the game's
                // actual transition at 12.0 - she's stuck in her closed-eyes "reform" pose and
                // the pull-out dolly gets crushed into the final second). Layer-derived signals
                // (activeUntil windows, colour-curve reveals still visible at their own end) are
                // ALWAYS folded in via max() regardless of transform presence - they're genuine
                // content-completion beats, not the buggy curve-tail artifact, and Skadi2's white
                // reveal-overlay colour curve (ending visible at 22.43, well after its own 9.5
                // transform) depends on this to keep deferring the handoff until the flash
                // clears (unchanged from the existing fix for that skin).
                //
                // Skins with NO authored transform (Mlynar) keep the original raw-tail
                // computation and the `entAnimDur` guard, unchanged.
                if (opts.mode === "entrance" && scene && scene.data.entranceCamCenterCurve?.length && entranceFrameSize) {
                    const xform = scene.data.entranceTransform;
                    let end = xform ?? 0;
                    if (xform == null) {
                        const oc = scene.data.entranceOrthoCurve;
                        if (oc?.length) end = Math.max(end, oc[oc.length - 1][0]);
                        const ccv = scene.data.entranceCamCenterCurve;
                        if (ccv?.length) end = Math.max(end, ccv[ccv.length - 1][0]);
                    }
                    for (const l of scene.data.layers) {
                        if (l.activeUntil != null && Number.isFinite(l.activeUntil)) end = Math.max(end, l.activeUntil);
                        const cc = l.colorCurve;
                        // Only a colour curve that ends still VISIBLE (final alpha > 0) can defer the
                        // handoff. A curve fading to alpha 0 is a hide/fade-off with no visible payload;
                        // letting it push the scene end out holds a peaked reveal-overlay opaque past the
                        // cinematic. (Skadi2: its white cover-overlay peaks at 22.43s, but two invisible
                        // fade-to-0 layers ran to 23.67s - the opaque white then held through the handoff
                        // seam and BLANKED the frame. Mlynar/cello curves all end visible → unaffected.)
                        if (cc?.length && cc[cc.length - 1][4] > 0.02) end = Math.max(end, cc[cc.length - 1][0]);
                    }
                    if (xform != null) {
                        // The authored entrance TOTAL duration is a genuine content-completion
                        // beat and must floor the handoff: `entranceTransform` marks the pose/
                        // expression reform (cello 12.0), but the game holds the tight portrait
                        // through the whole entrance and snaps wide only at `entranceDuration`
                        // (cello 18.0 - matching her voice line to ~18.3s). Firing at the raw
                        // transform (12.0) snaps ~6s early and cuts the line. Fold it into the
                        // max as a floor. No-op where a later term already dominates: Skadi2's
                        // visible colour curve ends at 22.43, past its own entranceDuration
                        // (22.33), so its end is unchanged. (Mlynar has no transform → the else
                        // branch below, untouched.)
                        end = Math.max(end, scene.data.entranceDuration ?? 0);
                        // Data-derived beat - fire the tick-driven handoff at this track time
                        // even when it's EARLIER than the spine's own natural animation
                        // completion. That's the whole point: the pose swap must not wait for
                        // the baked idle-hold tail to finish playing out.
                        //
                        // ...but never LATER than the director's own `duration`. The layer terms
                        // above exist to let a late REVEAL finish; a curve that merely happens to
                        // end still-visible past the authored end is baked idle-hold tail, the
                        // same artifact this block already discounts for the camera curves. The
                        // handoff carries the white screen fade with it, so over-deferring holds
                        // the frame pure white long after the game has cut back.
                        //
                        // Measured on the two captures that cover the transition, both settle to
                        // the idle at `duration + 0.35`: Mlynar (duration 16.5) is back at 16.87,
                        // Muelsyse (duration 20.0) at 20.35. Muelsyse is the skin this fixes -
                        // her `end` runs to 20.93, deferred +0.93s by an ordinary sort-1 layer
                        // sitting at alpha 0.69, and we held pure white until 21.08 where the game
                        // was live at 20.35. Every other benchmark already lands within 0.1s of
                        // `duration` (Mlynar 16.50, Virtuosa 18.00, Skadi 2 22.43 → 22.33), so
                        // this is a no-op for them.
                        //
                        // 🔑 2026-08-25, read the two lines together: the floor above raises `end`
                        // to at least `duration`, so this min ALWAYS returns exactly `duration` and
                        // is unreachable rather than binding. Every layer term computed above is
                        // therefore dead code on this arm, and all ten skins that take it hand off
                        // at their own `duration`. The captures say that is the right beat, so the
                        // arm stays as it is; it is the OTHER one that was wrong. Do not "simplify"
                        // this to a bare assignment without re-reading the else branch below, which
                        // now depends on the same value being used on both sides.
                        deferEndUntil = Math.min(end, scene.data.entranceDuration ?? end);
                    } else {
                        // NO authored transform. This arm used to hand off at `end`, the layer and
                        // camera-curve tail, and that is WRONG: the game cuts at the director's
                        // `duration`.
                        //
                        // Measured on three captures, game clock converted to render clock with
                        // each skin's own REFOFF. The cut is the largest frame-to-frame delta in
                        // the ending, and every one of them sits perfectly frozen at 253.000 with
                        // `dPrev` 0.000 for the ten frames before it, so the beat is unambiguous:
                        //
                        //     skin    duration   flat 253.000 span (game)   cut, render   delta
                        //     fugue      9.767   9.4333..9.7333 = 0.300       9.8830     +0.116
                        //     kalts     14.500  13.9667..14.2667 = 0.300     14.7000     +0.200
                        //     chyue     20.000  19.8333..19.8667 = 0.033     20.2670     +0.267
                        //
                        // Ch'ien Yu is the case that settles it, because she is on THIS arm: her
                        // `end` is 19.267 and her camera tail 18.567, which sit 1.000s and 1.700s
                        // before her observed cut, the second of those in the dark beat at luma
                        // 33.920 before her white-out has even started. The tails are not the beat.
                        //
                        // ⚠️ The residual +0.116/+0.200/+0.267 is DELIBERATELY not folded in. It
                        // does not yet separate a constant lead near 0.150 from a proportional
                        // 1.188%/1.379%/1.335% of duration, and three points cannot decide it. A
                        // capture of Executor (duration 6.500) would: constant predicts ~0.150,
                        // proportional ~0.077, a 2x separation on one clip.
                        //
                        // The `end > entAnimDur + 0.05` guard is unchanged - it decides WHETHER a
                        // tick-driven deferral exists at all, and only the VALUE was ever wrong.
                        // All three skins on this arm pass it (pasngr 15.967 > 11.717, mlynar
                        // 16.500 > 14.717, chyue 19.267 > 18.983), so nothing starts or stops
                        // deferring, and Mlynar's `end` already equals his `duration` at 16.500.
                        //
                        // `?handoffdur=0` restores `end`. Read as an explicit "0" so an ABSENT
                        // parameter keeps the fix; never a falsy test, which `Number(null)` = 0
                        // has silently defeated here before.
                        const off = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("handoffdur") === "0";
                        const entAnimDur = spine.spineData.animations.find((a: { name: string }) => a.name === entranceAnim)?.duration ?? 0;
                        if (end > entAnimDur + 0.05) deferEndUntil = off ? end : (scene.data.entranceDuration ?? end);
                    }
                }
                // THE DIRECTOR'S `duration` IS THE AUTHORED END, whatever the spine does - and
                // whether or not the skin ships a camera track. The block above is gated on the
                // camera curve, so for a skin without one nothing sets a deferral at all and the
                // handoff falls back to the spine's own `complete`. When the baked `Start`
                // animation outlasts the authored duration that strands the cinematic: the white
                // screen fade has already ramped to FULL at `duration − HOLD` and then holds,
                // waiting for a handoff that will not come until the animation tail finishes.
                // Wiš'adel "sale#15" (duration 14.5) held PURE WHITE from 14.5s to past 18s for
                // exactly this reason. Firing at `duration` puts the handoff back on the same
                // schedule the fade already runs on.
                //
                // Never moves an EARLIER deferral later (the min below), and engages only when
                // the animation actually outlasts the duration - otherwise `complete` already
                // lands at or before it and this is a no-op.
                if (opts.mode === "entrance" && scene?.data.entranceDuration != null) {
                    const dur = scene.data.entranceDuration;
                    const animDur = spine.spineData.animations.find((a: { name: string }) => a.name === entranceAnim)?.duration ?? 0;
                    if (animDur > dur + 0.05) deferEndUntil = Math.min(deferEndUntil ?? dur, dur);
                }
                // VIEWPORT APERTURE (Executor's rifle scope). Seated INSIDE the scene container
                // so the entrance camera's pan/zoom carries it exactly as it carries the layers
                // - the aperture is scene-fixed, and its apparent expansion is the camera pulling
                // back, not an animation. Entrance-only: the settled idle is never scoped.
                const aperture = opts.mode === "entrance" ? (scene?.aperture ?? null) : null;
                let apertureMask: PIXI.Graphics | null = null;
                if (aperture) {
                    // A black COVER with a circular hole rather than a mask on the scene: the
                    // surround has to be pure black (measured max luma outside the scope: 0.0),
                    // and masking the scene alone would leave the viewer's own environment fill
                    // showing through. As the LAST child of the composite root it also covers the
                    // static backdrop, particles and spine - everything the entrance draws.
                    //
                    // Drawn at the local origin and MOVED each tick onto the live camera centre
                    // (the shot tracks the scope). Seated in the scene container so the camera's
                    // zoom scales it - the aperture's apparent growth is entirely that zoom. The
                    // plate is far wider than any authored scene so it covers at every zoom.
                    // Filled WHITE and TINTED by the rim's authored curve - a tint multiplies, so a
                    // black fill could never flash white. The curve supplies black, the flash and
                    // the end time (see {@link sceneAperture}).
                    const PLATE = 50000;
                    apertureMask = new PIXI.Graphics();
                    apertureMask.beginFill(0xffffff);
                    apertureMask.drawRect(-PLATE, -PLATE, PLATE * 2, PLATE * 2);
                    apertureMask.beginHole().drawCircle(0, 0, aperture.radius).endHole();
                    apertureMask.endFill();
                    apertureMask.tint = 0x000000;
                    const c0 = sampleCurveXY(scene?.data.entranceCamCenterCurve ?? null, 0);
                    if (c0) apertureMask.position.set(c0[0], c0[1]);
                    sceneContainer.addChild(apertureMask);
                }
                // ENTRANCE LETTERBOX. Civilight Eterna's cinematic renders into a hard 16:9 window
                // inside the 2340x1080 screen, and there is no per-skin rule behind it: her prefab
                // ships FOUR opaque planes whose inner edges bound the window (2809x1581, aspect
                // 1.7767, height = her `entranceViewPx` exactly). The exporter reads that geometry
                // and emits the rect; here it becomes four black quads drawn over everything.
                //
                // Built in the SAME space as the camera centre, as a child of the scene container,
                // so it inherits the camera pan/zoom/roll for free and disappears with the entrance
                // composite at the hand-off. `?letterbox=0` disables it.
                //
                // ⚠️ The bars are NOT automatically topmost. They carry an ordinary
                // `m_SortingOrder` like every other quad (hers is 100), and it is exactly TIED
                // with the two full-screen `Transition_*` planes the director flashes - all six
                // also sit at world z 0.000, so depth does not separate them either. The game
                // paints her end-of-cinematic white fade OVER the bars: its pillarbox ramps to
                // 0.99 alpha while ours, with the bars pinned last, stayed at 0.00 for the whole
                // fade. That is ~10.5% of the scored columns held at black against the game's
                // ~194 luma, and it was essentially the entire t=18.5 beat.
                //
                // So place them by sort, ties going to the OTHER layer: the bars are inserted
                // before the first hoisted layer that sorts at or above them. Sort-driven and
                // symmetric with the above-every-emitter hoist a few hundred lines up.
                // `?barsort=0` pins them last again (the old behaviour).
                const apRect = opts.mode === "entrance" ? scene?.data.entranceAperturePx : null;
                if (apRect && (typeof window === "undefined" || new URLSearchParams(window.location.search).get("letterbox") !== "0")) {
                    const [ax0, ay0, ax1, ay1] = apRect;
                    const F = 50000;
                    const bars = new PIXI.Graphics();
                    bars.beginFill(0x000000);
                    bars.drawRect(-F, -F, F + ax0, F * 2); // left
                    bars.drawRect(ax1, -F, F * 2, F * 2); // right
                    bars.drawRect(ax0, -F, ax1 - ax0, F + ay0); // top
                    bars.drawRect(ax0, ay1, ax1 - ax0, F * 2); // bottom
                    bars.endFill();
                    const barSort = scene?.data.entranceApertureSort;
                    const bySort = typeof window === "undefined" || new URLSearchParams(window.location.search).get("barsort") !== "0";
                    const above = sceneOverlay && bySort && barSort != null ? sceneOverlay.children.findIndex((m) => ((m as unknown as ISceneLayerRuntime).__sort ?? Number.NEGATIVE_INFINITY) >= barSort) : -1;
                    if (sceneOverlay && above === 0) {
                        // Every overlay layer sorts at/above the bars (Civilight Eterna: both
                        // transitions at 100, tied), so "just before the overlay" is the same
                        // draw order - and keeps the bars OUT of the overlay container, so
                        // `?abl=overlay` still attributes scene layers rather than silently
                        // deleting the pillarbox too (it read as a +6.9 luma "finding" once).
                        sceneContainer.addChildAt(bars, sceneContainer.getChildIndex(sceneOverlay));
                    } else if (sceneOverlay && above > 0) sceneOverlay.addChildAt(bars, above);
                    else sceneContainer.addChild(bars);
                }
                const scrollLayers: PIXI.Mesh[] = [];
                const ramLayers: PIXI.Mesh[] = [];
                const followLayers: PIXI.Mesh[] = [];
                if (scene) {
                    const scan = (cont: PIXI.Container | null | undefined) => {
                        if (!cont) return;
                        for (const m of cont.children) {
                            const rt = m as unknown as ISceneLayerRuntime;
                            if (rt.__uvScroll && !rt.__stCurve) scrollLayers.push(m as PIXI.Mesh);
                            if (rt.__ramSpeed) ramLayers.push(m as PIXI.Mesh);
                            if (rt.__follow) followLayers.push(m as PIXI.Mesh);
                        }
                    };
                    scan(scene.background);
                    scan(scene.foreground);
                    scan(sceneOverlay);
                }
                return {
                    spine,
                    root: sceneContainer,
                    separatorWash,
                    isScene: true,
                    cameraSizePx: scene?.data.cameraSizePx ?? null,
                    contentBounds: appRef.current?.renderer ? paintedLocalBounds(appRef.current.renderer, sceneContainer) : null,
                    hasDarkBackdrop: scene?.hasDarkBackdrop ?? false,
                    bounds,
                    authoredDisplayBounds,
                    authoredTightBounds,
                    authoredTightShift,
                    previewBounds,
                    settleFix,
                    entranceViewRatio: authoredFrame?.viewPx2 && authoredFrame?.viewPx ? (authoredFrame.viewPx2 as number) / authoredFrame.viewPx : null,
                    entranceDuration: scene?.data.entranceDuration ?? null,
                    entranceFadeEnd: entranceFadeEnd(scene?.data ?? null),
                    entranceFade: sceneDrivesEntranceFade(scene?.data ?? null) ? null : (scene?.data.entranceFade ?? null),
                    entranceClearColor: (scene?.data.entranceClearColor as [number, number, number] | undefined) ?? null,
                    entranceTransform: scene?.data.entranceTransform ?? null,
                    entranceOrthoCurve: (scene?.data.entranceOrthoCurve as [number, number][] | undefined) ?? null,
                    entrancePostFx: (scene?.data.entrancePostFx as IComposite["entrancePostFx"]) ?? null,
                    entranceCamCenterCurve: (scene?.data.entranceCamCenterCurve as [number, number, number][] | undefined) ?? null,
                    entranceCamRollCurve: (scene?.data.entranceCamRollCurve as [number, number][] | undefined) ?? null,
                    entranceFrameSize,
                    sceneLayers: opts.mode === "entrance" && scene ? [scene.background, scene.foreground, ...(sceneOverlay ? [sceneOverlay] : [])] : null,
                    aperture,
                    apertureMask,
                    scrollLayers,
                    ramLayers,
                    followLayers,
                    entranceSceneEnd: deferEndUntil,
                    requestEntranceEnd: opts.mode === "entrance" ? fireEntranceEnd : null,
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
                cameraSizePx: null,
                contentBounds: null,
                hasDarkBackdrop: false,
                bounds,
                separatorWash: [],
                authoredDisplayBounds: null,
                authoredTightBounds: null,
                authoredTightShift: null,
                previewBounds: null,
                settleFix: null,
                entranceViewRatio: null,
                entranceDuration: null,
                entranceFadeEnd: null,
                entranceFade: null,
                entranceClearColor: null,
                entranceTransform: null,
                entranceOrthoCurve: null,
                entrancePostFx: null,
                entranceCamCenterCurve: null,
                entranceCamRollCurve: null,
                entranceFrameSize: null,
                aperture: null,
                apertureMask: null,
                sceneLayers: null,
                scrollLayers: [],
                ramLayers: [],
                followLayers: [],
                entranceSceneEnd: null,
                requestEntranceEnd: opts.mode === "entrance" ? fireEntranceEnd : null,
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
                // Live per-emitter state for the parity harness (see ILoadedParticles.probe).
                // DEV-only and read-only: a missing effect cannot be diagnosed from the exported
                // JSON, because "emitted nothing" and "emitted off-frame" look identical there and
                // need opposite fixes. Call it from puppeteer after pumping the virtual clock.
                if (import.meta.env.DEV && typeof window !== "undefined") {
                    const w = window as unknown as { __dynProbe?: () => unknown; __dynXform?: () => unknown };
                    w.__dynProbe = () => particlesRef.current?.probe(app.screen.width, app.screen.height) ?? null;
                    // Resolved world transforms, for locating a placement divergence. The SCENE
                    // layers align to the capture within 0.10 px at every beat, so `root` is the
                    // known-good reference: any beat where the particle containers' matrix departs
                    // from it is where particle placement goes wrong. `paintedPx` shows Skadi's
                    // particle layer collapsing 1.69M -> 5.3K px^2 between t=9 and t=13 while the
                    // scene stays aligned, so the two are expected to diverge somewhere after t~11.
                    const m = (d: PIXI.Container | null | undefined) => {
                        if (!d) return null;
                        const t = d.worldTransform;
                        return { a: t.a, b: t.b, c: t.c, d: t.d, tx: t.tx, ty: t.ty };
                    };
                    w.__dynXform = () => {
                        const c = compositesRef.current?.find((x) => x.spine === spineRef.current) ?? compositesRef.current?.[0] ?? null;
                        return {
                            sceneRoot: m(c?.root),
                            spine: m(c?.spine as unknown as PIXI.Container),
                            particlesBg: m(particlesRef.current?.background),
                            particlesFg: m(particlesRef.current?.foreground),
                            screen: { w: app.screen.width, h: app.screen.height },
                        };
                    };
                    // DIAGNOSTIC (`__settleProbe`, DEV only): where the SETTLE actually lands, and
                    // what it was aimed at. The panel terminus is `main.contentBounds` fitted
                    // `mode: "contain"`, the same expression the 69 non-entrance skins use, so a
                    // panel that settles somewhere else has to be explained by one of three things
                    // and this tells them apart:
                    //
                    //   `contentBounds`   the box itself. Measured by `paintedLocalBounds` over
                    //                     THIS composite's own container, at `main` build time,
                    //                     which is BEFORE the `_Start` composite is built at all.
                    //   `dolly`           non-null means the post-hand-off move is still running,
                    //                     so any margin read at this instant is mid-flight. Grade
                    //                     on `dolly == null`, never on a guessed settle time.
                    //   `root`            the live world transform, i.e. what the camera ended up
                    //                     doing regardless of what it was handed.
                    //
                    // 🔑 Read `dolly` before believing a margin. A "framing" failure and a sample
                    // taken 0.4 s early look identical in the pixels.
                    const wS = window as unknown as { __settleProbe?: () => unknown };
                    wS.__settleProbe = () => {
                        const ez = entranceZoomRef.current;
                        return {
                            surface,
                            contentBounds: main.contentBounds,
                            bounds: main.bounds,
                            live: boundsRef.current,
                            dolly: ez ? { from: ez.from, to: ez.to, elapsed: ez.elapsed, duration: ez.duration, delay: ez.delay, fit: ez.fit ?? null } : null,
                            root: m(main.root),
                            screen: { w: app.screen.width, h: app.screen.height },
                        };
                    };
                    // Spine CLIPPING attachments, and whether pixi-spine actually applied them.
                    // Civilight Eterna's entrance is the only captured skin the game pillarboxes
                    // (a hard 1920x1080 aperture inside 2340x1080), and hers is the only `_Start`
                    // skeleton that ships clipping attachments - so the two need to be compared
                    // directly. `applied` distinguishes "the skeleton has no clip" from "it has one
                    // and our draw-order splicing tore the clippingContainer apart", which look the
                    // same in a render but need opposite fixes.
                    // Per-SLOT visibility: name, live attachment and colour alpha, plus the
                    // screen box of what it actually draws. Civilight Eterna's aperture EDGES are
                    // painted by the skeleton's own background attachments (ablating the spine
                    // flattens them; ablating scene or particles does not), and they go dark for
                    // exactly the 6→12 s window her side strips are empty - so "which slot stopped
                    // drawing, and was it the alpha or the attachment" is the question this answers.
                    const w3 = window as unknown as { __dynSlots?: () => unknown };
                    w3.__dynSlots = () => {
                        const c = compositesRef.current?.find((x) => x.spine === spineRef.current) ?? compositesRef.current?.[0] ?? null;
                        const sp = c?.spine as unknown as (PIXI.Container & { skeleton?: Record<string, unknown>; slotContainers?: PIXI.Container[] }) | undefined;
                        const sk = sp?.skeleton as unknown as { slots?: unknown[] } | undefined;
                        if (!sk?.slots) return "NO SKELETON";
                        const out: unknown[] = [];
                        (sk.slots as unknown[]).forEach((raw, i) => {
                            const sl = raw as {
                                data?: { name?: string };
                                color?: { a?: number };
                                attachment?: { name?: string } | null;
                            };
                            const cont = sp?.slotContainers?.[i];
                            let box: { x: number; y: number; w: number; h: number } | null = null;
                            try {
                                const b = cont?.getBounds();
                                if (b && b.width > 0 && b.height > 0) box = { x: b.x, y: b.y, w: b.width, h: b.height };
                            } catch {
                                box = null;
                            }
                            out.push({
                                i,
                                slot: sl.data?.name ?? "?",
                                att: sl.attachment?.name ?? null,
                                alpha: sl.color?.a ?? null,
                                vis: cont?.visible ?? null,
                                rend: cont?.renderable ?? null,
                                box,
                            });
                        });
                        return out;
                    };
                    const w2 = window as unknown as { __dynClip?: () => unknown };
                    w2.__dynClip = () => {
                        const c = compositesRef.current?.find((x) => x.spine === spineRef.current) ?? compositesRef.current?.[0] ?? null;
                        const sp = c?.spine as unknown as (PIXI.Container & { skeleton?: Record<string, unknown> }) | undefined;
                        const sk = sp?.skeleton as unknown as { slots?: unknown[] } | undefined;
                        if (!sk?.slots) return "NO SKELETON";
                        const wt = sp?.worldTransform;
                        const out: unknown[] = [];
                        for (const raw of sk.slots) {
                            const slot = raw as Record<string, unknown> & { getAttachment?: () => unknown };
                            const att = slot.getAttachment?.() as Record<string, unknown> | null;
                            if (!att || att.endSlot === undefined) continue;
                            const n = (att.worldVerticesLength as number) ?? 0;
                            const verts = new Float32Array(n);
                            (att.computeWorldVertices as (s: unknown, a: number, b: number, v: Float32Array, o: number, st: number) => void)?.(slot, 0, n, verts, 0, 2);
                            let x0 = Infinity;
                            let x1 = -Infinity;
                            let y0 = Infinity;
                            let y1 = -Infinity;
                            for (let i = 0; i < n; i += 2) {
                                x0 = Math.min(x0, verts[i]);
                                x1 = Math.max(x1, verts[i]);
                                y0 = Math.min(y0, verts[i + 1]);
                                y1 = Math.max(y1, verts[i + 1]);
                            }
                            const sx = (x: number, y: number) => (wt ? { x: wt.a * x + wt.c * y + wt.tx, y: wt.b * x + wt.d * y + wt.ty } : { x, y });
                            const p0 = sx(x0, y0),
                                p1 = sx(x1, y1);
                            const cc = slot.clippingContainer as PIXI.Container | undefined;
                            out.push({
                                slot: (slot.data as Record<string, unknown> | undefined)?.name,
                                attachment: att.name,
                                endSlot: (att.endSlot as Record<string, unknown> | null)?.name ?? null,
                                verts: n / 2,
                                localBox: { x: x0, y: y0, w: x1 - x0, h: y1 - y0, aspect: (x1 - x0) / (y1 - y0) },
                                screenBox: { x: Math.min(p0.x, p1.x), y: Math.min(p0.y, p1.y), w: Math.abs(p1.x - p0.x), h: Math.abs(p1.y - p0.y) },
                                applied: { hasGraphics: !!slot.currentGraphics, clipChildren: cc ? cc.children.length : -1 },
                            });
                        }
                        return { screen: { w: app.screen.width, h: app.screen.height }, clips: out };
                    };
                }

                const { width, height } = app.screen;
                // The viewer's own backdrop at the BACK of the stage (see createEnvironmentBgTexture):
                // the skin's camera clears to alpha 0, so this is what the game shows wherever the art
                // doesn't reach. Added here (after the scene has loaded) so it appears WITH the
                // illustration rather than covering the static load placeholder, and behind every
                // scene layer so they composite over it as they do in game. ALWAYS created - even a
                // scene that owns its own dark painted backdrop has real coverage gaps at its widest
                // camera framing (the opening pan, the post-handoff wide settle) where this is the
                // ONLY fill; omitting it reopens black voids there. Only the fill's COLOUR is gated,
                // on the data-derived `hasDarkBackdrop` (cello-only by construction).
                envBgDarkRef.current = !!main.hasDarkBackdrop;
                const envBg = new PIXI.Sprite(createEnvironmentBgTexture(main.hasDarkBackdrop));
                // DIAGNOSTIC (`?fill=RRGGBB`): repaint the viewer fill so the spine's EFFECTIVE
                // transparency can be measured - the mean shifts by (fillDelta x uncovered area).
                if (typeof window !== "undefined") {
                    const f = new URLSearchParams(window.location.search).get("fill");
                    if (f) envBg.tint = parseInt(f, 16);
                }
                resizeEnvironmentBg(envBg, width, height);
                envBgRef.current = envBg;
                app.stage.addChildAt(envBg, 0);
                // 🚨 `?abl=` WALKS SCENE CONTAINERS ONLY, and this sprite lives on the STAGE.
                // That blind spot has now cost three separate investigations: `srconly:` could
                // not reach `sceneOverlay`, and nothing could reach THIS, so a full-frame 77.3
                // fill read as an unexplained "grey floor" through several rounds of ablation
                // that all came back clean. Two fixes, both here because this is where the
                // stage is in scope:
                //   `?abl=envbg` hides the fill, so it can be attributed like anything else;
                //   any `abl=` token at all now logs the stage children, so a null result
                //   NAMES what it could not reach instead of reading as a clean negative.
                if (typeof window !== "undefined") {
                    const q = new URLSearchParams(window.location.search).get("abl");
                    if (q) {
                        if (q.split(",").includes("envbg")) envBg.renderable = false;
                        const names = app.stage.children.map((c, i) => `${i}:${(c as { name?: string }).name ?? c.constructor.name}`);
                        console.warn(`[abl] scene containers only. STAGE children NOT walked by bg:/fg:/src:/part*/scene* tokens: ${names.join(" ")}. Use abl=envbg for the environment fill.`);
                    }
                }
                // Settled ground, stacked directly above the backdrop and faded in by the tick.
                // Skipped for the dark-backdrop family, which keeps its own measured fill.
                if (!main.hasDarkBackdrop) {
                    const sb = new PIXI.Sprite(createEnvironmentBgTexture(false, SETTLED_GROUND));
                    sb.alpha = 0;
                    resizeEnvironmentBg(sb, width, height);
                    // DIAGNOSTIC (`?settledbg=0`): draw nothing for the settled ground while leaving
                    // every other consequence of `settledGroundOn()` exactly as shipped.
                    //
                    // `?settledground=0` CANNOT answer "how much is the white ground worth", because
                    // it is a COMPOUND switch gating three things: this sprite's alpha ramp, the
                    // `settledRef` latch that retires the gap-fill sprites, and the `gapFill` gate
                    // that decides whether a post-latch composite builds one at all. Its own doc
                    // comment says the halves are one defect and neither works alone (Muelsyse t=18:
                    // gap fill alone 49.142 -> 52.632, WORSE; together 49.142 -> 26.910). Reading its
                    // result as the ground's cost attributes three behaviours to one. Same shape as
                    // `?entfade=` needing `?fadesprite=` before the screen fade could be isolated.
                    //
                    // 🔑 Suppressing at CREATION rather than in the tick is deliberate. The alpha is
                    // written only inside `if (efd)`, and `efd` is nulled once the fade lifts, so
                    // past the hand-off nothing assigns it again and it FREEZES at whatever it last
                    // held, which is 1. A tick-side suppression would therefore stop reaching the
                    // exact region under investigation.
                    if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("settledbg") === "0") sb.renderable = false;
                    settledBgRef.current = sb;
                    app.stage.addChildAt(sb, 1);
                }
                // HDR bloom pass: render the scene into a half-float target so additive
                // light/flame stacks don't clip to white, then tonemap to screen. Created
                // once and re-pointed at whichever composite is live. Spine-only art never
                // used HDR, so gate on the main being a scene composite. Falls back to plain
                // 8-bit compositing (add the live composite straight to the stage) when
                // float targets aren't available.
                // DIAGNOSTIC (`?nohdr=1`): bypass the HDR float target + tonemap entirely and
                // draw straight to the canvas, so a residual can be attributed to (or cleared
                // of) that pass. The non-HDR path already exists for non-scene composites.
                const noHdr = typeof window !== "undefined" && !!new URLSearchParams(window.location.search).get("nohdr");
                // The composite transfer rides on this pass because it IS the final assembly
                // of the frame - the stage it was measured on. Derived per scene from the
                // authored camera size; `?gamma=1` disables it, `?gamma=<f>` forces a value.
                const hdr = main.isScene && !noHdr ? createHDRScene(app.renderer, width, height, app.renderer.resolution, undefined, sceneCompositeGamma(main.cameraSizePx)) : null;
                if (hdr) {
                    hdrRef.current = hdr;
                    app.stage.addChild(hdr.mesh);
                }

                // Make `c` the live composite: point the tick loop at it, attach it (under
                // the HDR pass, or straight onto the stage), and start its playback.
                const activate = (c: IComposite, playOpts?: { skipStart?: boolean }) => {
                    // Arm the end-of-entrance screen fade. Colour and end time both come from the
                    // director (`fadeColor`, `duration`); the sprite sits ABOVE the tonemap so it
                    // covers the assembled frame exactly as a client-side screen fade would, and
                    // is disabled by `?entfade=0`.
                    if (c.entranceFade && c.entranceDuration && entranceFadeRef.current === null) {
                        const off = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("entfade") === "0";
                        if (!off) {
                            const [fr, fg, fb] = c.entranceFade;
                            const sp = new PIXI.Sprite(PIXI.Texture.WHITE);
                            sp.tint = ((Math.round(fr * 255) << 16) | (Math.round(fg * 255) << 8) | Math.round(fb * 255)) >>> 0;
                            sp.width = width;
                            sp.height = height;
                            sp.alpha = 0;
                            app.stage.addChild(sp);
                            entranceFadeRef.current = { sprite: sp, elapsed: 0, duration: c.entranceFadeEnd ?? c.entranceDuration, out: null, transform: c.entranceTransform ?? null };
                        }
                    }
                    // ENTRANCE POST-PROCESS: greyscale and mobile blur.
                    //
                    // ⚠️ NOT nested in the screen-fade block above, and NOT driven off its clock.
                    // It used to be both, which silently disabled the whole subsystem for any skin
                    // whose director authors no fade colour: Civilight Eterna ships an
                    // `HGMobileBlur` volume and `entranceFade` null, so her blur never ran.
                    //
                    // The BLUR radius comes from the profile, not a fitted constant.
                    // `HGMobileBlur`'s shader (`Hidden/Torappu/PostEffect/MobileBlurWithMask`) is a
                    // 4-tap box at +-0.5 TEXEL of its render target, iterated `blurDegree` times,
                    // with the target downsampled by `resMode` - so one pass spans
                    // `blurSpread * 2^resMode` full-res px. She authors 1 / 1 / 1 -> 2 px.
                    //
                    // Measured: where her weight is high our render is 1.5-3.8x SHARPER than the
                    // capture (gradient ratio), and where it is 0 we are 0.83x, the usual softness.
                    // ⚠️ Her scored beats sample NEITHER blur window (5.2-6.8 and 11.6-13.0), so
                    // this cannot move her MADC - validate it at a beat inside a window.
                    const pf = c.entrancePostFx;
                    if (pf && postFxOn() && pf.weightCurve.length > 1 && /grey|gray|saturat|blur/i.test(pf.effect)) {
                        const blur = /blur/i.test(pf.effect);
                        const q = pf.params ?? {};
                        entrancePostFxRef.current = {
                            filter: blur ? new PIXI.BlurFilter() : new PIXI.ColorMatrixFilter(),
                            kind: blur ? "blur" : "saturation",
                            blurPx: blurPxOverride() ?? (q.blurSpread ?? 1) * 2 ** (q.resMode ?? 0),
                            curve: pf.weightCurve,
                            intensity: pf.intensity,
                            target: c.root,
                            elapsed: 0,
                        };
                        if (blur) {
                            // `blurDegree` is the iteration count the shader runs.
                            (entrancePostFxRef.current.filter as PIXI.BlurFilter).quality = Math.max(1, Math.round(q.blurDegree ?? 1));
                        }
                    }
                    spineRef.current = c.spine;
                    particlesRef.current = c.particles;
                    separatorWashRef.current = c.separatorWash ?? [];
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

                // `authoredDisplayBounds` is null whenever the skin's `_adjustes[0]` extent is
                // unusable - Unity writes an uninitialised stop as -FLT_MAX, which `usableExtent`
                // (above) correctly rejects. That guard was necessary but not sufficient: EVERY
                // path below runs through `openStandingIdle`, which bailed on a null `gameFrame`
                // BEFORE calling `attach()`, so the composite root was never added to the stage
                // and the skin rendered as a completely EMPTY frame - a healthy skeleton with 205
                // renderable meshes and valid textures, simply never drawn. The composite has
                // already computed a sane framing box for exactly this case (the authored
                // `cameraSizePx` square centred on the visible art), so fall back to it. Same
                // `?? bounds` chain the entrance paths below already use; a no-op for every skin
                // whose `_adjustes[0]` is a real number.
                // The post-entrance idle settles on the TIGHT `_adjustes[1]` stop, not the wide
                // `_adjustes[0]` one (`?settletight=0` reverts).
                //
                // This path used to disagree with itself. A skin with NO authored pull-out already
                // settled tight - see the `fromEntrance && !entrancePullOut && openTight` branch
                // below, which carries "verified against the recording, the steady settle is a
                // knees-up shot, not a full-body pull-out". A skin WITH one dollied out to the wide
                // box instead, purely because it happened to have a usable `entranceTransform`.
                // So Mlynar (no transform) looked right and Virtuosa (transform 12.0) did not, for
                // no reason connected to how either is framed.
                //
                // The wide box is the ARCHIVE crop: this file already records that it "renders the
                // subject ~1.6x too small against the captures", with authored tight/wide ratios
                // ska 1.598, cel 1.548, mly 1.484.
                //
                // MEASURED against the Android captures, aligned by cross-correlating each raw
                // against its own trimmed clip (t0 = 4.10-4.40 s, correlation 0.987-1.000) and
                // sampled after each skin's pull-out would have completed (`duration +
                // (duration - transform)`). All five affected skins improve and none regresses:
                //
                //     cel  wide needs 1.75-1.90x to match, tight needs 1.10x
                //     ska / mue / wis / exc   wide renders small and vignetted; tight fills the
                //                             frame at the capture's scale
                //
                // ⚠️ Judge this VISUALLY. Gradient-NCC is useless on the settle - our settled idle
                // is also markedly darker than the capture, which pins correlation near 0.05 at
                // every scale, and a scale search silently clips at its own search bounds.
                // ⚠️ MADC cannot see this at all: every scored beat lands before the hand-off, so
                // all eight baselines are bit-identical either way. That is why this survived so
                // long.
                // The post-entrance idle settles on the PREVIEW box (`cameraSizePx x 0.9565`) -
                // `?settlebasis=tight` for the `_adjustes[1]` stop, `=wide` for the old archive crop.
                //
                // The game returns to its SKIN-PREVIEW shot after the cinematic, and `previewBounds`
                // is exactly that shot, already derived and calibrated here: of the three candidate
                // bases it has the lowest cross-skin spread (3.0% against 4.6% and 5.1%) and is
                // "exact on ska and cel". It lands ~0.806x the tight `_adjustes[1]` box.
                //
                // Two steps got here. First the settle stopped dollying out to the WIDE archive crop
                // (it only did so for skins that happened to carry an `entranceTransform`, which is
                // why Mlynar looked right and Virtuosa did not). Then the residual showed up as the
                // idle looking too DARK - which was not tone at all: our view was still wider than
                // the game's, so dark surround was being pulled into frame. Virtuosa's periphery
                // measured 54.7 against the capture's 143.4, with only 2.3% of it at the environment
                // fill value, i.e. real content beyond the game's edge rather than uncovered area.
                //
                // Mean settled-frame level against the captures (ours/game, 1.00 = match):
                //
                //     skin   wide   tight   PREVIEW
                //     ska           0.88     1.00
                //     wis           0.92     1.01
                //     mue           0.89     0.97
                //     cel    0.73   0.73*    0.83     (*her tight settle still ran ~1.10x wide)
                //     mly           0.91     0.91     (no transform - unaffected by any of this)
                //     exc           1.14     1.45     <- the one regression, see below
                //
                // ⚠️ Executor is worse on this proxy. Her entrance is a dark scope shot and her
                // `cameraSizePx` is the least representative of the set; visually her preview
                // framing is no worse than the tight one, and the level difference is tonal rather
                // than geometric. Shipped anyway because the basis is the calibrated one and four
                // skins improve, but she is the skin to re-check if this is ever revisited.
                // ⚠️ MADC cannot see ANY of this - every scored beat lands before the hand-off.
                const settleBasis = typeof window === "undefined" ? "" : (new URLSearchParams(window.location.search).get("settlebasis") ?? "");
                const settleBox = settleBasis === "wide" ? main.authoredDisplayBounds : settleBasis === "tight" ? (main.authoredTightBounds ?? main.authoredDisplayBounds) : (main.previewBounds ?? main.authoredTightBounds ?? main.authoredDisplayBounds);
                // DIAGNOSTIC (`?settlescale=`): multiply the SETTLED frame extent about its own
                // centre - the twin of `?camscale=`, which only reaches the entrance camera. The
                // settled frame is `bodyFrameBox(cameraSizePx * PREVIEW_CAM_FRAC)`, i.e. sized
                // from `cameraSizePx` but CENTRED ON THE BODY, so a skin whose backdrop is not
                // centred on its character can frame off the art. Exists so "what extent does the
                // game actually settle at" is a measurement rather than an argument. Inert at 1.
                const settleScaleRaw = typeof window === "undefined" ? Number.NaN : parseFloat(new URLSearchParams(window.location.search).get("settlescale") ?? "");
                const settleScale = Number.isFinite(settleScaleRaw) && settleScaleRaw > 0 ? settleScaleRaw : 1;
                const scaledSettleBox =
                    settleBox && settleScale !== 1
                        ? {
                              x: settleBox.x + (settleBox.width * (1 - settleScale)) / 2,
                              y: settleBox.y + (settleBox.height * (1 - settleScale)) / 2,
                              width: settleBox.width * settleScale,
                              height: settleBox.height * settleScale,
                          }
                        : settleBox;
                // THE SETTLED FRAMING CORRECTION (see `settleFix`), applied here because this is
                // the last point the settled box exists before it becomes the camera.
                //
                // The parameters are in OUTPUT pixels: our render must SHRINK by `s` and move by
                // (dx, dy) to land on the game. Content apparent size goes as 1/box.height, so
                // shrinking the content by `s` means GROWING the box by 1/s. The fit maps the box
                // HEIGHT onto the container height, so one output pixel is `box.height / 416`
                // render units, and moving content right means moving the box LEFT, hence both
                // offsets are subtracted. `?settleyflip=1` inverts the vertical, which is the one
                // term a reading of the code cannot settle.
                //
                // `asSettled` says the box is the SETTLED TARGET rather than a dolly endpoint, and
                // it decides whether the extent is re-normalised. `openTight` has both roles: for a
                // skin with an authored pull-out it is the tight endpoint the dolly starts from,
                // where its own extent is the authored meaning and must be preserved; for a skin
                // without one it is the frame that is HELD, i.e. the settled shot. Normalising it in
                // the first role collapses the endpoint onto the settled size and measured whitw2
                // 64.203 -> 74.555, which is how the two roles were told apart.
                const applySettleFix = (box: IAnimationBounds | null, asSettled: boolean): IAnimationBounds | null => {
                    const fx = main.settleFix;
                    if (!box || !fx) return box;
                    const [s, dx, dy] = fx;
                    // The corrected extent is defined by the PREVIEW box, whatever box we start
                    // from. `scale = cameraViewPx / 1935.2` was calibrated on subjects that settle
                    // at `previewBounds`, so 1935.2 is normalised against THAT extent; applying the
                    // same ratio to a different box divides by the wrong number.
                    //
                    // Chyue is the case, and it is exact rather than approximate. She settles at
                    // `authoredTightBounds` (extent `cameraViewPx2` = 1254.140) where the others
                    // settle at `previewBounds` (extent `cameraSizePx * PREVIEW_CAM_FRAC` =
                    // 1004.325). Their ratio is 1.2487 and her measured post-correction residual was
                    // 1.260; equivalently `s_rule * tight/preview` = 0.7899 against a measured 0.790.
                    // No constant is chosen here: the ratio is authored fields only, and for every
                    // subject that DOES settle at the preview box it is exactly 1, which is the
                    // built-in control.
                    const target = asSettled ? (main.previewBounds?.height ?? box.height) : box.height;
                    const h = target / s;
                    const w = box.width * (h / box.height);
                    const yf = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("settleyflip") === "1" ? -1 : 1;
                    const cx = box.x + box.width / 2 - (dx * h) / 416;
                    const cy = box.y + box.height / 2 - (yf * dy * h) / 416;
                    return { x: cx - w / 2, y: cy - h / 2, width: w, height: h };
                };
                const fixedSettleBox = applySettleFix(scaledSettleBox, true);
                const gameFrame = fixedSettleBox ?? main.bounds;
                const openTight =
                    main.authoredTightBounds && settleScale !== 1
                        ? {
                              x: main.authoredTightBounds.x + (main.authoredTightBounds.width * (1 - settleScale)) / 2,
                              y: main.authoredTightBounds.y + (main.authoredTightBounds.height * (1 - settleScale)) / 2,
                              width: main.authoredTightBounds.width * settleScale,
                              height: main.authoredTightBounds.height * settleScale,
                          }
                        : main.authoredTightBounds; // the game's `_adjustes[1]`, else null
                // ...and the SAME correction on the tight box. A skin that arrives from `_Start`
                // with no authored pull-out HOLDS this frame as its settled shot (see the
                // `fromEntrance && !entrancePullOut` branch below) and never reaches `gameFrame`,
                // so correcting only the settle box leaves it untouched. Chyue is that case: she
                // was inert even to an absurd `?settledxy=0.5,200,200` while `?settlescale=`
                // reached her here, which is what located the second path.
                // `openTight` in its SETTLED role: the frame a skin with no authored pull-out HOLDS
                // as its steady shot. Chyue is the only settled subject there. Its dolly-endpoint
                // uses (`openFrom`, `startBox` below) keep the RAW box, because there the authored
                // extent is the meaning and the correction belongs only to the settled frame.
                const heldSettleBox = applySettleFix(openTight, true);
                if (gameFrame) main.bounds = gameFrame; // the idle settles at the game display frame

                // When we arrive from the `_Start` cinematic, the settled idle continues the
                // camera move: the game HELD the tight close-up through the whole transform
                // (seated → apple → reform) and only pulls back to the wide throne AFTERWARD.
                // So the idle OPENS on the same tight close-up the entrance ended on - the wide
                // frame scaled by the entrance camera's ratio (`entranceViewRatio`) - and dollies
                // OUT to the wide throne over the POST-reform entrance time (`duration−transform`).
                // Set by the entrance branch below; null for the no-`_Start` fallback (which opens
                // on the idle `_adjustes[1]` tight stop instead).
                let entrancePullOut: { ratio: number; dur: number } | null = null;

                // Open on the standing idle at the settled frame, then dolly-out. Used both as the
                // fallback (no `_Start`) and as the hand-off target after the `_Start` cinematic.
                const openStandingIdle = (opts?: { fromEntrance?: boolean; handoffPanDelta?: [number, number] | null }) => {
                    if (aborted() || !appRef.current || !gameFrame) return;
                    const { width: sw, height: sh } = appRef.current.screen;
                    // An explicit `?framebox=` wins over every other framing, and over any
                    // dolly - the point is a camera that CANNOT move between renders.
                    const pinned = frameBoxParam();
                    if (pinned) {
                        main.bounds = pinned;
                        boundsRef.current = pinned;
                        layoutSpine(main.root, sw, sh, pinned, { mode: "contain", align: fitRef.current.align });
                        return;
                    }
                    // Arriving from a `_Start` cinematic WITHOUT an authored pull-out window
                    // (no `_transform` beat in the gamedata, e.g. Mlynar): the game HOLDS the
                    // tight `_adjustes[1]` frame after the white-out (verified against the
                    // recording - the steady settle is a knees-up shot, not a full-body pull-out).
                    // A pull-out is only performed when the data authorizes one (`entrancePullOut`,
                    // cello) or when there was no cinematic at all (the standard archive open).
                    // THE PANEL'S ENTRANCE TERMINATES AT THE CONTAIN BOX. The cinematic plays
                    // on every surface; only where it ENDS differs. A non-entrance skin settles at
                    // `whole = main.contentBounds` fitted `mode: "contain"`, which is what
                    // letterboxes, while the entrance settle height-fits through `fitRef` and crops
                    // the width into full bleed. On a panel the entrance now lands in the same box
                    // the 69 non-entrance skins already use, so the idle after the cinematic
                    // matches the idle without one.
                    //
                    // 🔑 It is a DOLLY TARGET, not a snap. Returning early and assigning the box
                    // would jump at the hand-off, because the cinematic ends at the height-fit
                    // camera. Feeding it to `entranceZoomRef` instead means the existing post-
                    // hand-off dolly drives the whole move, and it carries its own `fit` so the
                    // contain mode applies across the interpolation rather than switching under it.
                    //
                    // ⚠️ Terminating at contain does NOT license contain during the cinematic. The
                    // entrance's own framing is untouched; only the terminus moves.
                    //
                    // `?panelsettle=0` reverts a panel to the viewer's full-bleed terminus, kept as
                    // the one deliberate A/B here. Read as an explicit "0" so an ABSENT parameter
                    // keeps the shipped behaviour. (The previous `?entwhole=1` and `?panelentrance=1`
                    // are GONE rather than left as dead toggles: this IS the entwhole behaviour, now
                    // surface-gated, and the entrance is never suppressed.)
                    const panelSettleOn = typeof window === "undefined" || new URLSearchParams(window.location.search).get("panelsettle") !== "0";
                    const panelTerminus = surface === "panel" && opts?.fromEntrance && panelSettleOn && fitWholeArt() && !staticCamOn() ? main.contentBounds : null;
                    const panelFit: ISpineFit | undefined = panelTerminus ? { mode: "contain", align: fitRef.current.align } : undefined;
                    if (opts?.fromEntrance && !entrancePullOut && heldSettleBox && !panelTerminus) {
                        main.bounds = heldSettleBox; // resizes keep the held tight frame
                        layoutSpine(main.root, sw, sh, heldSettleBox, fitRef.current);
                        boundsRef.current = heldSettleBox;
                        return;
                    }
                    // Plain archive open (no `_Start` cinematic, so not an entrance hand-off) with no
                    // authored pull-out: the skin has NO camera move at all, so it must open STATIC at
                    // the settled game frame. The former fallback dollied `openTight` (`_adjustes[1]`)
                    // → `gameFrame` (`_adjustes[0]`) over an invented 2s, which read as an unwanted pan
                    // on skins that never had an entrance. A dolly is only performed for a genuine
                    // entrance hand-off (`fromEntrance`) or a data-authored pull-out (`entrancePullOut`).
                    if (!opts?.fromEntrance && !entrancePullOut) {
                        // A skin with NO `_Start` has no authored shot to hold - the game's viewer
                        // shows the whole cut-out with margins, and the `_adjustes[0]` box crops
                        // well inside it (Ch'en the Holungday: 2704x1804 of content against a
                        // 1500 px view). Frame the drawn CONTENT instead. 69 of the 82 dynchar
                        // skins take this path; the 13 with an entrance are untouched, which is
                        // why the three measured reference skins cannot move.
                        // `?statcam=1` reproduces the game's SKIN-PREVIEW shot, which is NOT this
                        // archive shot. The archive view contains the whole cut-out with margins;
                        // the preview runs scene content edge to edge with no margin on any side,
                        // so it is an authored camera box fitted by HEIGHT (see `fitRef`), and the
                        // TIGHT `_adjustes[1]` stop rather than the wide `_adjustes[0]` one: the
                        // wide box renders the subject ~1.6x too small against the captures, and
                        // the authored tight/wide ratio is exactly that (ska 1934.6/1211 = 1.598,
                        // cel 1929/1246 = 1.548, mly 1990/1341 = 1.484). Gamedata, not a fitted
                        // constant. Falls back to the wide box for any skin shipping no [1] stop.
                        // Half the wide `_adjustes[0]` view (see `previewBounds`), else the tight
                        // box re-seated on its own authored centre (see `authoredTightShift`;
                        // `?tightoff=0` A/Bs that), else the wide box.
                        let previewBox = main.previewBounds ?? main.authoredTightBounds ?? gameFrame;
                        const tshift = main.authoredTightShift;
                        if (staticCamOn() && !main.previewBounds && tshift && tightOffOn() && main.authoredTightBounds) {
                            previewBox = { ...previewBox, x: previewBox.x + tshift[0], y: previewBox.y + tshift[1] };
                        }
                        const preview = staticCamOn() ? staticCamBox(previewBox) : null;
                        // SURFACE-GATED (2026-08-31): the whole-cut-out contain belongs to the
                        // PANEL only. Ch'en the Holungday's two outfits captured in the game's
                        // full-screen viewer ("Ten Thousand Mountains", "Holiday HD79", 60 fps,
                        // DIAG/) render FULL-BLEED, and our render with this branch skipped
                        // (`wholeart=0`, i.e. the authored `_adjustes[0]` gameFrame below)
                        // matches their crop while the contain arm floats the cut-out in a grey
                        // field over 46.7% fill. The 13 entrance skins never reach this branch
                        // (they arrive `fromEntrance`), so the corpus cannot move. `?fitwhole=1`
                        // restores the contain on the viewer for A/B.
                        const fitWholeOverride = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("fitwhole") === "1";
                        const whole = (surface === "panel" || fitWholeOverride) && fitWholeArt() && !staticCamOn() ? main.contentBounds : null;
                        const settle = preview ?? whole ?? gameFrame;
                        // The authored framing fits by HEIGHT, which crops the width - fine for a
                        // shot composed around the character, wrong for "show the whole picture".
                        // A content box is only meaningful CONTAINED, which is also what the game's
                        // viewer does (it letterboxes the cut-out with margins).
                        const settleFit: ISpineFit = whole ? { mode: "contain", align: fitRef.current.align } : fitRef.current;
                        main.bounds = settle;
                        boundsRef.current = settle;
                        layoutSpine(main.root, sw, sh, settle, settleFit);
                        return;
                    }
                    let openFrom = entrancePullOut ? inflateBounds(gameFrame, entrancePullOut.ratio) : openTight;
                    // Hand-off continuity (Sf fix): `inflateBounds` above is a pure symmetric scale
                    // around `gameFrame`'s OWN static centre - it has no notion of the entrance rig
                    // camera's actual PAN over the shot. Re-base the box by the live camera's real
                    // pan delta (hand-off centre minus its own t=0 centre, curve-space units added
                    // directly onto `gameFrame`'s centre - the same cross-space-additive convention
                    // `centerBlend` already uses in the tick above), so the dolly starts exactly
                    // where the entrance camera actually ended instead of snapping to an
                    // independently-recomputed box. Zero pan (or no captured delta - no `_Start`,
                    // or the no-pull-out branch above) leaves this byte-identical to before.
                    // SANITY BOUND: this cross-space add is only meaningful for a genuine "same shot,
                    // slightly drifted" case (Skadi2: ~370-unit lateral drift). A skin whose rig camera
                    // travels THROUGH the scene in depth (Virtuosa's shaft plunge: ~3400 authored units,
                    // ~9× her own entrance frame size) isn't drifting off a shared reference - applying
                    // the raw delta there blows the box off-frame entirely (verified: produced a
                    // black/void hand-off). Gate on the delta being smaller than the target frame's own
                    // extent - a data-derived bound (not a per-skin constant), true for every drift-style
                    // pan and false for every plunge-style one measured so far.
                    if (openFrom && opts?.handoffPanDelta) {
                        const [dx, dy] = opts.handoffPanDelta;
                        const maxDelta = Math.max(gameFrame.width, gameFrame.height);
                        if (Math.abs(dx) < maxDelta && Math.abs(dy) < maxDelta) {
                            openFrom = { ...openFrom, x: openFrom.x + dx, y: openFrom.y + dy };
                        }
                    }
                    const dur = entrancePullOut ? entrancePullOut.dur : 2.0;
                    // The panel's terminus replaces `gameFrame` as the dolly TARGET and carries the
                    // contain fit with it, so the move is interpolated rather than snapped. A skin
                    // with no pull-out has no `openFrom`, so it starts the dolly from the tight
                    // frame the cinematic ended on.
                    const target = panelTerminus ?? gameFrame;
                    // A resize re-lays out from `main.bounds`, which was set to `gameFrame` before
                    // the cinematic. Point it at the terminus or the panel loses its box on resize.
                    if (panelTerminus) {
                        main.bounds = panelTerminus;
                        boundsRef.current = panelTerminus;
                    }
                    const startBox = openFrom ?? (panelTerminus ? openTight : null);
                    // The authored transform -> duration pull-out happens INSIDE the cinematic:
                    // the game performs it before `duration` and then CUTS to the settled frame
                    // (measured on fugue/kalts/chyue: the cut lands at dur +0.116..0.267 and the
                    // settled camera is STATIC afterwards, cel_settled2 phase-corr shift (0,0)
                    // scale 1.00). Replaying that dolly on the MAIN composite after the hand-off
                    // kept our settled camera moving for (duration - transform) seconds per skin
                    // (cel 6.0, mue 7.0, ska 12.833, kalts 4.4, each matching its audited settle
                    // transient), so the main now opens AT the terminus. The durations above are
                    // read from the authored fields, never chosen. `?handoffdolly=1` restores the
                    // previous post-handoff replay exactly.
                    const replayDolly = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("handoffdolly") === "1";
                    if (startBox && replayDolly) {
                        layoutSpine(main.root, sw, sh, startBox, panelFit ?? fitRef.current);
                        entranceZoomRef.current = { container: main.root, from: startBox, to: target, elapsed: 0, duration: dur, delay: 0.15, fit: panelFit };
                    } else {
                        layoutSpine(main.root, sw, sh, target, panelFit ?? fitRef.current);
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
                    const mu = mainUnderRef.current;
                    if (mu) {
                        // Put the main spine back where the composite expects it before the
                        // hand-off cross-fade wraps main.root, and lift the guard's renderable flags.
                        mu.spineParent.addChildAt(mu.main.spine, Math.min(mu.spineIndex, mu.spineParent.children.length));
                        mu.underRoot.parent?.removeChild(mu.underRoot);
                        mu.underRoot.destroy({ children: false });
                        type GuardSlot = { currentMesh?: PIXI.DisplayObject; currentSprite?: PIXI.DisplayObject };
                        for (const sl of (mu.main.spine.skeleton as unknown as { slots: GuardSlot[] }).slots) {
                            const disp = sl.currentMesh ?? sl.currentSprite;
                            if (disp) disp.renderable = true;
                        }
                        mainUnderRef.current = null;
                    }
                    const ent = composites.find((c) => c !== main);
                    // Capture the entrance camera's actual pan (its live centre at hand-off minus
                    // its own t=0 centre) BEFORE `entranceFollowRef` is nulled below - this is what
                    // lets `openStandingIdle` start the post-handoff dolly continuously from where
                    // the eye actually was, instead of an independently-recomputed box (see the Sf
                    // handoff-continuity fix there).
                    const ef = entranceFollowRef.current;
                    const handoffPanDelta: [number, number] | null = ef?.lastLiveCenter ? [ef.lastLiveCenter[0] - ef.startCenter[0], ef.lastLiveCenter[1] - ef.startCenter[1]] : null;
                    entranceZoomRef.current = null;
                    entranceFollowRef.current = null;
                    // The client stops rendering through the entrance cameras here and resumes
                    // drawing the square 2048 RT quad; match it. Happens UNDER the held fade below.
                    raiseToIdleResolution();
                    // Retire the scope with the sequencer that drives it. The cover is only ever
                    // updated from the entrance tick, so leaving it visible here would black the
                    // frame for the whole hand-off window (Executor renders pure black at 5.5s
                    // otherwise - the reveal is at 5.2 but the composite swap comes later).
                    if (entranceSeqRef.current?.apertureMask) entranceSeqRef.current.apertureMask.visible = false;
                    entranceSeqRef.current = null;
                    // The fade is at (or near) full here - hold it briefly so the swap happens
                    // UNDER it, then lift, which is what the recordings show.
                    if (entranceFadeRef.current) entranceFadeRef.current.out = 0;
                    activate(main, { skipStart: true }); // straight to standing idle (no second intro beat)
                    openStandingIdle({ fromEntrance: true, handoffPanDelta });
                    if (ent && hdr) {
                        // Crossfade: render BOTH roots in one wrapper (HDR renders it), main behind
                        // fading in, the dissolved entrance on top fading out - the character reforms
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
                // `?statcam=1` declines to build the entrance at all, so the fall-through below opens
                // the standing idle STATIC at the settled frame - the game's own preview shot. Gating
                // the BUILD (rather than the playback) also keeps the `_Start` skel/atlas/scene off
                // the wire, so a statcam render is not perturbed by the entrance's assets at all.
                // The entrance BUILDS AND PLAYS ON EVERY SURFACE. What differs by surface is where
                // it ENDS, which `openStandingIdle` handles: a panel terminates at the contain box
                // the non-entrance skins settle into, a viewer keeps the full-bleed authored frame.
                //
                // ⚠️ An earlier pass gated the BUILD here so a panel skipped the cinematic entirely.
                // That framed the idle correctly by deleting the thing being framed. The cinematic
                // is wanted on the surface people browse; only its terminus was wrong.
                const entrance = staticCamOn()
                    ? null
                    : await buildComposite(startSkel, startAtlas, {
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
                // No cinematic: the idle path is live already, so take its target now.
                if (!built) raiseToIdleResolution();
                if (built) {
                    // Repaint the environment fill with the entrance camera's authored clear colour
                    // (see `camClearOn`). The fill sprite is created with the main composite, before
                    // the `_Start` scene is loaded, so the swap happens here.
                    if (camClearOn() && built.entranceClearColor && envBgRef.current) {
                        const old = envBgRef.current.texture;
                        envBgRef.current.texture = createEnvironmentBgTexture(false, cssHex(built.entranceClearColor));
                        old.destroy(true);
                        resizeEnvironmentBg(envBgRef.current, app.screen.width, app.screen.height);
                    }
                    // The game HOLDS the entrance shot steadily through the whole transform and only
                    // pulls back to the wide frame AFTERWARD, on the settled idle (see
                    // `entrancePullOut` → `openStandingIdle`). Measured against the reference
                    // recording, the entrance holds the `_Start` camera's OWN close-up - the ANIMATED
                    // t=0 ortho view (`entranceFrameSize`, Virtuosa 374px: a tight head-and-shoulders
                    // shot) - TIGHTER than both the `_adjustes[1]` stop (1246px, the settled idle's
                    // open) and the exporter's static `entranceViewPx` (598px, pre-override ortho).
                    composites.push(built);
                    activate(built);
                    // The idle continues the shot: open on this same `_adjustes[1]` framing and dolly
                    // OUT to the wide `_adjustes[0]` throne over the POST-reform entrance window
                    // (`duration − transform` = the ~6s the game takes to pull back). All gamedata.
                    // Computed BEFORE the camera-follow below: a skin with NO authored transform
                    // beat instead gets the late-entrance steady-camera takeover (`centerBlend`).
                    if (built.entranceViewRatio && built.entranceDuration && built.entranceTransform) {
                        entrancePullOut = { ratio: built.entranceViewRatio, dur: Math.max(0.5, built.entranceDuration - built.entranceTransform) };
                    }
                    let tight = built.authoredTightBounds ?? built.authoredDisplayBounds ?? built.bounds;
                    // DIAGNOSTIC (`?entscale=`, `?entview=1`): a skin with NO camera track frames its
                    // entrance on the IDLE's tight bounds, and its own `entranceViewPx` (the `_Start`
                    // camera's serialized ortho view) is never consulted. `?entscale=` scales the box;
                    // `?entview=1` replaces its SIZE with `entranceViewPx`, keeping the centre.
                    if (tight && typeof window !== "undefined") {
                        const q = new URLSearchParams(window.location.search);
                        const es = parseFloat(q.get("entscale") ?? "");
                        const evPx = built.entranceFrameSize;
                        const cx0 = tight.x + tight.width / 2;
                        const cy0 = tight.y + tight.height / 2;
                        if (q.get("entview") === "1" && evPx) {
                            tight = { x: cx0 - evPx / 2, y: cy0 - evPx / 2, width: evPx, height: evPx };
                        } else if (Number.isFinite(es) && es > 0) {
                            tight = { x: cx0 - (tight.width * es) / 2, y: cy0 - (tight.height * es) / 2, width: tight.width * es, height: tight.height * es };
                        }
                    }
                    if (tight) {
                        layoutSpine(built.root, width, height, tight, fitRef.current);
                        const sl = built.sceneLayers ?? [];
                        for (const c of sl) c.alpha = 1;
                        // Sequence this entrance's layers regardless of whether it also has a
                        // camera track (see `entranceSeqRef`).
                        entranceSeqRef.current = {
                            spine: built.spine,
                            sceneLayers: sl,
                            endAt: built.entranceSceneEnd,
                            fireEnd: built.entranceSceneEnd != null ? built.requestEntranceEnd : null,
                            aperture: built.aperture,
                            apertureMask: built.apertureMask,
                        };
                        // `?mainunder=1`: the MAIN skeleton live under the rig from t=0, on its own
                        // Start clip then the idle (see mainUnderOn and the frame-loop guard). Only the
                        // spine is taken under; the main's scene and particles stay where they are and
                        // stay unrendered until the hand-off, which puts the spine back.
                        if (mainUnderOn() && main.spine !== built.spine && main.spine.spineData.findAnimation("Start")) {
                            const spineParent = main.spine.parent as PIXI.Container | null;
                            if (spineParent) {
                                const spineIndex = spineParent.getChildIndex(main.spine);
                                const underRoot = new PIXI.Container();
                                underRoot.addChild(main.spine);
                                const wrap = new PIXI.Container();
                                const rigSpineParent = built.spine.parent as PIXI.Container | null;
                                if (mainAtRigSpine() && rigSpineParent) {
                                    rigSpineParent.addChildAt(underRoot, rigSpineParent.getChildIndex(built.spine) + 1);
                                } else {
                                    if (mainAboveRig()) {
                                        wrap.addChild(built.root);
                                        wrap.addChild(underRoot);
                                    } else {
                                        wrap.addChild(underRoot);
                                        wrap.addChild(built.root);
                                    }
                                    if (hdr) {
                                        hdrSceneRef.current = wrap;
                                    } else {
                                        app.stage.removeChild(built.root);
                                        app.stage.addChild(wrap);
                                    }
                                }
                                // A delayed Start holds an empty animation first; the frame-loop guard
                                // draws nothing while that empty entry is current (the setup pose has
                                // kalts's crystals at alpha 1, so the hold must not render).
                                const msd = mainStartDelay();
                                if (msd > 0) {
                                    main.spine.state.setEmptyAnimation(0, 0);
                                    main.spine.state.addAnimation(0, "Start", false, msd);
                                } else {
                                    main.spine.state.setAnimation(0, "Start", false);
                                }
                                if (main.spine.spineData.findAnimation(IDLE_ANIMATION)) main.spine.state.addAnimation(0, IDLE_ANIMATION, true, 0);
                                // THE GUARD'S SET, derived from the rig's own data: a rig slot is "never
                                // visible" when no colour timeline of any rig animation ever keys its alpha
                                // above 0 and its setup alpha is 0, or every key is 0. Only main slots whose
                                // rig twin is in this set may draw: the first guard (rig alpha 0 at the
                                // current frame) admitted body parts the rig fades in later or draws under
                                // other names (cel +27, ska +63, kalts double-drawn at t=1).
                                const rigNever = new Set<string>();
                                {
                                    const rd = built.spine.skeleton.data as unknown as { slots: { name: string; color: { a: number } }[]; animations: { timelines: unknown[] }[] };
                                    const setupA = new Map<string, number>();
                                    for (const sl of rd.slots) setupA.set(sl.name, sl.color.a);
                                    const keyedMax = new Map<string, number>();
                                    for (const an of rd.animations) {
                                        for (const tlU of an.timelines) {
                                            const tl = tlU as { slotIndex?: number; frames?: ArrayLike<number>; getFrameEntries?: () => number; constructor: { name: string } };
                                            if (typeof tl.slotIndex !== "number" || !tl.frames) continue;
                                            const cn = tl.constructor.name;
                                            if (!/Color|RGBA|Alpha/.test(cn)) continue;
                                            const stride = typeof tl.getFrameEntries === "function" ? tl.getFrameEntries() : cn.includes("Two") ? 8 : 5;
                                            const fr = tl.frames;
                                            const nm = rd.slots[tl.slotIndex]?.name;
                                            if (!nm) continue;
                                            let hi = 0;
                                            for (let i = 0; i + stride - 1 < fr.length; i += stride) hi = Math.max(hi, /^Alpha/.test(cn) ? fr[i + 1] : fr[i + 4]);
                                            keyedMax.set(nm, Math.max(hi, keyedMax.get(nm) ?? 0));
                                        }
                                    }
                                    // a keyed slot's visibility is what its keys say; an unkeyed one keeps its setup pose
                                    for (const [nm, a] of setupA) if ((keyedMax.has(nm) ? (keyedMax.get(nm) as number) : a) <= 0.001) rigNever.add(nm);
                                }
                                mainUnderRef.current = { main, ent: built, underRoot, spineParent, spineIndex, noCounterpart: 0, rigNever };
                                if (import.meta.env.DEV) {
                                    // PROBE hook for the arm: where the main spine sits and whether it renders.
                                    (window as unknown as { __mainUnder?: () => unknown }).__mainUnder = () => {
                                        const ms = main.spine as unknown as { visible: boolean; renderable: boolean; worldAlpha: number; alpha: number; parent: unknown; worldTransform: PIXI.Matrix; skeleton: { slots: { data: { name: string }; currentMesh?: PIXI.DisplayObject; currentSprite?: PIXI.DisplayObject }[] } };
                                        const rendered = ms.skeleton.slots
                                            .filter((sl) => (sl.currentMesh ?? sl.currentSprite)?.renderable)
                                            .map((sl) => {
                                                const d = (sl.currentMesh ?? sl.currentSprite) as unknown as { getBounds?: () => { x: number; y: number; width: number; height: number }; alpha?: number } | undefined;
                                                const b = d?.getBounds?.();
                                                const c = (sl as unknown as { color: { a: number }; getAttachment?: () => { name?: string } | null }).color;
                                                return {
                                                    n: sl.data.name,
                                                    a: Number(c.a.toFixed(3)),
                                                    da: d?.alpha ?? null,
                                                    att: (sl as unknown as { getAttachment?: () => { name?: string } | null }).getAttachment?.()?.name ?? null,
                                                    box: b ? [Math.round(b.x), Math.round(b.y), Math.round(b.width), Math.round(b.height)] : null,
                                                };
                                            });
                                        const es = built.spine as unknown as { worldTransform: PIXI.Matrix };
                                        return {
                                            visible: ms.visible,
                                            renderable: ms.renderable,
                                            worldAlpha: ms.worldAlpha,
                                            alpha: ms.alpha,
                                            hasParent: !!ms.parent,
                                            under: underRoot.transform.worldTransform.toArray(false),
                                            rig: es.worldTransform.toArray(false),
                                            main: ms.worldTransform.toArray(false),
                                            renderedSlots: rendered.length,
                                            sample: rendered,
                                            noCounterpart: mainUnderRef.current?.noCounterpart ?? null,
                                            wrapKids: wrap.children.length,
                                            hdrIsWrap: hdrSceneRef.current === wrap,
                                        };
                                    };
                                }
                            }
                        }
                        // Drive the entrance camera PURELY from gamedata: the exporter-accumulated camera
                        // rig track (`entranceCamCenterCurve`, absolute mesh-px frame centre) for the
                        // pan/dolly, and the `_adjustes[1]` view extent (`entranceFrameSize`) × the ortho
                        // zoom for the frame size. No measured bounds, no per-skin tuning. Only when both
                        // gamedata inputs exist - otherwise the static `tight` framing already laid out
                        // above holds (spine-only / skins with no camera track).
                        if (built.entranceCamCenterCurve?.length && built.entranceFrameSize) {
                            // Steady framing (see the tick): skins WITHOUT an authored transform beat
                            // HOLD the horizontal centre on the settle-open box (dropping the rig's
                            // right-swinging X excursion), keep the rig's vertical motion re-based onto
                            // the settle centre, and follow the ortho zoom-out to full body uncapped.
                            // Skins WITH a transform (cello, skadi2) keep the pure rig camera → null.
                            let centerBlend: { cx0: number; cy0: number; rigEndY: number } | null = null;
                            const settleOpen = main.authoredTightBounds ?? main.authoredDisplayBounds;
                            const rigEnd = built.entranceCamCenterCurve[built.entranceCamCenterCurve.length - 1];
                            if (!entrancePullOut && settleOpen && rigEnd) {
                                centerBlend = { cx0: settleOpen.x + settleOpen.width / 2, cy0: settleOpen.y + settleOpen.height / 2, rigEndY: rigEnd[2] };
                            }
                            entranceFollowRef.current = {
                                spine: built.spine,
                                root: built.root,
                                ortho: built.entranceOrthoCurve ?? null,
                                camCenter: built.entranceCamCenterCurve,
                                camRoll: built.entranceCamRollCurve,
                                camCuts: detectCurveCuts(built.entranceCamCenterCurve, built.entranceFrameSize),
                                frameSize: built.entranceFrameSize,
                                centerBlend,
                                startCenter: sampleCurveXY(built.entranceCamCenterCurve, 0) ?? [0, 0],
                                lastLiveCenter: null,
                                lastCamRaw: null,
                                sceneLayers: sl,
                                endAt: built.entranceSceneEnd,
                                fireEnd: built.entranceSceneEnd != null ? built.requestEntranceEnd : null,
                            };
                        }
                    }
                    // The "Start" clip plays STRAIGHT THROUGH to its own end - the reform, the voice
                    // beat and the settle all live inside the clip, and the live-follow camera
                    // (entranceFollowRef) keeps the reformed standing form in frame - so no early
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
            // The client's target is a FIXED size, so the resolution that reproduces it moves as
            // the container does.
            {
                const want = dynRenderResolution(h, entranceFollowRef.current ? "entrance" : "idle");
                if (Math.abs(currentApp.renderer.resolution - want) > 0.01) currentApp.renderer.resolution = want;
            }
            currentApp.renderer.resize(w, h);
            hdrRef.current?.resize(w, h, currentApp.renderer.resolution);
            if (envBgRef.current) resizeEnvironmentBg(envBgRef.current, w, h);
            if (settledBgRef.current) resizeEnvironmentBg(settledBgRef.current, w, h);
            const target = sceneContainerRef.current ?? spineRef.current;
            layoutSpine(target, w, h, boundsRef.current, fitRef.current);
        });
        resizeObserver.observe(container);

        return () => {
            mountedRef.current = false;
            resizeObserver.disconnect();
            cleanup();
        };
    }, [files.skel, files.atlas, files.png, server, backdrop, surface]);

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
