import { describe, expect, it } from "vitest";
import { type ThumbCell, thumbArtFit, thumbCells, thumbSpacing, thumbWidth } from "./thumb";

const amiya: ThumbCell = { kind: "operator", icon: "/avatar/char_002_amiya", server: null };

describe("thumbCells draws the grid's shape", () => {
    it("keeps every cell of a full list in order", () => {
        const preview: ThumbCell[] = [amiya, null, { ...amiya, server: "cn" }, null];
        expect(thumbCells(preview, 2, 2)).toEqual(preview);
    });

    it("draws a 6 x 6 as 36 cells, not a 2 x 2 crop", () => {
        expect(thumbCells(Array(36).fill(amiya), 6, 6)).toHaveLength(36);
    });

    it("pads a short list with empty cells and cuts a long one", () => {
        expect(thumbCells([amiya], 1, 3)).toEqual([amiya, null, null]);
        expect(thumbCells([amiya, amiya, amiya], 1, 2)).toEqual([amiya, amiya]);
        expect(thumbCells([], 10, 10)).toEqual(Array(100).fill(null));
    });
});

describe("thumbSpacing", () => {
    it("thins the gutters as the board gets denser, by its longer side", () => {
        expect(thumbSpacing(1, 1)).toEqual({ gap: 4, radius: 4 });
        expect(thumbSpacing(3, 3)).toEqual({ gap: 4, radius: 4 });
        expect(thumbSpacing(2, 6)).toEqual({ gap: 3, radius: 3 });
        expect(thumbSpacing(10, 1)).toEqual({ gap: 2, radius: 2 });
    });
});

describe("thumbArtFit follows the board's art fit", () => {
    it("covers for square art, insets objects and glyphs", () => {
        expect(thumbArtFit(amiya as NonNullable<ThumbCell>)).toBe("cover");
        expect(thumbArtFit({ kind: "module", icon: "/module/ui_equip_type_direction/x.png", server: null })).toBe("object");
        expect(thumbArtFit({ kind: "module", icon: "/module/uniequip_002_amiya.png", server: null })).toBe("cover");
        expect(thumbArtFit({ kind: "faction", icon: "/faction/logo_rhodes.png", server: null })).toBe("glyph");
    });
});

describe("thumbWidth fits square cells to the box", () => {
    it("solves the board's height for the box's height", () => {
        expect(thumbWidth(1, 1, 4)).toBe("min(100cqw, calc((100cqh - 0px) * 1 + 0px))");
        expect(thumbWidth(6, 6, 3)).toBe("min(100cqw, calc((100cqh - 15px) * 1 + 15px))");
        expect(thumbWidth(10, 1, 2)).toBe("min(100cqw, calc((100cqh - 18px) * 0.1 + 0px))");
        expect(thumbWidth(1, 10, 2)).toBe("min(100cqw, calc((100cqh - 0px) * 10 + 18px))");
    });

    it("gives a board exactly the box's height when the height binds", () => {
        // A 120 px tall box, 4 x 2 at a 3 px gap: width w makes cells (w - 3) / 2, so height 4 * cell + 9.
        const h = 120;
        const w = (h - 3 * 3) * (2 / 4) + 3;
        expect(4 * ((w - 3) / 2) + 3 * 3).toBeCloseTo(h);
    });
});
