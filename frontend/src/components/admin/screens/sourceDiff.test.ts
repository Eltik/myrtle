import { describe, expect, it } from "vitest";

import { diffWords } from "./sourceDiff";

const render = (before: string, after: string) => diffWords(before, after).map((s) => `${s.op[0]}:${s.text}`);

describe("diffWords", () => {
    it("marks an unchanged string as one span", () => {
        expect(diffWords("Next page", "Next page")).toEqual([{ op: "same", text: "Next page" }]);
    });

    it("isolates a single replaced word", () => {
        expect(render("Showing 1 of 20 Doctors", "Showing 1 of 20 players")).toEqual(["s:Showing 1 of 20 ", "r:Doctors", "a:players"]);
    });

    it("reassembles to the original texts", () => {
        const before = "The EXP and LMD still needed to reach the cap.";
        const after = "The EXP and LMD needed to reach every cap, against what you hold.";
        const spans = diffWords(before, after);
        expect(
            spans
                .filter((s) => s.op !== "added")
                .map((s) => s.text)
                .join(""),
        ).toBe(before);
        expect(
            spans
                .filter((s) => s.op !== "removed")
                .map((s) => s.text)
                .join(""),
        ).toBe(after);
    });

    it("treats punctuation as part of its word, so a trailing change is visible", () => {
        expect(render("Saved", "Saved.")).toEqual(["r:Saved", "a:Saved."]);
    });

    it("handles an empty before and after", () => {
        expect(diffWords("", "Added")).toEqual([{ op: "added", text: "Added" }]);
        expect(diffWords("Removed", "")).toEqual([{ op: "removed", text: "Removed" }]);
        expect(diffWords("", "")).toEqual([]);
    });

    it("degrades to a plain replace past the token guard rather than allocating a huge table", () => {
        const before = Array.from({ length: 401 }, (_, i) => `w${i}`).join(" ");
        const after = `${before} tail`;
        expect(diffWords(before, after).map((s) => s.op)).toEqual(["removed", "added"]);
    });
});
