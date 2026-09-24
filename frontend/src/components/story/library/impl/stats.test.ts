import { describe, expect, it } from "vitest";
import { emptyProgress, type StoryProgress } from "#/lib/story/progress";
import type { LibEntry, LibGroup, LibIndex, LibRecord } from "./derive";
import { readingStats, sectionTotals, setWords } from "./stats";

function entry(id: string, words: number | undefined, over: Partial<LibEntry> = {}): LibEntry {
    return { id, name: id, sort: 1, groupId: "g", hasScript: true, wordCount: words, requiredStages: [], ...over };
}

function group(id: string, over: Partial<LibGroup> = {}): LibGroup {
    return { id, name: id, category: "side", entryType: "ACTIVITY", actType: "NONE", startTime: 0, stories: [], ...over };
}

function progressOf(over: Partial<StoryProgress>): StoryProgress {
    return { ...emptyProgress(), ...over };
}

const NO_GAME: ReadonlySet<string> = new Set<string>();

/**
 * A hand-built index: one mainline chapter, one event, one intermezzo, one
 * vignette, one record group, and the same record listed a SECOND time under
 * `records`, which is the shape the wire actually sends.
 */
function index(): LibIndex {
    const record: LibRecord = { charId: "char_002_amiya", name: "Amiya", stories: [entry("rec_1", 400)] };
    return {
        groups: [
            group("main_1", { category: "main", name: "Chapter 1", stories: [entry("m1", 9000), entry("m2", 1000), entry("m3", undefined, { hasScript: false })] }),
            group("act1", { name: "An Event", displayType: "SIDESTORY", stories: [entry("e1", 5000), entry("e2", 3000)] }),
            group("act2", { name: "An Intermezzo", displayType: "BRANCHLINE", stories: [entry("i1", 2000)] }),
            group("act3", { name: "A Vignette", category: "vignette", stories: [entry("v1", 700)] }),
            group("rec_amiya", { category: "record", name: "Amiya", stories: [entry("rec_1", 400)] }),
        ],
        records: [record],
    };
}

describe("readingStats", () => {
    it("counts every story ONCE, never the record set twice", () => {
        const stats = readingStats(index(), emptyProgress(), NO_GAME);
        // 7 readable, not 8: `m3` carries no script, and `rec_1` is listed by
        // both the record GROUP and the record SET and counts once.
        expect(stats.total).toBe(7);
        expect(stats.wordsTotal).toBe(9000 + 1000 + 5000 + 3000 + 2000 + 700 + 400);
        expect(stats.wordsRead).toBe(0);
        expect(stats.counted).toBe(true);
    });

    it("sums the words of the stories READ, the game's verdict included", () => {
        const stats = readingStats(index(), progressOf({ read: { m1: 2 } }), new Set(["e1"]));
        expect(stats.read).toBe(2);
        expect(stats.wordsRead).toBe(9000 + 5000);
    });

    it("puts each group in the one bucket its browse pill answers to", () => {
        const stats = readingStats(index(), emptyProgress(), NO_GAME);
        expect(stats.rows.map((r) => [r.key, r.total, r.words])).toEqual([
            ["main", 2, 10000],
            ["events", 2, 8000],
            // Side stories is the vignette plus the intermezzo, both short-form shelves.
            ["side", 2, 2700],
            ["records", 1, 400],
        ]);
    });

    it("ranks the longest chapters and events, and lists no operator record among them", () => {
        const stats = readingStats(index(), emptyProgress(), NO_GAME);
        expect(stats.longest.map((row) => [row.name, row.words])).toEqual([
            ["Chapter 1", 10000],
            ["An Event", 8000],
            ["An Intermezzo", 2000],
            ["A Vignette", 700],
        ]);
    });

    it("cuts the longest table at the limit", () => {
        expect(readingStats(index(), emptyProgress(), NO_GAME, 2).longest).toHaveLength(2);
    });

    it("carries the read fraction of each long row", () => {
        const stats = readingStats(index(), progressOf({ read: { m1: 2 } }), NO_GAME);
        expect(stats.longest[0]).toMatchObject({ id: "main_1", read: 1, total: 2 });
    });

    it("reports UNCOUNTED rather than zero when the wire sends no word counts", () => {
        const bare: LibIndex = { groups: [group("act1", { stories: [entry("e1", undefined)] })], records: [] };
        const stats = readingStats(bare, emptyProgress(), NO_GAME);
        expect(stats.counted).toBe(false);
        expect(stats.wordsTotal).toBe(0);
        expect(stats.longest).toEqual([]);
    });

    it("counts words even with nothing read, because a zero read is a true number", () => {
        const stats = readingStats(index(), emptyProgress(), NO_GAME);
        expect(stats.counted).toBe(true);
        expect(stats.wordsRead).toBe(0);
    });
});

describe("setWords", () => {
    const stories = [entry("a", 1000), entry("b", 500), entry("c", 9000, { hasScript: false })];

    it("counts the whole set and what is LEFT of it, script-less stories excluded from both", () => {
        expect(setWords(stories, emptyProgress(), NO_GAME)).toEqual({ total: 1500, left: 1500 });
    });

    it("drops what is read out of `left`, the game's verdict included", () => {
        expect(setWords(stories, progressOf({ read: { a: 2 } }), new Set(["b"]))).toEqual({ total: 1500, left: 0 });
    });

    it("is null on a set the wire counts no words for, never zero", () => {
        expect(setWords([entry("a", undefined)], emptyProgress(), NO_GAME)).toEqual({ total: null, left: null });
    });
});

describe("sectionTotals", () => {
    it("sums a block's words and counts the chapters in it that are FINISHED", () => {
        const blocks = [group("g1", { stories: [entry("a", 1000), entry("b", 500)] }), group("g2", { stories: [entry("c", 250)] })];
        expect(sectionTotals(blocks, progressOf({ read: { c: 2 } }), NO_GAME)).toEqual({ total: 1750, left: 1500, done: 1, groups: 2 });
    });

    it("never calls a block with nothing readable in it finished: 0 of 0 is not an achievement", () => {
        const blocks = [group("g1", { stories: [entry("a", 400, { hasScript: false })] })];
        expect(sectionTotals(blocks, emptyProgress(), NO_GAME)).toEqual({ total: null, left: null, done: 0, groups: 1 });
    });
});
