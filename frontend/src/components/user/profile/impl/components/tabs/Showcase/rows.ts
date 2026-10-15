import type { ShowcaseDraftBlock } from "../../../showcase";

// How the showcase lays its blocks out, in the owner's order. Blocks size to
// their content instead of each spanning the page: a grid is as wide as its
// board, so at 1440 px a 6 x 6 board (457 px) would leave ~2/3 of a full-width card
// empty. A grid ANCHORS a row and the small blocks after it stack in a lane
// beside it; a tier list is wide by nature and takes a row to itself; any
// other run of small blocks wraps in a row of its own.

export type ShowcaseBlockFit = "anchor" | "full" | "flow";

/** A grid anchors; a tier list takes the full width; favourites, plans and removed blocks flow. */
export function blockFit(block: Pick<ShowcaseDraftBlock, "type" | "removed">): ShowcaseBlockFit {
    if (block.removed) return "flow";
    if (block.type === "grid") return "anchor";
    if (block.type === "tier_list") return "full";
    return "flow";
}

/**
 * Most small blocks a lane beside a grid takes. A TRADE, not a derivation: a
 * 6 x 6 compact grid card is ~705 px tall, and a favourites block (~330 px with
 * portraits) over a plan card (311 px) is ~657 px, so two fill it; a third
 * would outgrow the grid and leave the empty slab under the board instead.
 * Heights are content-dependent, so no count is right for every showcase.
 */
export const LANE_MAX = 2;

export type ShowcaseRow = { kind: "full"; index: number } | { kind: "anchor"; index: number; lane: number[] } | { kind: "flow"; items: number[] };

/** Groups block indices into rows. Every index appears once, in order. */
export function showcaseRows(fits: readonly ShowcaseBlockFit[]): ShowcaseRow[] {
    const rows: ShowcaseRow[] = [];
    let i = 0;
    while (i < fits.length) {
        const fit = fits[i];
        if (fit === "full") {
            rows.push({ kind: "full", index: i });
            i += 1;
        } else if (fit === "anchor") {
            const lane: number[] = [];
            let j = i + 1;
            while (j < fits.length && fits[j] === "flow" && lane.length < LANE_MAX) {
                lane.push(j);
                j += 1;
            }
            rows.push({ kind: "anchor", index: i, lane });
            i = j;
        } else {
            const items: number[] = [];
            while (i < fits.length && fits[i] === "flow") {
                items.push(i);
                i += 1;
            }
            rows.push({ kind: "flow", items });
        }
    }
    return rows;
}
