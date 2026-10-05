import { describe, expect, it } from "vitest";
import type { StoryCommand } from "#/types/generated/StoryCommand";
import type { StoryScript } from "#/types/generated/StoryScript";
import { haltForLine } from "./scrub";
import { summarizeHalts } from "./useStoryPlayer";

function cmd(line: number, kind: string, args: Record<string, string> = {}, text?: string): StoryCommand {
    return text === undefined ? { kind, args, line } : { kind, args, text, line };
}

// Lines 2 and 3 are spoken, 5 is a decision, 7 is only in branch "2" (the
// probe takes "1"), 9 is in branch "1", 11 follows the reconvergence.
const SCRIPT: StoryScript = {
    id: "t",
    name: "t",
    groupId: "g",
    wordCount: 0,
    assets: { backgrounds: {}, images: {}, characters: {}, music: {}, sounds: {}, avatars: {}, imageSizes: {}, videos: {} },
    commands: [
        cmd(1, "header"),
        cmd(2, "name", { name: "A" }, "one"),
        cmd(3, "name", { name: "B" }, "two"),
        cmd(5, "decision", { options: "Go;Stay", values: "1;2" }),
        cmd(6, "predicate", { references: "2" }),
        cmd(7, "name", { name: "A" }, "stay line"),
        cmd(8, "predicate", { references: "1" }),
        cmd(9, "name", { name: "A" }, "go line"),
        cmd(10, "predicate"),
        cmd(11, "name", { name: "B" }, "after"),
    ],
};

describe("?line= as a halt", () => {
    const summaries = summarizeHalts(SCRIPT, "Doctor", true, "cutscene");

    it("carries each halt's script line on the probe walk", () => {
        expect(summaries.map((s) => [s.haltIndex, s.kind, s.line])).toEqual([
            [0, "line", 2],
            [1, "line", 3],
            [2, "decision", 5],
            [3, "line", 9],
            [4, "line", 11],
        ]);
    });

    it("lands on the line, on the decision, past an unchosen branch, and clamps past the end", () => {
        expect(haltForLine(summaries, 3)).toBe(1);
        expect(haltForLine(summaries, 4)).toBe(2);
        expect(haltForLine(summaries, 5)).toBe(2);
        // Line 7 sits in branch "2", which the probe does not take: the next halt is line 9.
        expect(haltForLine(summaries, 7)).toBe(3);
        expect(haltForLine(summaries, 999)).toBe(4);
        expect(haltForLine(summaries, 1)).toBe(0);
        expect(haltForLine([], 5)).toBe(0);
    });
});
