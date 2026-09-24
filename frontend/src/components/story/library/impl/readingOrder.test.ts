import { describe, expect, it } from "vitest";
import { emptyProgress } from "#/lib/story/progress";
import type { LibEntry, LibGroup } from "./derive";
import { mainlineDateSplit, nextInOrder, orderRowModel, orderTotal, readingOrder, releaseOrder, releaseTime, storylineOrder } from "./readingOrder";

function entry(id: string, over: Partial<LibEntry> = {}): LibEntry {
    return { id, name: id, sort: 1, groupId: "g", hasScript: true, requiredStages: [], ...over };
}

function group(id: string, over: Partial<LibGroup> = {}): LibGroup {
    return { id, name: id, category: "side", entryType: "ACTIVITY", actType: "ACTIVITY_STORY", startTime: 0, stories: [entry(`${id}_1`)], ...over };
}

function main(id: string, chapterName: string): LibGroup {
    return group(id, { category: "main", entryType: "MAINLINE", actType: "MAIN_STORY", startTime: -1, zone: { chapterName } });
}

const ids = (sections: ReturnType<typeof releaseOrder>) => sections.map((s) => ({ kind: s.kind, title: s.title, rows: s.rows.map((r) => r.group.id) }));

describe("releaseTime", () => {
    it("reads the mainline's -1 and a record set's 0 as no date at all", () => {
        expect(releaseTime({ startTime: -1 })).toBeNull();
        expect(releaseTime({ startTime: 0 })).toBeNull();
        expect(releaseTime({ startTime: 1580943600 })).toBe(1580943600);
    });
});

describe("orderRowModel", () => {
    it("draws the key visual when there is one, the cover when there is not, and nothing when there is neither", () => {
        expect(orderRowModel(group("a", { bannerUrl: "/kv.png", coverUrl: "/bg_black.png" })).plate).toEqual({ kind: "banner", url: "/kv.png" });
        expect(orderRowModel(group("b", { coverUrl: "/cover.png" })).plate).toEqual({ kind: "cover", url: "/cover.png" });
        expect(orderRowModel(group("c")).plate).toEqual({ kind: "none" });
    });

    it("prints the code only where there is no picture at all", () => {
        expect(orderRowModel(group("act5d0", { zone: { nameThird: "WALK IN THE DUST" } })).code).toBe("WALK IN THE DUST");
    });

    it("wears the Browse cards' own badge, per in-game shelf", () => {
        expect(orderRowModel(main("main_3", "a")).kind).toBe("main");
        expect(orderRowModel(group("act5d0")).kind).toBe("event");
        expect(orderRowModel(group("act9d0", { displayType: "BRANCHLINE" })).kind).toBe("intermezzo");
        expect(orderRowModel(group("act4d0", { category: "vignette" })).kind).toBe("vignette");
    });

    it("reads the chapter number off the wire first and re-derives it from the id when the field is missing", () => {
        expect(orderRowModel(group("main_x", { category: "main", chapterNumber: 17 })).chapter).toBe(17);
        expect(orderRowModel(main("main_10", "a")).chapter).toBe(10);
        // Chapter 0 is the Prologue, the most common mainline number on the
        // page: a falsy check here would re-parse every one of the 17.
        expect(orderRowModel(group("main_0", { category: "main", chapterNumber: 0 })).chapter).toBe(0);
        expect(orderRowModel(group("act5d0")).chapter).toBeNull();
    });
});

describe("mainlineDateSplit", () => {
    it("counts the chapters the zone dates and the chapters it writes -1, and ignores everything that is not mainline", () => {
        const groups = [...Array.from({ length: 12 }, (_, i) => main(`main_${i}`, "a")), ...[10, 11, 12, 13, 14].map((n) => group(`main_${n}`, { category: "main", startTime: 1666180800 + n })), group("act5d0", { startTime: 1590602400 }), group("rec", { category: "record", startTime: 0 })];
        expect(mainlineDateSplit(groups)).toEqual({ undated: 12, dated: 5 });
    });

    it("is all zeroes over a roster with no mainline in it", () => {
        expect(mainlineDateSplit([group("act5d0", { startTime: 1590602400 })])).toEqual({ undated: 0, dated: 0 });
    });
});

