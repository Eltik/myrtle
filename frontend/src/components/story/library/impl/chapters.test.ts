import { describe, expect, it } from "vitest";
import { chapterNumberOf, chipLines, glyphIndexFor, hashKey, type IChipModel, parseMainOrdinal, rangeIsSingle, releaseYear, SECTION_GLYPHS, sectionChapters, specModel, splitActOrdinal } from "./chapters";
import type { LibEntry, LibGroup } from "./derive";

function entry(id: string, over: Partial<LibEntry> = {}): LibEntry {
    return { id, name: id, sort: 1, groupId: "g", hasScript: true, requiredStages: [], ...over };
}

function group(id: string, over: Partial<LibGroup> = {}): LibGroup {
    return { id, name: id, category: "side", entryType: "ACTIVITY", actType: "ACTIVITY_STORY", startTime: 0, stories: [entry(`${id}_1`)], ...over };
}

describe("parseMainOrdinal", () => {
    it("reads the number out of a mainline group id", () => {
        expect(parseMainOrdinal("main_0")).toBe(0);
        expect(parseMainOrdinal("main_9")).toBe(9);
        expect(parseMainOrdinal("main_14")).toBe(14);
    });

    it("refuses anything that is not exactly main_<digits>", () => {
        expect(parseMainOrdinal("main_")).toBeNull();
        expect(parseMainOrdinal("main_x")).toBeNull();
        expect(parseMainOrdinal("act18d0")).toBeNull();
        expect(parseMainOrdinal("main_1_extra")).toBeNull();
        expect(parseMainOrdinal("")).toBeNull();
    });
});

describe("chapterNumberOf", () => {
    it("prefers the wire field when the backend sends one", () => {
        expect(chapterNumberOf(group("main_3", { category: "main", chapterNumber: 7 }))).toBe(7);
    });

    it("reads chapter ZERO off the wire rather than falling back, because 0 is not missing", () => {
        // `group.chapterNumber || parse(id)` would answer 9 here. Chapter 0 is
        // the Prologue and the most common mainline number on the page.
        expect(chapterNumberOf(group("main_9", { category: "main", chapterNumber: 0 }))).toBe(0);
    });

    it("parses the id while the field is absent, which is what :3060 sends", () => {
        expect(chapterNumberOf(group("main_0", { category: "main" }))).toBe(0);
        expect(chapterNumberOf(group("main_14", { category: "main" }))).toBe(14);
    });

    it("has no number for an event, a vignette or a record", () => {
        expect(chapterNumberOf(group("act18d0"))).toBeNull();
        expect(chapterNumberOf(group("act5d0", { category: "vignette" }))).toBeNull();
        expect(chapterNumberOf(group("char_002_amiya", { category: "record" }))).toBeNull();
    });

    it("treats a non-finite field as absent", () => {
        expect(chapterNumberOf(group("main_4", { category: "main", chapterNumber: Number.NaN }))).toBe(4);
    });
});

describe("releaseYear", () => {
    it("reads the year off the release time", () => {
        expect(releaseYear({ startTime: 1666180800 })).toBe(2022);
    });

    it("has no year for the 17 mainline groups the table writes -1 on", () => {
        expect(releaseYear({ startTime: -1 })).toBeNull();
        expect(releaseYear({ startTime: 0 })).toBeNull();
    });
});

describe("specModel", () => {
    it("prints a chapter number for the mainline instead of the group id", () => {
        const g = group("main_0", { category: "main", stories: [entry("a"), entry("b")] });
        expect(specModel(g, "EPISODE 00")).toEqual({ kind: "chapter", chapter: 0, entries: 2 });
    });

    it("keeps the operation code for an event and adds the year it ran", () => {
        const g = group("act18d0", { startTime: 1666180800, stories: [entry("a")] });
        expect(specModel(g, "GT")).toEqual({ kind: "code", code: "GT", year: 2022, entries: 1 });
    });

    it("leaves the year out when nothing dates the group", () => {
        expect(specModel(group("act1d0", { startTime: -1 }), "GT")).toEqual({ kind: "code", code: "GT", year: null, entries: 1 });
    });
});

