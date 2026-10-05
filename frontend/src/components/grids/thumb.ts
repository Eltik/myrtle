import { type ArtFit, entityArtFit, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import type { GridPreviewIcon } from "#/types/generated/GridPreviewIcon";

// The card thumbnail is the board in miniature: every cell in the grid's
// shape, art where a pick has some, the board's empty grey elsewhere. These
// are its pure parts; `GridCard` draws it.

/** One thumbnail cell: the pick's art, or `null` for a cell drawn empty. */
export type ThumbCell = GridPreviewIcon | null;

/**
 * `preview` as exactly `rows * cols` cells, row-major. The backend already
 * sends that many; a list that is not (a response from an older build, which
 * sent up to four icons) is cut or padded with empty cells so the card still
 * draws the grid's shape.
 */
export function thumbCells(preview: readonly ThumbCell[], rows: number, cols: number): ThumbCell[] {
    const n = Math.max(0, rows) * Math.max(0, cols);
    return Array.from({ length: n }, (_, i) => preview[i] ?? null);
}

/** Gap and corner radius in px: a denser board draws thinner gutters, so a 10 x 10 still reads as cells rather than lines. */
export function thumbSpacing(rows: number, cols: number): { gap: number; radius: number } {
    const side = Math.max(rows, cols);
    if (side <= 3) return { gap: 4, radius: 4 };
    if (side <= 6) return { gap: 3, radius: 3 };
    return { gap: 2, radius: 2 };
}

/**
 * The thumbnail board's CSS width inside a container of size containment: the
 * box's full width, unless that would make the board taller than the box. The
 * cells are square, so a board `w` wide is `rows * cell + (rows - 1) * gap`
 * tall with `cell = (w - (cols - 1) * gap) / cols`; solving that height for
 * the box's height gives the second term. A 1 x 10 fills the width, a 10 x 1
 * the height, a 6 x 6 whichever is shorter.
 */
export function thumbWidth(rows: number, cols: number, gap: number): string {
    return `min(100cqw, calc((100cqh - ${(rows - 1) * gap}px) * ${cols / rows} + ${(cols - 1) * gap}px))`;
}

/**
 * How a cell's art sits, by the board's rules (`entityArtFit`) read off what a
 * thumbnail carries: the kind and the icon path. A module badge is told by its
 * path, as on the board. An Integrated Strategies theme is told by a facet the
 * thumbnail does not carry, so it takes its kind's `object` fit (a 6% inset)
 * where the board covers the cell: a TRADE, a pixel or two at thumbnail size,
 * for not sending every pick's facets on every card.
 */
export function thumbArtFit(cell: GridPreviewIcon): ArtFit {
    return entityArtFit(toTierEntity(cell.kind, "", { kind: cell.kind, id: "", name: "", icon: cell.icon, href: null, facets: {} }, UNPLACED));
}
