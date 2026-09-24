import { describe, expect, it } from "vitest";
import type { Storyline } from "#/types/generated/Storyline";
import type { LibEntry, LibGroup, LibIndex, LibRecord } from "./derive";
import { buildSections, cardCode, FILTER_ORDER, fallbackSections, kindOf, matchesFilter, sectionLibrary } from "./sections";

function entry(id: string, over: Partial<LibEntry> = {}): LibEntry {
    return { id, name: id, sort: 1, groupId: "g", hasScript: true, requiredStages: [], ...over };
}

function group(id: string, over: Partial<LibGroup> = {}): LibGroup {
    return { id, name: id, category: "side", entryType: "ACTIVITY", actType: "ACTIVITY_STORY", startTime: 0, stories: [entry(`${id}_1`)], ...over };
}

function line(id: string, sort: number, groupIds: string[], arcs: Storyline["arcs"] = []): Storyline {
    return { id, name: `line ${id}`, sort, groupIds, arcs };
}

const keepAll = () => true;
const keepNone = () => false;

describe("kindOf", () => {
    it("reads the in-game label off the category and the Archives shelf", () => {
        expect(kindOf({ category: "main", displayType: undefined })).toBe("main");
        expect(kindOf({ category: "side", displayType: "SIDESTORY" })).toBe("event");
        expect(kindOf({ category: "side", displayType: "BRANCHLINE" })).toBe("intermezzo");
        expect(kindOf({ category: "vignette", displayType: "MINISTORY" })).toBe("vignette");
        expect(kindOf({ category: "record", displayType: undefined })).toBe("record");
    });

    it("keeps the 13 EN side groups with no activity row as EVENTS rather than dropping them", () => {
        expect(kindOf({ category: "side", displayType: undefined })).toBe("event");
        expect(kindOf({ category: "side", displayType: "NONE" })).toBe("event");
    });
});

describe("matchesFilter", () => {
    it("puts vignettes and intermezzi behind one Side stories pill", () => {
        expect(matchesFilter("vignette", "side")).toBe(true);
        expect(matchesFilter("intermezzo", "side")).toBe(true);
        expect(matchesFilter("event", "side")).toBe(false);
    });

    it("lets every kind through All and nothing else through the wrong pill", () => {
        for (const kind of ["main", "event", "intermezzo", "vignette", "record"] as const) {
            expect(matchesFilter(kind, "all")).toBe(true);
        }
        expect(matchesFilter("main", "events")).toBe(false);
        expect(matchesFilter("record", "records")).toBe(true);
        expect(matchesFilter("record", "main")).toBe(false);
    });

    it("has a pill for every kind, so nothing is unreachable", () => {
        for (const kind of ["main", "event", "intermezzo", "vignette", "record"] as const) {
            expect(FILTER_ORDER.some((f) => f !== "all" && matchesFilter(kind, f))).toBe(true);
        }
    });
});

describe("buildSections", () => {
    const groups = [group("main_0", { category: "main" }), group("main_1", { category: "main" }), group("act9d0"), group("act5d0"), group("act77side")];

    it("cuts the mainline shelf into its arcs and the rest into one section each", () => {
        const storylines = [
            line(
                "mainLine",
                0,
                ["main_0", "main_1", "act9d0"],
                [
                    { name: "ARC ONE", sort: 9, groupIds: ["main_0"] },
                    { name: "ARC TWO", sort: 101, groupIds: ["main_1", "act9d0"] },
                ],
            ),
            line("ssLine_1", 1, ["act5d0"]),
        ];
        const sections = buildSections(groups, storylines);
        expect(sections.map((s) => [s.title, s.groups.map((g) => g.id)])).toEqual([
            ["ARC ONE", ["main_0"]],
            ["ARC TWO", ["main_1", "act9d0"]],
            ["line ssLine_1", ["act5d0"]],
            ["", ["act77side"]],
        ]);
        expect(sections[3]?.kind).toBe("other");
    });

    it("gives a group on several shelves to the LOWEST-sorted one and never twice", () => {
        const storylines = [line("ssLine_1", 1, ["act9d0", "act5d0"]), line("ssLine_2", 2, ["act9d0"])];
        const sections = buildSections(groups, storylines);
        const owner = sections.filter((s) => s.groups.some((g) => g.id === "act9d0"));
        expect(owner).toHaveLength(1);
        expect(owner[0]?.id).toBe("line-ssLine_1");
        expect(sections.find((s) => s.id === "line-ssLine_2")).toBeUndefined();
    });

    it("drops a shelf that names nothing the index carries", () => {
        expect(buildSections(groups, [line("ssLine_9", 9, ["act_does_not_exist"])]).map((s) => s.id)).toEqual(["other-events"]);
    });

    it("gives an arc its OWN icon and its OWN chapter range, and a shelf without arcs the shelf's", () => {
        const groups = [group("main_0", { category: "main" }), group("main_1", { category: "main" }), group("act18d0")];
        const mainLine: Storyline & { iconUrl?: string } = { ...line("mainLine", 0, ["act18d0"], [{ name: "HOUR OF AN AWAKENING", sort: 0, groupIds: ["main_0", "main_1"], iconUrl: "/act_0.png", chapterRange: { from: 0, to: 3 } }]), iconUrl: "/mainline_logo.png" };
        const sections = buildSections(groups, [mainLine]);
        expect(sections.map((s) => [s.id, s.iconUrl, s.iconWide, s.chapterRange])).toEqual([
            ["arc-mainLine-0", "/act_0.png", true, { from: 0, to: 3 }],
            ["line-mainLine", "/mainline_logo.png", undefined, undefined],
        ]);
    });

    it("falls back to the shelf's glyph on an arc the wire gives no icon, and to no icon at all when neither has one", () => {
        const groups = [group("main_0", { category: "main" })];
        const withShelf: Storyline & { iconUrl?: string } = { ...line("mainLine", 0, [], [{ name: "arc", sort: 0, groupIds: ["main_0"] }]), iconUrl: "/mainline_logo.png" };
        expect(buildSections(groups, [withShelf]).map((s) => [s.iconUrl, s.iconWide])).toEqual([["/mainline_logo.png", false]]);
        expect(buildSections(groups, [line("mainLine", 0, [], [{ name: "arc", sort: 0, groupIds: ["main_0"] }])]).map((s) => [s.iconUrl, s.iconWide])).toEqual([[undefined, false]]);
    });

    it("never shelves an operator record group", () => {
        const withRecord = [...groups, group("story_x_set_1", { category: "record" })];
        const sections = buildSections(withRecord, [line("ssLine_1", 1, ["story_x_set_1"])]);
        expect(sections.flatMap((s) => s.groups.map((g) => g.id))).not.toContain("story_x_set_1");
    });
});

