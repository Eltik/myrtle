/**
 * THE TOP PROGRESS BAR AS A SCRUBBER: the track, the knob, and the tooltip that
 * follows the pointer.
 *
 * The visible track is 3 px at rest and 6 px under the pointer; the HIT band is
 * the 12 px this element declares on a fine pointer and the 44 px touch minimum
 * on a coarse one, where the toolbar drops to 48 px to stay clear of it.
 * MEASURED: a 16 px band at 1440x900 was intercepted by the toolbar across
 * x 700..1428, half the bar.
 *
 * `scrub.ts` owns the two lookups (`fractionAt`, `haltAtFraction`) and
 * `chrome.ts` owns the tooltip clamp, both pure and both tested. What is left
 * here is the rect, which is the one thing a test cannot supply.
 */
import type React from "react";
import { useLayoutEffect, useRef, useState } from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { clampTooltipLeft } from "./chrome";
import type { messages } from "./reader.messages";
import { fractionAt, type HaltSummary, haltAtFraction } from "./scrub";
import { speakerColor } from "./TextBox";

/** Where the pointer is on the bar, and where its tooltip may sit. Null means the pointer is off the bar. */
interface IScrubAt {
    summary: HaltSummary;
    pointerX: number;
    barWidth: number;
    tipTop: number;
}

/**
 * `useT` is bound HERE rather than taken as a prop, and that is not a style
 * choice: `scripts/i18n-extract.mjs` binds a `t` to its namespace by the
 * `useT(...)` call in the SAME file, so a component handed its translator
 * through a prop takes every key it uses out of the catalogue's reach. Passing
 * it down measured 54 keys newly unused and 59 call sites lost.
 */
export interface IScrubberProps {
    /** Fades with the toolbar. */
    shown: boolean;
    summaries: HaltSummary[];
    totalHalts: number;
    /** 0..1, the halt the reader is at over the total. */
    progress: number;
    /** The toolbar's rect, so the tooltip can drop below the bar instead of intersecting it. */
    barRect: () => DOMRect | undefined;
    onPointerEnter: () => void;
    onPointerLeave: () => void;
    /** A pointer UP jumps, which is what makes a mouse click and a touch drag the same gesture. */
    onJump: (haltIndex: number) => void;
}

export function Scrubber({ shown, summaries, totalHalts, progress, barRect, onPointerEnter, onPointerLeave, onJump }: IScrubberProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const [at, setAt] = useState<IScrubAt | null>(null);
    const tipRef = useRef<HTMLDivElement>(null);
    const [tipWidth, setTipWidth] = useState(0);

    // The tip's own width, measured before paint: the first render of a tip
    // positions it at width 0 and this corrects it in the same frame, so the
    // clamp never shows an unclamped tip.
    useLayoutEffect(() => {
        if (!at) return;
        const w = tipRef.current?.offsetWidth ?? 0;
        if (w !== tipWidth) setTipWidth(w);
    });

    const read = (e: React.PointerEvent<HTMLDivElement>): HaltSummary | null => {
        const rect = e.currentTarget.getBoundingClientRect();
        const summary = haltAtFraction(summaries, fractionAt(e.clientX, rect));
        // The scrubber spans the stage and starts at its top edge, so its own
        // rect IS the stage's x and top. While the bar is on screen the tip
        // drops BELOW the bar's bottom edge, which is what keeps the two rects
        // from ever intersecting; with the bar faded out it sits under the track.
        const bar = barRect();
        const tipTop = shown && bar ? bar.bottom - rect.top + 8 : 12;
        if (summary) setAt({ summary, pointerX: e.clientX - rect.left, barWidth: rect.width, tipTop });
        return summary;
    };

    const left = at ? clampTooltipLeft(at.pointerX, tipWidth, at.barWidth) : 0;

    return (
        <div
            data-story-ui
            data-story-scrubber
            data-story-chrome-hidden={shown ? "false" : "true"}
            className={cn("group absolute inset-x-0 top-0 z-40 h-3 pointer-coarse:h-11 touch-none transition-opacity", shown ? "opacity-100 duration-150" : "pointer-events-none opacity-0 duration-200")}
            onPointerEnter={onPointerEnter}
            onPointerMove={(e) => {
                read(e);
            }}
            onPointerLeave={() => {
                onPointerLeave();
                setAt(null);
            }}
            onPointerDown={(e) => {
                e.currentTarget.setPointerCapture(e.pointerId);
                read(e);
            }}
            onPointerUp={(e) => {
                const summary = read(e);
                if (summary) onJump(summary.haltIndex);
            }}
        >
            <div className="absolute inset-x-0 top-0 h-0.75 bg-white/15 transition-[height,background-color] duration-150 group-hover:h-1.5 group-hover:bg-white/25" role="progressbar" aria-label={t("reader.progress")} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(progress * 100)}>
                <div className="relative h-full bg-primary transition-[width] duration-300 group-hover:shadow-[0_0_10px_1px_var(--color-primary)]" style={{ width: `${progress * 100}%` }}>
                    {/* The knob marks the halt the reader is AT. It only appears
                        under the pointer, so the resting bar is a hairline and
                        nothing else. */}
                    <span className="pointer-events-none absolute top-1/2 right-0 size-3 translate-x-1/2 -translate-y-1/2 rounded-full bg-primary opacity-0 shadow-[0_0_0_2px_rgba(0,0,0,.45),0_0_10px_2px_var(--color-primary)] transition-opacity duration-150 group-hover:opacity-100" />
                </div>
            </div>
            {at ? (
                <div ref={tipRef} className="pointer-events-none absolute z-50 w-max max-w-[min(22rem,80vw)] rounded-md border border-white/15 bg-black/92 px-2.5 py-1.5 text-white text-xs shadow-lg backdrop-blur-sm" style={{ left: `${left}px`, top: `${at.tipTop}px` }} data-story-scrub-tip>
                    {/* The arrow only exists while the bar is on screen and the
                        tip has been pushed below it; a tip riding on the track
                        needs none. */}
                    {shown ? <span className="absolute -top-1 size-2 rotate-45 border-white/15 border-t border-l bg-black/92" style={{ left: `${Math.min(Math.max(8, at.pointerX - left - 4), Math.max(8, tipWidth - 16))}px` }} /> : null}
                    <span className="block font-mono text-[0.95em] text-white/60 tabular-nums">{t("reader.scrub.line", { line: at.summary.haltIndex + 1, total: totalHalts })}</span>
                    {at.summary.preview || at.summary.speaker ? (
                        <span className="mt-0.5 block truncate">
                            {at.summary.speaker ? (
                                <span className="me-1.5 font-semibold" style={{ color: speakerColor(at.summary.speaker, false) }}>
                                    {at.summary.speaker}
                                </span>
                            ) : null}
                            {at.summary.preview}
                        </span>
                    ) : null}
                </div>
            ) : null}
        </div>
    );
}
