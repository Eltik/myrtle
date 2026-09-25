import { describe, expect, it } from "vitest";
import type { StoryEntry } from "#/types/generated/StoryEntry";
import type { StoryGroup } from "#/types/generated/StoryGroup";
import type { StoryIndex } from "#/types/generated/StoryIndex";
import { ogHash } from "./hash";
import { buildStoryOgData, parseStoryOgId, storyOgHashParts, storyOgId } from "./story";

function entry(id: string, over: Partial<StoryEntry> = {}): StoryEntry {
    return { id, name: id, sort: 0, groupId: "g", hasScript: true, wordCount: 450, hasVideo: false, requiredStages: [], ...over };
}

function group(id: string, stories: StoryEntry[], over: Partial<StoryGroup> = {}): StoryGroup {
    return { id, name: id, category: "side", entryType: "ACTIVITY", actType: "ACTIVITY_STORY", startTime: -1, wordCount: 0, illustrationCount: 0, spriteCount: 0, stories, ...over };
}

/** The builder's result where the test has already established it exists. */
function must<T>(value: T | null | undefined): T {
    if (value === null || value === undefined) throw new Error("expected a card");
    return value;
}

function index(groups: StoryGroup[], records: StoryIndex["records"] = []): StoryIndex {
    return { groups, records, storylines: [], totals: {} as StoryIndex["totals"] };
}

const main0 = group("main_0", [entry("main_0_level_main_00-01_end", { name: "Collapse", code: "0-1", sort: 4, avgTag: "After Operation" }), entry("main_0_level_main_00-01_beg", { name: "Collapse", code: "0-1", sort: 3, avgTag: "Before Operation", wordCount: 416 })], {
    name: "Evil Time Part 1",
    category: "main",
    chapterNumber: 0,
    zone: { chapterName: "Hour of An Awakening", nameFirst: "Prologue", nameSecond: "Evil Time Part 1", nameThird: "EPISODE 00" } as StoryGroup["zone"],
    bannerUrl: "/textures/spritepack/mixstory_kv_sprites_0/kv_evil_time_part1.png",
    coverUrl: "/textures/avg/bg/avg_bkg_h1_bg_in_0/bg_indoor_1.png",
});

