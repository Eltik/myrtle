import { describe, expect, it } from "vitest";
import { blockFit, LANE_MAX, type ShowcaseBlockFit, showcaseRows } from "./rows";

function flatten(rows: ReturnType<typeof showcaseRows>): number[] {
    return rows.flatMap((r) => (r.kind === "full" ? [r.index] : r.kind === "anchor" ? [r.index, ...r.lane] : r.items));
}

describe("showcaseRows keeps the owner's order and sizes blocks to their content", () => {
    it("puts the small blocks after a grid in a lane beside it", () => {
        // A grid, a favourites block and a plan: the showcase this layout was measured on.
        expect(showcaseRows(["anchor", "flow", "flow"])).toEqual([{ kind: "anchor", index: 0, lane: [1, 2] }]);
    });

    it("caps the lane and wraps the rest into a row of their own", () => {
        const fits: ShowcaseBlockFit[] = ["anchor", ...Array<ShowcaseBlockFit>(LANE_MAX + 2).fill("flow")];
        const rows = showcaseRows(fits);
        expect(rows[0]).toEqual({ kind: "anchor", index: 0, lane: Array.from({ length: LANE_MAX }, (_, k) => k + 1) });
        expect(rows[1]).toEqual({ kind: "flow", items: [LANE_MAX + 1, LANE_MAX + 2] });
    });

    it("gives a tier list a row to itself and never pulls blocks across it", () => {
        expect(showcaseRows(["flow", "full", "anchor", "full", "flow"])).toEqual([
            { kind: "flow", items: [0] },
            { kind: "full", index: 1 },
            { kind: "anchor", index: 2, lane: [] },
            { kind: "full", index: 3 },
            { kind: "flow", items: [4] },
        ]);
    });

    it("never lanes a grid beside another grid", () => {
        expect(showcaseRows(["anchor", "anchor", "flow"])).toEqual([
            { kind: "anchor", index: 0, lane: [] },
            { kind: "anchor", index: 1, lane: [2] },
        ]);
    });

    it("places every block exactly once, in order", () => {
        const kinds: ShowcaseBlockFit[] = ["anchor", "flow", "flow", "flow", "full", "flow", "anchor", "anchor", "flow", "full"];
        expect(flatten(showcaseRows(kinds))).toEqual(kinds.map((_, k) => k));
        expect(showcaseRows([])).toEqual([]);
    });

    it("lets a removed block flow, whatever it was", () => {
        expect(blockFit({ type: "grid", removed: true })).toBe("flow");
        expect(blockFit({ type: "grid", removed: false })).toBe("anchor");
        expect(blockFit({ type: "tier_list", removed: false })).toBe("full");
        expect(blockFit({ type: "favourites", removed: false })).toBe("flow");
        expect(blockFit({ type: "plan", removed: false })).toBe("flow");
    });
});
