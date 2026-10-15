import { describe, expect, it } from "vitest";
import { categoryTicks, endAnchoredTicks, labelStride, timeTicks } from "./ticks";

describe("endAnchoredTicks", () => {
    it("always keeps the last value and steps back from it", () => {
        expect(endAnchoredTicks([0, 2, 4, 6, 8, 10, 12], 3)).toEqual([0, 6, 12]);
        expect(endAnchoredTicks([0, 2, 4, 6, 8, 10], 4)).toEqual([2, 10]);
    });

    it("keeps every value at stride 1 and treats a bad stride as 1", () => {
        expect(endAnchoredTicks([1, 2, 3], 1)).toEqual([1, 2, 3]);
        expect(endAnchoredTicks([1, 2, 3], 0)).toEqual([1, 2, 3]);
    });

    it("returns nothing for no values", () => {
        expect(endAnchoredTicks([], 5)).toEqual([]);
    });
});

describe("labelStride", () => {
    it("spaces labels by their width plus the 30 px gap", () => {
        // 151 rows over a 1,040 px plot: 6.93 px a row; a 3-digit label needs 49.5 px, so 8 rows.
        expect(labelStride(151, 1080, 3)).toBe(8);
    });

    it("widens the stride as the chart narrows", () => {
        expect(labelStride(151, 360, 3)).toBeGreaterThan(labelStride(151, 1080, 3));
    });

    it("never goes below 1, even for one row", () => {
        expect(labelStride(1, 500, 4)).toBe(1);
        expect(labelStride(3, 5000, 1)).toBe(1);
    });
});

describe("categoryTicks", () => {
    it("labels the odds curve's 0..300 in steps that end on 300", () => {
        const pulls = Array.from({ length: 151 }, (_, i) => i * 2);
        const ticks = categoryTicks(pulls, 1080);
        expect(ticks.at(-1)).toBe(300);
        expect(ticks.length).toBeGreaterThan(10);
        expect(ticks.length).toBeLessThan(30);
        for (const v of ticks) expect(pulls).toContain(v);
    });
});

describe("timeTicks", () => {
    const DAY = 86_400_000;
    const start = Date.UTC(2026, 9, 14);

    it("ends on the last day and steps back in whole days", () => {
        const end = start + 180 * DAY;
        const ticks = timeTicks(start, end, 1134, 12);
        expect(ticks.at(-1)).toBe(end);
        expect(ticks[0]).toBeGreaterThanOrEqual(start);
        const step = ticks[1] - ticks[0];
        expect(step % DAY).toBe(0);
        for (let i = 1; i < ticks.length; i++) expect(ticks[i] - ticks[i - 1]).toBe(step);
    });

    it("thins out on a narrow chart", () => {
        const end = start + 180 * DAY;
        expect(timeTicks(start, end, 360, 12).length).toBeLessThan(timeTicks(start, end, 1134, 12).length);
    });

    it("gives the one point of a single-day range", () => {
        expect(timeTicks(start, start, 800, 12)).toEqual([start]);
    });
});
