import { describe, expect, it } from "vitest";
import { nameShare, STRAY_CAP, storyNode, strayPage } from "./NameChip";

describe("the name popover and the stray list", () => {
    it("caps the open stray list at 30 until it is expanded", () => {
        const fifty = Array.from({ length: 50 }, (_, i) => i);
        expect(STRAY_CAP).toBe(30);
        expect(strayPage(fifty, false)).toEqual({ shown: fifty.slice(0, 30), hidden: 20 });
        expect(strayPage(fifty, true)).toEqual({ shown: fifty, hidden: 0 });
        expect(strayPage([1, 2], false)).toEqual({ shown: [1, 2], hidden: 0 });
    });

    it("names a story by its node and gives a share of the folder", () => {
        expect(storyNode({ name: "Twin Swallow Pincer", code: "TA-5", tag: "Before Operation" })).toBe("Twin Swallow Pincer · TA-5 Before Operation");
        expect(storyNode({ name: "Relit" })).toBe("Relit");
        expect(nameShare(14, 269)).toBeCloseTo(0.052, 3);
        expect(nameShare(3, 0)).toBeNull();
    });
});
