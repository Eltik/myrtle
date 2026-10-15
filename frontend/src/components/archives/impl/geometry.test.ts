import { describe, expect, it } from "vitest";
import { directionVector, hexColor, outlineSegments, toSvg, zoomBox } from "./geometry";

describe("archives geometry", () => {
    it("points each direction where the tables put the neighbour", () => {
        // Rhodes Island's centre hex and its six neighbours, Unity space.
        const neighbours: Record<number, [number, number]> = { 0: [-75, 43.3], 1: [0, 86.6], 2: [75, 43.3], 3: [75, -43.3], 4: [0, -86.6], 5: [-75, -43.3] };
        for (const [d, [x, y]] of Object.entries(neighbours)) {
            const v = directionVector(Number(d));
            const want = toSvg({ x, y });
            const len = Math.hypot(want.x, want.y);
            expect(v.x).toBeCloseTo(want.x / len, 3);
            expect(v.y).toBeCloseTo(want.y / len, 3);
        }
    });

    it("draws only the outside of a cluster", () => {
        const one = outlineSegments([{ x: 0, y: 0 }], 50);
        expect(one).toHaveLength(6);
        // Two hexes side by side share one side, which is drawn by neither.
        const two = outlineSegments(
            [
                { x: 0, y: 0 },
                { x: 0, y: 86.6 },
            ],
            50,
        );
        expect(two).toHaveLength(10);
    });

    it("zooms around the anchor", () => {
        const box = zoomBox({ x: 0, y: 0, w: 100, h: 100 }, 0.5, { x: 100, y: 100 });
        expect(box).toEqual({ x: 50, y: 50, w: 50, h: 50 });
    });

    it("reads the tables' colours", () => {
        expect(hexColor("0098DC")).toBe("#0098DC");
        expect(hexColor(null, "#000000")).toBe("#000000");
        expect(hexColor("nope", "#111111")).toBe("#111111");
    });
});