describe("sectionChapters", () => {
    it("makes a mainline arc a chapter RUN, which is its primary label", () => {
        const run = [group("main_0", { category: "main" }), group("main_1", { category: "main" }), group("main_3", { category: "main" })];
        expect(sectionChapters(run)).toEqual({ primary: { from: 0, to: 3 }, includes: null });
    });

    it("keeps the run on a mainline arc that also shelves side material, which 3 of the 4 EN arcs do", () => {
        // arc-mainLine-201 is 6 numbered of 9, the narrowest of the four.
        const arc = [group("main_9", { category: "main" }), group("main_10", { category: "main" }), group("main_11", { category: "main" }), group("main_12", { category: "main" }), group("main_13", { category: "main" }), group("main_14", { category: "main" }), group("act18d0"), group("act22side"), group("act33side")];
        expect(sectionChapters(arc, { from: 9, to: 14 })).toEqual({ primary: { from: 9, to: 14 }, includes: null });
    });

    it("takes the wire's own range as the primary on a run, and ignores a non-finite one", () => {
        expect(sectionChapters([group("main_2", { category: "main" })], { from: 0, to: 3 })).toEqual({ primary: { from: 0, to: 3 }, includes: null });
        expect(sectionChapters([group("main_2", { category: "main" })], { from: Number.NaN, to: 3 })).toEqual({ primary: { from: 2, to: 2 }, includes: null });
    });

    it("gives a THEMED shelf that owns no mainline group nothing at all, whatever range the wire sends", () => {
        // The defect: ssLine_2 "The Blessed" carries chapterRange 15 to 15 and
        // owns 5 groups, none of them a chapter, because mainLine sorts 0 and
        // took main_15 first. It headed "Main story · Chapter 15".
        expect(sectionChapters([group("act18d0"), group("act5d0")], { from: 15, to: 15 })).toEqual({ primary: null, includes: null });
        expect(sectionChapters([group("act18d0"), group("act5d0")])).toEqual({ primary: null, includes: null });
        // ssLine_1 "The Ark", chapterRange 7 to 14 over 3 owned event groups.
        expect(sectionChapters([group("act11d0"), group("act12side"), group("act17side")], { from: 7, to: 14 })).toEqual({ primary: null, includes: null });
    });

    it("gives a themed shelf that DOES own mainline groups the muted includes, never the heading", () => {
        const shelf = [group("main_7", { category: "main" }), group("main_14", { category: "main" }), group("act18d0"), group("act22side"), group("act33side")];
        expect(sectionChapters(shelf, { from: 7, to: 14 })).toEqual({ primary: null, includes: { from: 7, to: 14 } });
    });

    it("reads includes off the section's OWN groups, not off the shelf's range", () => {
        const shelf = [group("main_9", { category: "main" }), group("act18d0"), group("act22side")];
        expect(sectionChapters(shelf, { from: 4, to: 14 })).toEqual({ primary: null, includes: { from: 9, to: 9 } });
    });

    it("says nothing about an empty section", () => {
        expect(sectionChapters([])).toEqual({ primary: null, includes: null });
    });
});

describe("rangeIsSingle", () => {
    it("separates a one-chapter section from a run", () => {
        expect(rangeIsSingle({ from: 9, to: 9 })).toBe(true);
        expect(rangeIsSingle({ from: 0, to: 3 })).toBe(false);
    });
});

describe("glyphIndexFor", () => {
    it("is stable and inside the glyph table", () => {
        for (const id of ["mainLine", "ssLine_1", "ssLine_12", "other-events", "operator-records"]) {
            const at = glyphIndexFor(id);
            expect(at).toBe(glyphIndexFor(id));
            expect(at).toBeGreaterThanOrEqual(0);
            expect(at).toBeLessThan(SECTION_GLYPHS.length);
        }
    });

    it("gives the thirteen EN shelves more than one glyph between them", () => {
        const ids = ["mainLine", "ssLine_1", "ssLine_2", "ssLine_3", "ssLine_4", "ssLine_5", "ssLine_6", "ssLine_7", "ssLine_8", "ssLine_9", "ssLine_10", "ssLine_11", "ssLine_12"];
        expect(new Set(ids.map(glyphIndexFor)).size).toBeGreaterThan(1);
    });

    it("hashes the id, not its length", () => {
        expect(hashKey("ssLine_1")).not.toBe(hashKey("ssLine_2"));
    });
});

describe("splitActOrdinal", () => {
    it("finds no ordinal in any of the four EN arc names, because the ordinal is in the banner art", () => {
        for (const name of ["HOUR OF AN AWAKENING", "SHATTER OF A VISION", "SHADOW OF A DYING SUN", "NEXUS POINT OF FUTURE"]) {
            expect(splitActOrdinal(name)).toEqual({ ordinal: null, name });
        }
    });

    it("reads a roman or arabic ordinal off a name that carries one, and hands back the name without it", () => {
        expect(splitActOrdinal("Act I: Shatter of a Vision")).toEqual({ ordinal: "I", name: "Shatter of a Vision" });
        expect(splitActOrdinal("Act 2 - Shadow of a Dying Sun")).toEqual({ ordinal: "2", name: "Shadow of a Dying Sun" });
        expect(splitActOrdinal("act iii · Nexus Point")).toEqual({ ordinal: "III", name: "Nexus Point" });
    });

    it("leaves a bare ordinal with an empty name rather than printing it twice", () => {
        expect(splitActOrdinal("Act 1")).toEqual({ ordinal: "1", name: "" });
    });

    it("does not read ACT out of a name that merely starts with those letters", () => {
        expect(splitActOrdinal("Actors of the Dawn").ordinal).toBeNull();
        expect(splitActOrdinal("Action").ordinal).toBeNull();
    });
});

