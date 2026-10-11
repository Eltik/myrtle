import { describe, expect, it } from "vitest";
import { fitBox } from "./PanZoom";

/** Where a content point lands on screen, in pixels, for view box `b` on a `w`-wide stage. */
const screen = (b: { x: number; y: number; w: number }, w: number, p: { x: number; y: number }) => ({ x: ((p.x - b.x) / b.w) * w, y: ((p.y - b.y) / b.w) * w });

describe("fitBox", () => {
    it("centres content in the stage with no insets", () => {
        const b = fitBox({ x: -100, y: -50, w: 200, h: 100 }, 1000, 500);
        expect(b).toEqual({ x: -100, y: -50, w: 200, h: 100 });
    });

    it("keeps content inside the area the controls leave free, centred there", () => {
        const c = { x: 0, y: 0, w: 400, h: 400 };
        const w = 1000;
        const h = 500;
        const b = fitBox(c, w, h, { top: 80, right: 0, bottom: 20, left: 0 });
        const tl = screen(b, w, { x: c.x, y: c.y });
        const br = screen(b, w, { x: c.x + c.w, y: c.y + c.h });
        expect(tl.y).toBeCloseTo(80, 6);
        expect(br.y).toBeCloseTo(480, 6);
        // Horizontally centred: equal margins left and right.
        expect(tl.x).toBeCloseTo(w - br.x, 6);
        // Same scale on both axes.
        expect(b.h / b.w).toBeCloseTo(h / w, 9);
    });

    it("drops insets that would leave less than half the stage", () => {
        const c = { x: 0, y: 0, w: 100, h: 100 };
        expect(fitBox(c, 300, 200, { top: 150, right: 0, bottom: 0, left: 0 })).toEqual(fitBox(c, 300, 200));
    });
});
