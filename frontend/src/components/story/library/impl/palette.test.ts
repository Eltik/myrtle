import { describe, expect, it } from "vitest";
import { badgeTextFor, clampLightness, contrastRatio, floorSaturation, fromHex, ON_ACCENT_DARK, padInk, paletteFromPixels, pickTriad, quantise, rgbDistance, rgbToHsl, SAMPLE_H, SAMPLE_W, toHex } from "./palette";

/** A synthetic cover: `bands` are [colour, share of the pixels] in order, painted row by row. */
function synthetic(bands: readonly [[number, number, number], number][], alpha = 255): Uint8ClampedArray {
    const total = SAMPLE_W * SAMPLE_H;
    const data = new Uint8ClampedArray(total * 4);
    let at = 0;
    bands.forEach(([rgb, share], index) => {
        const last = index === bands.length - 1;
        const n = last ? total - at : Math.round(total * share);
        for (let i = 0; i < n; i += 1) {
            const p = (at + i) * 4;
            data[p] = rgb[0];
            data[p + 1] = rgb[1];
            data[p + 2] = rgb[2];
            data[p + 3] = alpha;
        }
        at += n;
    });
    return data;
}

const RED: [number, number, number] = [200, 60, 60];
const BLUE: [number, number, number] = [60, 160, 200];
const GREEN: [number, number, number] = [120, 200, 80];

describe("quantise", () => {
    it("ranks bins by population and averages their members", () => {
        const bins = quantise(
            synthetic([
                [RED, 0.6],
                [BLUE, 0.25],
                [GREEN, 0.15],
            ]),
        );
        expect(bins).toHaveLength(3);
        expect(bins[0]?.rgb).toEqual(RED);
        expect(bins[0]?.count).toBe(Math.round(SAMPLE_W * SAMPLE_H * 0.6));
        expect(bins[1]?.rgb).toEqual(BLUE);
        expect(bins[2]?.rgb).toEqual(GREEN);
    });

    it("folds colours that differ below the 5-bit step into one bin", () => {
        // 200 and 204 both land in bin 25 (200 >> 3 === 204 >> 3), so one bin holds both and carries their mean.
        const bins = quantise(
            synthetic([
                [[200, 60, 60], 0.5],
                [[204, 60, 60], 0.5],
            ]),
        );
        expect(bins).toHaveLength(1);
        expect(bins[0]?.rgb[0]).toBe(202);
    });

    it("ignores pixels under the alpha floor", () => {
        expect(quantise(synthetic([[RED, 1]], 100))).toHaveLength(0);
    });
});

describe("clampLightness", () => {
    it("leaves a colour already inside 0.22..0.7 untouched", () => {
        expect(clampLightness(RED)).toEqual(RED);
    });

    it("lifts a near-black sample to lightness 0.22 without moving its hue", () => {
        const before = rgbToHsl([10, 10, 40]);
        const after = rgbToHsl(clampLightness([10, 10, 40]));
        expect(before[2]).toBeCloseTo(0.098, 3);
        expect(after[2]).toBeCloseTo(0.22, 2);
        expect(after[0]).toBeCloseTo(before[0], 1);
    });

    it("drops a near-white sample to lightness 0.7", () => {
        expect(rgbToHsl(clampLightness([250, 246, 240]))[2]).toBeCloseTo(0.7, 2);
    });
});

describe("pickTriad", () => {
    it("takes the three most populous inks, in population order", () => {
        expect(
            pickTriad(
                quantise(
                    synthetic([
                        [RED, 0.6],
                        [BLUE, 0.25],
                        [GREEN, 0.15],
                    ]),
                ),
            ),
        ).toEqual([toHex(RED), toHex(BLUE), toHex(GREEN)]);
    });

    it("skips a second bin that is the same ink as the first", () => {
        // 200,60,60 and 196,64,64 are 2.2 degrees and 0.008 lightness apart: one ink, two bins.
        const triad = pickTriad(
            quantise(
                synthetic([
                    [[200, 60, 60], 0.5],
                    [[196, 64, 64], 0.3],
                    [BLUE, 0.2],
                ]),
            ),
        );
        expect(triad[1]).toBe(toHex(BLUE));
    });

    it("pads a monochrome cover by rotating the hue rather than returning one flat band", () => {
        const triad = pickTriad(quantise(synthetic([[RED, 1]])));
        expect(triad[0]).toBe(toHex(RED));
        expect(new Set(triad).size).toBe(3);
        expect(rgbToHsl(fromHex(triad[1] as string))[0]).toBeCloseTo(40, 0);
    });

    it("falls back to the accent triad when nothing is opaque enough to sample", () => {
        expect(pickTriad([])).toEqual(["#df202e", "#008c7d", "#006088"]);
    });
});

describe("floorSaturation", () => {
    it("lifts a low-chroma ink to 0.32 and keeps its hue and lightness", () => {
        // #35353b, sampled from chapter MAIN_1's cover on the running page: saturation 0.054.
        const before = rgbToHsl([53, 53, 59]);
        const after = rgbToHsl(floorSaturation([53, 53, 59]));
        expect(before[1]).toBeCloseTo(0.054, 3);
        expect(after[1]).toBeCloseTo(0.32, 2);
        expect(after[0]).toBeCloseTo(before[0], 0);
        expect(after[2]).toBeCloseTo(before[2], 2);
    });

    it("leaves a colour already above the floor alone", () => {
        expect(floorSaturation(RED)).toEqual(RED);
    });

    it("leaves an achromatic bin grey rather than inventing a hue for it", () => {
        expect(floorSaturation([56, 56, 56])).toEqual([56, 56, 56]);
    });
});

