// The compact board: a grid shown inside another surface (a profile showcase
// card), not on its own page. The full board sizes its cells off the page
// width, up to 180 px, so a 6 x 6 in a wide card draws 180 px cells and a
// ~1470 px tall board. The compact one sizes its cells off a height budget
// instead, so a typical grid fits in one view without scrolling inside a card.

/** Tallest the cells of a compact board grow, together, before the cells stop shrinking. */
export const COMPACT_GRID_HEIGHT = 640;
/** Smallest a compact cell gets: the full board's own minimum. Past it the board grows taller instead. */
export const COMPACT_CELL_MIN = 48;
/** Largest a compact cell gets, so a 1 x 3 does not draw poster-sized art. */
export const COMPACT_CELL_MAX = 120;
/** The label strip's text size, the same at every density: the strip is two lines of it. */
export const COMPACT_LABEL_PX = 11;
export const COMPACT_GAP_PX = 5;
/** The strip's padding, top plus bottom (4 px each, set in GridBoard.module.css). */
export const COMPACT_STRIP_PAD_PX = 8;
/** The compact board's own padding on each side (GridBoard.module.css): none, the host card pads it. */
export const COMPACT_BOARD_PAD_PX = 0;

/** One label strip's height: two lines at a 1.2 line height plus its padding. */
export const COMPACT_STRIP_PX = COMPACT_LABEL_PX * 1.2 * 2 + COMPACT_STRIP_PAD_PX;

export interface ICompactGridSize {
    /** Each cell's width, and its art's height. */
    cellPx: number;
    gapPx: number;
    labelPx: number;
    /** The board's widest, padding included: the grid's own width at `cellPx`. */
    boardMaxPx: number;
}

/**
 * The cell size that fits `rows` labelled rows into the height budget,
 * clamped to [COMPACT_CELL_MIN, COMPACT_CELL_MAX]. The board can still be
 * narrower than `boardMaxPx` on a narrow screen: there the cells shrink to the
 * width, as the full board's do.
 */
export function compactGridSize(rows: number, cols: number): ICompactGridSize {
    const r = Math.max(1, Math.round(rows));
    const c = Math.max(1, Math.round(cols));
    const fit = Math.floor((COMPACT_GRID_HEIGHT - r * COMPACT_STRIP_PX - (r - 1) * COMPACT_GAP_PX) / r);
    const cellPx = Math.min(COMPACT_CELL_MAX, Math.max(COMPACT_CELL_MIN, fit));
    return {
        cellPx,
        gapPx: COMPACT_GAP_PX,
        labelPx: COMPACT_LABEL_PX,
        boardMaxPx: c * cellPx + (c - 1) * COMPACT_GAP_PX + 2 * COMPACT_BOARD_PAD_PX,
    };
}
