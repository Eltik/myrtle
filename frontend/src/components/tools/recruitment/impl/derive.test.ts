import { describe, expect, it } from "vitest";

import { buildOperatorTagList, professionTagName } from "./derive";

// KR's own recruitment tag names for ids 1 (Guard) and 9 (Melee).
const KR_TAGS = new Map([
    [1, "근위"],
    [9, "근거리"],
]);

describe("professionTagName", () => {
    it("names the class in the server's own wording by tag id", () => {
        expect(professionTagName("WARRIOR", KR_TAGS)).toBe("근위");
    });

    it("falls back to the English label when the list lacks the id", () => {
        expect(professionTagName("SNIPER", KR_TAGS)).toBe("Sniper");
        expect(professionTagName("WARRIOR")).toBe("Guard");
    });

    it("returns an unknown profession code as itself", () => {
        expect(professionTagName("TOKEN", KR_TAGS)).toBe("TOKEN");
    });
});

describe("buildOperatorTagList", () => {
    it("uses the same class name the popover shows", () => {
        const tags = buildOperatorTagList({ position: "MELEE", profession: "WARRIOR", tagList: [] }, 4, KR_TAGS);
        expect(tags).toEqual(["근거리", "근위"]);
    });
});
