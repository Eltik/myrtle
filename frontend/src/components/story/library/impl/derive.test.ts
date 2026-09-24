import { describe, expect, it } from "vitest";
import { emptyProgress, type StoryProgress } from "#/lib/story/progress";
import {
    buildTabs,
    filterGroups,
    filterRecords,
    finishedGroups,
    groupCode,
    groupOperations,
    groupWords,
    type LibEntry,
    type LibGroup,
    type LibIndex,
    type LibRecord,
    matchesReadState,
    pickContinue,
    pickHero,
    progressSummary,
    READ_FILTERS,
    readFraction,
    readStateOf,
    sortedStories,
    sortGroups,
    sortLibrary,
    tabOf,
} from "./derive";

const collator = new Intl.Collator("en", { sensitivity: "base", numeric: true });

function entry(id: string, over: Partial<LibEntry> = {}): LibEntry {
    return { id, name: id, sort: 1, groupId: "g", hasScript: true, requiredStages: [], ...over };
}

function group(id: string, over: Partial<LibGroup> = {}): LibGroup {
    return { id, name: id, category: "side", entryType: "ACTIVITY", actType: "NONE", startTime: 0, stories: [entry(`${id}_1`)], ...over };
}

function progressOf(over: Partial<StoryProgress>): StoryProgress {
    return { ...emptyProgress(), ...over };
}

/** The game's verdict, which most of these cases do not exercise. */
const NO_GAME: ReadonlySet<string> = new Set<string>();

describe("readFraction", () => {
    it("counts only stories that have a script", () => {
        const stories = [entry("a"), entry("b"), entry("c", { hasScript: false })];
        expect(readFraction(stories, progressOf({ read: { a: 1 } }), NO_GAME)).toEqual({ read: 1, total: 2, listed: 3, done: false });
    });

    it("is done when every readable story is read, script-less ones notwithstanding", () => {
        const stories = [entry("a"), entry("b", { hasScript: false })];
        expect(readFraction(stories, progressOf({ read: { a: 1 } }), NO_GAME).done).toBe(true);
    });

    it("is never done with nothing to read", () => {
        expect(readFraction([entry("a", { hasScript: false })], emptyProgress(), NO_GAME)).toEqual({ read: 0, total: 0, listed: 1, done: false });
    });

    it("counts a story the GAME reports read, which the document never carries", () => {
        const stories = [entry("a"), entry("b")];
        expect(readFraction(stories, emptyProgress(), new Set(["a"]))).toMatchObject({ read: 1, total: 2, done: false });
        expect(readFraction(stories, progressOf({ read: { b: 1 } }), new Set(["a"])).done).toBe(true);
    });
});

describe("tabOf and buildTabs", () => {
    it("shelves a BRANCHLINE event under Intermezzi rather than Side Stories", () => {
        expect(tabOf({ category: "side", displayType: "BRANCHLINE" })).toBe("intermezzi");
        expect(tabOf({ category: "side", displayType: "SIDESTORY" })).toBe("side");
        expect(tabOf({ category: "main", displayType: undefined })).toBe("main");
    });

    it("keeps operator records out of the Stories tabs", () => {
        expect(tabOf({ category: "record", displayType: undefined })).toBeNull();
    });

    it("emits only non-empty sub-tabs, in game order", () => {
        const tabs = buildTabs([group("v", { category: "vignette" }), group("m", { category: "main" }), group("b", { displayType: "BRANCHLINE" }), group("s", { displayType: "SIDESTORY" }), group("r", { category: "record" })]);
        expect(tabs.map((entry) => entry.key)).toEqual(["main", "side", "intermezzi", "vignette"]);
        expect(tabs.map((entry) => entry.groups.length)).toEqual([1, 1, 1, 1]);
    });
});

describe("filterGroups", () => {
    const groups = [group("a", { name: "Under Tides" }), group("b", { name: "Dossoles Holiday", stories: [entry("b_1", { name: "Beach Episode" })] }), group("c", { name: "Ancient Forge" })];

    it("returns everything for an empty query", () => {
        expect(filterGroups("  ", groups)).toHaveLength(3);
    });

    it("matches a group name and preserves input order", () => {
        expect(filterGroups("tides", groups).map((g) => g.id)).toEqual(["a"]);
    });

    it("matches a story name inside a group", () => {
        expect(filterGroups("beach", groups).map((g) => g.id)).toEqual(["b"]);
    });

    it("drops everything that scores zero", () => {
        expect(filterGroups("zzzzqqq", groups)).toEqual([]);
    });
});