describe("fallbackSections", () => {
    it("orders the main chapters numerically, then events newest year first, then the undated", () => {
        const groups = [group("main_10", { category: "main" }), group("main_2", { category: "main" }), group("act_2024", { startTime: Date.UTC(2024, 5, 1) / 1000 }), group("act_2023", { startTime: Date.UTC(2023, 5, 1) / 1000 }), group("act_undated", { startTime: -1 })];
        expect(fallbackSections(groups).map((s) => [s.id, s.groups.map((g) => g.id)])).toEqual([
            ["main-story", ["main_2", "main_10"]],
            ["year-2024", ["act_2024"]],
            ["year-2023", ["act_2023"]],
            ["undated", ["act_undated"]],
        ]);
    });
});

describe("sectionLibrary", () => {
    const index: LibIndex = {
        groups: [group("main_0", { category: "main" }), group("act9d0", { displayType: "BRANCHLINE" }), group("act5d0", { displayType: "SIDESTORY" }), group("story_x_set_1", { category: "record" })],
        records: [{ charId: "char_x", name: "X", stories: [entry("e")] } as LibRecord],
        storylines: [line("mainLine", 0, ["main_0"]), line("ssLine_1", 1, ["act9d0", "act5d0"])],
    };

    it("reports the game's own cut when the wire carries storylines", () => {
        const library = sectionLibrary(index, "all", keepAll, keepAll);
        expect(library.fallback).toBe(false);
        expect(library.sections.map((s) => s.id)).toEqual(["line-mainLine", "line-ssLine_1"]);
        expect(library.records).toHaveLength(1);
    });

    it("falls back to the chapter-and-year cut when the wire carries none", () => {
        const library = sectionLibrary({ ...index, storylines: undefined }, "all", keepAll, keepAll);
        expect(library.fallback).toBe(true);
        expect(library.sections.map((s) => s.id)).toEqual(["main-story", "undated"]);
    });

    it("drops a section the pill empties, and drops the records under any pill but All and Operator records", () => {
        const library = sectionLibrary(index, "main", keepAll, keepAll);
        expect(library.sections.map((s) => s.id)).toEqual(["line-mainLine"]);
        expect(library.records).toHaveLength(0);
        expect(sectionLibrary(index, "records", keepAll, keepAll).sections).toHaveLength(0);
        expect(sectionLibrary(index, "records", keepAll, keepAll).records).toHaveLength(1);
    });

    it("leaves nothing standing when the search matches nothing", () => {
        const library = sectionLibrary(index, "all", keepNone, keepNone);
        expect(library.sections).toHaveLength(0);
        expect(library.records).toHaveLength(0);
    });
});

describe("cardCode", () => {
    it("prints the chapter's own third name on the mainline", () => {
        expect(cardCode(group("main_0", { category: "main", zone: { nameThird: "EPISODE 00" } }))).toBe("EPISODE 00");
    });

    it("skips the table's ENTRY placeholder and reads the act's real code, which is the Code of Brawl defect", () => {
        // act5d0 ships storyCode ENTRY on its first-enter entry and CB-1 on the
        // rest; the card printed "ENTRY · 2020 · 27 entries".
        const act5d0 = group("act5d0", { stories: [entry("a", { code: "ENTRY" }), entry("b", { code: "CB-1" }), entry("c", { code: "CB-2" })] });
        expect(cardCode(act5d0)).toBe("CB");
        // act3d0 is the only other one in the EN corpus.
        const act3d0 = group("act3d0", { stories: [entry("a", { code: "ENTRY" }), entry("b", { code: "OF-ST1" })] });
        expect(cardCode(act3d0)).toBe("OF");
    });

    it("falls back to the uppercased group id, never to a placeholder word", () => {
        expect(cardCode(group("act10mini", { stories: [entry("a"), entry("b")] }))).toBe("ACT10MINI");
        expect(cardCode(group("act5d0", { stories: [entry("a", { code: "ENTRY" })] }))).toBe("ACT5D0");
    });
});
