/**
 * Where a story sits in the library decides what the END CARD offers. The
 * reference reader puts Previous and Next in the same group, so the question
 * this pins is what happens at a group BOUNDARY (nothing, rather than a link
 * into an unrelated event) and on an operator record, whose neighbours come
 * from the operator's own list and not from the 364-group `obt/memory` shelf.
 */
import { describe, expect, it, vi } from "vitest";
import type { StoryEntry } from "#/types/generated/StoryEntry";
import type { StoryIndex } from "#/types/generated/StoryIndex";

vi.mock("@tanstack/react-router", () => ({
    createFileRoute: () => (opts: unknown) => opts,
    Link: () => null,
    useNavigate: () => () => undefined,
}));
vi.mock("@tanstack/react-query", () => ({ useQuery: () => ({ data: undefined }), queryOptions: (o: unknown) => o }));

const { placeStory } = await import("#/routes/stories_.$storyId");

function entry(id: string, sort: number, groupId: string): StoryEntry {
    return { id, name: id, sort, groupId, hasScript: true, wordCount: 0, hasVideo: false, requiredStages: [] };
}

const index: StoryIndex = {
    groups: [
        { id: "main_0", name: "Chapter 0", category: "main", entryType: "MAINLINE", actType: null, displayType: null, coverUrl: null, startTime: 0, zone: null, stories: [entry("a", 0, "main_0"), entry("b", 1, "main_0"), entry("c", 2, "main_0")] },
        { id: "main_1", name: "Chapter 1", category: "main", entryType: "MAINLINE", actType: null, displayType: null, coverUrl: null, startTime: 0, zone: null, stories: [entry("d", 0, "main_1")] },
        { id: "rec_amiya", name: "Records", category: "record", entryType: "NONE", actType: null, displayType: null, coverUrl: null, startTime: 0, zone: null, stories: [entry("r1", 0, "rec_amiya"), entry("r2", 1, "rec_amiya")] },
    ],
    records: [{ charId: "char_002_amiya", name: "Amiya", stories: [entry("r1", 0, "rec_amiya"), entry("r2", 1, "rec_amiya")] }],
} as unknown as StoryIndex;

describe("placeStory", () => {
    it("offers both neighbours in the middle of a group", () => {
        const p = placeStory(index, "b");
        expect(p.previous?.id).toBe("a");
        expect(p.next?.id).toBe("c");
    });

    it("stops at the group boundary rather than crossing into the next group", () => {
        expect(placeStory(index, "a").previous).toBeNull();
        expect(placeStory(index, "c").next).toBeNull();
        const only = placeStory(index, "d");
        expect(only.previous).toBeNull();
        expect(only.next).toBeNull();
    });

    it("has no neighbours at all for a story the index does not carry, so the arrows are dead rather than wrong", () => {
        const missing = placeStory(index, "not_a_story");
        expect(missing.entry).toBeNull();
        expect(missing.previous).toBeNull();
        expect(missing.next).toBeNull();
    });

    it("walks an operator record through the operator's own list", () => {
        const p = placeStory(index, "r1");
        expect(p.category).toBe("record");
        expect(p.groupName).toBe("Amiya");
        expect(p.previous).toBeNull();
        expect(p.next?.id).toBe("r2");
        expect(placeStory(index, "r2").previous?.id).toBe("r1");
    });
});
