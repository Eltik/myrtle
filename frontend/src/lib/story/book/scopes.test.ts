import { describe, expect, it } from "vitest";
import type { Storyline } from "#/types/generated/Storyline";
import { type BookIndex, bookOf, scopeFileName, scopeStories } from "./book";
import { imagePaths } from "./epub";
import { estimateBook } from "./estimate";
import { scriptOf } from "./fixtures";
import { arcOf, arcScope, orderRange, rangeScope, storylineOf, storylineScope } from "./scopes";

const entry = (id: string, code: string, sort: number) => ({ id, name: id.toUpperCase(), code, sort, hasScript: true, wordCount: 100 });
const index: BookIndex = {
    groups: [
        { id: "main_0", name: "Evil Time Part 1", bannerUrl: "/kv0.png", illustrationCount: 4, stories: [entry("m0b", "0-2", 2), entry("m0a", "0-1", 1)] },
        { id: "main_1", name: "Evil Time Part 2", coverUrl: "/c1.png", illustrationCount: 4, stories: [entry("m1a", "1-1", 1)] },
        { id: "main_4", name: "Stinging Shock", stories: [entry("m4a", "4-1", 1)] },
        { id: "act_a", name: "Event A", stories: [entry("aa", "A-1", 1), entry("m0a", "0-1", 9)] },
        { id: "act_b", name: "Event B", stories: [entry("bb", "B-1", 1)] },
    ],
};
const line = (id: string, sort: number, groupIds: string[], arcs: Storyline["arcs"] = []): Storyline => ({ id, name: id, sort, groupIds, arcs });
const storylines: Storyline[] = [
    line("ssLine_2", 3, ["act_b", "act_a"]),
    line(
        "mainLine",
        1,
        ["main_0", "main_1", "main_4"],
        [
            { name: "HOUR OF AN AWAKENING", sort: 1, groupIds: ["main_0", "main_1"] },
            { name: "SHATTER OF A VISION", sort: 2, groupIds: ["main_4"] },
        ],
    ),
    line("ssLine_1", 2, ["act_a"]),
];

describe("scope builders", () => {
    it("the arc is the mainline arc holding the chapter, in the arc's order", () => {
        expect(arcOf(storylines, "main_1")).toEqual({ name: "HOUR OF AN AWAKENING", groupIds: ["main_0", "main_1"], key: "mainLine-1" });
        expect(arcOf(storylines, "act_a")).toBeNull();
    });

    it("the storyline is the lowest-sort themed shelf holding the group; the mainline shelf is never one", () => {
        expect(storylineOf(storylines, "act_a")?.key).toBe("ssLine_1");
        expect(storylineOf(storylines, "act_b")?.groupIds).toEqual(["act_b", "act_a"]);
        expect(storylineOf(storylines, "main_0")).toBeNull();
    });

    it("a reading-order range is inclusive and runs in the order's direction whichever end was picked first", () => {
        const order = ["main_0", "act_a", "main_1", "act_b"];
        expect(orderRange(order, "act_a", "act_b")).toEqual(["act_a", "main_1", "act_b"]);
        expect(orderRange(order, "act_b", "act_a")).toEqual(["act_a", "main_1", "act_b"]);
        expect(orderRange(order, "nope", "act_b")).toEqual([]);
    });
});

describe("a book over several groups", () => {
    const scope = rangeScope("release", ["main_0", "act_a", "act_b"], "main_0", "act_a", "Release order: Evil Time Part 1 to Event A");
    const scripts = new Map(
        ["m0a", "m0b", "aa", "bb", "m1a"].map((id) => [
            id,
            scriptOf(
                id,
                [
                    ["background", { image: "bg" }],
                    ["image", { image: "cg" }],
                ],
                { backgrounds: { bg: "/bg.png" }, images: { cg: "/cg.png" } },
            ),
        ]),
    );

    it("one part per group in the scope's order, stories in sort order, a story shared by two groups printed once", () => {
        expect(scopeStories(index, scope).map((s) => `${s.group.id}/${s.entry.id}`)).toEqual(["main_0/m0a", "main_0/m0b", "act_a/aa"]);
        const book = bookOf({ index, scripts, server: "en" }, scope, { nickname: "Doctor", images: "cg+bg", branches: "all" });
        expect(book.parts.map((p) => [p.title, p.sections.map((s) => s.id)])).toEqual([
            ["Evil Time Part 1", ["m0a", "m0b"]],
            ["Event A", ["aa"]],
        ]);
        expect(book.meta).toMatchObject({ title: "Release order: Evil Time Part 1 to Event A", identifier: "urn:myrtle:story:en:order-release-main_0-act_a", partArt: { main_0: "/kv0.png", act_a: undefined } });
        // Every section names the same two images; the EPUB writes each once for the whole book.
        const keys = book.parts.flatMap((p) => p.sections).flatMap((s) => imagePaths(s.blocks, "cg+bg"));
        expect(new Set(keys)).toEqual(new Set(["thumb:/bg.png", "/cg.png"]));
    });

    it("arc and storyline scopes resolve, are named and file-named, and estimate over the whole scope", () => {
        const arc = arcScope(storylines, "main_0");
        expect(arc).toEqual({ kind: "groups", ids: ["main_0", "main_1"], title: "HOUR OF AN AWAKENING", id: "arc-mainLine-1" });
        expect(scopeFileName(index, arc as NonNullable<typeof arc>, "epub")).toBe("HOUR OF AN AWAKENING · 0-1–1-1.epub");
        expect(storylineScope(storylines, "act_a")).toMatchObject({ ids: ["act_a"], id: "storyline-ssLine_1" });
        const e = estimateBook(index, arc as NonNullable<typeof arc>, { images: "none", format: "text", typeface: "device" });
        expect([e.stories, e.words]).toEqual([3, 300]);
    });
});
