import { describe, expect, it } from "vitest";
import type { StoryIllustrations } from "#/types/generated/StoryIllustrations";
import type { LibGroup, LibRecord } from "./derive";
import { artSummary, artTabs, backgroundRows, categoryTotal, cgRows, countOf, defaultPane, type IArtGroup, illustrationRows, paneCounts, paneRows, spriteRows } from "./illustrations";

type Counts = { illustrationCount?: number; spriteCount?: number };

function group(id: string, over: Partial<LibGroup> & Counts = {}): LibGroup {
    return { id, name: id, category: "side", entryType: "ACTIVITY", actType: "NONE", startTime: 0, stories: [], ...over } as LibGroup;
}

function record(charId: string, over: Partial<LibRecord> & Counts = {}): LibRecord {
    return { charId, name: charId, stories: [], ...over } as LibRecord;
}

function card(over: Partial<IArtGroup> = {}): IArtGroup {
    return { id: "a", name: "a", art: { kind: "none" }, code: "A", kind: "event", chapter: null, ...over };
}

function illus(name: string, url: string | null, storyIds: string[] = ["s1"]) {
    return { name, url, storyIds };
}

function payload(over: Partial<StoryIllustrations> = {}): StoryIllustrations {
    return { groupId: "g", backgrounds: [], images: [], sprites: [], ...over };
}

describe("artTabs", () => {
    it("shelves story groups under the Stories tab's own categories and the records last", () => {
        const tabs = artTabs([group("a", { category: "main" }), group("b"), group("c", { displayType: "BRANCHLINE" })], [record("char_1")]);
        expect(tabs.map((t) => t.key)).toEqual(["main", "side", "intermezzi", "record"]);
        expect(tabs[3]?.groups).toEqual([{ id: "char_1", name: "char_1", art: { kind: "none" }, code: "char_1", kind: "record", chapter: null, illustrationCount: undefined, spriteCount: undefined }]);
    });

    it("drops a category with no groups, and the records entirely when there are none", () => {
        expect(artTabs([group("a")], []).map((t) => t.key)).toEqual(["side"]);
    });

    it("reads the chapter's own third name as the typographic fallback, as the Stories tab does", () => {
        const [main] = artTabs([group("a", { category: "main", zone: { nameThird: "EPISODE 00" } })], []);
        expect(main?.groups[0]?.code).toBe("EPISODE 00");
    });

    it("carries the counts through and draws the key visual over the derived cover", () => {
        const [side] = artTabs([group("a", { bannerUrl: "/kv.png", coverUrl: "/cover.png", illustrationCount: 7, spriteCount: 3 })], []);
        expect(side?.groups[0]).toEqual({ id: "a", name: "a", art: { kind: "banner", url: "/kv.png" }, code: "A", kind: "event", chapter: null, illustrationCount: 7, spriteCount: 3 });
    });

    it("falls back to the cover, then to the typographic code, exactly as every other library surface does", () => {
        const [side] = artTabs([group("a", { coverUrl: "/cover.png" }), group("b")], []);
        expect(side?.groups.map((g) => g.art)).toEqual([{ kind: "cover", url: "/cover.png" }, { kind: "none" }]);
    });

    it("badges a mainline card and gives it its chapter number", () => {
        const [main] = artTabs([group("a", { category: "main", chapterNumber: 4 })], []);
        expect(main?.groups[0]).toMatchObject({ kind: "main", chapter: 4 });
    });

    it("keeps an operator's AVATAR as its own kind of picture and treats the empty string as absent", () => {
        const [records] = artTabs([], [record("char_1", { avatarUrl: "" }), record("char_2", { avatarUrl: "/a.png" })]);
        expect(records?.groups.map((g) => g.art)).toEqual([{ kind: "none" }, { kind: "avatar", url: "/a.png" }]);
    });
});

describe("countOf and categoryTotal", () => {
    it("is null, never zero, when the wire carries no count", () => {
        expect(countOf(card(), "illustrations")).toBeNull();
        expect(countOf(card({ illustrationCount: 0 }), "illustrations")).toBe(0);
    });

    it("reads the sub-tab's own field", () => {
        const g = card({ illustrationCount: 7, spriteCount: 3 });
        expect(countOf(g, "illustrations")).toBe(7);
        expect(countOf(g, "sprites")).toBe(3);
    });

    it("sums only the cards that carry a count, and is null when none do", () => {
        const groups = [card({ id: "a", illustrationCount: 7 }), card({ id: "b" }), card({ id: "c", illustrationCount: 5 })];
        expect(categoryTotal(groups, "illustrations")).toBe(12);
        expect(categoryTotal(groups, "sprites")).toBeNull();
        expect(categoryTotal([], "illustrations")).toBeNull();
    });
});

