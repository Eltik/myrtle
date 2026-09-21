import { describe, expect, it } from "vitest";
import { fitOperatorCount } from "./use-fitted-op-count";

// A browse-card row at 4-5 tiers: 236px wide, 22px tiles, 1px gap. The
// expected counts were measured in the browser after the fix shipped.
const row = { width: 236, tile: 22, gap: 1 };

describe("fitOperatorCount", () => {
    it("shows every operator when they all fit (10 slots at 236px)", () => {
        expect(fitOperatorCount(row, 3)).toBe(3);
        expect(fitOperatorCount(row, 10)).toBe(10);
    });

    it("gives up one slot to a one- or two-digit badge", () => {
        expect(fitOperatorCount(row, 17)).toBe(9); // "+8" ends 6.2px from the edge
        expect(fitOperatorCount(row, 25)).toBe(9); // "+16" ends 0.7px from the edge
        expect(fitOperatorCount(row, 56)).toBe(9); // "+47"
    });

    it("gives up two slots to a three-digit badge", () => {
        expect(fitOperatorCount(row, 110)).toBe(8); // "+102"
        expect(fitOperatorCount(row, 1000)).toBe(8); // "+992"
    });

    it("keeps at least one tile on a row too narrow for tile plus badge", () => {
        expect(fitOperatorCount({ width: 30, tile: 22, gap: 1 }, 5)).toBe(1);
    });

    it("scales with the tile size (one-tier thumbs use 56px tiles)", () => {
        const wide = { width: 236, tile: 56, gap: 1 };
        expect(fitOperatorCount(wide, 4)).toBe(4);
        expect(fitOperatorCount(wide, 9)).toBe(3); // 4 slots; "+6" needs ~2 avatars' width
    });
});