describe("buildStoryOgData", () => {
    it("prints the code, the phase, the chapter tag and the story's place, and draws the key visual before the cover", () => {
        const data = buildStoryOgData(index([main0]), "main_0_level_main_00-01_beg");
        expect(data).toMatchObject({
            name: "Collapse",
            groupName: "Evil Time Part 1",
            categoryLabel: "Main story",
            chapterTag: "EPISODE 00",
            code: "0-1",
            phase: "Before Operation",
            artKind: "banner",
            artPath: "/textures/spritepack/mixstory_kv_sprites_0/kv_evil_time_part1.png",
            artPosition: "50% 20%",
            watermark: "EP 00",
            server: "en",
        });
        // Sorted by `sort`, not by wire order: the `_beg` entry is listed second and plays first.
        expect(data?.stats).toEqual([
            { label: "Story", value: "1 / 2" },
            { label: "Words", value: "416" },
            { label: "Read time", value: "~2m" },
        ]);
        expect(data?.cutsceneLabel).toBeUndefined();
        expect(data?.noScriptLabel).toBeUndefined();
    });

    it("falls back to the cover, then to no art, never to a broken path", () => {
        const withCover = group("act11d7", [entry("s1", { code: "FA-1" })], { coverUrl: "/cover.png" });
        expect(buildStoryOgData(index([withCover]), "s1")).toMatchObject({ artKind: "cover", artPath: "/cover.png", artPosition: "50% 50%" });
        const bare = group("act99", [entry("s2", { code: "XX-ST-1" })]);
        const data = buildStoryOgData(index([bare]), "s2");
        expect(data?.artKind).toBe("none");
        expect(data?.artPath).toBeUndefined();
        expect(data?.watermark).toBe("XX");
        expect(storyOgHashParts(must(data))).toContain("none");
    });

    it("names a record story by its OPERATOR and draws the operator, in the rarity colour", () => {
        const story = entry("story_kalts_set_1_story_1", { name: "End of a Long Journey", groupId: "story_kalts_set_1", sort: 101, wordCount: 3109 });
        const recGroup = group("story_kalts_set_1", [story], { name: "Kal'tsit's record set", category: "record", coverUrl: "/textures/avg/bg/bg_battlefield.png" });
        const records: StoryIndex["records"] = [{ charId: "char_003_kalts", name: "Kal'tsit", rarity: 6, profession: "MEDIC", avatarUrl: "/a.png", wordCount: 3109, illustrationCount: 0, spriteCount: 0, stories: [story] }];
        const data = buildStoryOgData(index([recGroup], records), story.id);
        expect(data).toMatchObject({ name: "End of a Long Journey", groupName: "Kal'tsit", categoryLabel: "Operator records", artKind: "operator", artPath: "/textures/chararts/char_003_kalts/char_003_kalts_1.png", accent: "#f7a452" });
        expect(data?.code).toBeUndefined();
        // A lone story has no position to print.
        expect(data?.stats.map((s) => s.label)).toEqual(["Words", "Read time"]);
    });

    it("marks a cutscene and a story with no script, and prints no reading stats for the latter", () => {
        const g = group("act51side", [entry("st01", { code: "PA-ST-1", hasScript: false, wordCount: 0 }), entry("st02", { code: "PA-1", sort: 1, hasVideo: true })]);
        const noScript = buildStoryOgData(index([g]), "st01");
        expect(noScript?.noScriptLabel).toBe("No script yet");
        expect(noScript?.stats.map((s) => s.label)).toEqual(["Story"]);
        expect(buildStoryOgData(index([g]), "st02")?.cutsceneLabel).toBe("Cutscene");
    });

    it("returns null for a story the index does not list", () => {
        expect(buildStoryOgData(index([main0]), "nope")).toBeNull();
    });

    it("resolves the card's words from the locale's catalog when it has them", () => {
        const source = { locale: "de", messages: { "meta.og.story.words": "Wörter", "story.category.main": "Hauptgeschichte" } };
        const data = buildStoryOgData(index([main0]), "main_0_level_main_00-01_beg", { source });
        expect(data?.categoryLabel).toBe("Hauptgeschichte");
        expect(data?.stats[1]?.label).toBe("Wörter");
        // Untranslated keys fall back to the English.
        expect(data?.stats[2]?.label).toBe("Read time");
    });
});

describe("story OG hash", () => {
    it("is stable for the same index and moves when a drawn field moves", () => {
        const a = buildStoryOgData(index([main0]), "main_0_level_main_00-01_beg");
        const b = buildStoryOgData(index([main0]), "main_0_level_main_00-01_beg");
        expect(ogHash(storyOgHashParts(must(a)))).toBe(ogHash(storyOgHashParts(must(b))));
        const renamed = buildStoryOgData(index([{ ...main0, name: "Evil Time Part 2" }]), "main_0_level_main_00-01_beg");
        expect(ogHash(storyOgHashParts(must(renamed)))).not.toBe(ogHash(storyOgHashParts(must(a))));
    });

    it("ignores the data URI, which only the handler fills", () => {
        const a = must(buildStoryOgData(index([main0]), "main_0_level_main_00-01_beg"));
        expect(ogHash(storyOgHashParts({ ...a, artURL: "data:image/png;base64,AAAA" }))).toBe(ogHash(storyOgHashParts(a)));
    });
});

describe("storyOgId", () => {
    it("keeps the default server's id bare and prefixes any other", () => {
        expect(storyOgId("main_0_level_main_00-01_beg")).toBe("main_0_level_main_00-01_beg");
        expect(storyOgId("act51side_level_act51side_st01", "cn")).toBe("cn:act51side_level_act51side_st01");
        expect(storyOgId("x", "not-a-server")).toBe("x");
    });

    it("round-trips through parseStoryOgId", () => {
        expect(parseStoryOgId("cn:act51side_level_act51side_st01")).toEqual({ server: "cn", storyId: "act51side_level_act51side_st01", explicit: true });
        expect(parseStoryOgId("main_0_level_main_00-01_beg")).toEqual({ server: "en", storyId: "main_0_level_main_00-01_beg", explicit: false });
    });
});