describe("chipLines", () => {
    type Chip = Pick<IChipModel, "name" | "range" | "includes" | "iconWide" | "iconLogo" | "abbr">;
    const arc: Chip = { name: "SHATTER OF A VISION", range: { from: 4, to: 8 }, includes: null, iconWide: true };
    // Every one of the twelve EN shelves carries the 108x108 `storyline_*`
    // logo, which is the mark the collapsed chip is.
    const shelf: Chip = { name: "The Ark", range: null, includes: null, iconWide: false, iconLogo: true };

    it("collapses a mainline arc to its range, because the banner beside it already prints the name", () => {
        expect(chipLines(arc, false)).toEqual({ name: null, abbr: null, range: { from: 4, to: 8 }, holds: false, ordinal: null });
    });

    it("expands the active arc to the name over the range", () => {
        expect(chipLines(arc, true)).toEqual({ name: "SHATTER OF A VISION", abbr: null, range: { from: 4, to: 8 }, holds: false, ordinal: null });
    });

    it("keeps an arc's name collapsed when there is no banner to carry it", () => {
        expect(chipLines({ ...arc, iconWide: false }, false).name).toBe("SHATTER OF A VISION");
    });

    it("carries a single chapter as a range of one rather than dropping it", () => {
        const one = chipLines({ ...arc, range: { from: 9, to: 9 } }, false);
        expect(one.range).toEqual({ from: 9, to: 9 });
        expect(one.range && rangeIsSingle(one.range)).toBe(true);
    });

    it("collapses a themed shelf to its monogram, which prints nothing at all, and expands the active one to its name", () => {
        expect(chipLines(shelf, false)).toEqual({ name: null, abbr: null, range: null, holds: false, ordinal: null });
        expect(chipLines(shelf, true)).toEqual({ name: "The Ark", abbr: null, range: null, holds: false, ordinal: null });
    });

    it("keeps a themed shelf's name where the wire sends no monogram to collapse to", () => {
        // A shelf with only the 44x36 abbreviation art, or none at all: the
        // 108x108 logo is what the collapse trades the name for, and 12 of the
        // 12 EN shelves have one.
        expect(chipLines({ ...shelf, iconLogo: false }, false).name).toBe("The Ark");
        expect(chipLines({ ...shelf, iconLogo: undefined }, false).name).toBe("The Ark");
    });

    it("prints neither line where a section has both a mark and an abbreviation, because the mark is the stronger stand-in", () => {
        expect(chipLines({ ...shelf, abbr: "ARK" }, false)).toEqual({ name: null, abbr: null, range: null, holds: false, ordinal: null });
    });

    it("holds back the weaker includes claim until the chip is the active one", () => {
        const holding = { ...shelf, includes: { from: 7, to: 14 } };
        expect(chipLines(holding, false).range).toBeNull();
        expect(chipLines(holding, true)).toEqual({ name: "The Ark", abbr: null, range: { from: 7, to: 14 }, holds: true, ordinal: null });
    });

    it("collapses the two unmarked sections to their mono abbreviation and gives the name back when they are active", () => {
        // Other events and Operator records are the only sections the game
        // marks with nothing, so a lucide glyph plus MISC or REC is all a
        // collapsed chip has; the name is in the tooltip and the aria-label.
        const misc: Chip = { name: "Other events", range: null, includes: null, abbr: "MISC" };
        const rec: Chip = { name: "Operator records", range: null, includes: null, abbr: "REC" };
        expect(chipLines(misc, false)).toEqual({ name: null, abbr: "MISC", range: null, holds: false, ordinal: null });
        expect(chipLines(rec, false)).toEqual({ name: null, abbr: "REC", range: null, holds: false, ordinal: null });
        expect(chipLines(misc, true)).toEqual({ name: "Other events", abbr: null, range: null, holds: false, ordinal: null });
        expect(chipLines(rec, true)).toEqual({ name: "Operator records", abbr: null, range: null, holds: false, ordinal: null });
    });

    it("keeps the name on a section with neither a mark nor an abbreviation, which is every fallback section", () => {
        const year: Chip = { name: "2022", range: null, includes: null };
        expect(chipLines(year, false)).toEqual({ name: "2022", abbr: null, range: null, holds: false, ordinal: null });
    });

    it("prints the act ordinal on the expanded chip alone, and strips it from the name", () => {
        const named = { ...arc, name: "Act I: Shatter of a Vision" };
        expect(chipLines(named, true)).toEqual({ name: "Shatter of a Vision", abbr: null, range: { from: 4, to: 8 }, holds: false, ordinal: "I" });
        expect(chipLines(named, false).ordinal).toBeNull();
    });

    it("prints no name at all for a bare ordinal, rather than repeating it on both lines", () => {
        expect(chipLines({ ...arc, name: "Act 1", iconWide: false }, true)).toEqual({ name: null, abbr: null, range: { from: 4, to: 8 }, holds: false, ordinal: "1" });
    });
});
