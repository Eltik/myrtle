import { describe, expect, it } from "vitest";
import type { StoryArtGallery } from "#/types/generated/StoryArtGallery";
import type { StoryGallery } from "#/types/generated/StoryGallery";
import { anyGalleryFilter, archiveCategory, archiveTiles, categoryOptions, filterForSource, filterGallery, groupOptions, type IGalleryFilter, NO_GALLERY_FILTER, pagesFor, searchGroups, storyArtTiles, TILE_PAGE, visibleTiles } from "./gallery";

const picture = (id: string, title: string) => ({ id, title, pictureType: "IMAGE" });

const GALLERY: StoryGallery = {
    groups: [
        { id: "act13side", name: "Near Light", pictures: [picture("act13side_pic_0", "Long Night, Near Light"), picture("act13side_pic_1", "Flickering Candle, Fleeting Shadow")] },
        { id: "rogue_2", name: "Mizuki & Caerula Arbor", pictures: [picture("pic_rogue_2_KV1", "Key Visual"), picture("pic_rogue_2_1", "Candlelight")] },
    ],
};

const art = (id: string) => ({ id, width: 1600, height: 900 });

const CGS: StoryArtGallery = {
    kind: "cg",
    groups: [
        { id: "main_0", name: "Evil Time Part 1", category: "main", pictures: [art("avg_1_1"), art("avg_1_2")] },
        { id: "act13side", name: "Near Light", category: "side", pictures: [art("ac13_2")] },
        { id: "main_1", name: "Separated Hearts", category: "main", pictures: [art("avg_2_1")] },
    ],
};

const ids = (tiles: readonly { id: string }[]) => tiles.map((tile) => tile.id);
const f = (part: Partial<IGalleryFilter>): IGalleryFilter => ({ ...NO_GALLERY_FILTER, ...part });

describe("archive tiles", () => {
    const tiles = archiveTiles(GALLERY);

    it("keeps the archive's own order, group by group, with no filter", () => {
        expect(ids(filterGallery(tiles, NO_GALLERY_FILTER))).toEqual(["act13side_pic_0", "act13side_pic_1", "pic_rogue_2_KV1", "pic_rogue_2_1"]);
        expect(filterGallery(tiles, f({ query: "  " }))[2]).toMatchObject({ kind: "archive_pic", groupId: "rogue_2", groupName: "Mizuki & Caerula Arbor", category: "is" });
    });

    it("files a theme under Integrated Strategies and an event under Side Stories", () => {
        expect(archiveCategory("rogue_5")).toBe("is");
        expect(archiveCategory("act25side")).toBe("side");
        expect(archiveCategory("rogue_x")).toBe("side");
    });

    it("filters to one group and to a category", () => {
        expect(ids(filterGallery(tiles, f({ group: "rogue_2" })))).toEqual(["pic_rogue_2_KV1", "pic_rogue_2_1"]);
        expect(ids(filterGallery(tiles, f({ categories: ["side"] })))).toEqual(["act13side_pic_0", "act13side_pic_1"]);
        expect(filterGallery(tiles, f({ group: "act99side" }))).toEqual([]);
    });

    it("ranks a title match first and finds a group by its name", () => {
        expect(ids(filterGallery(tiles, f({ query: "candle" })))).toEqual(["pic_rogue_2_1", "act13side_pic_1"]);
        expect(ids(filterGallery(tiles, f({ query: "mizuki" })))).toEqual(["pic_rogue_2_KV1", "pic_rogue_2_1"]);
        expect(ids(filterGallery(tiles, f({ group: "act13side", query: "candle" })))).toEqual(["act13side_pic_1"]);
    });
});

