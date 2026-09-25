import { describe, expect, it } from "vitest";
import { nextPage, pageOf, RECORD_PAGE } from "./paging";

const items = Array.from({ length: 95 }, (_, i) => i);

describe("pageOf", () => {
    it("shows the first page and counts the rest", () => {
        const { shown, hidden } = pageOf(items, RECORD_PAGE);
        expect(shown).toEqual(items.slice(0, 60));
        expect(hidden).toBe(35);
    });

    it("returns the input untouched when it fits", () => {
        const few = items.slice(0, 12);
        const { shown, hidden } = pageOf(few, RECORD_PAGE);
        expect(shown).toBe(few);
        expect(hidden).toBe(0);
    });

    it("never shows less than one page", () => {
        expect(pageOf(items, 0).shown).toHaveLength(60);
        expect(pageOf(items, -3).shown).toHaveLength(60);
    });

    it("shows everything once the limit passes the length", () => {
        expect(pageOf(items, 120)).toEqual({ shown: items, hidden: 0 });
    });
});

describe("nextPage", () => {
    it("reveals a page, or the remainder", () => {
        expect(nextPage(75)).toBe(60);
        expect(nextPage(15)).toBe(15);
        expect(nextPage(0)).toBe(0);
    });
});
