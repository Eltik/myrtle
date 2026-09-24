import { describe, expect, it } from "vitest";
import { contrastRatio, DARK_BOX_WORST, fitToBox, isHexColor, LIGHT_BOX_WORST, relativeLuminance, resolveTextColor, swatchColor, TEXT_SWATCHES } from "./colors";

describe("contrast", () => {
    it("is 21 between black and white and 1 against itself", () => {
        expect(contrastRatio("#000000", "#ffffff")).toBeCloseTo(21, 5);
        expect(contrastRatio("#4d4d4d", "#4d4d4d")).toBeCloseTo(1, 10);
    });

    it("does not depend on the order of its arguments", () => {
        expect(contrastRatio("#a8d8f5", DARK_BOX_WORST)).toBeCloseTo(contrastRatio(DARK_BOX_WORST, "#a8d8f5"), 10);
    });

    it("reads shorthand hex and treats junk as black", () => {
        expect(relativeLuminance("#fff")).toBeCloseTo(relativeLuminance("#ffffff"), 10);
        expect(relativeLuminance("nonsense")).toBe(0);
    });
});

describe("curated swatches", () => {
    it("has eight pairs and every dark tint clears 4.5:1 on the dark box at its lightest", () => {
        expect(TEXT_SWATCHES).toHaveLength(8);
        for (const s of TEXT_SWATCHES) expect(contrastRatio(s.dark, DARK_BOX_WORST)).toBeGreaterThanOrEqual(4.5);
    });

    it("every light tint clears 4.5:1 on the light box at its darkest", () => {
        for (const s of TEXT_SWATCHES) expect(contrastRatio(s.light, LIGHT_BOX_WORST)).toBeGreaterThanOrEqual(4.5);
    });

    // The refutation this file records: no single hex can do both, so the row is surface-aware.
    it("cannot be one hex per swatch: the two luminance bounds do not overlap", () => {
        const needAtLeast = 4.5 * (relativeLuminance(DARK_BOX_WORST) + 0.05) - 0.05;
        const needAtMost = (relativeLuminance(LIGHT_BOX_WORST) + 0.05) / 4.5 - 0.05;
        expect(needAtLeast).toBeGreaterThan(needAtMost);
    });
});

describe("resolveTextColor", () => {
    it("returns undefined for the default, which is the kill switch", () => {
        expect(resolveTextColor("", false)).toBeUndefined();
        expect(resolveTextColor("", true)).toBeUndefined();
    });

    it("picks the surface's tint for a swatch id", () => {
        expect(resolveTextColor("s3", false)).toBe(TEXT_SWATCHES[3].dark);
        expect(resolveTextColor("s3", true)).toBe(TEXT_SWATCHES[3].light);
        expect(swatchColor(TEXT_SWATCHES[3], true)).toBe(TEXT_SWATCHES[3].light);
    });

    it("passes a custom hex through on both surfaces and drops anything else", () => {
        expect(resolveTextColor("#ff00aa", false)).toBe("#ff00aa");
        expect(resolveTextColor("#ff00aa", true)).toBe("#ff00aa");
        expect(resolveTextColor("s99", false)).toBeUndefined();
        expect(resolveTextColor("red", false)).toBeUndefined();
    });

    it("accepts the hex shapes the colour input produces", () => {
        expect(isHexColor("#abc")).toBe(true);
        expect(isHexColor("#AABBCC")).toBe(true);
        expect(isHexColor("#abcd")).toBe(false);
    });
});

describe("fitToBox", () => {
    const sampled = ["#2f4f8f", "#a03040", "#3f8f4f", "#8f7a2f", "#5a2e8a", "#0d1a33", "#e5c0a0"];

    it("clears 4.5:1 on the surface it was fitted to, both ways round", () => {
        for (const hex of sampled) {
            expect(contrastRatio(fitToBox(hex, false), DARK_BOX_WORST)).toBeGreaterThanOrEqual(4.5);
            expect(contrastRatio(fitToBox(hex, true), LIGHT_BOX_WORST)).toBeGreaterThanOrEqual(4.5);
        }
    });

    it("moves the ink AWAY from the box: lighter on the dark box, darker on the light one", () => {
        for (const hex of sampled) {
            expect(relativeLuminance(fitToBox(hex, false))).toBeGreaterThanOrEqual(relativeLuminance(hex));
            expect(relativeLuminance(fitToBox(hex, true))).toBeLessThanOrEqual(relativeLuminance(hex));
        }
    });

    it("leaves an ink that already reads alone", () => {
        // #ffffff is 8.453:1 on the dark box, so the walk stops on the first step.
        expect(fitToBox("#ffffff", false)).toBe("#ffffff");
    });

    it("hands junk back untouched for the caller to drop", () => {
        expect(fitToBox("nonsense", false)).toBe("nonsense");
    });
});
