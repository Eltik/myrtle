import { describe, expect, it } from "vitest";
import { conditionInk, FALLBACK_SPRITE, fromHex, isExcludedInk, quantise, rgbToHsl, spriteInksFromPixels, toHex } from "./palette";

/** A flat RGBA block of `n` pixels of one colour at one alpha. */
function block(rgb: [number, number, number], n: number, alpha = 255): number[] {
    const out: number[] = [];
    for (let i = 0; i < n; i++) out.push(rgb[0], rgb[1], rgb[2], alpha);
    return out;
}

function pixels(...blocks: number[][]): Uint8ClampedArray {
    return new Uint8ClampedArray(blocks.flat());
}

describe("the sprite alpha floor", () => {
    it("keeps the cover's own floor for the cover, and drops the silhouette rim for a sprite", () => {
        // 200 opaque red, 200 at alpha 200: the second block is the antialiased
        // rim a body has and a cover does not.
        const data = pixels(block([200, 40, 40], 200), block([40, 40, 200], 200, 200));
        // The cover default (128) admits both.
        expect(quantise(data)).toHaveLength(2);
        // The sprite floor (250) admits only the opaque one.
        expect(quantise(data, 250)).toHaveLength(1);
    });
});

describe("the excluded bands", () => {
    it("drops near-white and near-black whatever the hue", () => {
        expect(isExcludedInk([250, 250, 252])).toBe(true);
        expect(isExcludedInk([8, 8, 10])).toBe(true);
    });

    it("drops a skin bin and keeps a saturated orange coat and a dark tan", () => {
        // hue 27.7, sat 0.46, lightness 0.72: inside the band.
        expect(isExcludedInk(fromHex("#e5c0a0"))).toBe(true);
        // hue 27.4, sat 0.90, lightness 0.50: too saturated to be skin at that lightness.
        expect(isExcludedInk(fromHex("#f27a0d"))).toBe(false);
        // hue 27.7, sat 0.46, lightness 0.36: too dark to be skin.
        expect(isExcludedInk(fromHex("#7a5030"))).toBe(false);
        // A warm grey is stone or cloth, not a face: under the saturation floor.
        expect(isExcludedInk(fromHex("#a49d9a"))).toBe(false);
    });

    it("catches the PALE skin the first band missed, which is the whole reason it was retuned", () => {
        // Dobermann's face, lightness 0.896 and saturation 0.623: over the old
        // 0.85 ceiling AND over the old 0.6 cap, so the old band kept it.
        expect(isExcludedInk(fromHex("#f5e5d4"))).toBe(true);
        // The npc's, lightness 0.861 at saturation 0.915.
        expect(isExcludedInk(fromHex("#fcd5bb"))).toBe(true);
    });

    it("keeps every one of them when the caller asks for the unfiltered run", () => {
        expect(isExcludedInk(fromHex("#e5c0a0"), false)).toBe(false);
        expect(isExcludedInk([250, 250, 252], false)).toBe(true);
    });
});

describe("spriteInksFromPixels", () => {
    it("ranks by population after the exclusions, so the skin-dominant sprite prints its coat", () => {
        // 600 skin, 300 blue coat, 150 red hair: skin is the most populous bin
        // and would be c1 with no exclusion.
        const data = pixels(block(fromHex("#e5c0a0"), 600), block(fromHex("#2f4f8f"), 300), block(fromHex("#a03040"), 150));
        const withSkin = spriteInksFromPixels(data, false);
        const withoutSkin = spriteInksFromPixels(data, true);
        expect(withSkin.c1).toBe(toHex(conditionInk(fromHex("#e5c0a0"))));
        expect(withoutSkin.c1).toBe(toHex(conditionInk(fromHex("#2f4f8f"))));
        expect(withoutSkin.c2).toBe(toHex(conditionInk(fromHex("#a03040"))));
    });

    it("falls back rather than inking a body with nothing left after the exclusions", () => {
        expect(spriteInksFromPixels(pixels(block([255, 255, 255], 100), block([2, 2, 2], 100)))).toEqual(FALLBACK_SPRITE);
    });

    it("conditions every ink into the legible band the ticket uses", () => {
        const p = spriteInksFromPixels(pixels(block(fromHex("#0d1a33"), 400), block(fromHex("#123a12"), 200)));
        for (const hex of [p.c1, p.c2, p.c3]) {
            const [, , l] = rgbToHsl(fromHex(hex));
            expect(l).toBeGreaterThanOrEqual(0.22 - 1e-3);
            expect(l).toBeLessThanOrEqual(0.7 + 1e-3);
        }
    });

    it("ignores fully transparent pixels entirely, which is most of a body plate", () => {
        const data = pixels(block([0, 0, 0], 3000, 0), block(fromHex("#2f4f8f"), 100));
        expect(spriteInksFromPixels(data).c1).toBe(toHex(conditionInk(fromHex("#2f4f8f"))));
    });
});