describe("pickTriad hue gate", () => {
    it("reads two near-greys of the same lightness as ONE ink, because hue is noise below 0.12 saturation", () => {
        // Before the gate these two were accepted as distinct on a 150-degree hue difference
        // that is three RGB steps of sensor noise.
        const triad = pickTriad(
            quantise(
                synthetic([
                    [[53, 53, 59], 0.5],
                    [[57, 55, 57], 0.3],
                    [BLUE, 0.2],
                ]),
            ),
        );
        expect(triad[1]).toBe(toHex(BLUE));
    });

    it("still separates two saturated inks on hue alone", () => {
        const triad = pickTriad(
            quantise(
                synthetic([
                    [RED, 0.5],
                    [[60, 200, 90], 0.3],
                    [BLUE, 0.2],
                ]),
            ),
        );
        expect(new Set(triad).size).toBe(3);
        expect(triad[0]).toBe(toHex(RED));
    });
});

describe("badgeTextFor", () => {
    it("puts white on a dark fill and the site's dark on-accent on a light one", () => {
        expect(badgeTextFor("#df202e")).toBe("#ffffff");
        expect(badgeTextFor("#f7e79e")).toBe(ON_ACCENT_DARK);
    });

    it("picks whichever of the two actually has the higher ratio", () => {
        const fill = "#7a8a3d";
        const chosen = badgeTextFor(fill);
        const other = chosen === "#ffffff" ? ON_ACCENT_DARK : "#ffffff";
        expect(contrastRatio(fromHex(fill), fromHex(chosen))).toBeGreaterThanOrEqual(contrastRatio(fromHex(fill), fromHex(other)));
    });
});

describe("paletteFromPixels", () => {
    it("reads a synthetic cover end to end", () => {
        expect(
            paletteFromPixels(
                synthetic([
                    [RED, 0.6],
                    [BLUE, 0.25],
                    [GREEN, 0.15],
                ]),
            ),
        ).toEqual({ c1: "#c83c3c", c2: "#3ca0c8", c3: "#78c850", badgeText: "#ffffff" });
    });
});

describe("padInk", () => {
    it("moves an ACHROMATIC dominant ink off itself, which a hue rotation alone cannot", () => {
        // hslToRgb returns the same grey for every hue at saturation 0, so the
        // old pad (hue + 40 at the base saturation) was a no-op and Stormwatch
        // printed #383838 / #383838 / #383838.
        const grey: [number, number, number] = [56, 56, 56];
        const one = padInk(grey, 1);
        const two = padInk(grey, 2);
        expect(rgbDistance(grey, one)).toBeGreaterThanOrEqual(24);
        expect(rgbDistance(grey, two)).toBeGreaterThanOrEqual(24);
        expect(rgbDistance(one, two)).toBeGreaterThanOrEqual(24);
    });

    it("is deterministic", () => {
        expect(padInk([56, 56, 56], 1)).toEqual(padInk([56, 56, 56], 1));
    });
});

describe("pickTriad separation", () => {
    it("prints three inks a hard stripe boundary can be seen across, on a flat black cover", () => {
        const triad = pickTriad(quantise(synthetic([[[8, 8, 8], 1]])));
        const rgb = triad.map(fromHex) as [number, number, number][];
        expect(rgbDistance(rgb[0] as [number, number, number], rgb[1] as [number, number, number])).toBeGreaterThanOrEqual(24);
        expect(rgbDistance(rgb[1] as [number, number, number], rgb[2] as [number, number, number])).toBeGreaterThanOrEqual(24);
        expect(rgbDistance(rgb[2] as [number, number, number], rgb[0] as [number, number, number])).toBeGreaterThanOrEqual(24);
    });

    it("drops a bin that CONDITIONING flattens onto an accepted ink, not only one the raw rule rejects", () => {
        // Two near-black browns 16 RGB units apart in lightness: the raw rule
        // admits the second, and clampLightness then pulls both to 0.22.
        const triad = pickTriad(
            quantise(
                synthetic([
                    [[26, 17, 13], 0.4],
                    [[10, 7, 5], 0.35],
                    [[60, 140, 200], 0.25],
                ]),
            ),
        );
        const rgb = triad.map(fromHex) as [number, number, number][];
        for (const [a, b] of [
            [0, 1],
            [1, 2],
            [2, 0],
        ] as const) {
            expect(rgbDistance(rgb[a] as [number, number, number], rgb[b] as [number, number, number])).toBeGreaterThanOrEqual(24);
        }
    });

    it("leaves a three-ink cover exactly as it was", () => {
        expect(
            pickTriad(
                quantise(
                    synthetic([
                        [RED, 0.6],
                        [BLUE, 0.25],
                        [GREEN, 0.15],
                    ]),
                ),
            ),
        ).toEqual([toHex(RED), toHex(BLUE), toHex(GREEN)]);
    });
});
