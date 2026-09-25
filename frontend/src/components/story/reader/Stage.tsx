/**
 * THE FRAME THE SCENE IS DRAWN INTO: the 16:9 canvas box, what fills the stage
 * outside it, and the camera shake over the whole thing.
 *
 * The stage spans the viewport edge to edge; the CANVAS inside it is the
 * largest 16:9 box, centred, which is what the client's `fit_mode="BLACK_MASK"`
 * makes (`docs/story-reader-captures.md`, 0). What goes outside that box is our
 * named trade: the background redrawn blurred by default, the client's black
 * under `?mask=1` and the "Letterbox like the game" setting.
 *
 * What is drawn INSIDE lives beside this file: `StageLayers.tsx` for the scene,
 * `StageSprites.tsx` for the characters, `stageFx.tsx` for the four effects
 * both use.
 */
import type React from "react";
import { useEffect, useRef } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { CANVAS_H, CANVAS_W, CPX_BOX, CPX_STRETCH, cpx } from "#/lib/story/canvas";
import type { CameraShake } from "#/lib/story/engine";
import type { FocusOut, Frame, Slot, SlotState } from "#/lib/story/scene";
import { cn } from "#/lib/utils";
import { CurtainFill, CutinPlate, InterludePanels, Layer, OverlayText, Panels } from "./StageLayers";
import { Sprite } from "./StageSprites";
import { fade, joinFilters } from "./stageFx";

/**
 * How the stage's canvas is mapped. `box` is the client: the canvas is the
 * largest 16:9 rectangle that fits the stage, centred, and one canvas pixel is
 * its width over 1280 (`docs/story-reader-captures.md`, 0). `stretch` is the
 * kill switch `?canvas=stretch`, the pre-capture mapping where the canvas width
 * was the whole stage width.
 */
export type CanvasMode = "box" | "stretch";

/**
 * What fills the stage OUTSIDE the canvas box. `mask` is the client's own
 * black (`?mask=1`, and the "Letterbox like the game" setting). `extend` is
 * OUR trade, the default: the current background is redrawn cover-scaled across
 * the whole stage, blurred and darkened, so a wide screen sees a continuation
 * of the scene instead of two bars. On a 16:9 stage the box is the stage and
 * the two are identical.
 */
export type StageFill = "extend" | "mask";

/** The extend fill's blur radius and brightness. 24 px at 45% of the plate's own light. */
const EXTEND_FILTER = "blur(24px) brightness(0.45)";

export interface IStageProps {
    frame: Frame | null;
    /** The box the reader gives the stage. The CANVAS is the largest 16:9 rectangle inside it. Defaults to a 16:9 box. */
    className?: string;
    /** The live `[camerashake]`, with the sequence number that restarts it. */
    shake?: { seq: number; params: CameraShake } | null;
    /** `?canvas=stretch`: the pre-capture mapping, canvas width = stage width. */
    canvasMode?: CanvasMode;
    /** `?mask=1` and the "Letterbox like the game" setting: black outside the box, as the client draws it. */
    fill?: StageFill;
    /** `?plate=0`: ignore `CharacterSprite.plate` and draw every body at the 1024-at-203 slot template. */
    plateFromWire?: boolean;
    children?: React.ReactNode;
    onClick?: (e: React.MouseEvent) => void;
    onContextMenu?: (e: React.MouseEvent) => void;
    label: string;
}

/**
 * The stage OUTSIDE the canvas box, under our `extend` trade: the background
 * the scene is showing, cover-scaled across the whole stage, blurred and
 * darkened so it reads as a continuation rather than a second picture. The
 * client fills this region with pure black and `?mask=1` does the same.
 *
 * The 4% overscale is not decoration: a 24 px blur samples past the plate's own
 * edges and would fade to transparent against the black stage without it.
 */
function ExtendFill({ url }: { url: string }): React.ReactElement {
    return <img key={url} src={asset(url)} alt="" aria-hidden="true" draggable={false} data-story-extend-fill className="pointer-events-none absolute inset-0 size-full select-none object-cover" style={{ filter: EXTEND_FILTER, scale: "1.04", ...fade("in", 0.4) }} />;
}