describe("story art tiles", () => {
    const tiles = storyArtTiles(CGS);

    it("files each picture under its group's library category, in library order", () => {
        expect(ids(tiles)).toEqual(["avg_1_1", "avg_1_2", "ac13_2", "avg_2_1"]);
        expect(tiles[0]).toMatchObject({ kind: "story_cg", title: "", groupId: "main_0", category: "main" });
        expect(ids(filterGallery(tiles, f({ categories: ["main"] })))).toEqual(["avg_1_1", "avg_1_2", "avg_2_1"]);
        expect(ids(filterGallery(tiles, f({ categories: ["main", "side"] })))).toEqual(ids(tiles));
        expect(storyArtTiles({ ...CGS, kind: "scene" })[0]?.kind).toBe("story_scene");
    });

    it("searches the story's name, and the asset key as a fallback", () => {
        expect(ids(filterGallery(tiles, f({ query: "separated" })))).toEqual(["avg_2_1"]);
        expect(ids(filterGallery(tiles, f({ query: "near light" })))).toEqual(["ac13_2"]);
    });

    it("combines category, story and search", () => {
        expect(ids(filterGallery(tiles, f({ categories: ["main"], group: "main_0", query: "evil" })))).toEqual(["avg_1_1", "avg_1_2"]);
        expect(filterGallery(tiles, f({ categories: ["side"], group: "main_0" }))).toEqual([]);
    });
});

describe("facet counts", () => {
    const tiles = storyArtTiles(CGS);

    it("counts each category under the story and search, not under the category choice", () => {
        expect(categoryOptions(tiles, NO_GALLERY_FILTER)).toEqual([
            { id: "main", count: 3 },
            { id: "side", count: 1 },
        ]);
        expect(categoryOptions(tiles, f({ categories: ["main"] }))).toEqual([
            { id: "main", count: 3 },
            { id: "side", count: 1 },
        ]);
        expect(categoryOptions(tiles, f({ group: "act13side" }))).toEqual([
            { id: "main", count: 0 },
            { id: "side", count: 1 },
        ]);
    });

    it("lists each story once in source order, counted under the categories, dropping the empty ones but the chosen", () => {
        expect(groupOptions(tiles, NO_GALLERY_FILTER)).toEqual([
            { id: "main_0", name: "Evil Time Part 1", category: "main", count: 2 },
            { id: "act13side", name: "Near Light", category: "side", count: 1 },
            { id: "main_1", name: "Separated Hearts", category: "main", count: 1 },
        ]);
        expect(groupOptions(tiles, f({ categories: ["main"] })).map((g) => g.id)).toEqual(["main_0", "main_1"]);
        expect(groupOptions(tiles, f({ categories: ["main"], group: "act13side" }))).toContainEqual({ id: "act13side", name: "Near Light", category: "side", count: 0 });
    });

    it("searches the story list by name", () => {
        const groups = groupOptions(tiles, NO_GALLERY_FILTER);
        expect(searchGroups(groups, "").map((g) => g.id)).toEqual(["main_0", "act13side", "main_1"]);
        expect(searchGroups(groups, "near").map((g) => g.id)).toEqual(["act13side"]);
    });
});

describe("filterForSource", () => {
    it("drops a category or story the source does not have, and keeps the same object when nothing drops", () => {
        const archive = archiveTiles(GALLERY);
        const kept = f({ categories: ["is"], group: "rogue_2" });
        expect(filterForSource(archive, kept)).toBe(kept);
        expect(filterForSource(archive, f({ categories: ["main", "is"], group: "main_0", query: "x" }))).toEqual(f({ categories: ["is"], query: "x" }));
    });

    it("says whether anything filters", () => {
        expect(anyGalleryFilter(NO_GALLERY_FILTER)).toBe(false);
        expect(anyGalleryFilter(f({ query: " " }))).toBe(false);
        expect(anyGalleryFilter(f({ group: "g" }))).toBe(true);
        expect(anyGalleryFilter(f({ categories: ["side"] }))).toBe(true);
    });
});

describe("visibleTiles", () => {
    const many = Array.from({ length: TILE_PAGE * 2 + 5 }, (_, i) => i);

    it("mounts one page at first, a page more per step, and never past the end", () => {
        expect(visibleTiles(many, 1)).toHaveLength(TILE_PAGE);
        expect(visibleTiles(many, 0)).toHaveLength(TILE_PAGE);
        expect(visibleTiles(many, 2)).toHaveLength(TILE_PAGE * 2);
        expect(visibleTiles(many, 9)).toHaveLength(many.length);
    });

    it("mounts enough pages to hold a tile", () => {
        expect(pagesFor(-1)).toBe(1);
        expect(pagesFor(0)).toBe(1);
        expect(pagesFor(TILE_PAGE - 1)).toBe(1);
        expect(pagesFor(TILE_PAGE)).toBe(2);
    });
});
