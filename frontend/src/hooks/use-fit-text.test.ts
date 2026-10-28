import { describe, expect, it } from "vitest";
import { fitTextScale } from "./use-fit-text";

describe("fitTextScale", () => {
    it("keeps the size when the text fits", () => {
        expect(fitTextScale(120, 80)).toBe(1);
        expect(fitTextScale(120, 120)).toBe(1);
    });

    it("shrinks an overflowing word to the available width, with slack", () => {
        // "Inconsequential" at 16px in a ~110px label: ~132px wide.
        const scale = fitTextScale(110, 132);
        expect(scale).toBeLessThan(110 / 132);
        expect(scale * 132).toBeGreaterThan(105);
    });

    it("stops at the minimum scale and leaves the rest to overflow-wrap", () => {
        expect(fitTextScale(100, 400)).toBe(0.55);
        expect(fitTextScale(100, 400, 0.7)).toBe(0.7);
    });

    it("keeps the size when nothing was measured", () => {
        expect(fitTextScale(0, 100)).toBe(1);
        expect(fitTextScale(100, 0)).toBe(1);
    });
});