describe("releaseOrder", () => {
    const groups = [group("act5d0", { startTime: 1590602400 }), main("main_1", "Hour of An Awakening"), group("1stact", { startTime: 1580943600 }), main("main_0", "Hour of An Awakening"), group("rec", { category: "record", startTime: 0 })];

    it("puts the undated mainline first in chapter order and the dated groups oldest first", () => {
        expect(ids(releaseOrder(groups))).toEqual([
            { kind: "undated", title: null, rows: ["main_0", "main_1"] },
            { kind: "dated", title: null, rows: ["1stact", "act5d0"] },
        ]);
    });

    it("keeps operator record sets out of every reading order", () => {
        expect(orderTotal(releaseOrder(groups))).toBe(4);
    });

    it("interleaves a dated chapter with the events and shelves only the ones the zone writes -1", () => {
        const sections = releaseOrder([main("main_9", "a"), group("act5d0", { startTime: 1590602400 }), group("main_10", { category: "main", startTime: 1666180800 }), group("act21side", { startTime: 1700000000 })]);
        expect(ids(sections)).toEqual([
            { kind: "undated", title: null, rows: ["main_9"] },
            { kind: "dated", title: null, rows: ["act5d0", "main_10", "act21side"] },
        ]);
    });

    it("orders chapters numerically, not as strings", () => {
        const rows = releaseOrder([main("main_10", "a"), main("main_2", "a"), main("main_1", "a")])[0].rows;
        expect(rows.map((r) => r.group.id)).toEqual(["main_1", "main_2", "main_10"]);
    });

    it("draws no heading over an empty block", () => {
        expect(releaseOrder([group("act5d0", { startTime: 1590602400 })]).map((s) => s.kind)).toEqual(["dated"]);
        expect(releaseOrder([main("main_0", "a")]).map((s) => s.kind)).toEqual(["undated"]);
        expect(releaseOrder([])).toEqual([]);
    });
});

describe("storylineOrder", () => {
    const groups = [main("main_4", "Shatter of A Vision"), main("main_0", "Hour of An Awakening"), main("main_1", "Hour of An Awakening"), group("act9d0", { startTime: 1607612400, displayType: "BRANCHLINE" }), group("1stact", { startTime: 1580943600 })];

    it("cuts the mainline into the arcs the zone names, ordered by their first chapter", () => {
        expect(ids(storylineOrder(groups)).slice(0, 2)).toEqual([
            { kind: "arc", title: "Hour of An Awakening", rows: ["main_0", "main_1"] },
            { kind: "arc", title: "Shatter of A Vision", rows: ["main_4"] },
        ]);
    });

    it("trails everything it cannot link to a chapter in release order, as its own block", () => {
        const last = storylineOrder(groups).at(-1);
        expect(last?.kind).toBe("unlinked");
        expect(last?.rows.map((r) => r.group.id)).toEqual(["1stact", "act9d0"]);
    });

    it("names an arc after the chapter itself when the zone carries no chapter name", () => {
        const bare = group("main_15", { category: "main", name: "Dissociative Recombination", startTime: -1 });
        expect(storylineOrder([bare])[0]).toMatchObject({ kind: "arc", title: "Dissociative Recombination" });
    });

    it("lists every story group exactly once", () => {
        expect(orderTotal(storylineOrder(groups))).toBe(5);
    });
});

describe("readingOrder", () => {
    it("routes each mode, and hands the timeline nothing because nothing derives it", () => {
        const groups = [main("main_0", "Hour of An Awakening"), group("1stact", { startTime: 1580943600 })];
        expect(readingOrder("release", groups)).toEqual(releaseOrder(groups));
        expect(readingOrder("storyline", groups)).toEqual(storylineOrder(groups));
        expect(readingOrder("timeline", groups)).toEqual([]);
    });
});

describe("nextInOrder", () => {
    const NO_GAME: ReadonlySet<string> = new Set<string>();

    /** An undated chapter and a dated event, each with two stories, so the pointer has to walk the sections in the order's own sequence rather than the input's. */
    const groups = [group("act1", { startTime: 1580943600, stories: [entry("a1_a", { sort: 1 }), entry("a1_b", { sort: 2 })] }), group("main_1", { category: "main", startTime: -1, stories: [entry("m1_a", { sort: 1 }), entry("m1_b", { sort: 2 })] })];

    it("points at the first unread story of the first unfinished chapter, in the ORDER'S own sequence", () => {
        const sections = releaseOrder(groups);
        expect(nextInOrder(sections, emptyProgress(), NO_GAME)).toMatchObject({ group: { id: "main_1" }, entry: { id: "m1_a" } });
    });

    it("walks past what is read, the game's verdict included", () => {
        const sections = releaseOrder(groups);
        const progress = { ...emptyProgress(), read: { m1_a: 2 } };
        expect(nextInOrder(sections, progress, new Set(["m1_b"]))).toMatchObject({ entry: { id: "a1_a" } });
    });

    it("skips a story with no script rather than pointing at a 404", () => {
        const sections = releaseOrder([group("act9", { startTime: 1580943600, stories: [entry("x", { sort: 1, hasScript: false }), entry("y", { sort: 2 })] })]);
        expect(nextInOrder(sections, emptyProgress(), NO_GAME)).toMatchObject({ entry: { id: "y" } });
    });

    it("is null once everything in the order is read, which is a finished library and not a missing answer", () => {
        const sections = releaseOrder(groups);
        const progress = { ...emptyProgress(), read: { m1_a: 2, m1_b: 2, a1_a: 2, a1_b: 2 } };
        expect(nextInOrder(sections, progress, NO_GAME)).toBeNull();
    });
});
