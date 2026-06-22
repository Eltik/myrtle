import { describe, expect, it } from "vitest";
import type { IBoardCell } from "#/lib/api/level-adapter";
import { bfs, type Pixel, pointAtFraction, polylineLength, rangeTiles, rerouteAround, routeTiles, toPixelPolyline } from "./sim";

describe("polyline geometry", () => {
    const pts: Pixel[] = [
        { x: 0, y: 0 },
        { x: 10, y: 0 },
        { x: 10, y: 10 },
    ];

    it("measures total length", () => {
        expect(polylineLength(pts)).toBe(20);
    });

    it("interpolates endpoints and midpoint", () => {
        expect(pointAtFraction(pts, 0)).toEqual({ x: 0, y: 0 });
        expect(pointAtFraction(pts, 1)).toEqual({ x: 10, y: 10 });
        // Halfway along a 20-unit path lands at the first corner.
        expect(pointAtFraction(pts, 0.5)).toEqual({ x: 10, y: 0 });
    });

    it("clamps out-of-range fractions", () => {
        expect(pointAtFraction(pts, -1)).toEqual({ x: 0, y: 0 });
        expect(pointAtFraction(pts, 2)).toEqual({ x: 10, y: 10 });
    });

    it("converts tile coords to pixel centers", () => {
        expect(
            toPixelPolyline(
                [
                    { x: 0, y: 0 },
                    { x: 2, y: 0 },
                ],
                52,
            ),
        ).toEqual([
            { x: 26, y: 26 },
            { x: 130, y: 26 },
        ]);
    });
});

describe("routeTiles", () => {
    it("covers every tile along a straight segment", () => {
        const tiles = routeTiles([
            { x: 0, y: 0 },
            { x: 3, y: 0 },
        ]);
        expect(tiles).toEqual([
            { x: 0, y: 0 },
            { x: 1, y: 0 },
            { x: 2, y: 0 },
            { x: 3, y: 0 },
        ]);
    });
});

// A 3-wide, 2-tall passable grid:  (0,0)(1,0)(2,0) / (0,1)(1,1)(2,1)
function makeGrid(): Record<string, IBoardCell> {
    const grid: Record<string, IBoardCell> = {};
    for (let y = 0; y < 2; y++) {
        for (let x = 0; x < 3; x++) {
            grid[`${x},${y}`] = { x, y, kind: "road", passable: true, deployable: "NONE", heightType: "" };
        }
    }
    return grid;
}

describe("bfs / rerouteAround", () => {
    it("finds a straight path when unobstructed", () => {
        const path = bfs(makeGrid(), { x: 0, y: 0 }, { x: 2, y: 0 }, new Set());
        expect(path).toEqual([
            { x: 0, y: 0 },
            { x: 1, y: 0 },
            { x: 2, y: 0 },
        ]);
    });

    it("detours around a blocked tile", () => {
        // Block (1,0): the only way across is down through row 1.
        const path = bfs(makeGrid(), { x: 0, y: 0 }, { x: 2, y: 0 }, new Set(["1,0"]));
        expect(path).not.toBeNull();
        expect(path?.some((p) => p.x === 1 && p.y === 1)).toBe(true);
    });

    it("rerouteAround leaves fliers untouched", () => {
        const original = [
            { x: 0, y: 0 },
            { x: 2, y: 0 },
        ];
        expect(rerouteAround(makeGrid(), original, "FLY", new Set(["1,0"]))).toBe(original);
    });

    it("rerouteAround detours walkers around blockers", () => {
        const rerouted = rerouteAround(
            makeGrid(),
            [
                { x: 0, y: 0 },
                { x: 2, y: 0 },
            ],
            "WALK",
            new Set(["1,0"]),
        );
        // the path dips down into row 1 to get around the blocked (1,0)
        expect(rerouted.some((p) => p.y === 1)).toBe(true);
        expect(rerouted[0]).toEqual({ x: 0, y: 0 });
        expect(rerouted[rerouted.length - 1]).toEqual({ x: 2, y: 0 });
    });
});

describe("rangeTiles", () => {
    it("projects a forward range facing right", () => {
        const hits = rangeTiles(
            [
                { col: 0, row: 0 },
                { col: 1, row: 0 },
            ],
            { x: 5, y: 5 },
            "right",
        );
        expect(hits).toContainEqual({ x: 5, y: 5, self: true });
        expect(hits).toContainEqual({ x: 6, y: 5, self: false });
    });

    it("rotates the range when facing up", () => {
        // (1,0) facing up rotates to (0,-1).
        const hits = rangeTiles([{ col: 1, row: 0 }], { x: 5, y: 5 }, "up");
        expect(hits).toContainEqual({ x: 5, y: 4, self: false });
    });
});
