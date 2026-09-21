import { type RefObject, useLayoutEffect, useState } from "react";

/**
 * The "+N" badge's width in avatar units, from its CSS: `padding: 0
 * calc(var(--op-size) * 0.22)` around mono digits at `font-size:
 * calc(var(--op-size) * 0.42)`, and a mono glyph is ~0.6em. So a badge with g
 * glyphs (the "+" and N's digits) is (0.44 + 0.252 * g) avatars wide, plus its
 * 2px `margin-left`. Measured: "+48" at --op-size 22px renders 26.3px and the
 * formula gives 26.3px; one extra px of slack absorbs font rounding.
 */
const BADGE_PAD_RATIO = 0.44;
const BADGE_GLYPH_RATIO = 0.6 * 0.42;
const BADGE_MARGIN_PX = 2;
const BADGE_SLACK_PX = 1;

export interface IRowMetrics {
    /** Inner width of the row's tile container, px. */
    width: number;
    /** Tile edge, px (`--op-size`). */
    tile: number;
    /** Gap between tiles, px (`column-gap`). */
    gap: number;
}

function badgeReservePx(tile: number, digits: number): number {
    const glyphs = 1 + digits;
    return Math.ceil(tile * (BADGE_PAD_RATIO + BADGE_GLYPH_RATIO * glyphs)) + BADGE_MARGIN_PX + BADGE_SLACK_PX;
}

function tilesThatFit(width: number, tile: number, gap: number): number {
    return Math.floor((width + gap) / (tile + gap));
}

function digitCount(n: number): number {
    return String(n).length;
}

/**
 * How many of `total` tiles to show on a row of the given metrics. Shows all
 * of them when they fit; otherwise reserves room for the "+N" badge. The
 * badge's width depends on N's digit count and N depends on the tile count,
 * so the reserve starts at the fewest digits the badge can have and widens
 * until the digit count is self-consistent (one extra pass per digit at most).
 */
export function fitOperatorCount({ width, tile, gap }: IRowMetrics, total: number): number {
    const withoutBadge = tilesThatFit(width, tile, gap);
    if (withoutBadge >= total) return total;

    let digits = digitCount(Math.max(1, total - withoutBadge));
    for (;;) {
        const tiles = Math.max(1, tilesThatFit(width - badgeReservePx(tile, digits), tile, gap));
        const actualDigits = digitCount(total - tiles);
        if (actualDigits <= digits) return tiles;
        digits = actualDigits;
    }
}

/** Reads the row's metrics from its computed style; null until the row has a usable size. */
function readRowMetrics(el: HTMLElement, width: number): IRowMetrics | null {
    const style = getComputedStyle(el);
    const tile = Number.parseFloat(style.getPropertyValue("--op-size"));
    const gap = Number.parseFloat(style.columnGap) || 0;
    if (!Number.isFinite(tile) || tile <= 0 || width <= 0) return null;
    return { width, tile, gap };
}

/**
 * How many fixed-size avatars fit on one thumbnail row, measured from the
 * row's rendered width. The avatar size comes from the row's inherited
 * `--op-size` and the gap from its `column-gap`, so the CSS that sizes the
 * tiles stays the only place those numbers live.
 *
 * Returns `fallback` until the element has been measured (server render and
 * the first client paint), so the pre-measurement output is identical to the
 * fixed cap the cards used before.
 */
export function useFittedOpCount(ref: RefObject<HTMLElement | null>, total: number, fallback: number): number {
    const [fitted, setFitted] = useState<number | null>(null);

    useLayoutEffect(() => {
        const el = ref.current;
        if (!el || typeof ResizeObserver === "undefined") return;

        const measure = (width: number) => {
            const metrics = readRowMetrics(el, width);
            if (metrics) setFitted(fitOperatorCount(metrics, total));
        };

        measure(el.clientWidth);
        const observer = new ResizeObserver((entries) => {
            for (const entry of entries) measure(entry.contentRect.width);
        });
        observer.observe(el);
        return () => observer.disconnect();
    }, [ref, total]);

    return Math.min(fitted ?? fallback, total);
}
