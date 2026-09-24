/**
 * The community shaper on a hand-built index: what gets ranked, what gets
 * dropped, and the one verdict that is not a number, "finished is not
 * measurable here".
 *
 * The fixture keeps the shapes the real index has and the real aggregate
 * answered on 2026-09-24: a gated mainline chapter, a gated event, a MINISTORY
 * vignette with no gate anywhere in it, a record set that the wire lists twice,
 * a chapter nobody has read, and a chapter the aggregate does not mention.
 */
import { describe, expect, it } from "vitest";
import type { StoryCommunity } from "#/types/generated/StoryCommunity";
import { DEFAULT_DEPTH_GROUP, finishedIsMeasurable, rankCommunity } from "./community";
import type { LibEntry, LibGroup, LibIndex } from "./derive";

function story(id: string, name: string, gated: boolean, extra: Partial<LibEntry> = {}): LibEntry {
    return { id, name, groupId: "", sort: 0, hasScript: true, requiredStages: gated ? ["main_01-01"] : [], ...extra } as LibEntry;
}

function group(id: string, name: string, category: string, stories: LibEntry[], displayType?: string): LibGroup {
    return { id, name, category, displayType, entryType: "NONE", actType: "NONE", startTime: -1, stories: stories.map((s) => ({ ...s, groupId: id })) } as unknown as LibGroup;
}

const index: LibIndex = {
    groups: [
        group("main_1", "Black Trail", "main", [story("m1a", "Prologue", false, { code: "1-1" }), story("m1b", "Trail", true, { code: "1-2" }), story("m1c", "Ash", true, { code: "1-3" })]),
        group("act1side", "Grani and the Knights", "side", [story("a1a", "Arrival", true, { code: "GT-1" }), story("a1b", "Departure", true, { code: "GT-2" })], "SIDESTORY"),
        group("act4d0", "Twilight of Wolumonde", "vignette", [story("v1a", "Snow", false), story("v1b", "Thaw", false)], "MINISTORY"),
        group("act99side", "Unread Event", "side", [story("u1a", "Nobody", true, { code: "UN-1" })], "SIDESTORY"),
        group("act98side", "Unmentioned Event", "side", [story("x1a", "Absent", true, { code: "XX-1" })], "SIDESTORY"),
        group("story_kalts_set_1", "Kal'tsit", "record", [story("r1a", "End of a Long Journey", false, { hasScript: false })]),
    ],
    records: [],
};

const community: StoryCommunity = {
    players: 100,
    computedAt: 1_758_700_000,
    stories: [
        { id: "m1a", readers: 90 },
        { id: "m1b", readers: 70 },
        { id: "m1c", readers: 80 },
        { id: "a1a", readers: 60 },
        { id: "a1b", readers: 55 },
        { id: "v1a", readers: 12 },
        { id: "v1b", readers: 9 },
        { id: "u1a", readers: 0 },
        { id: "r1a", readers: 7 },
    ],
    groups: [
        { id: "main_1", readers: 90, finished: 65, depth: [90, 70, 80] },
        { id: "act1side", readers: 60, finished: 40, depth: [60, 55] },
        { id: "act4d0", readers: 12, finished: 0, depth: [12, 9] },
        { id: "act99side", readers: 0, finished: 0, depth: [0] },
        { id: "story_kalts_set_1", readers: 7, finished: 0 },
        { id: "char_003_kalts", readers: 7, finished: 0 },
    ],
};

const ranked = rankCommunity(index, community);