describe("filterRecords", () => {
    const records: LibRecord[] = [
        { charId: "char_003_kalts", name: "Kal'tsit", stories: [entry("k1", { name: "End of a Long Journey" })] },
        { charId: "char_002_amiya", name: "Amiya", stories: [entry("a1", { name: "Incoming Mail" })] },
    ];

    it("matches on the operator name through the site's normalization", () => {
        expect(filterRecords("kaltsit", records).map((r) => r.charId)).toEqual(["char_003_kalts"]);
    });

    it("matches on a record's own name", () => {
        expect(filterRecords("incoming", records).map((r) => r.charId)).toEqual(["char_002_amiya"]);
    });
});

describe("sortGroups", () => {
    it("puts the newest release first", () => {
        const sorted = sortGroups([group("old", { startTime: 100 }), group("new", { startTime: 900 })], "release", collator);
        expect(sorted.map((g) => g.id)).toEqual(["new", "old"]);
    });

    it("reads the mainline's id numerically when every startTime is -1", () => {
        const mains = [group("main_10", { startTime: -1 }), group("main_2", { startTime: -1 }), group("main_0", { startTime: -1 })];
        expect(sortGroups(mains, "release", collator).map((g) => g.id)).toEqual(["main_0", "main_2", "main_10"]);
    });

    it("sorts by name alphabetically", () => {
        const sorted = sortGroups([group("b", { name: "Zwillingstürme" }), group("a", { name: "Ancient Forge" })], "alpha", collator);
        expect(sorted.map((g) => g.id)).toEqual(["a", "b"]);
    });
});

describe("groupCode and groupWords", () => {
    it("prefers the chapter's own third name", () => {
        expect(groupCode(group("main_0", { zone: { nameThird: "EPISODE 00" } }))).toBe("EPISODE 00");
    });

    it("falls back to the operation prefix, then to the id", () => {
        expect(groupCode(group("act1d0", { stories: [entry("x", { code: "GT-1" })] }))).toBe("GT");
        expect(groupCode(group("act1d0", { stories: [entry("x")] }))).toBe("ACT1D0");
    });

    it("sums the stories' word counts, and stays absent when none carry one", () => {
        expect(groupWords(group("a", { stories: [entry("x", { wordCount: 10 }), entry("y", { wordCount: 5 }), entry("z")] }))).toBe(15);
        expect(groupWords(group("a", { stories: [entry("x")] }))).toBeNull();
        expect(groupWords(group("a", { wordCount: 42, stories: [entry("x", { wordCount: 10 })] }))).toBe(42);
    });
});

describe("sortedStories", () => {
    it("orders by the table's sort key", () => {
        expect(sortedStories([entry("b", { sort: 2 }), entry("a", { sort: 1 })]).map((s) => s.id)).toEqual(["a", "b"]);
    });
});

