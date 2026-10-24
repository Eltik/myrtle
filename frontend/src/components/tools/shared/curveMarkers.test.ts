import { describe, expect, it } from "vitest";
import { curveMarkersFor, findBreakPoint, findHalfPoint } from "./curveMarkers";

/** One physical hit: max(atk - def, 5% atk). With atk 1000 the floor (50) starts at DEF 950. */
function physical(atk: number, defs: number[]) {
    return defs.map((x) => ({ x, y: Math.max(atk - x, 0.05 * atk) }));
}

const DEFS = Array.from({ length: 21 }, (_, i) => i * 100);

describe("findBreakPoint", () => {
    it("recovers the exact floor DEF between two samples", () => {
        expect(findBreakPoint(physical(1000, DEFS))).toEqual({ x: 950, y: 50 });
    });

    it("lands on a sample when the floor starts exactly there", () => {
        // atk 1000 / 0.95 floors at DEF 1000 exactly: 0.95 * atk = 1000.
        const atk = 1000 / 0.95;
        const p = findBreakPoint(physical(atk, DEFS));
        expect(p?.x).toBeCloseTo(1000, 9);
        expect(p?.y).toBeCloseTo(0.05 * atk, 9);
    });

    it("shows nothing when the floor is not reached in range", () => {
        expect(findBreakPoint(physical(5000, DEFS))).toBeNull();
    });

    it("shows nothing for a curve that never falls", () => {
        expect(findBreakPoint(DEFS.map((x) => ({ x, y: 1234 })))).toBeNull();
    });

    it("needs two flat samples: one trailing flat sample could be a shallower slope", () => {
        // atk 2000: floor 100 at DEF 1900; the last two samples are 1900 -> 100 and 2000 -> 100.
        expect(findBreakPoint(physical(2000, DEFS))?.x).toBe(1900);
        // atk 2050: the floor is only hit at the very last sample.
        expect(findBreakPoint(physical(2050, DEFS))).toBeNull();
    });

    it("uses the floor sample itself when there is no earlier segment to extend", () => {
        expect(
            findBreakPoint([
                { x: 0, y: 100 },
                { x: 100, y: 5 },
                { x: 200, y: 5 },
            ]),
        ).toEqual({ x: 100, y: 5 });
    });
});

describe("findHalfPoint", () => {
    it("interpolates between the straddling samples", () => {
        // 1000 - x falls to 500 at DEF 500.
        expect(findHalfPoint(physical(1000, DEFS))).toEqual({ x: 500, y: 500 });
        const p = findHalfPoint([
            { x: 0, y: 100 },
            { x: 10, y: 80 },
            { x: 20, y: 40 },
        ]);
        expect(p?.x).toBeCloseTo(17.5, 9);
        expect(p?.y).toBe(50);
    });

    it("measures from the value at the sweep's start, not at zero", () => {
        const pts = physical(1000, DEFS).filter((p) => p.x >= 200);
        // start 800 at DEF 200, half 400 at DEF 600.
        expect(findHalfPoint(pts)).toEqual({ x: 600, y: 400 });
    });

    it("shows nothing when the curve never halves in range or starts at zero", () => {
        expect(findHalfPoint(physical(10_000, DEFS))).toBeNull();
        expect(
            findHalfPoint([
                { x: 0, y: 0 },
                { x: 1, y: 0 },
            ]),
        ).toBeNull();
    });
});

describe("curveMarkersFor", () => {
    it("reads one series out of chart rows and gives nothing for a partial curve", () => {
        const rows = physical(1000, DEFS).map(({ x, y }) => ({ x, a: y, b: x === 500 ? null : y }));
        expect(curveMarkersFor(rows, "a")).toEqual({ breakPoint: { x: 950, y: 50 }, halfPoint: { x: 500, y: 500 } });
        expect(curveMarkersFor(rows, "b")).toEqual({ breakPoint: null, halfPoint: null });
    });
});
