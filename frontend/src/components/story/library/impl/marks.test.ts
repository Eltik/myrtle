import { describe, expect, it } from "vitest";
import { emptyProgress, isStoryRead, readSourceOf, type StoryProgress } from "#/lib/story/progress";
import type { LibEntry } from "./derive";
import { readFraction } from "./derive";
import { clearMarks, markAllRead, markable, markedCount, toggleRead } from "./marks";

function entry(id: string, over: Partial<LibEntry> = {}): LibEntry {
    return { id, name: id, sort: 1, groupId: "g", hasScript: true, requiredStages: [], ...over };
}

const NO_GAME: ReadonlySet<string> = new Set<string>();

describe("toggleRead", () => {
    it("marks an untouched story read, as the reader's OWN mark", () => {
        const next = toggleRead(emptyProgress(), "a", false, 1_700_000_000_000);
        expect(readSourceOf(next, NO_GAME, "a")).toBe("own");
        expect(isStoryRead(next, NO_GAME, "a")).toBe(true);
    });

    it("walks a GAME-read story through game -> cleared -> own", () => {
        const game: ReadonlySet<string> = new Set(["a"]);
        const start = emptyProgress();
        expect(readSourceOf(start, game, "a")).toBe("game");

        const cleared = toggleRead(start, "a", true, 1_700_000_000_000);
        expect(readSourceOf(cleared, game, "a")).toBe("cleared");
        expect(isStoryRead(cleared, game, "a")).toBe(false);

        const own = toggleRead(cleared, "a", false, 1_700_000_001_000);
        expect(readSourceOf(own, game, "a")).toBe("own");
        expect(isStoryRead(own, game, "a")).toBe(true);
    });

    it("leaves the reader's own mark withdrawn, not merely absent", () => {
        const own = toggleRead(emptyProgress(), "a", false);
        const off = toggleRead(own, "a", true);
        expect(off.read.a).toBeUndefined();
        expect(off.unread?.a).toBeGreaterThan(0);
        expect(readSourceOf(off, NO_GAME, "a")).toBe("none");
    });

    it("carries a v1 document's version through, so the merge can still withdraw its baked marks", () => {
        const v1: StoryProgress = { ...emptyProgress(), v: 1 };
        expect(toggleRead(v1, "a", false).v).toBe(1);
    });
});

describe("markable", () => {
    it("drops the stories with no script", () => {
        expect(markable([entry("a"), entry("b", { hasScript: false })]).map((s) => s.id)).toEqual(["a"]);
    });
});

describe("markAllRead", () => {
    it("moves the fraction to full and never marks a script-less story", () => {
        const stories = [entry("a"), entry("b"), entry("c", { hasScript: false })];
        const before = readFraction(stories, emptyProgress(), NO_GAME);
        expect(before).toEqual({ read: 0, total: 2, listed: 3, done: false });

        const next = markAllRead(emptyProgress(), stories);
        expect(readFraction(stories, next, NO_GAME)).toEqual({ read: 2, total: 2, listed: 3, done: true });
        expect(next.read.c).toBeUndefined();
    });
});

describe("clearMarks", () => {
    it("empties the fraction across BOTH sources, the game's included", () => {
        const stories = [entry("a"), entry("b")];
        const game: ReadonlySet<string> = new Set(["b"]);
        const marked = markAllRead(emptyProgress(), [entry("a")]);
        expect(readFraction(stories, marked, game)).toEqual({ read: 2, total: 2, listed: 2, done: true });

        const cleared = clearMarks(marked, stories, game);
        expect(readFraction(stories, cleared, game)).toEqual({ read: 0, total: 2, listed: 2, done: false });
        expect(readSourceOf(cleared, game, "b")).toBe("cleared");
    });

    it("writes no override against a story nobody ever marked", () => {
        const cleared = clearMarks(emptyProgress(), [entry("a")], NO_GAME);
        expect(cleared.unread).toBeUndefined();
    });
});

describe("markedCount", () => {
    it("counts what Clear all would withdraw, over both sources", () => {
        const stories = [entry("a"), entry("b"), entry("c"), entry("d", { hasScript: false })];
        const game: ReadonlySet<string> = new Set(["b", "d"]);
        const progress = markAllRead(emptyProgress(), [entry("a")]);
        expect(markedCount(stories, progress, game)).toBe(2);
    });
});
