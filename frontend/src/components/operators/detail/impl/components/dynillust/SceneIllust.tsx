import * as PIXI from "pixi.js";
import { useEffect, useRef, useState } from "react";
import { Spinner } from "#/components/ui/spinner";
import type { IChibiSpineFiles } from "#/lib/api/chibis";
import { cn } from "#/lib/utils";
import { ANIMATION_SPEED } from "../chibi/constants";
import { chibiAssetURL, DEFAULT_SPINE_FIT, type IAnimationBounds, type ISpineFit, layoutSpine, loadSpineWithEncodedURLs, measureAnimationBounds, visibleRect } from "../chibi/helpers";
import { createHDRScene, type IHDRScene, sceneCompositeGamma } from "./hdrTonemap";
import {
    particleCensus, ensureAdditiveSpriteBoost, type FindBone, type ILoadedParticles, loadParticles } from "./particles";
import {
    applySceneLayerColor,
    applySceneLayerFollow,
    applySceneLayerRamScroll,
    applySceneLayerSt,
    applySceneLayerUvScroll,
    detectCurveCuts,
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

/** DIAGNOSTIC (`?framebox=x,y,w,h`): pin the settled camera to an EXPLICIT box, in the same
 *  space as {@link IComposite.bounds}.
 *
 *  Exists because the two other framings are each unusable for per-layer attribution. The
 *  authored `_adjustes` camera sits INSIDE the art on most skins (Ch'en's sky quad fills
 *  99.94% of it), so anything at the art's edge is off-frame entirely; and the whole-art
 *  framing is derived from PAINTED content, so every `?abl=` ablation silently re-frames the
 *  shot and an ablated render cannot be compared with an unablated one. A fixed box is
 *  independent of both — identical across every ablation, and wide enough to include the
 *  silhouette boundary. Rendered with a `contain` fit so the whole box is visible.
 *
 *  Returns null when absent or malformed, leaving the normal framing untouched. */
/** Draw the skin's static illustration, DEFOCUSED, behind a scene that does not span the camera
 *  view — filling the bare canvas the game fills with vista. **SHIPPED, default ON; `?gapfill=0`
 *  disables.**
 *
 *  Virtuosa's entrance scene genuinely does not reach the frame (0 of 132 layers span the view,
 *  the largest reaches 0.88) and nothing is dropped by the exporter, so the uncovered margin falls
 *  through to bare canvas — a hard-edged near-black wedge where the game shows misty vista. What
 *  fell through was our own invention: a flat neutral #4d4d4e studio fill, a colour the game never
 *  shows. Replacing it with the skin's own art, blurred, does exactly what it should:
 *
 *      cel  19.422 -> 19.270     t=2 -0.678, t=5 -0.387, every other beat EXACTLY 0.000
 *      mly  17.721 -> 17.721     bit-identical
 *      ska  10.505 -> 10.505     bit-identical
 *
 *  The two beats that move are the two with the camera at its widest — exactly where the scene
 *  leaves the most frame unfilled. Everything else is untouched to the bit.
 *
 *  **Why blurred.** The static illustration contains the CHARACTER, so drawn sharp behind a
 *  translucent scene it can ghost her, misregistered. That cannot be masked away: the art sits at
 *  index 0 and the spine draws in FRONT of it, so a spine-shaped mask can only remove pixels that
 *  were already hidden — built and measured, it moved 786 px of 374400 on `lin_nian#10`, all of
 *  them antialiasing at the spine's edge. The ghost lives precisely where the spine ISN'T. But the
 *  art is only here to be a distant vista, and a vista needs its low-frequency colour and
 *  luminance, not its detail. Defocusing keeps the fill and destroys the recognisable second
 *  character.
 *
 *  The radius is a fraction of the art's own on-screen height, so it is a spatial-frequency cutoff
 *  rather than a pixel constant, and it is not a fitted one — the parity win is a broad plateau,
 *  19.270 to 19.284 over the whole range 1/48 to 1/6, and every value from 1/96 to 1/3 beats the
 *  unblurred 19.361.
 *
 *  Refuted variants are recorded in `hdrTonemap.ts` so none is retried. NOTE for the harness:
 *  `rec.js` must be given `&backdrop=<url>` or this whole path is inert — the production viewer
 *  always passes it (`SkinsContent.tsx`), the recorder does not. */
function gapFillOn(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("gapfill") !== "0";
}
/** Blur radius as a fraction of the backdrop's on-screen height. `?gapblur=<f>` sweeps it. */
const GAP_BLUR_FRACTION = 1 / 24;
function gapBlurFraction(): number {
    if (typeof window === "undefined") return GAP_BLUR_FRACTION;
    const v = parseFloat(new URLSearchParams(window.location.search).get("gapblur") ?? "");
    return Number.isFinite(v) && v > 0 ? v : GAP_BLUR_FRACTION;
}

/** `?matlead=<seconds>` — offset applied to material colour-curve sampling only. Inert (0) by
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

/** Frame a no-entrance skin to everything it PAINTS rather than to its authored `_adjustes`
 *  camera. **DISABLED by default — the target is real but only ONE reference exists.**
 *
 *  The observation is solid. 70 of the 82 dynchar skins have no `_Start`, and for those the
 *  game's viewer shows the whole cut-out with margins while the authored `_adjustes[0]` box
 *  crops well inside it — content exceeds the authored view on 61 of 80 skins (Ch'en the
 *  Holungday 2690x1789 against a 1171 px view; Virtuosa 2.7x; Ch'en Wei 3.8x). Against the
 *  in-game recording of Ch'en, framing the painted content is plainly the better match.
 *
 *  It is off because ONE reference cannot calibrate 70 skins, and two concrete problems
 *  remain:
 *    • `fitRef` is `mode: "height"` for authored framing, which CROPS the width — so a
 *      content box wider than the viewport still does not show the whole art. Needs a
 *      "contain" fit, which changes layout for every skin.
 *    • `paintedLocalBounds`' alpha floor is a tuned constant. At >8/255 it catches faint
 *      haze that extends far past the visible composition: Nian's painted box comes out
 *      3037x2267 against a 1640 px authored view, dropping her art from 45% of the frame to
 *      25%. Her authored crop already showed the whole composition — but that judgement is
 *      aesthetic, not measured, because there is no recording of her to check against.
 *
 *  TO ENABLE: capture 2-3 in-game recordings of no-entrance skins with DIFFERENT content
 *  ratios (a tight one and a sprawling one), fix the fit mode to "contain", and calibrate the
 *  alpha floor against them. `?wholeart=1` turns it on now for inspection. */
function fitWholeArt(): boolean {
    if (typeof window === "undefined") return true;
    return new URLSearchParams(window.location.search).get("wholeart") !== "0";
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
 *  actually is. Model-free and fully general — no per-skin data, no assumptions about which
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
    // a full one then measures the ZOOM CHANGE, not the ablated element — which inverted a
    // real result once: Ch'en's foreground read as DARKENING the art by 11.2 when, framed
    // identically, it BRIGHTENS it by 11.4 (verified by bounding box: the content-fit pair was
    // framed differently, the authored pair identically). **Always pass `wholeart=0` on BOTH
    // sides of an ablation comparison.** Forcing children renderable here was tried and does
    // NOT fix it — the ablation is applied before this runs, through more than one flag.
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
function calibrationParam(name: string, fallback: number): number {
    try {
        return parseFloat(new URLSearchParams(window.location.search).get(name) || String(fallback)) || fallback;
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
 *  faithfully — and the animated character still renders over the static — we hide
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
    /** The scene's authored orthographic camera size in px (`cameraSize × 100`). Drives the
     *  composite transfer — see {@link sceneCompositeGamma}. Null for spine-only art. */
    cameraSizePx: number | null;
    /** Union of everything the composite actually draws (all mesh layers + the skeleton), in
     *  the same space as {@link bounds}. The authored `_adjustes` camera CROPS INTO this for
     *  most skins — content is larger than the authored view on 61 of 80 — which is right for
     *  a skin whose entrance cinematic frames a shot, and wrong for one that has none, where
     *  the game's viewer fits the whole cut-out. Null for spine-only art. */
    contentBounds: IAnimationBounds | null;
    /** True when the scene owns a DARK opaque painted backdrop (see
     *  {@link ILoadedScene.hasDarkBackdrop}) — the scene supplies its own environment, so the
     *  light-grey studio gradient must NOT be composited behind it (it would bleed through the
     *  frame's un-covered edges and wash the deep colour to grey). */
    hasDarkBackdrop: boolean;
    bounds: IAnimationBounds | null;
    /** The game's AUTHORED display frame in frontend coords (`cameraViewPx` square
     *  centred on the Y-flipped `cameraOffsetPx`) — the exact camera the in-game
     *  viewer frames these skins with. Used to frame the special-entry entrance skins
     *  (and their idle) to match the game rather than the character-bounds heuristic.
     *  Null for spine-only art with no authored camera. */
    /** The backdrop wash's seat inside this composite's spine, when the game splits the
     *  skeleton. Null when the skin ships no `separatorSlots`. */
    separatorWash: ISeparatorWash[];
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
    /** Straight RGBA of the director's end-of-entrance screen fade (`_params.fadeColor`), or
     *  null when the skin ships no entrance director. See {@link ENTRANCE_FADE_IN}. */
    entranceFade: [number, number, number, number] | null;
    entranceTransform: number | null;
    /** The `_Start` camera dolly ZOOM curve (`[t_s, ortho]` keyframes) — the game's actual
     *  data-driven camera motion (extracted from the clip animating the Main Camera's orthographic
     *  size). Scales the entrance frame relative to its base extent as the shot pushes in/out. */
    entranceOrthoCurve: [number, number][] | null;
    /** The `_Start` camera's ABSOLUTE frame-centre track (`[t_s, cx, cy]`, mesh px) — the game's
     *  own camera rig (the animated camera-parent Transform) accumulated by the Rust exporter into
     *  a world-space centre curve. Drives the entrance pan/dolly directly; no measured bounds. */
    entranceCamCenterCurve: [number, number, number][] | null;
    /** The entrance frame EXTENT in mesh px — the `_Start` camera's view at its ANIMATED t=0
     *  ortho size (`2·ortho₀/skeletonScale`, falling back to the static `entranceViewPx`, then the
     *  tight `_adjustes[1]` stop) — the base frame size the live camera holds, scaled by the
     *  ortho-size ratio each tick. Same 100 px/world-unit scale as
     *  {@link entranceCamCenterCurve}, so they compose directly. */
    entranceFrameSize: number | null;
    /** The entrance's scene-mesh containers (background + foreground) — carry the per-layer
     *  `m_IsActive` gating meshes. Entrance-only. */
    sceneLayers: PIXI.Container[] | null;
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
     *  ancestry — Mlynar's sword flare). The always-running tick rebases them onto their
     *  live bone each frame, after the spine update. Empty for scenes with none. */
    followLayers: PIXI.Mesh[];
    /** When the authored SCENE timeline outlasts the spine's own "Start" animation
     *  (Mlynar: the spine ends at 14.33s but the white-flash plane + camera run to the
     *  clip stop 15.97s), the time (track seconds) the entrance actually ends — the
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
/** Seconds spent ramping INTO the fade colour, ending just before `entranceDuration`. */
const ENTRANCE_FADE_IN = 0.85;
/** Seconds the fade holds at full before lifting (covers the idle swap). */
const ENTRANCE_FADE_HOLD = 0.2;
/** Seconds spent lifting the fade once the idle is live — Mlynar's capture is fully white at
 *  `duration - 0.1` and back to the idle mean by `duration + 0.3`. */
const ENTRANCE_FADE_OUT = 0.35;

/** Does the SCENE already perform the end-of-entrance fade itself, making the client-side
 *  director fade a double-count?
 *
 *  Every skin authors a white overlay for the transition, so "has a fade layer" cannot be the
 *  test — all four benchmarks have one. What separates them is whether that layer SPANS THE
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
 *  236.5 — the user-visible "the glow is instant instead of fading". Mlynar is unaffected either
 *  way (his layer is already opaque white through the whole ramp, so the director fade adds
 *  nothing to add), which is what makes this safe to apply by rule rather than per skin.
 *
 *  Colour is compared to the director's own `fadeColor` rather than assumed white. */
function sceneDrivesEntranceFade(data: ISceneData | null): boolean {
    const fade = data?.entranceFade;
    const dur = data?.entranceDuration;
    if (!data || !fade || !dur || !data.cameraSizePx) return false;
    const ext = 2 * data.cameraSizePx;
    const rampStart = dur - ENTRANCE_FADE_HOLD - ENTRANCE_FADE_IN;
    const rampEnd = dur - ENTRANCE_FADE_HOLD;
    return data.layers.some((l: ISceneLayer) => {
        const cc = l.colorCurve;
        if (!cc?.length) return false;
        // Spans the camera view on BOTH axes — only then can it white out the frame alone.
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

function makeBackdropSprite(backdrop: ILoadedBackdrop, frame: ISceneFrame, spineCentroid: { x: number; y: number }): PIXI.Sprite {
    const { texture, centroid } = backdrop;
    const sprite = new PIXI.Sprite(texture);
    // Anchor at the illustration's own centroid so `position` places THAT point;
    // register it onto the spine's centroid so the two illustrations overlap by
    // mass (robust where a bounding-box centre is skewed by an arch/railing).
    sprite.anchor.set(centroid.nx, centroid.ny);
    // CONTAIN, not cover. Covering the visible rect was tried to kill the neutral fill showing
    // beyond a contained backdrop (Muelsyse's settled frame is mostly bare canvas on a 2.16:1
    // viewport) and is REFUTED: the game's surround there is a flat NEUTRAL grey/gradient, while
    // the blurred art is coloured — a cover fill puts (84,142,161) blue where the game holds
    // neutral. The surround is the viewer's own chrome, not the skin's art scaled up.
    sprite.scale.set((2 * frame.cameraSizePx) / Math.max(texture.width || 1, texture.height || 1));
    sprite.position.set(spineCentroid.x, spineCentroid.y);
    return sprite;
}

/** The in-game viewer composites the illustration over its OWN backdrop, and that backdrop
 *  is a flat NEUTRAL DARK GREY. Every skin's dynchar camera clears with `m_ClearFlags: 2`
 *  (solid colour) and `m_BackGroundColor.a = 0` — the skin renders into a TRANSPARENT
 *  target, so nothing in the scene fills the frame its art doesn't reach; whatever shows
 *  there belongs to the viewer, not to the skin. Measured on the in-game captures of three
 *  skins with three completely different worlds (Mlynar "Fields of Ruination", Virtuosa
 *  "Diversity Oneness", Skadi2 "Iteration") across ten beats: every uncovered corner reads
 *  32–39 in ALL channels — one neutral tone, the same for every skin at every time.
 *
 *  This supersedes two earlier guesses. A bright studio gradient (`#dddee2`) was ~3× too
 *  bright. Deriving the tone from the SCENE (the mean colour of its largest painted
 *  backdrop) made Mlynar's frame edges a light blue-GREY (73,81,95) — wrong in hue as well as
 *  level, because his backdrop bakes its vignette INTO the texture, so an alpha-weighted mean
 *  reports the lit painted INTERIOR. Any per-scene derivation is wrong in principle here: the
 *  tone is not the scene's to supply. `resizeEnvironmentBg` keeps it covering the viewport.
 *
 *  CORRECTED 2026-07-28: the level was `#232324` (35,35,36), taken from "every uncovered
 *  corner reads 32–39". Those corners were not uncovered. At almost every beat the art reaches
 *  the frame edge — sampled across all three captures, the margins are strongly coloured and
 *  vary 43→208 with the camera, i.e. they are painted scenery, and a dark corner of scenery
 *  reads ~35 exactly like a dark fill. The fill is genuinely exposed only at Mlynar's widest
 *  entrance framing, and there it is unambiguous: flat, neutral, and 76–83 down the FULL height
 *  of both margins and in all four corners. Sampling only the pre-flash window matters — from
 *  t≈13.2 his white transition flash ramps those same margins 79 → 125 → 202 → 255, so a
 *  sample taken a few frames later reads the flash, not the fill.
 *
 *  Measured over the frame INTERIOR this is a clean win with no cost anywhere: Mlynar 20.878 →
 *  19.733 (t=13 −5.57, t=12.4 −3.41, every other beat flat to ±0.02) and Skadi bit-identical,
 *  since his fill is never exposed. Virtuosa is untouched — she takes the `dark` branch.
 *
 *  Scored over the FULL frame it instead reads −0.12/+0.03, with apparent ~0.5 regressions at
 *  t=7 and t=11.8. That is an artifact of the metric, not of this constant: the default window
 *  includes columns 0–35 and 864–899, which are the CAPTURE'S OWN dark UI chrome (temporal SD
 *  ~1.0 vs ~70 in the interior — no renderer signal at all), so raising the fill is scored as a
 *  growing mismatch against a black border that is not ours to match. Confirmed by measuring the
 *  art's transmittance to this fill directly, from two renders that differ only in its value:
 *  it is 0 or 1 and never in between (≥99.5% of interior pixels fully opaque at t=4…11, the
 *  remainder fully uncovered), so no semi-transparent leak exists to trade against. An earlier
 *  note here claimed one and blamed thin layer alphas; that was reading the chrome. */
// Fresh per-app texture (the app is destroyed with `texture: true`, so a shared/cached
// texture would be torn down under later mounts).
const VIEWER_BACKDROP = "#4d4d4e";

function createEnvironmentBgTexture(dark = false): PIXI.Texture {
    const S = 512;
    const cvs = document.createElement("canvas");
    cvs.width = S;
    cvs.height = S;
    const ctx = cvs.getContext("2d");
    if (ctx) {
        if (dark) {
            // A scene that owns a self-lit DARK painted world (`hasDarkBackdrop`, Virtuosa's
            // mirror-world) composites that world SEMI-TRANSPARENTLY over this fill, so here the
            // fill is not only the frame surround but also a light LEAK through the art — and our
            // layer alphas run thinner than the game's. Its measured optimum therefore sits below
            // the viewer's own grey (Virtuosa MAD 31.89 with this dark gradient vs 32.01 with the
            // viewer grey), so that family keeps the fill as measured.
            const g = ctx.createRadialGradient(S / 2, S * 0.3, S * 0.08, S / 2, S * 0.5, S * 0.78);
            g.addColorStop(0, "#1a1b22");
            g.addColorStop(1, "#0c0d12");
            ctx.fillStyle = g;
        } else {
            ctx.fillStyle = VIEWER_BACKDROP;
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
 * skeleton already contains the ENTIRE scene — painted background, effect/glass
 * layers, sparkles — as Spine slots (with their blend modes baked in) and mesh
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
 *  front of the whole skeleton — but the game does not wash her body, because the game SPLITS
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
 *  SHIPPED (`?sepwash=0` disables). The per-frame re-seat once LEAKED — `spine.update()` re-appends slot
 *  containers each frame, so re-seating grows `spine.children` without bound: cello reaches
 *  849 children by t=17 against ~329 slots, one leaked per frame. Fix the leak before
 *  re-enabling; the 15.190 measurement below was taken WITH the leak active.
 *
 *  Previously shipped (`?sepwash=0` disables). At cello t=2 the deficit region goes 114.0 -> 173.8
 *  against the game's 193.8 with NO ghosting (her darks 26.8 vs the game's 25.5; undemoting
 *  gives 151.9). Seating all four sheets regressed the frame (21.969 -> 24.671) — the fix was
 *  to bind each sheet by SORT to the parts' own depths (see `sheetTarget` in particles.ts),
 *  which puts bg_ref/bg_tint_01/air_01 in the gap and leaves bg_rain_01 genuinely in front.
 *  cel 18.844 -> 17.167, ska 10.500 -> 10.465, mly bit-identical (ships no separator).
 */
function reseatSeparatorWash(spine: unknown, seps: ISeparatorWash[]): void {
    if (!seps.length) return;
    const sp = spine as unknown as { children: PIXI.DisplayObject[]; addChildAt(c: PIXI.DisplayObject, i: number): unknown; slotContainers?: PIXI.DisplayObject[] };
    const conts = sp.slotContainers;
    if (!conts) return;
    // pixi-spine re-adds every slot container each `update()` to restore `skeleton.drawOrder`.
    // With a FOREIGN child spliced in, its index bookkeeping stops matching and containers get
    // appended instead of moved, so `children` grows one per frame — cello reached 849 against
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
            console.log(`DBGBG seat k=${k} slotIndex=${sep.slotIndex} at=${at} washKids=${sep.wash.children.length} spineKids=${sp.children.length}`);
        }
    }
}

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
    const attachRestRef = useRef<Map<string, { x: number; y: number }> | null>(null);
    // HDR bloom pass: when float render targets are available, the scene is drawn
    // into `hdr.target` (half-float, additive stacks don't clip) and tonemapped to
    // screen via `hdr.mesh`. `hdrSceneRef` is the container rendered into it.
    const hdrRef = useRef<IHDRScene | null>(null);
    /** The gap-fill backdrop sprite, when it is composited by the tonemap threshold rather than
     *  drawn inline — rendered alone into `hdr.bdTarget` each frame (see the tick). */
    const hdrSceneRef = useRef<PIXI.Container | null>(null);
    /** The backdrop wash re-parented inside the spine (see {@link reseatSeparatorWash}). */
    const separatorWashRef = useRef<ISeparatorWash[]>([]);
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
    /** ENTRANCE LAYER SEQUENCING — the `_Start` clip's per-layer `m_IsActive` windows,
     *  material-colour curves and UV-ST curves. Kept SEPARATE from `entranceFollowRef`
     *  because that ref only exists when the skin ships a camera-centre curve, and layer
     *  sequencing has nothing to do with the camera: gating it on the camera track left
     *  three skins (Kalt'sits "boc#6", Cetsyr, Chongyue "epoque#7") replaying their entrance
     *  with EVERY layer at its static tint. For Kalt'sits that means the closing full-frame
     *  white flash — authored white at full alpha — paints the whole cinematic pure white. */
    const entranceSeqRef = useRef<{ spine: import("pixi-spine").Spine; sceneLayers: PIXI.Container[]; endAt: number | null; fireEnd: (() => void) | null } | null>(null);
    const entranceFollowRef = useRef<{
        spine: import("pixi-spine").Spine;
        root: PIXI.Container;
        ortho: [number, number][] | null;
        camCenter: [number, number, number][];
        /** Segment end-indices in `camCenter` that are HARD CUTS (Skadi the Corrupting Heart's
         *  rig repositions) — the sampler STEPS through these instead of interpolating, so an
         *  instant reposition isn't smeared into a visible pan. Empty for cut-free curves. */
        camCuts: Set<number>;
        frameSize: number;
        /** Steady framing (see the tick): for skins with NO authored transform beat (Mlynar), the
         *  baked rig-centre curve swings the hero into the right third (its X excursion) and the
         *  game instead holds him CENTRED, letting only the ortho zoom widen the shot. We HOLD the
         *  horizontal centre at the settle-open box centre (`cx0`), KEEP the rig's vertical motion
         *  re-based so it lands on `cy0` at the rig's end (`cy0 + (rigY − rigEndY)` — needed so the
         *  tight t=0 frame catches the reforming head), and let the ortho zoom pull out to full body
         *  uncapped (the wide→tight idle handoff hides behind the white flash — same settle centre).
         *  Null = pure rig camera (cello, skadi2). */
        centerBlend: { cx0: number; cy0: number; rigEndY: number } | null;
        /** The live rig camera's `camCenter` sample at track-time 0, captured once at build
         *  time — the reference point for `lastLiveCenter` below, so `swapToMainIdle` can
         *  compute how far the camera panned by hand-off (see Sf handoff-continuity fix in
         *  `openStandingIdle`). */
        startCenter: [number, number];
        /** The most recently sampled live camera centre (refreshed every tick, same value
         *  used to build `liveDisplayBox`). Read by `swapToMainIdle` at the exact hand-off
         *  instant — BEFORE this ref is nulled — so the post-handoff dolly can start from
         *  where the entrance camera actually ended rather than a value re-derived
         *  independently from the idle skeleton's own static bounds. Null until the first
         *  tick runs. */
        lastLiveCenter: [number, number] | null;
        sceneLayers: PIXI.Container[];
        /** Deferred entrance end (see IComposite.entranceSceneEnd): fire `fireEnd`
         *  when the track clock reaches `endAt`. Null when the spine's own
         *  `complete` handles the handoff. */
        endAt: number | null;
        fireEnd: (() => void) | null;
    } | null>(null);
    // Crossfade the `_Start` cinematic OUT (frozen at its dissolved final frame) while the main
    // gala idle fades IN — so the character is never absent during the transformation (the game
    // overlaps the reform with the dissolve). Both roots are wrapped in one container the HDR
    // pass renders; the tick ramps their alphas, then detaches the main and frees the entrance.
    // END-OF-ENTRANCE SCREEN FADE. The director carries the COLOUR (`_params.fadeColor`) and the
    // END TIME (`_params.duration`) as data; only the ramp lengths are client behaviour, and they
    // are measured, not guessed. Virtuosa is the one reference where the fade is unconfounded —
    // Mlynar's own white-transition plane already holds the frame white from t=15.0, and Skadi
    // reaches white through her authored fade LAYERS — and her recording runs
    // 134 → 146 → 178 → 211 → 240 → 254 over `duration - 1.0s` → `duration - 0.2s`.
    const entranceFadeRef = useRef<{ sprite: PIXI.Sprite; elapsed: number; duration: number; out: number | null } | null>(null);
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

        ensureAdditiveSpriteBoost(); // MUST precede Renderer construction (plugins bind at build)
        const app = new PIXI.Application({
            width: container.clientWidth || 600,
            height: container.clientHeight || 450,
            backgroundAlpha: 0,
            antialias: true,
            resolution: typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1,
            autoDensity: true,
            // The tick below drives EVERYTHING — clock, spine, particles, and both render
            // passes — so PIXI's own ticker must not render as well. Left on it did two
            // things: it drew `app.stage` a second time every frame (double the GPU work for
            // the whole viewer), and it could draw the stage BEFORE the frame's scene pass
            // had written the HDR target, so the tonemap quad sampled a render target that
            // nothing had filled in yet.
            autoStart: false,
        });
        app.ticker.stop();
        appRef.current = app;
        container.appendChild(app.view as HTMLCanvasElement);

        let lastTick = performance.now();
        let sceneClock = 0;
        const tick = (now: number) => {
            if (!mountedRef.current) return;
            const dt = Math.min((now - lastTick) / 1000, 0.1);
            lastTick = now;
            // Capability A — continuous shader UV-scroll. The Ram-family scene layers scroll
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
                // does not stick — this has to run after each update.
                reseatSeparatorWash(spineRef.current, separatorWashRef.current);
                if (typeof window !== "undefined" && new URLSearchParams(window.location.search).get("sepdbg") === "1") {
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
            // by the ortho ratio — no measured bounds, no easing, no tuning: purely the rig's motion.
            // Live authored display box for particle edge-clip culling this frame — the
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
                // keyframe timing IS the game's dolly/pan — replay it directly.
                const c = sampleCurveXY(ef.camCenter, tt, ef.camCuts) ?? [0, 0];
                // Camera SIZE: the gamedata frame extent (`_adjustes` view px) × the ortho-size ratio,
                // so the character grows into the frame exactly as the authored zoom dictates.
                // DIAGNOSTIC (`?ortholead=<seconds>`): sample the ZOOM curve at `tt + lead` only,
                // leaving the pan/centre on `tt`. Isolates a zoom-specific timing error from a
                // global clock error (which would move the centre too). Inert at 0.
                const orthoLead = typeof window !== "undefined" ? parseFloat(new URLSearchParams(window.location.search).get("ortholead") ?? "0") || 0 : 0;
                const size = ef.frameSize * orthoZoomRatio(ef.ortho, tt + orthoLead);
                // STEADY FRAMING (skins with no authored `_transform` beat, e.g. Mlynar "Fields of
                // Ruination"): measured against the game recording, the baked rig-centre curve swings
                // the character HORIZONTALLY into the right third (its X excursion reaches ~-355 off
                // the settle centre at t≈7) and its Y curve parks the reforming form low. The in-game
                // operator viewer instead holds the hero CENTRED the whole entrance and lets only the
                // ortho zoom widen the shot — pulling out to a near-FULL-BODY frame (feet + campfire)
                // by t≈12-13, then the post-flash idle settles back tighter. So we:
                //  - FOLLOW the rig's horizontal curve RAW (`cx = rigX`). Measured against the
                //    recording at nine beats spanning t=7→14, the game's horizontal framing tracks
                //    the baked curve exactly: converting each frame's best-fit shift back into mesh
                //    px reproduces `rigX` to within ~5 % over offsets from 441 px down to 39 px
                //    (t=7 predicts 441, measures 450; t=9.5 predicts 89, measures 85; t=13 predicts
                //    39, measures 40). Two earlier readings were wrong in opposite directions: the
                //    RE-BASED form `cx0 + (rigX − rigEndX)` added a constant ~+81 px (`cx0` sits
                //    that far from `rigEndX`) and threw the hero right, and the reaction to that —
                //    holding `cx0` — discarded a real authored pan. The raw curve is the one the
                //    game replays;
                //  - KEEP the rig's VERTICAL motion re-based onto the settle centre (`cy = cy0 +
                //    (rigY − rigEndY)`) — required, or the tight t=0 frame clips the reforming head
                //    (the rig lifts the frame to catch the pose, which sits higher than the idle rest).
                //    Y is NOT raw: raw `rigY` mispredicts by ~250 px where the re-based form is
                //    within ~20;
                //  - follow the ortho zoom-out to full body (NO size cap) — the wide→tight step at the
                //    idle handoff lands behind the white flash (both frames share the settle centre, so
                //    it is a pure scale change) + the 0.45s crossfade, matching the game.
                // All gamedata-derived, no magic constant. Skins WITH an authored transform beat (cello,
                // skadi2) keep the pure rig camera — `centerBlend` is null for them (verified parity).
                const cx = c[0];
                let cy = c[1];
                const cb = ef.centerBlend;
                if (cb) {
                    cy = cb.cy0 + (c[1] - cb.rigEndY);
                }
                liveDisplayBox = { x: cx - size / 2, y: cy - size / 2, width: size, height: size };
                ef.lastLiveCenter = [cx, cy];
                layoutSpine(ef.root, sw, sh, { x: cx - size / 2, y: cy - size / 2, width: size, height: size }, fitRef.current);
            }
            // ENTRANCE LAYER SEQUENCING. Runs for ANY live entrance composite, independent of
            // whether the skin ships a camera track — see `entranceSeqRef`.
            const eseq = entranceSeqRef.current;
            if (eseq && spineRef.current === eseq.spine) {
                const st = eseq.spine.state.tracks[0] as unknown as { trackTime?: number } | null;
                const tt = st?.trackTime ?? 0;
                // Per-layer `m_IsActive` window from the `_Start` clips (gamedata): a layer with
                // `activeFrom`/`activeUntil` renders only while `activeFrom <= t < activeUntil`.
                // Absent = always visible (Virtuosa's backdrop is entirely always-on).
                for (const c of eseq.sceneLayers) {
                    for (const m of c.children) {
                        const mm = m as unknown as ISceneLayerRuntime;
                        const af = mm.__activeFrom;
                        const au = mm.__activeUntil;
                        if (af != null || au != null) m.renderable = (af == null || tt >= af) && (au == null || tt < au);
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
                        // minimise away from 0 — and if it minimises AT 0, that is a clean kill.
                        if (mm.__colorCurve) applySceneLayerColor(m, sampleColorCurve(mm.__colorCurve, tt + matLead()));
                        if (mm.__stCurve) applySceneLayerSt(m, tt, sceneClock);
                    }
                }
                // Deferred entrance end: the spine's own "Start" animation ended before the
                // authored scene timeline, so its `complete` listener held off and the handoff
                // waits on the track clock instead — a hair EARLY so the dissolve starts on the
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
            const efd = entranceFadeRef.current;
            if (efd) {
                efd.elapsed += dt;
                let a = 0;
                if (efd.out !== null) {
                    efd.out += dt;
                    a = efd.out <= ENTRANCE_FADE_HOLD ? 1 : Math.max(0, 1 - (efd.out - ENTRANCE_FADE_HOLD) / ENTRANCE_FADE_OUT);
                    if (a <= 0) {
                        efd.sprite.parent?.removeChild(efd.sprite);
                        efd.sprite.destroy();
                        entranceFadeRef.current = null;
                    }
                } else {
                    // The ramp COMPLETES at `duration - HOLD`, not at `duration`: the capture is
                    // already pure white 0.2s before the authored end and holds there.
                    const end = efd.duration - ENTRANCE_FADE_HOLD;
                    a = Math.max(0, Math.min(1, (efd.elapsed - (end - ENTRANCE_FADE_IN)) / ENTRANCE_FADE_IN));
                }
                efd.sprite.alpha = a;
            }
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
                liveDisplayBox = b;
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
            if (!liveDisplayBox) {
                const activeComposite = compositesRef.current.find((c) => c.spine === spineRef.current);
                liveDisplayBox = activeComposite?.bounds ?? null;
            }
            // The particle off-screen cull must test what is actually VISIBLE, not the FRAMING
            // box. `layoutSpine` fits that box by its smaller axis, so the 2.16:1 viewport shows
            // ~2.16x its width — culling against the box itself throws away particles sitting
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
            // Set (below, once the scene loads) when the authored scene timeline outlasts
            // the spine's own "Start" animation: the `complete` listener then DEFERS the
            // handoff and the tick fires it at this track time instead (Mlynar's spine
            // anim ends 14.33s while the white-flash plane runs to the clip stop 15.97s —
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
            // GAP FILL (?gapfill=1): also load it for the ENTRANCE, where a scene that does
            // not span the camera view leaves bare canvas the game fills with vista.
            if (backdrop && (opts.mode === "main" || gapFillOn())) {
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
                    // Union of the character's own geometry bounds across its FULL played
                    // animation (idle loop, or the "Start" entrance clip) — used by particles.ts
                    // to detect a world-space background particle system whose static spawn disc
                    // sits on the character's own body (Mlynar's dominant rain system) so it can
                    // be un-occluded. A full-clip union (not a single-frame snapshot) is needed
                    // because the character's OWN pose/position moves substantially during the
                    // "Start" reform — a single frame would miss most of the overlap.
                    const characterBounds = measureAnimationBounds(spine, opts.mode === "entrance" ? entranceAnim : idle);
                    particles = await loadParticles(particlesUrl + bust, particlesTexBase, bust, characterBounds, scene.hasDarkBackdrop);
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
                    // ON by default; `?sepwash=0` disables. Data-gated — a skin with no
                    // `separatorSlots` is untouched (Mlynar ships none and is bit-identical).
                    const on = typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("sepwash") !== "0" : true;
                    if (on && particles && sepNames && sepNames.length > 0) {
                        const slots = (spine as unknown as { skeleton: { slots: { data: { name: string } }[] } }).skeleton.slots;
                        // Draw indices of the split slots, ASCENDING — gap k sits at the k-th
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
                // layer authored ABOVE every particle system must cover the particles too —
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
                        // `uColor=[0.121, 0.137, 0.294]`, `tint=None` — i.e. tint x alpha), and
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
                        // symptom to ONE layer — see the Ch'en silhouette hunt.
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
                                // ablation for exactly those layers — so a windowed layer always
                                // measured as "contributes 0 px" no matter what it drew, while
                                // window-less siblings ablated fine. Clear the window on the
                                // layers this token touches so the tick leaves them alone.
                                if (m[2] ? true : inRange) {
                                    const rt = c as unknown as ISceneLayerRuntime;
                                    rt.__activeFrom = undefined;
                                    rt.__activeUntil = undefined;
                                }
                            });
                        }
                        // `on:bg<i>` / `on:fg<i>` clears a layer's `m_IsActive` window so it
                        // stays drawn — tests whether missing content is a mis-timed window.
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
                        if (off.has("bgcount") && scene) console.log(`[abl] bgchildren=${scene.background.children.length}`);
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
                    (window as unknown as { __dumpSlots?: () => unknown }).__dumpSlots = () => {
                        const sk = (spine as unknown as { skeleton: { slots: unknown[] } }).skeleton;
                        return sk.slots.map((slotU) => {
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
                                    let b = (sl as unknown as { bone?: { parent?: unknown; data?: { name?: string } } }).bone as
                                        | { parent?: { parent?: unknown; data?: { name?: string } } | null; data?: { name?: string } }
                                        | undefined;
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
                        });
                    };
                    (window as unknown as { __dumpTex?: () => unknown }).__dumpTex = () => {
                        const cache = (PIXI.utils as unknown as { BaseTextureCache: Record<string, PIXI.BaseTexture> }).BaseTextureCache;
                        // `b.mipmap` is only the REQUESTED mode. PIXI silently falls back to no
                        // mipmap chain where GL cannot build one (an NPOT page on WebGL1), so
                        // also report what actually reached the GPU and the context version —
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
                                    // tint — its whole colour lives in the `uColor` uniform, so a
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
                                    // screen box, which does not survive the camera transform —
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
                // through to bare canvas — a hard-edged near-black wedge where the game shows
                // misty vista. Gated on the geometry, not on a skin: only when no layer spans
                // the view, so a scene that already fills the frame is untouched.
                const viewExt = 2 * (scene?.data.cameraSizePx ?? 0);
                const sceneCoversFrame =
                    !!scene &&
                    viewExt > 0 &&
                    scene.data.layers.some((l) => {
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
                        return x1 - x0 >= viewExt && y1 - y0 >= viewExt;
                    });
                const gapFill = !useStatic && gapFillOn() && !sceneCoversFrame && !!backdropData && !!backdropFrame;
                if ((useStatic || gapFill) && backdropData && backdropFrame) {
                    const bd = makeBackdropSprite(backdropData, backdropFrame, spineCentroid);
                    if (typeof window !== "undefined" && (new URLSearchParams(window.location.search).get("abl") || "").split(",").includes("backdrop")) bd.renderable = false;
                    if (gapFill) {
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
                // per-skin numbers. Skins with NO pull-out (Mlynar) hold the tight `_adjustes[1]`
                // stop as their steady frame; skins with a pull-out (Virtuosa/Skadi2) settle on the
                // wide `_adjustes[0]` stop — matching the game, whose steady camera frames each
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
                // game's side pillar-box slack — no stretch, no distortion. The box is CENTRED on the
                // character body — the raw `_adjustes` OFFSET points ~650px BELOW the body in export
                // space (verified), so it can't be used for centring; `VBIAS` corrects the residual
                // hair/mass drag.
                //
                // Re-swept by the same method as `VBIAS` after that constant was corrected (the
                // two interact: `cy0 = bodyCy - VBIAS*e` with `e = viewPx*RCAL`). Mlynar is again
                // the only surface that moves — Virtuosa and Skadi2 are BIT-IDENTICAL at every
                // value tried, since they keep the pure rig camera:
                //
                //     rcal   0.74    0.76    0.772   0.776   0.780   0.781   0.782   0.784   0.80
                //     MADC   29.697  25.814  22.085  20.534  19.352  19.278  19.364  19.820  24.999
                //
                // 0.78 was already right to ~0.1%; 0.781 is the reproducible optimum and is worth
                // only 0.074 MADC — kept because it IS the measured minimum, but do not read it as
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
                // residual, not noise — it held at +1.1 px on EVERY scored beat and cost ~16% of
                // his interior luma error. Re-swept at finer resolution against the current
                // pipeline (frames decoded and INDEXED, never `ffmpeg -ss`, which is a frame late):
                //
                //     vbias    0.1805   0.1810   0.1815   0.1820   0.1825   0.184
                //     MADC     20.649   19.863   19.352   19.387   19.902   22.226
                //
                // 0.1815 is the optimum and drops the residual shift +1.08 px → +0.18 px
                // (per-beat +1.2,+1.3,+1.1,+1.1,+1.1,+0.7 → +0.2,+0.2,+0.2,+0.2,+0.2,+0.1).
                // Virtuosa and Skadi2 are BIT-IDENTICAL across the change (29.904 / 18.671 at both
                // values) — they carry an authored transform beat, so they keep the pure rig camera
                // and never enter the `centerBlend` branch this constant feeds.
                const VBIAS = calibrationParam("vbias", 0.1815);
                const bodyCx = vb?.centroid ? vb.centroid.cx : vb ? vb.x + vb.width / 2 : frameCx;
                const bodyCy = (headY + feetY) / 2; // vertical body centre
                /** A body-centred square crop of authored extent `viewPx`, in render/vis space. */
                const bodyFrameBox = (viewPx: number): IAnimationBounds => {
                    const e = viewPx * RCAL;
                    return { x: bodyCx - e / 2, y: bodyCy - VBIAS * e - e / 2, width: e, height: e };
                };
                /** An authored `_adjustes` extent is only usable if it is a POSITIVE FINITE
                 *  number. Unity writes an uninitialised stop as `-FLT_MAX`
                 *  (-3.4028235e38), which the exporter carries through verbatim — and a
                 *  plain truthiness test accepts it, because it is non-zero. `bodyFrameBox`
                 *  then builds a crop 3e38 px wide, `tight` prefers it over the sane
                 *  `cameraSizePx` fallback, and the skin renders as an EMPTY frame.
                 *  Nearl the Radiant Knight "Epoque" is the one such skin in the corpus
                 *  (both of her stops are -FLT_MAX); she rendered blank. Property-driven —
                 *  a no-op for every skin whose stops are real numbers. */
                const usableExtent = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v) && v > 0;
                const authoredDisplayBounds: IAnimationBounds | null = usableExtent(authoredFrame?.viewPx) && vis ? bodyFrameBox(authoredFrame.viewPx as number) : null;
                // The TIGHT open endpoint: the exact `cameraViewPx2` box (2nd `_adjustes` stop),
                // same centre — the viewer dollies OUT from here. Null unless a 2nd camera stop.
                const authoredTightBounds: IAnimationBounds | null = usableExtent(authoredFrame?.viewPx2) && vis ? bodyFrameBox(authoredFrame.viewPx2 as number) : null;
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
                // Setup-pose CENTRE of every attachment, keyed by attachment name. A
                // `BoneFollower` rig that CONTINUES a spine prop (Virtuosa's falling apple:
                // bone `L_C_Apple_F` carries the attachment of the same name, which the
                // entrance fades out exactly as the particle copy spawns) must hand off from
                // where the ART is, not from the bone ORIGIN — the authored `followOffset` is
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
                // from frame 0 — for Virtuosa that's 598 px (ortho 2.99) vs the true animated 374 px
                // (ortho 1.87), framing the whole entrance 1.6× too wide. Measured against the game
                // recording, the animated view matches the hold beats to <1% (the game's seated shot
                // is a tight head-and-shoulders close-up). Fall back to the static `entranceViewPx`
                // (then the tight `_adjustes[1]` stop) only when no ortho curve/scale shipped.
                const entranceOrtho0 = scene?.data.entranceOrthoCurve?.[0]?.[1];
                const entranceSkelScale = scene?.data.skeletonScale;
                const entranceFrameSize = entranceOrtho0 && entranceSkelScale ? (2 * entranceOrtho0) / entranceSkelScale : ((authoredFrame?.entranceViewPx as number | undefined) ?? (authoredFrame?.viewPx2 as number | undefined) ?? null);
                // Authored SCENE-timeline end: when the entrance→idle handoff (pose swap +
                // pull-out dolly) fires, relative to the entrance track clock.
                //
                // Skins with an authored `entranceTransform` beat (the exported reform/
                // pose-transition timestamp, e.g. cello 12.0, skadi2 9.5) use it as the base
                // timing candidate INSTEAD OF the raw camera-curve tail: the ortho/cam-center
                // curves keep being keyed almost to the end of the cinematic (baked idle-hold/
                // sway data past the real transition), so their literal last keyframe over-
                // defers the handoff by several seconds (cello: curve tail 19.73 vs the game's
                // actual transition at 12.0 — she's stuck in her closed-eyes "reform" pose and
                // the pull-out dolly gets crushed into the final second). Layer-derived signals
                // (activeUntil windows, colour-curve reveals still visible at their own end) are
                // ALWAYS folded in via max() regardless of transform presence — they're genuine
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
                        // fade-to-0 layers ran to 23.67s — the opaque white then held through the handoff
                        // seam and BLANKED the frame. Mlynar/cello curves all end visible → unaffected.)
                        if (cc?.length && cc[cc.length - 1][4] > 0.02) end = Math.max(end, cc[cc.length - 1][0]);
                    }
                    if (xform != null) {
                        // The authored entrance TOTAL duration is a genuine content-completion
                        // beat and must floor the handoff: `entranceTransform` marks the pose/
                        // expression reform (cello 12.0), but the game holds the tight portrait
                        // through the whole entrance and snaps wide only at `entranceDuration`
                        // (cello 18.0 — matching her voice line to ~18.3s). Firing at the raw
                        // transform (12.0) snaps ~6s early and cuts the line. Fold it into the
                        // max as a floor. No-op where a later term already dominates: Skadi2's
                        // visible colour curve ends at 22.43, past its own entranceDuration
                        // (22.33), so its end is unchanged. (Mlynar has no transform → the else
                        // branch below, untouched.)
                        end = Math.max(end, scene.data.entranceDuration ?? 0);
                        // Data-derived beat — fire the tick-driven handoff at this track time
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
                        // Muelsyse (duration 20.0) at 20.35. Muelsyse is the skin this fixes —
                        // her `end` runs to 20.93, deferred +0.93s by an ordinary sort-1 layer
                        // sitting at alpha 0.69, and we held pure white until 21.08 where the game
                        // was live at 20.35. Every other benchmark already lands within 0.1s of
                        // `duration` (Mlynar 16.50, Virtuosa 18.00, Skadi 2 22.43 → 22.33), so
                        // this is a no-op for them.
                        deferEndUntil = Math.min(end, scene.data.entranceDuration ?? end);
                    } else {
                        const entAnimDur = spine.spineData.animations.find((a: { name: string }) => a.name === entranceAnim)?.duration ?? 0;
                        if (end > entAnimDur + 0.05) deferEndUntil = end;
                    }
                }
                // THE DIRECTOR'S `duration` IS THE AUTHORED END, whatever the spine does — and
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
                // the animation actually outlasts the duration — otherwise `complete` already
                // lands at or before it and this is a no-op.
                if (opts.mode === "entrance" && scene?.data.entranceDuration != null) {
                    const dur = scene.data.entranceDuration;
                    const animDur = spine.spineData.animations.find((a: { name: string }) => a.name === entranceAnim)?.duration ?? 0;
                    if (animDur > dur + 0.05) deferEndUntil = Math.min(deferEndUntil ?? dur, dur);
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
                    entranceViewRatio: authoredFrame?.viewPx2 && authoredFrame?.viewPx ? (authoredFrame.viewPx2 as number) / authoredFrame.viewPx : null,
                    entranceDuration: scene?.data.entranceDuration ?? null,
                    entranceFade: sceneDrivesEntranceFade(scene?.data ?? null) ? null : (scene?.data.entranceFade ?? null),
                    entranceTransform: scene?.data.entranceTransform ?? null,
                    entranceOrthoCurve: (scene?.data.entranceOrthoCurve as [number, number][] | undefined) ?? null,
                    entranceCamCenterCurve: (scene?.data.entranceCamCenterCurve as [number, number, number][] | undefined) ?? null,
                    entranceFrameSize,
                    sceneLayers: opts.mode === "entrance" && scene ? [scene.background, scene.foreground, ...(sceneOverlay ? [sceneOverlay] : [])] : null,
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
                entranceViewRatio: null,
                entranceDuration: null,
                entranceFade: null,
                entranceTransform: null,
                entranceOrthoCurve: null,
                entranceCamCenterCurve: null,
                entranceFrameSize: null,
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
                }

                const { width, height } = app.screen;
                // The viewer's own backdrop at the BACK of the stage (see createEnvironmentBgTexture):
                // the skin's camera clears to alpha 0, so this is what the game shows wherever the art
                // doesn't reach. Added here (after the scene has loaded) so it appears WITH the
                // illustration rather than covering the static load placeholder, and behind every
                // scene layer so they composite over it as they do in game. ALWAYS created — even a
                // scene that owns its own dark painted backdrop has real coverage gaps at its widest
                // camera framing (the opening pan, the post-handoff wide settle) where this is the
                // ONLY fill; omitting it reopens black voids there. Only the fill's COLOUR is gated,
                // on the data-derived `hasDarkBackdrop` (cello-only by construction).
                const envBg = new PIXI.Sprite(createEnvironmentBgTexture(main.hasDarkBackdrop));
                // DIAGNOSTIC (`?fill=RRGGBB`): repaint the viewer fill so the spine's EFFECTIVE
                // transparency can be measured — the mean shifts by (fillDelta x uncovered area).
                if (typeof window !== "undefined") {
                    const f = new URLSearchParams(window.location.search).get("fill");
                    if (f) envBg.tint = parseInt(f, 16);
                }
                resizeEnvironmentBg(envBg, width, height);
                envBgRef.current = envBg;
                app.stage.addChildAt(envBg, 0);
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
                // of the frame — the stage it was measured on. Derived per scene from the
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
                            entranceFadeRef.current = { sprite: sp, elapsed: 0, duration: c.entranceDuration, out: null };
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
                // unusable — Unity writes an uninitialised stop as -FLT_MAX, which `usableExtent`
                // (above) correctly rejects. That guard was necessary but not sufficient: EVERY
                // path below runs through `openStandingIdle`, which bailed on a null `gameFrame`
                // BEFORE calling `attach()`, so the composite root was never added to the stage
                // and the skin rendered as a completely EMPTY frame — a healthy skeleton with 205
                // renderable meshes and valid textures, simply never drawn. The composite has
                // already computed a sane framing box for exactly this case (the authored
                // `cameraSizePx` square centred on the visible art), so fall back to it. Same
                // `?? bounds` chain the entrance paths below already use; a no-op for every skin
                // whose `_adjustes[0]` is a real number.
                const gameFrame = main.authoredDisplayBounds ?? main.bounds;
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
                const openStandingIdle = (opts?: { fromEntrance?: boolean; handoffPanDelta?: [number, number] | null }) => {
                    if (aborted() || !appRef.current || !gameFrame) return;
                    const { width: sw, height: sh } = appRef.current.screen;
                    // An explicit `?framebox=` wins over every other framing, and over any
                    // dolly — the point is a camera that CANNOT move between renders.
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
                    // recording — the steady settle is a knees-up shot, not a full-body pull-out).
                    // A pull-out is only performed when the data authorizes one (`entrancePullOut`,
                    // cello) or when there was no cinematic at all (the standard archive open).
                    if (opts?.fromEntrance && !entrancePullOut && openTight) {
                        main.bounds = openTight; // resizes keep the held tight frame
                        layoutSpine(main.root, sw, sh, openTight, fitRef.current);
                        boundsRef.current = openTight;
                        return;
                    }
                    // Plain archive open (no `_Start` cinematic, so not an entrance hand-off) with no
                    // authored pull-out: the skin has NO camera move at all, so it must open STATIC at
                    // the settled game frame. The former fallback dollied `openTight` (`_adjustes[1]`)
                    // → `gameFrame` (`_adjustes[0]`) over an invented 2s, which read as an unwanted pan
                    // on skins that never had an entrance. A dolly is only performed for a genuine
                    // entrance hand-off (`fromEntrance`) or a data-authored pull-out (`entrancePullOut`).
                    if (!opts?.fromEntrance && !entrancePullOut) {
                        // A skin with NO `_Start` has no authored shot to hold — the game's viewer
                        // shows the whole cut-out with margins, and the `_adjustes[0]` box crops
                        // well inside it (Ch'en the Holungday: 2704x1804 of content against a
                        // 1500 px view). Frame the drawn CONTENT instead. 70 of the 82 dynchar
                        // skins take this path; the 12 with an entrance are untouched, which is
                        // why the three measured reference skins cannot move.
                        const whole = fitWholeArt() ? main.contentBounds : null;
                        const settle = whole ?? gameFrame;
                        // The authored framing fits by HEIGHT, which crops the width — fine for a
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
                    // around `gameFrame`'s OWN static centre — it has no notion of the entrance rig
                    // camera's actual PAN over the shot. Re-base the box by the live camera's real
                    // pan delta (hand-off centre minus its own t=0 centre, curve-space units added
                    // directly onto `gameFrame`'s centre — the same cross-space-additive convention
                    // `centerBlend` already uses in the tick above), so the dolly starts exactly
                    // where the entrance camera actually ended instead of snapping to an
                    // independently-recomputed box. Zero pan (or no captured delta — no `_Start`,
                    // or the no-pull-out branch above) leaves this byte-identical to before.
                    // SANITY BOUND: this cross-space add is only meaningful for a genuine "same shot,
                    // slightly drifted" case (Skadi2: ~370-unit lateral drift). A skin whose rig camera
                    // travels THROUGH the scene in depth (Virtuosa's shaft plunge: ~3400 authored units,
                    // ~9× her own entrance frame size) isn't drifting off a shared reference — applying
                    // the raw delta there blows the box off-frame entirely (verified: produced a
                    // black/void hand-off). Gate on the delta being smaller than the target frame's own
                    // extent — a data-derived bound (not a per-skin constant), true for every drift-style
                    // pan and false for every plunge-style one measured so far.
                    if (openFrom && opts?.handoffPanDelta) {
                        const [dx, dy] = opts.handoffPanDelta;
                        const maxDelta = Math.max(gameFrame.width, gameFrame.height);
                        if (Math.abs(dx) < maxDelta && Math.abs(dy) < maxDelta) {
                            openFrom = { ...openFrom, x: openFrom.x + dx, y: openFrom.y + dy };
                        }
                    }
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
                    // Capture the entrance camera's actual pan (its live centre at hand-off minus
                    // its own t=0 centre) BEFORE `entranceFollowRef` is nulled below — this is what
                    // lets `openStandingIdle` start the post-handoff dolly continuously from where
                    // the eye actually was, instead of an independently-recomputed box (see the Sf
                    // handoff-continuity fix there).
                    const ef = entranceFollowRef.current;
                    const handoffPanDelta: [number, number] | null = ef?.lastLiveCenter ? [ef.lastLiveCenter[0] - ef.startCenter[0], ef.lastLiveCenter[1] - ef.startCenter[1]] : null;
                    entranceZoomRef.current = null;
                    entranceFollowRef.current = null;
                    entranceSeqRef.current = null;
                    // The fade is at (or near) full here — hold it briefly so the swap happens
                    // UNDER it, then lift, which is what the recordings show.
                    if (entranceFadeRef.current) entranceFadeRef.current.out = 0;
                    activate(main, { skipStart: true }); // straight to standing idle (no second intro beat)
                    openStandingIdle({ fromEntrance: true, handoffPanDelta });
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
                    // `entrancePullOut` → `openStandingIdle`). Measured against the reference
                    // recording, the entrance holds the `_Start` camera's OWN close-up — the ANIMATED
                    // t=0 ortho view (`entranceFrameSize`, Virtuosa 374px: a tight head-and-shoulders
                    // shot) — TIGHTER than both the `_adjustes[1]` stop (1246px, the settled idle's
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
                    const tight = built.authoredTightBounds ?? built.authoredDisplayBounds ?? built.bounds;
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
                        };
                        // Drive the entrance camera PURELY from gamedata: the exporter-accumulated camera
                        // rig track (`entranceCamCenterCurve`, absolute mesh-px frame centre) for the
                        // pan/dolly, and the `_adjustes[1]` view extent (`entranceFrameSize`) × the ortho
                        // zoom for the frame size. No measured bounds, no per-skin tuning. Only when both
                        // gamedata inputs exist — otherwise the static `tight` framing already laid out
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
                                camCuts: detectCurveCuts(built.entranceCamCenterCurve, built.entranceFrameSize),
                                frameSize: built.entranceFrameSize,
                                centerBlend,
                                startCenter: sampleCurveXY(built.entranceCamCenterCurve, 0) ?? [0, 0],
                                lastLiveCenter: null,
                                sceneLayers: sl,
                                endAt: built.entranceSceneEnd,
                                fireEnd: built.entranceSceneEnd != null ? built.requestEntranceEnd : null,
                            };
                        }
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
