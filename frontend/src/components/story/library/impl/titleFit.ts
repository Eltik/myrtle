/**
 * HOW A TITLE IS SET WHEN THERE IS NO LOGOTYPE TO DRAW.
 *
 * This module used to fit the ghost echo behind a ticket. The ghost is gone
 * (the authored logotype is the title now), and what survives is the
 * measurement underneath it, generalised: given a box and a way to measure a
 * line, answer the largest size at which the title sets WITHOUT a mid-word
 * cut.
 *
 * WHY MEASURE AT ALL. The reviewer's case was "Grani and the Knights'
 * Treasure" printing as GRANI AND THE KNIGHTS'... : a fixed 28 px title with a
 * two-line clamp and an ellipsis drops the one word that names the story. CSS
 * cannot decide this, because a container unit scales with the CARD and never
 * with the title, so one `font-size` prints COLLAPSE and NECESSARY SOLUTIONS
 * alike. The title's own advance width only comes from a measurement.
 *
 * THE ORDER OF CONCESSIONS is fixed and is what the reviewer asked for: shrink
 * first, from the ceiling down to a floor, and only at the floor take another
 * line. `overflow-wrap: anywhere` is the last resort and fires only when a
 * single WORD is still wider than the box at the floor.
 *
 * `widthEm` returns a line's width at font-size 1, so the caller owns the
 * measurement and this stays pure. Sizes and the box are in the SAME unit,
 * whatever the caller chooses.
 */

import { useEffect, useRef, useState } from "react";

export interface ITitleFit {
    /** The lines, rendered as their own elements so the browser cannot re-wrap them. */
    lines: string[];
    /** The font size, in the unit the box was given in. */
    size: number;
    /** True only when a single word still overflows at the floor, which is what licenses breaking one. */
    breakAnywhere: boolean;
}

export interface IFitBounds {
    /** The box's inline size. */
    box: number;
    /** The size a title that fits is set at. */
    max: number;
    /** The size below which the answer is another line, not smaller type. */
    min: number;
    /** Lines allowed before the floor. */
    lines: number;
    /** Lines allowed at the floor. */
    floorLines: number;
    /** The sampling step from {@link IFitBounds.max} down to {@link IFitBounds.min}. */
    step?: number;
}

/** Greedy word wrap at one size: the lines the title takes, each one whole words. */
function wrap(words: string[], widthEm: (text: string) => number, capEm: number): string[] {
    const lines: string[] = [];
    let line = "";
    for (const word of words) {
        const next = line === "" ? word : `${line} ${word}`;
        if (line !== "" && widthEm(next) > capEm) {
            lines.push(line);
            line = word;
        } else line = next;
    }
    if (line !== "") lines.push(line);
    return lines;
}

/**
 * The largest size in `(min, max]` at which the title takes no more than
 * `lines` lines, else the floor with up to `floorLines`.
 */
export function fitTitle(title: string, widthEm: (text: string) => number, bounds: IFitBounds): ITitleFit {
    const words = title
        .trim()
        .split(/\s+/)
        .filter((w) => w !== "");
    if (words.length === 0) return { lines: [], size: bounds.max, breakAnywhere: false };

    // The line COUNT is not the whole test. A one-word title wraps to exactly one
    // line at every size, because the wrapper never cuts a word, so a count-only
    // gate hands SOMNILOQUIUM back at the ceiling and lets it run off the card.
    // Every accepted size has to clear the box on width as well.
    const widest = (set: string[]): number => Math.max(...set.map(widthEm));
    const step = bounds.step ?? 0.5;
    for (let size = bounds.max; size > bounds.min; size -= step) {
        const set = wrap(words, widthEm, bounds.box / size);
        if (set.length <= bounds.lines && widest(set) * size <= bounds.box) return { lines: set, size, breakAnywhere: false };
    }
    const set = wrap(words, widthEm, bounds.box / bounds.min);
    const longestWord = Math.max(...words.map(widthEm)) * bounds.min;
    return { lines: set.slice(0, bounds.floorLines), size: bounds.min, breakAnywhere: longestWord > bounds.box };
}

/** The face a fitted title is set in, which is the card's: `--font-heading` at 800. */
const TITLE_FONT = '800 100px "Inter Variable", ui-sans-serif, system-ui, sans-serif';

let context: CanvasRenderingContext2D | null | undefined;

function measureEm(text: string): number {
    if (context === undefined) {
        const canvas = document.createElement("canvas");
        context = canvas.getContext("2d");
        if (context) context.font = TITLE_FONT;
    }
    if (!context) return text.length * 0.62;
    return context.measureText(text.toUpperCase()).width / 100;
}

const cache = new Map<string, ITitleFit>();

/**
 * The fit for one title in one box, measured once and re-measured when the
 * webfont lands.
 *
 * The bounds arrive as a fresh object literal on every render, so they are
 * held in a ref and the KEY is what identifies a fit. Putting the object in
 * the dependency list re-runs the effect every render, and the effect calls
 * `setFit`, which renders again: that is an infinite loop, not a stale-closure
 * risk, which is why the deps are the key and the title.
 */
export function useTitleFit(title: string, bounds: IFitBounds): ITitleFit {
    const key = `${bounds.box}|${bounds.max}|${bounds.min}|${bounds.lines}|${title}`;
    const latest = useRef(bounds);
    latest.current = bounds;
    const [fit, setFit] = useState<ITitleFit>(() => cache.get(key) ?? { lines: [title], size: bounds.min, breakAnywhere: false });

    useEffect(() => {
        const measure = (): void => {
            const next = fitTitle(title, measureEm, latest.current);
            cache.set(key, next);
            setFit(next);
        };
        const cached = cache.get(key);
        if (cached) setFit(cached);
        else measure();
        // Until the webfont is ready the canvas reports the FALLBACK face's advances,
        // which are a few percent off; one re-measure afterwards settles it.
        const fonts = (document as Document & { fonts?: FontFaceSet }).fonts;
        if (!fonts || fonts.status === "loaded") return;
        let live = true;
        void fonts.ready.then(() => {
            if (!live) return;
            cache.delete(key);
            measure();
        });
        return () => {
            live = false;
        };
    }, [key, title]);

    return fit;
}
