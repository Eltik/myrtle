import { describe, expect, it } from "vitest";
import { fitTitle, type IFitBounds } from "./titleFit";

/** A stand-in measurer: every glyph is 0.6 em wide, so a line's width is its length times 0.6. */
const even = (text: string): number => text.length * 0.6;

/** The ticket's own bounds: a 216 px box (72% of the 300 px card the grid bottoms out at), 32 px down to a 21 px floor, 2 lines then 3. */
const CARD: IFitBounds = { box: 216, max: 32, min: 21, lines: 2, floorLines: 3 };

describe("fitTitle", () => {
    it("sets a short title at the ceiling on one line", () => {
        const fit = fitTitle("Stormwatch", even, CARD);
        expect(fit.size).toBe(32);
        expect(fit.lines).toEqual(["Stormwatch"]);
        expect(fit.breakAnywhere).toBe(false);
    });

    it("keeps TREASURE, which is the case the reviewer named: no line is a truncation", () => {
        const fit = fitTitle("Grani and the Knights' Treasure", even, CARD);
        expect(fit.lines.join(" ")).toBe("Grani and the Knights' Treasure");
        expect(fit.lines.at(-1)).toContain("Treasure");
        expect(fit.size).toBeGreaterThanOrEqual(21);
        expect(fit.size).toBeLessThan(32);
    });

    it("SHRINKS before it takes a line: Grani needs 4 lines at the ceiling and is set on 2 smaller instead", () => {
        expect(fitTitle("Grani and the Knights' Treasure", even, { ...CARD, max: 32, min: 32 }).lines.length).toBeGreaterThan(2);
        const fit = fitTitle("Grani and the Knights' Treasure", even, CARD);
        expect(fit.lines).toHaveLength(2);
        expect(fit.size).toBeLessThan(32);
        // The stand-in measurer is 0.6 em a glyph where Inter at 800 averages 0.55,
        // so Grani lands close to the floor here and above it in the browser.
        expect(fit.size).toBeGreaterThanOrEqual(21);
    });

    it("keeps a ONE-WORD title inside the box, which a line count alone never catches", () => {
        // SOMNILOQUIUM wraps to one line at every size, so only the width test shrinks it.
        const fit = fitTitle("Somniloquium", even, CARD);
        expect(fit.lines).toEqual(["Somniloquium"]);
        expect(even("Somniloquium") * fit.size).toBeLessThanOrEqual(216);
    });

    it("takes the third line only at the floor, never above it", () => {
        const fit = fitTitle("Come Catastrophes or Wakes of Vultures", even, CARD);
        expect(fit.size).toBe(21);
        expect(fit.lines).toHaveLength(3);
        expect(fit.breakAnywhere).toBe(false);
    });

    it("never breaks a word that fits: Zwillingstürme sets whole at or above the floor", () => {
        const fit = fitTitle("Zwillingstürme im Herbst", even, CARD);
        expect(fit.lines.some((l) => l.includes("Zwillingstürme"))).toBe(true);
        expect(fit.breakAnywhere).toBe(false);
        expect(fit.size).toBeGreaterThanOrEqual(21);
    });

    it("licenses `overflow-wrap: anywhere` ONLY when one word still overflows at the floor", () => {
        // 14 glyphs at 0.6 em is 8.4 em, which is 176.4 px at the 21 px floor: wider than a 60 px box.
        expect(fitTitle("Zwillingstürme", even, { ...CARD, box: 60 }).breakAnywhere).toBe(true);
        expect(fitTitle("Zwillingstürme", even, CARD).breakAnywhere).toBe(false);
    });

    it("wraps at whole words, so no line is a fragment of one", () => {
        const fit = fitTitle("Operation Lucent Arrowhead", even, CARD);
        for (const line of fit.lines) expect(line).not.toMatch(/^\s|\s$/);
        expect(fit.lines.join(" ")).toBe("Operation Lucent Arrowhead");
    });

    it("answers the ceiling and no lines for a blank title rather than dividing by a zero width", () => {
        expect(fitTitle("   ", even, CARD)).toEqual({ lines: [], size: 32, breakAnywhere: false });
    });
});
