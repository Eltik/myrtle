import { describe, expect, it } from "vitest";
import type { StoryIllustrations } from "#/types/generated/StoryIllustrations";
import { bookFileName, bookOf, scopeId, scopeStories } from "./book";
import { BACKGROUND_BYTES, CG_BYTES, CG_SHARE, EPUB_BYTES_PER_WORD, EPUB_FIXED_BYTES, estimateBook, FONT_BYTES, TEXT_BYTES_PER_WORD } from "./estimate";
import { sampleIndex, sampleSource } from "./sample";
import type { BookOptions } from "./types";

const OPTIONS: BookOptions = { nickname: "Doctor", images: "cg", branches: "all" };

describe("scopes", () => {
    it("a group is its scripted stories in sort order; a selection keeps its own order", () => {
        expect(scopeStories(sampleIndex, { kind: "group", groupId: "main_0" }).map((s) => s.entry.id)).toEqual(["s_01_beg", "s_01_end", "s_int"]);
        expect(scopeStories(sampleIndex, { kind: "selection", ids: ["x_1", "s_int", "s_none", "nope"] }).map((s) => s.entry.id)).toEqual(["x_1", "s_int"]);
        expect(scopeStories(sampleIndex, { kind: "story", storyId: "s_01_end" }).map((s) => s.group.id)).toEqual(["main_0"]);
    });

    it("identifiers are stable per scope", () => {
        expect(scopeId({ kind: "group", groupId: "main_0" })).toBe("main_0");
        expect(scopeId({ kind: "selection", ids: ["a", "b"] })).toBe(scopeId({ kind: "selection", ids: ["a", "b"] }));
        expect(scopeId({ kind: "selection", ids: ["a", "b"] })).not.toBe(scopeId({ kind: "selection", ids: ["b", "a"] }));
    });

    it("a selection across groups is one part per group, in first-appearance order", () => {
        const book = bookOf(sampleSource, { kind: "selection", ids: ["x_1", "s_01_beg", "s_int"] }, OPTIONS);
        expect(book.parts.map((p) => [p.title, p.sections.map((s) => s.id)])).toEqual([
            ["Other & Co", ["x_1"]],
            ["Evil Time Part 1", ["s_01_beg", "s_int"]],
        ]);
        expect(book.meta.identifier).toMatch(/^urn:myrtle:story:en:selection-[0-9a-f]{8}$/);
    });

    it("the book's metadata: title, identifier, cover art, synopses", () => {
        const book = bookOf(sampleSource, { kind: "group", groupId: "main_0" }, OPTIONS);
        expect(book.meta).toMatchObject({ title: "Evil Time Part 1", identifier: "urn:myrtle:story:en:main_0", language: "en", cover: { titleImageUrl: "/title.png", bannerUrl: "/kv.png" }, description: "0-1 Isolated Island: Amiya wakes the Doctor." });
        expect(bookOf(sampleSource, { kind: "story", storyId: "s_01_end" }, OPTIONS).meta.title).toBe("Evil Time Part 1: 0-1 Isolated Island");
    });
});

describe("file names", () => {
    it("<Group> · <first code>–<last code>, sanitised", () => {
        expect(bookFileName(bookOf(sampleSource, { kind: "group", groupId: "main_0" }, OPTIONS), "epub")).toBe("Evil Time Part 1 · 0-1.epub");
        expect(bookFileName(bookOf(sampleSource, { kind: "selection", ids: ["s_01_beg", "x_1"] }, OPTIONS), "md")).toBe("Evil Time Part 1 · 0-1–X-1.md");
        expect(bookFileName(bookOf(sampleSource, { kind: "story", storyId: "s_int" }, OPTIONS), "txt")).toBe("Evil Time Part 1 · Prologue 1.txt");
    });
});

describe("estimateBook", () => {
    it("words and minutes come from the index, nothing is fetched", () => {
        const e = estimateBook(sampleIndex, { kind: "group", groupId: "main_0" }, { images: "none", format: "text", typeface: "device" });
        expect(e).toMatchObject({ stories: 3, words: 80, images: 0, bytes: Math.round(80 * TEXT_BYTES_PER_WORD), exactImages: true });
        expect(e.minutes).toBeCloseTo(80 / 225, 6);
    });

    it("an illustrations listing makes the image count exact and deduplicated within the scope", () => {
        const listing: StoryIllustrations = {
            groupId: "main_0",
            backgrounds: [
                { name: "bg_room", url: "/bg/room.png", storyIds: ["s_01_beg"] },
                { name: "bg_street", url: "/bg/street.png", storyIds: ["s_01_beg", "s_01_end"] },
            ],
            images: [
                { name: "cg_one", url: "/cg/1.png", storyIds: ["s_01_beg", "s_01_end"] },
                { name: "cg_two", url: "/cg/2.png", storyIds: ["s_01_end"] },
                { name: "gone", url: null, storyIds: ["s_01_beg"] },
            ],
            sprites: [],
        };
        const illustrations = new Map([["main_0", listing]]);
        const cg = estimateBook(sampleIndex, { kind: "story", storyId: "s_01_beg" }, { images: "cg", format: "epub", typeface: "inter", illustrations });
        expect(cg.images).toBe(1);
        expect(cg.bytes).toBe(EPUB_FIXED_BYTES + FONT_BYTES.inter + Math.round(40 * EPUB_BYTES_PER_WORD) + CG_BYTES);
        const both = estimateBook(sampleIndex, { kind: "group", groupId: "main_0" }, { images: "cg+bg", format: "epub", typeface: "device", illustrations });
        expect(both.images).toBe(4);
        expect(both.bytes).toBe(EPUB_FIXED_BYTES + Math.round(80 * EPUB_BYTES_PER_WORD) + 2 * CG_BYTES + 2 * BACKGROUND_BYTES);
    });

    it("without a listing it falls back to the group's illustration count at the measured CG share", () => {
        const e = estimateBook(sampleIndex, { kind: "group", groupId: "main_0" }, { images: "cg", format: "epub", typeface: "device" });
        expect(e.exactImages).toBe(false);
        expect(e.images).toBe(Math.round(4 * CG_SHARE));
    });
});