describe("rankCommunity", () => {
    it("has nothing to shape when the backend served no aggregate", () => {
        expect(rankCommunity(index, null)).toBeNull();
        expect(rankCommunity(index, undefined)).toBeNull();
    });

    it("ranks the chapters and events by readers, and leaves the record sets out of that ranking entirely", () => {
        expect(ranked?.top.map((r) => r.id)).toEqual(["main_1", "act1side", "act4d0", "act99side"]);
        expect(ranked?.top.map((r) => r.readers)).toEqual([90, 60, 12, 0]);
        expect(ranked?.top.some((r) => r.id === "story_kalts_set_1" || r.id === "char_003_kalts")).toBe(false);
    });

    it("drops a group the aggregate never mentions rather than ranking it at zero", () => {
        expect(ranked?.top.some((r) => r.id === "act98side")).toBe(false);
    });

    it("carries the badge the Browse ticket wears and the share of players", () => {
        expect(ranked?.top.map((r) => r.kind)).toEqual(["main", "event", "vignette", "event"]);
        expect(ranked?.top[0]?.readerShare).toBeCloseTo(0.9, 10);
        expect(ranked?.top[0]?.finishedShare).toBeCloseTo(0.65, 10);
    });

    it("counts finishers only where a stage gates at least one story, and answers null where none does", () => {
        const by = new Map(ranked?.top.map((r) => [r.id, r]));
        expect(by.get("main_1")?.finished).toBe(65);
        expect(by.get("act1side")?.finished).toBe(40);
        expect(by.get("act4d0")?.finished).toBeNull();
        expect(by.get("act4d0")?.finishedShare).toBeNull();
    });

    it("keeps a chapter measurable when only some of its stories are gated, which is what main_0 looks like", () => {
        expect(finishedIsMeasurable(index.groups[0])).toBe(true);
        expect(finishedIsMeasurable(index.groups[2])).toBe(false);
    });

    it("reads a missing requiredStages as no gate rather than throwing, for a backend older than the field", () => {
        const older = { stories: [{ id: "z", name: "Z", groupId: "g", sort: 0, hasScript: true }] } as unknown as LibGroup;
        expect(finishedIsMeasurable(older)).toBe(false);
    });

    it("lists the least read from the bottom up and says how many it skipped for having no reader", () => {
        expect(ranked?.bottom.map((r) => r.id)).toEqual(["act4d0", "act1side", "main_1"]);
        expect(ranked?.bottom.every((r) => r.readers > 0)).toBe(true);
        expect(ranked?.skippedZero).toBe(1);
    });

    it("ranks single stories over the whole library, a record's story included, and names the group each belongs to", () => {
        expect(ranked?.stories.map((s) => s.id)).toEqual(["m1a", "m1c", "m1b", "a1a", "a1b", "v1a", "v1b", "r1a", "u1a"]);
        expect(ranked?.stories[0]).toMatchObject({ name: "Prologue", code: "1-1", groupName: "Black Trail", readers: 90, hasScript: true });
        expect(ranked?.stories.find((s) => s.id === "r1a")).toMatchObject({ groupName: "Kal'tsit", hasScript: false });
    });

    it("draws a depth curve only for a group whose curve lines up with the stories the index lists", () => {
        expect(ranked?.depth.map((d) => d.id)).toEqual(["main_1", "act1side", "act4d0"]);
        expect(ranked?.depth[0]?.bars.map((b) => b.readers)).toEqual([90, 70, 80]);
        expect(ranked?.depth[0]?.bars.map((b) => b.code)).toEqual(["1-1", "1-2", "1-3"]);
    });

    it("keeps a curve that rises again, because a special stage is entered by its own door", () => {
        expect(ranked?.depth[0]?.bars[1]?.readers).toBeLessThan(ranked?.depth[0]?.bars[2]?.readers ?? 0);
    });

    it("has no curve for a record group, which the wire omits `depth` on, nor for a one-story chapter", () => {
        expect(ranked?.depth.some((d) => d.id === "story_kalts_set_1")).toBe(false);
        expect(ranked?.depth.some((d) => d.id === "act99side")).toBe(false);
    });

    it("opens the strip on the first mainline chapter when it has a curve", () => {
        expect(ranked?.defaultDepth).toBe(DEFAULT_DEPTH_GROUP);
    });

    it("opens the strip on the first curve there is when the mainline chapter has none", () => {
        const without = { ...community, groups: community.groups.filter((g) => g.id !== "main_1") };
        expect(rankCommunity(index, without)?.defaultDepth).toBe("act1side");
        const none = { ...community, groups: community.groups.map((g) => ({ ...g, depth: undefined })) };
        expect(rankCommunity(index, none)?.defaultDepth).toBeNull();
        expect(rankCommunity(index, none)?.depth).toEqual([]);
    });

    it("answers a zero share rather than a NaN when the aggregate counted nobody", () => {
        const nobody = { ...community, players: 0 };
        const out = rankCommunity(index, nobody);
        expect(out?.top[0]?.readerShare).toBe(0);
        expect(out?.top[0]?.finishedShare).toBe(0);
        expect(out?.stories[0]?.readerShare).toBe(0);
        expect(out?.depth[0]?.bars[0]?.readerShare).toBe(0);
    });

    it("hands the wire's own computedAt and players through untouched", () => {
        expect(ranked?.players).toBe(100);
        expect(ranked?.computedAt).toBe(1_758_700_000);
    });
});
