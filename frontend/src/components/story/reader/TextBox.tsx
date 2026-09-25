import { ChevronDownIcon, GripHorizontalIcon } from "lucide-react";
import type React from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { BOX_MESSAGE_W, cpx } from "#/lib/story/canvas";
import { BOX_NUDGE_X, BOX_NUDGE_Y, type BoxPosition, boxPositionFromDrag, clampBoxPosition, MAX_BOX_LIFT, type StorySettings } from "#/lib/story/settings";
import { parseStoryText, plainStoryText, type TextNode } from "#/lib/story/text";
import { cn } from "#/lib/utils";
import { resolveTextColor } from "./colors";
import { fontFamilyFor, PRESET_FONT_FAMILY } from "./fonts";
import { lineColorFor } from "./speaker";

/**
 * Speaker plate hues, a curated set that reads on the dark box. The same name
 * always lands on the same hue (djb2 over the name), so a speaker keeps their
 * colour across stories.
 */
const SPEAKER_HUES = ["#e0a458", "#6fb3e0", "#e07a8c", "#6fcf97", "#b48ee0", "#f08a5d", "#5cc8c0"];

/**
 * The same hues darkened for the light reading surface. Each dark hue reads at
 * 6.912:1 or better on the dark plate (#0a0a0a), 6.318:1 with the plate's 5% bleed
 * from a white scene, and each light one at 6.970:1 or better on the light plate
 * (#fbf9f4). Nothing falls under 4.5:1, so no hue was replaced.
 */
const SPEAKER_HUES_LIGHT = ["#7a4c05", "#0d4f78", "#9c1f37", "#0a5c34", "#5a2e8a", "#8e3305", "#0a5a55"];

export function speakerColor(name: string, light = false): string {
    let h = 5381;
    for (let i = 0; i < name.length; i++) h = ((h << 5) + h + name.charCodeAt(i)) | 0;
    const i = Math.abs(h) % SPEAKER_HUES.length;
    return light ? SPEAKER_HUES_LIGHT[i] : SPEAKER_HUES[i];
}

/** Render nodes with a character budget: the typewriter reveals `limit` characters. */
function renderNodes(nodes: TextNode[], budget: { left: number }, keyPrefix: string): React.ReactNode[] {
    const out: React.ReactNode[] = [];
    for (let i = 0; i < nodes.length; i++) {
        if (budget.left <= 0) break;
        const n = nodes[i];
        const key = `${keyPrefix}${i}`;
        if (n.kind === "text") {
            const shown = n.value.slice(0, budget.left);
            budget.left -= shown.length;
            out.push(shown);
        } else if (n.kind === "inline") {
            const children = renderNodes(n.children, budget, `${key}.`);
            out.push(n.tag === "i" ? <em key={key}>{children}</em> : n.tag === "b" ? <strong key={key}>{children}</strong> : <u key={key}>{children}</u>);
        } else if (n.kind === "color") {
            out.push(
                <span key={key} style={{ color: n.color }}>
                    {renderNodes(n.children, budget, `${key}.`)}
                </span>,
            );
        } else {
            out.push(
                <span key={key} className="block">
                    {renderNodes(n.children, budget, `${key}.`)}
                </span>,
            );
        }
    }
    return out;
}

/** The same markup rendering with no reveal budget: every character, for a surface that is not typed out. */
export function renderStoryNodes(nodes: TextNode[]): React.ReactNode[] {
    return renderNodes(nodes, { left: Number.POSITIVE_INFINITY }, "n");
}

export interface ITextBoxProps {
    speaker?: string;
    text: string;
    isNarration: boolean;
    /** Restarts the reveal when it changes. */
    revealKey: number;
    /** When false the reveal waits (the scene is still transitioning). */
    armed: boolean;
    settings: StorySettings;
    /** Called with the `revealKey` whose line is now fully revealed. */
    onRevealDone: (revealKey: number) => void;
    /** Set by the reader on a click during the reveal: jump to the end. */
    completeSignal: number;
    continueLabel: string;
    hidden: boolean;
    /** True once the uploaded face is registered; until then a custom choice falls back to the preset. */
    customFontLoaded: boolean;
    /**
     * The speaker's own ink, sampled from the LIT sprite on stage and already
     * conditioned for this surface (`speaker.ts`, `useSpeakerTint`). Undefined
     * is the default and the kill switch: the djb2 hue below stands.
     */
    speakerTint?: string;
    /** Dragging the box's frame writes BOTH axes back to settings. */
    onPositionChange: (next: BoxPosition) => void;
    dragLabel: string;
}