describe("pickHero", () => {
    const main0 = group("main_0", { category: "main", startTime: -1, stories: [entry("m0_2", { sort: 2 }), entry("m0_1", { sort: 1 })] });
    const main1 = group("main_1", { category: "main", startTime: -1 });
    const side = group("act1d0", { stories: [entry("s1")] });
    const index: LibIndex = { groups: [side, main1, main0], records: [] };

    it("offers the first mainline story to a first-time visitor", () => {
        const hero = pickHero(index, emptyProgress());
        expect(hero?.entry.id).toBe("m0_1");
        expect(hero?.group.id).toBe("main_0");
        expect(hero?.fresh).toBe(true);
        expect(hero?.resumeHalt).toBeNull();
    });

    it("skips a first story with no script", () => {
        const noScript: LibIndex = { groups: [group("main_0", { category: "main", stories: [entry("a", { sort: 1, hasScript: false }), entry("b", { sort: 2 })] })], records: [] };
        expect(pickHero(noScript, emptyProgress())?.entry.id).toBe("b");
    });

    it("returns the last story read, with its saved halt", () => {
        const hero = pickHero(index, progressOf({ last: "s1", pos: { s1: { halt: 12, total: 40, ts: 1, choices: {} } } }));
        expect(hero?.entry.id).toBe("s1");
        expect(hero?.group.id).toBe("act1d0");
        expect(hero?.resumeHalt).toBe(12);
        expect(hero?.resumeTotal).toBe(40);
        expect(hero?.fresh).toBe(false);
    });

    it("names the operator when the last story is a record", () => {
        const recordGroup = group("story_kalts_set_1", { category: "record", stories: [entry("k1")] });
        const withRecords: LibIndex = { groups: [recordGroup], records: [{ charId: "char_003_kalts", name: "Kal'tsit", stories: [entry("k1")] }] };
        expect(pickHero(withRecords, progressOf({ last: "k1" }))?.recordName).toBe("Kal'tsit");
    });

    it("falls back when `last` names a story the index no longer holds", () => {
        expect(pickHero(index, progressOf({ last: "gone" }))?.entry.id).toBe("m0_1");
    });

    it("returns null with no mainline group at all", () => {
        expect(pickHero({ groups: [side], records: [] }, emptyProgress())).toBeNull();
    });
});

describe("progressSummary and finishedGroups", () => {
    const index: LibIndex = {
        groups: [group("main_0", { category: "main", stories: [entry("a", { wordCount: 100 }), entry("b", { wordCount: 50 })] }), group("act1d0", { stories: [entry("c"), entry("d", { hasScript: false })] }), group("story_x_set_1", { category: "record", stories: [entry("e")] })],
        records: [{ charId: "char_x", name: "X", stories: [entry("e")] }],
    };

    it("counts read stories over readable ones and leaves records out of the chapter count", () => {
        const summary = progressSummary(index, progressOf({ read: { a: 1, c: 1 } }), NO_GAME);
        expect(summary.read).toBe(2);
        expect(summary.total).toBe(4);
        expect(summary.fraction).toBeCloseTo(0.5, 10);
        expect(summary.groups).toBe(2);
        expect(summary.groupsDone).toBe(1);
        expect(summary.words).toBe(100);
    });

    it("reports words as absent only when NO story carries a count", () => {
        const wordless: LibIndex = { groups: [group("g", { stories: [entry("a")] })], records: [] };
        expect(progressSummary(wordless, progressOf({ read: { a: 1 } }), NO_GAME).words).toBeNull();
    });

    it("reports ZERO words, not absent, when the wire counts words and nothing is read", () => {
        expect(progressSummary(index, progressOf({}), NO_GAME)).toMatchObject({ read: 0, words: 0 });
    });

    it("sums the counts of the read stories", () => {
        expect(progressSummary(index, progressOf({ read: { a: 1, b: 1 } }), NO_GAME).words).toBe(150);
    });

    it("counts the game's read stories and their words, from outside the document", () => {
        const summary = progressSummary(index, emptyProgress(), new Set(["a", "b"]));
        expect(summary.read).toBe(2);
        expect(summary.words).toBe(150);
    });

    it("counts words off `totals` even when the read set is empty and the entries are bare", () => {
        const totalsOnly: LibIndex = { groups: [group("g", { stories: [entry("a")] })], records: [], totals: { stories: 1, withScript: 1, words: 4220743 } };
        expect(progressSummary(totalsOnly, progressOf({}), NO_GAME).words).toBe(0);
    });

    it("counts finished chapters in one sub-tab", () => {
        expect(finishedGroups([index.groups[0] as LibGroup, index.groups[1] as LibGroup], progressOf({ read: { c: 1 } }), NO_GAME)).toEqual({ done: 1, total: 2 });
    });
});