/**
 * `DOShakePosition`, driven from JS so it can last the command's own duration
 * (ten seconds by default) instead of a fixed keyframe, and so it never gates
 * the reader: the command returns non-blocking and the scene keeps advancing
 * underneath the shake. Strengths are canvas pixels, so they scale with the
 * stage the same way every other offset does.
 */
function useCameraShake(ref: React.RefObject<HTMLDivElement | null>, shake: { seq: number; params: CameraShake } | null | undefined): void {
    useEffect(() => {
        const el = ref.current;
        if (!el) return;
        const p = shake?.params;
        if (!p || p.stop || p.duration <= 0) {
            el.style.translate = "";
            return;
        }
        const scale = el.clientWidth / CANVAS_W || 1;
        const start = performance.now();
        // A deterministic phase per shake, so the same command shakes the same
        // way twice; `randomness` is DOTween's angle variance in degrees.
        const jitter = ((shake?.seq ?? 0) * 137.5) % 360;
        const phaseX = ((jitter * p.randomness) / 360) * (Math.PI / 180);
        const phaseY = phaseX + Math.PI / 3;
        let raf = 0;
        const tick = (now: number) => {
            const t = (now - start) / 1000;
            if (t >= p.duration) {
                el.style.translate = "";
                return;
            }
            const decay = p.fadeOut ? 1 - t / p.duration : 1;
            const turn = (t / p.duration) * p.vibrato * Math.PI * 2;
            const x = Math.sin(turn + phaseX) * p.xStrength * decay * scale;
            const y = Math.sin(turn * 1.13 + phaseY) * p.yStrength * decay * scale;
            el.style.translate = `${x.toFixed(2)}px ${y.toFixed(2)}px`;
            raf = requestAnimationFrame(tick);
        };
        raf = requestAnimationFrame(tick);
        return () => {
            cancelAnimationFrame(raf);
            el.style.translate = "";
        };
    }, [ref, shake]);
}

/**
 * The stage, and the CANVAS inside it.
 *
 * `fit_mode="BLACK_MASK"` is on every story in the EN corpus, and it is not an
 * overlay on a full-width canvas, it IS the canvas
 * (`docs/story-reader-captures.md`, 0): on the captured 2340x1080 screen the
 * non-black columns are exactly 210 through 2129, a centred 1920x1080 box, and
 * the canvas is 1280x720 across it at 1.5 device px per canvas px. So the box
 * below is the largest 16:9 rectangle that fits the stage, centred, everything
 * the script places lives inside it, and one canvas pixel is that box's width
 * over 1280 rather than the stage's.
 *
 * What the stage does with the region OUTSIDE the box is the one place this
 * reader knowingly departs from the client, and the trade is named: the client
 * paints it black, `fill="extend"` redraws the background across it blurred and
 * darkened so a wide screen sees a continuation, and `fill="mask"` (`?mask=1`,
 * or the "Letterbox like the game" setting) gives the client's black back. On a
 * 16:9 stage the box is the stage and the two are the same picture.
 *
 * The reader's own chrome, the toolbar and the text box, is `children` and
 * stays OUTSIDE the box against the stage, which is what keeps the text box's
 * stage-relative margins where they shipped.
 */