/**
 * The reveal progress, stamped with the line it belongs to.
 *
 * Stamped because a bare `shown` counter is STALE for one render every time
 * the line changes: `text` (and so `total`) is new while `shown` is still the
 * previous line's, and the effect that notifies completion ran against that
 * pair. Where the previous line was the longer one that read as "already
 * complete", so the notification fired for a line that had not started, and
 * the ref that guards it was spent; the real completion then never notified,
 * `revealDone` stayed false in the reader, and every click only re-sent the
 * complete signal. That was the reader STALLING at halt 6 of the tutorial.
 * Deriving `shown` from the stamp removes the stale render entirely.
 */
interface Reveal {
    key: number;
    shown: number;
}

export function TextBox({ speaker, text, isNarration, revealKey, armed, settings, onRevealDone, completeSignal, continueLabel, hidden, customFontLoaded, speakerTint, onPositionChange, dragLabel }: ITextBoxProps): React.ReactElement {
    const nodes = useMemo(() => parseStoryText(text), [text]);
    const total = useMemo(() => plainStoryText(nodes).length, [nodes]);
    const [reveal, setReveal] = useState<Reveal>({ key: revealKey, shown: 0 });
    // A reveal stamped with another line is that line's progress, so this one is at zero.
    const shown = reveal.key === revealKey ? Math.min(reveal.shown, total) : 0;
    const complete = shown >= total;

    useEffect(() => {
        if (!armed || complete) return;
        const perChar = 1000 / settings.cps;
        let last = performance.now();
        let acc = 0;
        let raf = 0;
        const tick = (now: number) => {
            acc += now - last;
            last = now;
            const chars = Math.floor(acc / perChar);
            if (chars > 0) {
                acc -= chars * perChar;
                setReveal((r) => ({ key: revealKey, shown: Math.min(total, (r.key === revealKey ? r.shown : 0) + chars) }));
            }
            raf = requestAnimationFrame(tick);
        };
        raf = requestAnimationFrame(tick);
        return () => cancelAnimationFrame(raf);
    }, [armed, complete, revealKey, total, settings.cps]);

    // Notifying on every render while complete is deliberate: the reader stores
    // the key, so a repeat is a no-op and no ordering between the two components
    // can lose the notification.
    useEffect(() => {
        if (complete) onRevealDone(revealKey);
    }, [complete, revealKey, onRevealDone]);

    // biome-ignore lint/correctness/useExhaustiveDependencies: the signal is the trigger; its value is irrelevant.
    useEffect(() => {
        if (completeSignal > 0) setReveal({ key: revealKey, shown: total });
    }, [completeSignal]);

    const budget = { left: shown };
    const fontSize = `${settings.textSize / 100}rem`;
    const light = settings.lightBox;
    // The sampled ink when the reader asked for it and a sprite answered for
    // this name, the hashed hue otherwise. Both are already conditioned for the
    // surface the box is on, so the plate reads either way.
    const plateColor = speaker ? (speakerTint ?? speakerColor(speaker, light)) : "";
    // Every one of these four is a SETTING now, written together by a reading
    // style and editable one at a time afterwards, so there is no second
    // rendering path a preset can disagree with.
    const fontFamily = fontFamilyFor(settings.font, PRESET_FONT_FAMILY, customFontLoaded);
    // The speaker plate keeps its own hue: the colour setting is the reader's
    // choice for the PROSE, and a plate recoloured with it stops identifying
    // who is speaking.
    const textColor = resolveTextColor(settings.textColor, light);
    // "Text and name": the SPOKEN line takes the plate's colour, which is the
    // sampled ink when a sprite answered for this name and the hashed hue when
    // none did. NARRATION keeps the text colour in every mode: it has no
    // speaker, so tinting it would print a voice on a line nobody is saying.
    // `readableLineTint` is what keeps a whole paragraph at 4.5:1 on this
    // surface's worst composite, and it moves the hashed hue only.
    const lineColor = lineColorFor({ mode: settings.speakerTint, isNarration, plateColor, textColor, light });

    // The drag measures the free travel ONCE, on pointer down: the box's own
    // `offsetParent` is the stage, and the panel is the thing the drag moves,
    // so reading either per move would measure a box the drag has already
    // moved. `travel.x` is the room the panel has to slide inside the
    // wrapper's content box and `travel.y` the lift's own 40% of the stage.
    const wrapRef = useRef<HTMLDivElement>(null);
    const panelRef = useRef<HTMLDivElement>(null);
    const position = clampBoxPosition({ x: settings.boxX, y: settings.boxY });
    const dragRef = useRef<{ startX: number; startY: number; from: BoxPosition; travel: { x: number; y: number }; moved: boolean } | null>(null);
    // A drag that MOVED must not also advance the story, and a click event is
    // not something `stopPropagation` on pointerup can reach, so the wrapper
    // swallows the next click itself.
    const swallowClick = useRef(false);

    const measureTravel = useCallback((): { x: number; y: number } => {
        const wrap = wrapRef.current;
        const panel = panelRef.current;
        const stage = wrap?.offsetParent as HTMLElement | null;
        const stageH = stage?.getBoundingClientRect().height ?? 0;
        if (!wrap || !panel) return { x: 0, y: 0 };
        const cs = getComputedStyle(wrap);
        const contentW = wrap.clientWidth - Number.parseFloat(cs.paddingInlineStart || "0") - Number.parseFloat(cs.paddingInlineEnd || "0");
        return { x: contentW - panel.getBoundingClientRect().width, y: (stageH * MAX_BOX_LIFT) / 100 };
    }, []);

    const onDragStart = useCallback(
        (e: React.PointerEvent<HTMLElement>) => {
            // The TEXT is not a handle: a drag that starts on the line is a
            // selection, and the frame around it is what moves the box.
            if (e.target instanceof Element && e.target.closest("[data-story-line]")) return;
            if (e.button !== 0 && e.pointerType === "mouse") return;
            dragRef.current = { startX: e.clientX, startY: e.clientY, from: position, travel: measureTravel(), moved: false };
        },
        [measureTravel, position],
    );
    const onDragMove = useCallback(
        (e: React.PointerEvent<HTMLElement>) => {
            const d = dragRef.current;
            if (!d) return;
            const dx = e.clientX - d.startX;
            const dy = e.clientY - d.startY;
            // Four pixels of slop: under it the gesture is still a click that
            // advances the line, over it the box is being moved.
            if (!d.moved && Math.hypot(dx, dy) < 4) return;
            if (!d.moved) {
                d.moved = true;
                // Only a REAL pointer can be captured: a synthetic event, which
                // is what a test dispatches, has no active pointer and
                // `setPointerCapture` throws `NotFoundError` on it. The drag
                // tracks from the frame's own move events either way.
                if (e.nativeEvent.isTrusted) e.currentTarget.setPointerCapture(e.pointerId);
            }
            onPositionChange(boxPositionFromDrag(d.from, dx, dy, d.travel));
        },
        [onPositionChange],
    );
    const onDragEnd = useCallback(() => {
        swallowClick.current = dragRef.current?.moved === true;
        dragRef.current = null;
    }, []);

    const nudge = useCallback(
        (e: React.KeyboardEvent) => {
            const p = position;
            if (e.key === "ArrowUp") onPositionChange(clampBoxPosition({ x: p.x, y: p.y + BOX_NUDGE_Y }));
            else if (e.key === "ArrowDown") onPositionChange(clampBoxPosition({ x: p.x, y: p.y - BOX_NUDGE_Y }));
            else if (e.key === "ArrowLeft") onPositionChange(clampBoxPosition({ x: p.x - BOX_NUDGE_X, y: p.y }));
            else if (e.key === "ArrowRight") onPositionChange(clampBoxPosition({ x: p.x + BOX_NUDGE_X, y: p.y }));
            else return;
            e.preventDefault();
            e.stopPropagation();
        },
        [onPositionChange, position],
    );

    // The panel's own width, declared twice: once as its `max-width` and once
    // inside the margin that places it. `ch` is resolved against the PANEL's
    // font size, which is why both live on the panel and not on the wrapper.
    const panelWidth = `min(${settings.lineWidth}ch, ${cpx(BOX_MESSAGE_W)})`;
    // x = 0 keeps the flex centring the box shipped with, untouched, so the
    // default reproduces the shipped rect exactly; any other x places the
    // panel by margin instead, over the free room `100% - panelWidth`.
    const centred = position.x === 0;

    // The box never takes more than 40% of the stage: `story-textbox` caps the
    // wrapper at 40% and the line scrolls inside it. A long multiline that used
    // to push the sprites off the stage now stays inside its own frame.
    return (
        <div
            ref={wrapRef}
            data-story-textbox
            data-story-box-x={position.x}
            data-story-box-y={position.y}
            className={cn("story-textbox absolute inset-x-0 flex max-h-[40%] transition-opacity duration-200", centred ? "justify-center" : "justify-start", hidden && "pointer-events-none opacity-0")}
            // `bottom` is a percentage of the STAGE height, which is what the
            // position pad and the drag both write. The default 0 resolves to
            // `bottom: 0%`, the `bottom-0` the box shipped with.
            style={{ "--story-textbox-side": `${settings.sideMargin}%`, "--story-textbox-bottom": `${settings.bottomMargin}%`, bottom: `${position.y * MAX_BOX_LIFT}%` } as React.CSSProperties}
            aria-hidden={hidden}
            onClickCapture={(e) => {
                if (!swallowClick.current) return;
                swallowClick.current = false;
                e.stopPropagation();
            }}
        >
            <div
                ref={panelRef}
                data-story-box-frame
                className={cn("group relative flex w-full min-w-0 cursor-grab flex-col rounded-xl border px-4 py-3 shadow-[0_12px_40px_rgba(0,0,0,.5)] backdrop-blur-md active:cursor-grabbing sm:px-6 sm:py-4", light ? "border-black/10 bg-[#f4f1ea]/95 text-neutral-900" : "border-white/10 bg-black/70 text-white")}
                onPointerDown={onDragStart}
                onPointerMove={onDragMove}
                onPointerUp={onDragEnd}
                onPointerCancel={onDragEnd}
                // The game's message column is 924.598 canvas px wide, which is
                // 72.234% of the stage. The `ch` slider still narrows the box;
                // the canvas number is the CEILING it can never pass, and at
                // 1440 the default 70ch (706.56 px) is well inside it. The
                // margin is the HORIZONTAL position: its percentage resolves
                // against the wrapper's content box, so `(1 + x) / 2` of the
                // room left over walks the panel from the left margin to the
                // right one. `max(0px, ...)` is the guard for a box the width
                // slider has already filled the stage with.
                style={{
                    maxWidth: panelWidth,
                    ...(centred ? {} : { marginInlineStart: `max(0px, calc(${(1 + position.x) / 2} * (100% - ${panelWidth})))` }),
                    fontFamily,
                    fontSize,
                    lineHeight: settings.lineHeight,
                    letterSpacing: `${settings.letterSpacing}em`,
                }}
            >
                {/* The grab affordance. The whole frame is the handle now, so
                    this is what SHOWS that: a pill that appears on hover or
                    focus and is always there on a coarse pointer, inside a
                    16 px target that grows to the 44 px touch minimum.
                    `touch-none` is what stops the drag scrolling the page. */}
                <button
                    type="button"
                    data-story-ui
                    data-story-drag
                    aria-label={dragLabel}
                    onPointerDown={onDragStart}
                    onPointerMove={onDragMove}
                    onPointerUp={onDragEnd}
                    onPointerCancel={onDragEnd}
                    onKeyDown={nudge}
                    className={cn("absolute inset-x-0 -top-2 pointer-coarse:-top-5 flex h-4 pointer-coarse:h-11 cursor-grab touch-none items-center justify-center active:cursor-grabbing", "opacity-0 pointer-coarse:opacity-100 transition-opacity focus-visible:opacity-100 group-hover:opacity-100")}
                >
                    <GripHorizontalIcon className={cn("size-4", light ? "text-neutral-900/50" : "text-white/60")} />
                </button>
                {speaker ? (
                    <div className={cn("absolute -top-3.5 left-4 rounded-md border px-2.5 py-0.5 font-heading font-semibold text-[0.8em] tracking-wide", light ? "bg-[#fbf9f4]/98" : "bg-neutral-950/95")} style={{ color: plateColor, borderColor: `${plateColor}55` }} data-story-speaker data-testid="story-speaker">
                        {speaker}
                    </div>
                ) : null}
                <p
                    className={cn("min-h-[2.6em] min-w-0 flex-1 overflow-y-auto whitespace-pre-wrap", speaker && "pt-1", isNarration && (light ? "text-neutral-900/80 italic" : "text-white/85 italic"))}
                    data-story-line
                    data-testid="story-line"
                    data-narration={isNarration ? "true" : "false"}
                    data-complete={complete ? "true" : "false"}
                    style={lineColor ? { color: lineColor } : undefined}
                >
                    {renderNodes(nodes, budget, "n")}
                </p>
                <ChevronDownIcon aria-label={continueLabel} className={cn("absolute right-3 bottom-2 size-4 transition-opacity", light ? "text-neutral-900/60" : "text-white/70", complete ? "story-bounce opacity-100" : "opacity-0")} />
            </div>
        </div>
    );
}
