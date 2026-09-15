import { describe, expect, it } from "vitest";

import { formatMessage } from "./format";

const en = (message: string, values?: Parameters<typeof formatMessage>[2]) => formatMessage(message, "en", values);

describe("formatMessage", () => {
    it("passes plain text through untouched", () => {
        expect(en("Next")).toBe("Next");
        expect(en("Search operators, pages, tools…")).toBe("Search operators, pages, tools…");
    });

    it("substitutes plain arguments", () => {
        expect(en("Showing {count} of {total}", { count: 30, total: 320 })).toBe("Showing 30 of 320");
    });

    // The bug this suite exists for. An apostrophe only opens an ICU quoted
    // run when it precedes { } # or | - everywhere else it is just an
    // apostrophe. Treating it as a quote ate contractions out of the English
    // source with no error.
    describe("apostrophes", () => {
        it("keeps a lone apostrophe in a contraction or possessive", () => {
            expect(en("Most E2'd")).toBe("Most E2'd");
            expect(en("This operator's files")).toBe("This operator's files");
            expect(en("{name} doesn't require materials", { name: "Amiya" })).toBe("Amiya doesn't require materials");
        });

        it("keeps both apostrophes in a sentence that has two", () => {
            expect(en("What's public, and what isn't")).toBe("What's public, and what isn't");
        });

        it("collapses a doubled apostrophe to one", () => {
            expect(en("Most E2''d")).toBe("Most E2'd");
        });

        it("still treats a quote before a brace as ICU quoting", () => {
            expect(en("'{'not an arg'}'")).toBe("{not an arg}");
        });
    });

    describe("plurals", () => {
        const loops = "{count, plural, one {# loop} other {# loops}}";

        it("selects the English categories", () => {
            expect(en(loops, { count: 1 })).toBe("1 loop");
            expect(en(loops, { count: 3 })).toBe("3 loops");
        });

        it("prefers an exact =N branch over the category", () => {
            expect(en("{count, plural, =0 {none} one {# loop} other {# loops}}", { count: 0 })).toBe("none");
        });

        it("uses the target locale's own rules, not English's", () => {
            // Japanese has a single category; Polish has four. Neither can be
            // expressed with a ternary on `count === 1`.
            expect(formatMessage(loops, "ja", { count: 1 })).toBe("1 loops");
            expect(formatMessage("{count, plural, one {plik} few {pliki} many {plików} other {pliku}}", "pl", { count: 5 })).toBe("plików");
        });

        it("formats # in the active locale", () => {
            expect(en("{count, plural, other {# operators}}", { count: 1234 })).toBe("1,234 operators");
        });
    });

    it("handles select", () => {
        const m = "{top, select, yes { - the most common choice} other {}}";
        expect(en(m, { top: "yes" })).toBe(" - the most common choice");
        expect(en(m, { top: "no" })).toBe("");
    });

    it("formats numbers, dates and times through Intl", () => {
        expect(en("{n, number}", { n: 1234567 })).toBe("1,234,567");
        expect(formatMessage("{n, number}", "ja", { n: 1234567 })).toBe("1,234,567");
        expect(en("{d, date}", { d: new Date("2026-09-15T00:00:00Z") })).toContain("2026");
    });

    it("returns the raw pattern rather than throwing on a malformed message", () => {
        // A stored translation must never be able to blank a subtree.
        expect(en("{count operators")).toBe("{count operators");
    });

    it("renders an absent argument as empty rather than 'undefined'", () => {
        expect(en("Hello {name}")).toBe("Hello ");
    });
});