export function Stage({ frame, shake, children, onClick, onContextMenu, label, className, canvasMode = "box", fill = "extend", plateFromWire = true }: IStageProps): React.ReactElement {
    const state = frame?.state;
    const sec = frame?.transitionSec ?? 0;
    const blocker = state?.blocker ?? { a: 0, r: 0, g: 0, b: 0 };
    const ease = frame?.ease;
    const focus: FocusOut = state?.focus ?? {};
    const grayscale = state?.effects.grayscale;
    const invert = state?.effects.colorInverse;
    const cameraFilter = joinFilters(grayscale ? `grayscale(${grayscale})` : undefined, invert ? `invert(${invert})` : undefined);
    const sceneRef = useRef<HTMLDivElement>(null);
    useCameraShake(sceneRef, shake);
    // The focused slot is raised to the top of its sibling order
    // (`SetAsLastSibling`), so a lit speaker draws in FRONT of a dimmed one.
    const slots = state ? (Object.entries(state.slots) as Array<[Slot, SlotState]>).sort((a, b) => Number(a[1].lit) - Number(b[1].lit)) : [];
    const stretch = canvasMode === "stretch";
    // The extend fill follows whatever the scene has behind it: a `largebg`
    // strip's first panel, else the background layer. With neither there is
    // nothing to continue and the stage stays black.
    const fillUrl = state?.panels?.urls[0] ?? state?.background?.url;
    return (
        // biome-ignore lint/a11y/useKeyWithClickEvents: the stage is a canvas-like surface; keyboard advance is the reader's window hotkeys (Space, Enter, ArrowRight).
        <section
            aria-label={label}
            onClick={onClick}
            onContextMenu={onContextMenu}
            className={cn("relative w-full select-none overflow-hidden bg-black outline-none [container-type:size]", className ?? "aspect-video")}
            style={{ "--story-cpx": stretch ? CPX_STRETCH : CPX_BOX } as React.CSSProperties}
            data-story-stage
            data-story-canvas-mode={canvasMode}
            data-story-fill={fill}
        >
            {fill === "extend" && fillUrl && !stretch ? <ExtendFill url={fillUrl} /> : null}
            <div
                data-story-canvas
                className="absolute overflow-hidden"
                // `calc(1280 * cpx)` and `calc(720 * cpx)` ARE the largest 16:9
                // box, because `cpx` is `min(width/1280, height*16/9/1280)`;
                // under `?canvas=stretch` the box is the whole stage again.
                style={{ left: "50%", top: "50%", translate: "-50% -50%", width: stretch ? "100%" : cpx(CANVAS_W), height: stretch ? "100%" : cpx(CANVAS_H) }}
            >
                <div ref={sceneRef} className="absolute inset-0 transition-[filter]" style={{ filter: cameraFilter, transitionDuration: `${sec}s` }}>
                    {state?.panels ? <Panels panels={state.panels} sec={sec} focus={focus.bg} /> : <Layer layer={state?.background} sec={sec} kind="background" focus={focus.bg} ease={ease} />}
                    {slots.map(([slot, s]) => (
                        <Sprite key={slot} slot={slot} state={s} sec={sec} focus={focus.char} plateFromWire={plateFromWire} />
                    ))}
                    <Layer layer={state?.image} sec={sec} kind="image" focus={focus.cg} ease={ease} />
                    {state ? <InterludePanels panels={state.interludes} sec={sec} /> : null}
                    {state?.cutin ? <CutinPlate cutin={state.cutin} sec={sec} plateFromWire={plateFromWire} /> : null}
                </div>
                <div data-story-blocker className="pointer-events-none absolute inset-0 transition-[background-color] ease-linear" style={{ backgroundColor: `rgba(${Math.round(blocker.r * 255)}, ${Math.round(blocker.g * 255)}, ${Math.round(blocker.b * 255)}, ${blocker.a})`, transitionDuration: `${sec}s` }} />
                {state?.curtain ? <CurtainFill curtain={state.curtain} sec={sec} /> : null}
                {state?.popupHead ? (
                    <img
                        key={state.popupHead.url}
                        src={asset(state.popupHead.url)}
                        alt=""
                        draggable={false}
                        data-story-popuphead={state.popupHead.key}
                        className="absolute size-[18%] select-none rounded-full object-cover ring-2 ring-white/70"
                        style={{ left: state.popupHead.x === undefined ? "6%" : cpx(state.popupHead.x), top: state.popupHead.y === undefined ? "62%" : cpx(state.popupHead.y), ...fade("in", 0.25) }}
                    />
                ) : null}
                {state?.subtitle ? <OverlayText overlay={state.subtitle} kind="subtitle" /> : null}
                {state ? Object.entries(state.stickers).map(([id, o]) => <OverlayText key={id} overlay={o} kind="sticker" />) : null}
            </div>
            {children}
        </section>
    );
}