describe("readStateOf and matchesReadState", () => {
    const fraction = (read: number, total: number): ReturnType<typeof readFraction> =>
        readFraction(
            Array.from({ length: total }, (_, i) => entry(`s${i}`)),
            progressOf({ read: Object.fromEntries(Array.from({ length: read }, (_, i) => [`s${i}`, 2])) }),
            NO_GAME,
        );

    it("classifies nothing read, some read and all read", () => {
        expect(readStateOf(fraction(0, 3))).toBe("unread");
        expect(readStateOf(fraction(1, 3))).toBe("progress");
        expect(readStateOf(fraction(3, 3))).toBe("done");
    });

    it("calls a group with nothing readable UNREAD, never finished", () => {
        const nothing = readFraction([entry("a", { hasScript: false })], emptyProgress(), NO_GAME);
        expect(nothing).toMatchObject({ read: 0, total: 0, done: false });
        expect(readStateOf(nothing)).toBe("unread");
    });

    it("weighs the GAME's verdict the same way a card does", () => {
        const stories = [entry("a"), entry("b")];
        expect(readStateOf(readFraction(stories, emptyProgress(), new Set(["a"])))).toBe("progress");
        expect(readStateOf(readFraction(stories, emptyProgress(), new Set(["a", "b"])))).toBe("done");
    });

    it("lets everything through on `any` and only its own state otherwise", () => {
        expect(READ_FILTERS).toEqual(["any", "unread", "progress", "done"]);
        for (const f of [fraction(0, 3), fraction(1, 3), fraction(3, 3)]) expect(matchesReadState("any", f)).toBe(true);
        expect(matchesReadState("unread", fraction(0, 3))).toBe(true);
        expect(matchesReadState("unread", fraction(1, 3))).toBe(false);
        expect(matchesReadState("progress", fraction(1, 3))).toBe(true);
        expect(matchesReadState("done", fraction(3, 3))).toBe(true);
        expect(matchesReadState("done", fraction(2, 3))).toBe(false);
    });
});

describe("sortLibrary", () => {
    const dated = (id: string, startTime: number, name: string): LibGroup => group(id, { startTime, name });
    const none = (): ReturnType<typeof readFraction> => readFraction([], emptyProgress(), NO_GAME);

    it("returns the shelf order untouched on `default`", () => {
        const groups = [dated("b", 3, "Beta"), dated("a", 1, "Alpha")];
        expect(sortLibrary(groups, "default", none, collator).map((g) => g.id)).toEqual(["b", "a"]);
    });

    it("orders by release in both directions and sorts the UNDATED last in both", () => {
        const groups = [dated("old", 100, "Old"), group("main_2", { startTime: -1, category: "main" }), dated("new", 300, "New"), group("main_10", { startTime: -1, category: "main" })];
        expect(sortLibrary(groups, "newest", none, collator).map((g) => g.id)).toEqual(["new", "old", "main_2", "main_10"]);
        expect(sortLibrary(groups, "oldest", none, collator).map((g) => g.id)).toEqual(["old", "new", "main_2", "main_10"]);
    });

    it("orders by the read SHARE, not the read count", () => {
        const small = group("small", { stories: [entry("s1")] });
        const big = group("big", { stories: [entry("b1"), entry("b2"), entry("b3"), entry("b4")] });
        const progress = progressOf({ read: { s1: 2, b1: 2, b2: 2 } });
        const fractionOf = (g: LibGroup): ReturnType<typeof readFraction> => readFraction(g.stories, progress, NO_GAME);
        // `big` has twice the reads, `small` is 1/1 against 2/4.
        expect(sortLibrary([big, small], "mostRead", fractionOf, collator).map((g) => g.id)).toEqual(["small", "big"]);
        expect(sortLibrary([small, big], "leastRead", fractionOf, collator).map((g) => g.id)).toEqual(["big", "small"]);
    });

    it("orders by title with a numeric, case-insensitive collator", () => {
        const groups = [dated("c", 1, "chapter 10"), dated("a", 1, "Chapter 2"), dated("b", 1, "Abyssal")];
        expect(sortLibrary(groups, "title", none, collator).map((g) => g.name)).toEqual(["Abyssal", "Chapter 2", "chapter 10"]);
    });

    it("never mutates the caller's array", () => {
        const groups = [dated("b", 3, "Beta"), dated("a", 1, "Alpha")];
        sortLibrary(groups, "title", none, collator);
        expect(groups.map((g) => g.id)).toEqual(["b", "a"]);
    });
});