describe("the panel's three panes", () => {
    // main_0 as :3060 answers it on 2026-09-24: 4 backgrounds, 13 images, 11
    // sprites, and `illustrationCount` 17. The reference site's "13
    // illustrations" for the Prologue is the CG pane alone.
    const prologue = payload({ backgrounds: [illus("bg_a", "/a.png"), illus("bg_b", "/b.png"), illus("bg_c", "/c.png"), illus("bg_d", "/d.png")], images: Array.from({ length: 13 }, (_, i) => illus(`cg_${i}`, `/cg_${i}.png`)), sprites: [{ base: "avg_x", bodyUrl: "/x.png", storyIds: ["s1"], faces: 3 }] });

    it("splits the card's sum into the two halves it is made of", () => {
        expect(paneCounts(prologue)).toEqual({ backgrounds: 4, cgs: 13, sprites: 1 });
        expect(paneCounts(prologue).backgrounds + paneCounts(prologue).cgs).toBe(illustrationRows(prologue).length);
    });

    it("keeps a name that both halves list out of the CG pane, so the two still add up", () => {
        const both = payload({ backgrounds: [illus("shared", "/bg.png")], images: [illus("shared", "/cg.png"), illus("cg_1", "/1.png")] });
        expect(backgroundRows(both).map((r) => r.name)).toEqual(["shared"]);
        expect(cgRows(both).map((r) => r.name)).toEqual(["cg_1"]);
        expect(paneCounts(both)).toEqual({ backgrounds: 1, cgs: 1, sprites: 0 });
    });

    it("routes each pane, and lands on the sub-tab the card was opened from", () => {
        expect(paneRows(prologue, "backgrounds")).toEqual(backgroundRows(prologue));
        expect(paneRows(prologue, "cgs")).toEqual(cgRows(prologue));
        expect(paneRows(prologue, "sprites")).toEqual(spriteRows(prologue));
        expect(defaultPane("illustrations")).toBe("backgrounds");
        expect(defaultPane("sprites")).toBe("sprites");
    });

    it("is empty, never a throw, while the endpoint answers null", () => {
        expect(paneCounts(null)).toEqual({ backgrounds: 0, cgs: 0, sprites: 0 });
        expect(cgRows(undefined)).toEqual([]);
    });
});

describe("illustrationRows and spriteRows", () => {
    it("lists the backgrounds before the CGs in wire order", () => {
        const rows = illustrationRows(payload({ backgrounds: [illus("bg_a", "/a.png"), illus("bg_b", null)], images: [illus("cg_1", "/c.png")] }));
        expect(rows.map((r) => r.name)).toEqual(["bg_a", "bg_b", "cg_1"]);
    });

    it("lists a name that appears in both halves once, under the half that saw it first", () => {
        const rows = illustrationRows(payload({ backgrounds: [illus("shared", "/bg.png")], images: [illus("shared", "/cg.png")] }));
        expect(rows).toHaveLength(1);
        expect(rows[0]?.url).toBe("/bg.png");
    });

    it("is empty, never a throw, while the endpoint answers null", () => {
        expect(illustrationRows(null)).toEqual([]);
        expect(spriteRows(undefined)).toEqual([]);
    });

    it("de-duplicates sprite bases", () => {
        const rows = spriteRows(
            payload({
                sprites: [
                    { base: "avg_x", bodyUrl: "/x.png", storyIds: ["s1"], faces: 4 },
                    { base: "avg_x", bodyUrl: null, storyIds: ["s2"], faces: 1 },
                ],
            }),
        );
        expect(rows.map((r) => r.base)).toEqual(["avg_x"]);
        expect(rows[0]?.faces).toBe(4);
    });
});

describe("artSummary", () => {
    it("counts the rows, the unresolved ones and the distinct stories", () => {
        expect(artSummary([illus("a", "/a.png", ["s1", "s2"]), illus("b", null, ["s2"])])).toEqual({ rows: 2, missing: 1, stories: 2 });
    });

    it("reads a sprite's bodyUrl for the same question", () => {
        expect(artSummary([{ base: "x", bodyUrl: null, storyIds: ["s1"], faces: 2 }])).toEqual({ rows: 1, missing: 1, stories: 1 });
    });

    it("is all zeroes over nothing", () => {
        expect(artSummary([])).toEqual({ rows: 0, missing: 0, stories: 0 });
    });
});
