import { describe, expect, it } from "vitest";
import { IMAGE_HEIGHT, IMAGE_WIDTH, type IRect, shareImageLayout } from "./shareImage";

const overlaps = (a: IRect, b: IRect) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
const inside = (r: IRect) => r.x >= 0 && r.y >= 0 && r.x + r.w <= IMAGE_WIDTH && r.y + r.h <= IMAGE_HEIGHT;

describe("shareImageLayout", () => {
    const layout = shareImageLayout();
    const boxes = [layout.title, ...layout.seats, layout.supportLabel, layout.support, layout.facts, layout.footer];

    it("lays out two rows of six seats", () => {
        expect(layout.seats).toHaveLength(12);
        expect(new Set(layout.seats.map((r) => r.y)).size).toBe(2);
        expect(layout.seats[6].x).toBe(layout.seats[0].x);
    });

    it("keeps every box on the canvas and apart from the others", () => {
        for (const r of boxes) expect(inside(r)).toBe(true);
        for (let i = 0; i < boxes.length; i++) for (let j = i + 1; j < boxes.length; j++) expect(overlaps(boxes[i], boxes[j])).toBe(false);
    });

    it("leaves room for the caption under the art", () => {
        expect(layout.seats[0].h - layout.artHeight).toBeGreaterThanOrEqual(56);
    });
});
