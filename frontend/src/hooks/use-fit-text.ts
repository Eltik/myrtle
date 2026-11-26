import { type RefObject, useLayoutEffect } from "react";

/** Below this the label is unreadable; the CSS breaks the word instead (`overflow-wrap: anywhere`). */
const DEFAULT_MIN_SCALE = 0.55;

/** Headroom so sub-pixel rounding at the scaled size never clips the last glyph. */
const SLACK = 0.98;

/**
 * The font scale that makes text `needed` px wide fit in `available` px, between `minScale` and 1.
 * Text that already fits keeps its size; nothing measured (0 or negative) also keeps it.
 */
export function fitTextScale(available: number, needed: number, minScale = DEFAULT_MIN_SCALE): number {
    if (!(available > 0) || !(needed > 0) || needed <= available) return 1;
    return Math.max(minScale, Math.min(1, (available / needed) * SLACK));
}

function contentWidth(el: HTMLElement): number {
    const style = getComputedStyle(el);
    return el.clientWidth - (Number.parseFloat(style.paddingLeft) || 0) - (Number.parseFloat(style.paddingRight) || 0);
}

/**
 * Shrinks a label's font so its longest word fits its parent's width. Writes the result to the
 * element's `--fit-scale` custom property, which its CSS multiplies into `font-size`; no React
 * state, so a resize never re-renders the row.
 *
 * The measure switches `overflow-wrap` to `normal` for one synchronous read so a long word
 * overflows (and shows in `scrollWidth`) instead of breaking. Spaces still wrap, so a multi-word
 * label is sized by its longest word, not its full length. The element needs `min-width: 0` and
 * `max-width: 100%` so it cannot grow past its parent, and `overflow-wrap: anywhere` as the last
 * resort when even `minScale` does not fit.
 */
export function useFitText(ref: RefObject<HTMLElement | null>, text: string, minScale = DEFAULT_MIN_SCALE): void {
    // biome-ignore lint/correctness/useExhaustiveDependencies: re-measure when the label is renamed; the observer only sees the parent resize
    useLayoutEffect(() => {
        const el = ref.current;
        const parent = el?.parentElement;
        if (!el || !parent || typeof ResizeObserver === "undefined") return;

        const measure = () => {
            el.style.setProperty("--fit-scale", "1");
            el.style.overflowWrap = "normal";
            const scale = fitTextScale(contentWidth(parent), el.scrollWidth, minScale);
            el.style.overflowWrap = "";
            el.style.setProperty("--fit-scale", String(scale));
        };

        measure();
        const observer = new ResizeObserver(measure);
        observer.observe(parent);
        let live = true;
        document.fonts?.ready.then(() => {
            if (live) measure();
        });
        return () => {
            live = false;
            observer.disconnect();
        };
    }, [ref, text, minScale]);
}