describe("groupOperations", () => {
    const op = (id: string, sort: number, code: string, avgTag: string, name: string): LibEntry => entry(id, { sort, code, avgTag, name });

    it("merges a _beg and _end pair that share a code into ONE row with two segments", () => {
        const rows = groupOperations([op("a_beg", 1, "1-1", "Before Operation", "Isolated Island"), op("a_end", 2, "1-1", "After Operation", "Isolated Island")]);
        expect(rows).toHaveLength(1);
        expect(rows[0]).toMatchObject({ key: "a_beg", code: "1-1", title: "Isolated Island" });
        expect(rows[0]?.segments.map((s) => s.phase)).toEqual(["before", "after"]);
        expect(rows[0]?.segments.map((s) => s.entry.id)).toEqual(["a_beg", "a_end"]);
    });

    it("gives an Interlude its own row, with one segment", () => {
        const rows = groupOperations([op("m8_1", 1, "M8-1", "Interlude", "Today, Sanguine Overlows"), op("eg1", 2, "EG-1", "Interlude", "Burnt Fragment 1")]);
        expect(rows.map((r) => r.code)).toEqual(["M8-1", "EG-1"]);
        expect(rows.every((r) => r.segments.length === 1 && r.segments[0]?.phase === "interlude")).toBe(true);
    });

    it("keeps a lone Before or a lone After as its own row", () => {
        const rows = groupOperations([op("x_beg", 1, "1-4", "Before Operation", "Omen"), op("y_end", 2, "1-10", "After Operation", "Remnants")]);
        expect(rows).toHaveLength(2);
        expect(rows.map((r) => r.segments.length)).toEqual([1, 1]);
        expect(rows.map((r) => r.segments[0]?.phase)).toEqual(["before", "after"]);
    });

    it("does NOT merge across an interlude that plays between the two halves", () => {
        // `act19side`: DV-S-1 sits between DV-5's two halves, twice in the corpus.
        const rows = groupOperations([op("dv5_beg", 1, "DV-5", "Before Operation", "Dorothy's Promise"), op("dvs1", 2, "DV-S-1", "Interlude", "Dilemma"), op("dv5_end", 3, "DV-5", "After Operation", "Dorothy's Promise")]);
        expect(rows.map((r) => r.code)).toEqual(["DV-5", "DV-S-1", "DV-5"]);
        expect(rows.map((r) => r.segments.length)).toEqual([1, 1, 1]);
    });

    it("keeps the row order equal to the index order", () => {
        const rows = groupOperations([op("b_end", 4, "R8-1", "After Operation", "Yesterday"), op("a_beg", 3, "R8-1", "Before Operation", "Yesterday"), op("i", 5, "M8-1", "Interlude", "Today")]);
        expect(rows.map((r) => r.key)).toEqual(["a_beg", "i"]);
        expect(rows[0]?.segments).toHaveLength(2);
    });

    it("carries a null code for the 494 stories the table gives none", () => {
        const rows = groupOperations([entry("welcome", { sort: 1, avgTag: "Interlude" })]);
        expect(rows[0]?.code).toBeNull();
    });
});

describe("pickContinue", () => {
    const stories = [entry("a", { sort: 1 }), entry("b", { sort: 2 }), entry("c", { sort: 3 })];

    it("offers the FIRST story of an untouched chapter, and does not call it a resume", () => {
        expect(pickContinue(stories, emptyProgress(), NO_GAME)).toMatchObject({ entry: { id: "a" }, resumed: false });
    });

    it("offers the first story with no read mark", () => {
        expect(pickContinue(stories, progressOf({ read: { a: 2 } }), NO_GAME)).toMatchObject({ entry: { id: "b" }, resumed: true });
    });

    it("offers the LAST story once every one of them is read", () => {
        expect(pickContinue(stories, progressOf({ read: { a: 2, b: 2, c: 2 } }), NO_GAME)).toMatchObject({ entry: { id: "c" }, resumed: true });
    });

    it("skips a story with no script, and answers null when the chapter has none at all", () => {
        expect(pickContinue([entry("x", { sort: 1, hasScript: false }), entry("y", { sort: 2 })], emptyProgress(), NO_GAME)).toMatchObject({ entry: { id: "y" } });
        expect(pickContinue([entry("x", { hasScript: false })], emptyProgress(), NO_GAME)).toBeNull();
    });

    it("counts a story the GAME reports read", () => {
        expect(pickContinue(stories, emptyProgress(), new Set(["a"]))).toMatchObject({ entry: { id: "b" } });
    });
});
