import { describe, expect, it } from "vitest";
import { clampRect, defaultRect, moveRect, parseRect, RECT_MARGIN, RECT_MIN_HEIGHT, RECT_MIN_WIDTH, resizeRect, WIDE_WIDTH } from "./picker-rect";

const VW = 1500;
const VH = 713;

describe("picker rectangle", () => {
    it("opens docked lower left, inside the margins", () => {
        // 713 tall: the 720 default height yields to 713 - 32.
        expect(defaultRect(VW, VH)).toEqual({ x: RECT_MARGIN, y: RECT_MARGIN, width: 960, height: 681 });
        expect(defaultRect(1500, 900)).toEqual({ x: RECT_MARGIN, y: 900 - RECT_MARGIN - 720, width: 960, height: 720 });
        // A short, narrow window caps both at the viewport less its margins.
        expect(defaultRect(900, 600)).toEqual({ x: 16, y: 16, width: 868, height: 568 });
        expect(defaultRect(1280, 600).width).toBeGreaterThanOrEqual(WIDE_WIDTH);
    });

    it("never leaves the viewport when moved", () => {
        const start = defaultRect(VW, VH);
        expect(moveRect(start, 5000, 5000, VW, VH)).toMatchObject({ x: VW - RECT_MARGIN - 960, y: VH - RECT_MARGIN - 681 });
        expect(moveRect(start, -5000, -5000, VW, VH)).toMatchObject({ x: RECT_MARGIN, y: RECT_MARGIN });
    });

    it("resizes between the minimum and the viewport, the opposite edge fixed", () => {
        const start = { x: 300, y: 50, width: 720, height: 600 };
        expect(resizeRect(start, { right: true, bottom: true }, -1000, -1000, VW, VH)).toEqual({ x: 300, y: 50, width: RECT_MIN_WIDTH, height: RECT_MIN_HEIGHT });
        expect(resizeRect(start, { right: true, bottom: true }, 5000, 5000, VW, VH)).toEqual({ x: 300, y: 50, width: VW - RECT_MARGIN - 300, height: VH - RECT_MARGIN - 50 });
        // The left edge stops at the minimum instead of pushing the right edge.
        expect(resizeRect(start, { left: true }, 1000, 0, VW, VH)).toEqual({ x: 300 + 720 - RECT_MIN_WIDTH, y: 50, width: RECT_MIN_WIDTH, height: 600 });
        expect(resizeRect(start, { left: true, top: true }, -1000, -1000, VW, VH)).toEqual({ x: RECT_MARGIN, y: RECT_MARGIN, width: 1020 - RECT_MARGIN, height: 650 - RECT_MARGIN });
    });

    it("yields the minimum to a viewport smaller than it", () => {
        expect(clampRect({ x: 0, y: 0, width: 900, height: 900 }, 500, 400)).toEqual({ x: 16, y: 16, width: 468, height: 368 });
    });

    it("reads back only a well-formed stored rectangle", () => {
        expect(parseRect(null)).toBeNull();
        expect(parseRect("not json")).toBeNull();
        expect(parseRect('{"x":1,"y":2,"width":3}')).toBeNull();
        expect(parseRect('{"x":1,"y":2,"width":3,"height":"4"}')).toBeNull();
        expect(parseRect('{"x":1,"y":2,"width":3,"height":4}')).toEqual({ x: 1, y: 2, width: 3, height: 4 });
    });
});
