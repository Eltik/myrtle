import { describe, expect, it } from "vitest";
import { fitTitle, TICKET_FIT, TICKET_FIT_PHONE } from "./titleFit";

/** A stand-in measurer: every glyph is 0.6 em wide, so a line's width is its length times 0.6. */
const even = (text: string): number => text.length * 0.6;

/** The ticket's own bounds: a 144 px box (the content box of the 200 px card the grid bottoms out at), 21 px down to a 14 px floor, 2 lines then 3. */
const CARD = TICKET_FIT;

describe("fitTitle", () => {
    it("sets a short title at the ceiling on one line", () => {
        const fit = fitTitle("Stormwatch", even, CARD);
        expect(fit.size).toBe(21);
        expect(fit.lines).toEqual(["Stormwatch"]);
        expect(fit.breakAnywhere).toBe(false);
    });

    it("keeps TREASURE, which is the case the reviewer named: no line is a truncation", () => {
        const fit = fitTitle("Grani and the Knights' Treasure", even, CARD);
        expect(fit.lines.join(" ")).toBe("Grani and the Knights' Treasure");
        expect(fit.lines.at(-1)).toContain("Treasure");
        expect(fit.size).toBeGreaterThanOrEqual(14);
        expect(fit.size).toBeLessThan(21);
    });

    it("SHRINKS before it takes a line: Grani needs 4 lines at the ceiling and is set on 2 smaller instead", () => {
        expect(fitTitle("Grani and the Knights' Treasure", even, { ...CARD, min: CARD.max }).lines.length).toBeGreaterThan(2);
        const fit = fitTitle("Grani and the Knights' Treasure", even, CARD);
        expect(fit.lines).toHaveLength(2);
        expect(fit.size).toBeLessThan(21);
        // The stand-in measurer is 0.6 em a glyph where Inter at 800 averages 0.55,
        // so Grani lands on the floor here and above it in the browser.
        expect(fit.size).toBeGreaterThanOrEqual(14);
    });

    it("keeps a ONE-WORD title inside the box, which a line count alone never catches", () => {
        // SOMNILOQUIUM wraps to one line at every size, so only the width test shrinks it.
        const fit = fitTitle("Somniloquium", even, CARD);
        expect(fit.lines).toEqual(["Somniloquium"]);
        expect(even("Somniloquium") * fit.size).toBeLessThanOrEqual(144);
        // 7.2 em at the 21 px ceiling is 151.2 px, so the width test is what shrank it, to 20.
        expect(fit.size).toBe(20);
    });

    it("takes the third line only at the floor, never above it", () => {
        const fit = fitTitle("Come Catastrophes or Wakes of Vultures", even, CARD);
        expect(fit.size).toBe(14);
        expect(fit.lines).toHaveLength(3);
        expect(fit.breakAnywhere).toBe(false);
    });

    it("never breaks a word that fits: Zwillingstürme sets whole at or above the floor", () => {
        const fit = fitTitle("Zwillingstürme im Herbst", even, CARD);
        expect(fit.lines.some((l) => l.includes("Zwillingstürme"))).toBe(true);
        expect(fit.breakAnywhere).toBe(false);
        expect(fit.size).toBeGreaterThanOrEqual(14);
    });

    it("licenses `overflow-wrap: anywhere` ONLY when one word still overflows at the floor", () => {
        // 14 glyphs at 0.6 em is 8.4 em, which is 117.6 px at the 14 px floor: wider than a 60 px box.
        expect(fitTitle("Zwillingstürme", even, { ...CARD, box: 60 }).breakAnywhere).toBe(true);
        expect(fitTitle("Zwillingstürme", even, CARD).breakAnywhere).toBe(false);
    });

    it("wraps at whole words, so no line is a fragment of one", () => {
        const fit = fitTitle("Operation Lucent Arrowhead", even, CARD);
        for (const line of fit.lines) expect(line).not.toMatch(/^\s|\s$/);
        expect(fit.lines.join(" ")).toBe("Operation Lucent Arrowhead");
    });

    it("answers the ceiling and no lines for a blank title rather than dividing by a zero width", () => {
        expect(fitTitle("   ", even, CARD)).toEqual({ lines: [], size: 21, breakAnywhere: false });
    });
});

describe("the ticket's fit bounds", () => {
    it("keep the old 216 px box's step ratios on both grids: ceiling 0.148 and floor 0.097 of the box", () => {
        for (const bounds of [TICKET_FIT, TICKET_FIT_PHONE]) {
            expect(bounds.max / bounds.box).toBeCloseTo(32 / 216, 2);
            expect(bounds.min / bounds.box).toBeCloseTo(21 / 216, 2);
        }
    });

    it("measure against the NARROWEST card each grid makes: 200 px and 160 px tracks, less 23% and 10 px", () => {
        expect(TICKET_FIT.box).toBe(Math.floor(200 * 0.77 - 10));
        expect(TICKET_FIT_PHONE.box).toBe(Math.floor(160 * 0.77 - 10));
    });

    it("set the phone title smaller than the desktop one and never under 11 px", () => {
        const phone = fitTitle("Somniloquium", even, TICKET_FIT_PHONE);
        // 7.2 em in a 113 px box: 16.5 and 16 overflow, 15.5 is 111.6 px.
        expect(phone.size).toBe(15.5);
        expect(phone.size).toBeLessThan(fitTitle("Somniloquium", even, TICKET_FIT).size);
        expect(fitTitle("Come Catastrophes or Wakes of Vultures", even, TICKET_FIT_PHONE).size).toBe(11);
    });
});
