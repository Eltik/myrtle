import { describe, expect, it } from "vitest";
import type { IRecruitableOperator } from "#/components/tools/recruitment/impl/types";
import type { IRecruitmentData } from "#/lib/api/recruitment";
import { ogHashers } from "./hashers";
import { normalizeRecruitmentTagIds, parseRecruitmentOgId, recruitmentOgId } from "./recruitment";
import { buildRecruitmentOgData } from "./recruitment.build";

function op(id: string, rarity: number, profession: string, position: string, tagList: string[] = []): IRecruitableOperator {
    return { id, name: id, rarity, profession, professionName: profession, position, tagList, potentials: [] };
}

const DATA: IRecruitmentData = {
    tags: [
        { tagId: 1, tagName: "Guard", tagGroup: 0 },
        { tagId: 9, tagName: "Melee", tagGroup: 0 },
        { tagId: 11, tagName: "Top Operator", tagGroup: 0 },
        { tagId: 14, tagName: "Senior Operator", tagGroup: 0 },
        { tagId: 1012, tagName: "Male", tagGroup: 0 },
    ],
    operators: [op("six", 6, "WARRIOR", "MELEE", ["Top Operator"]), op("five", 5, "WARRIOR", "MELEE", ["Senior Operator"]), op("three", 3, "WARRIOR", "MELEE")],
};

describe("recruitment card id", () => {
    it("sorts, dedupes and drops anything that is not a positive integer", () => {
        expect(normalizeRecruitmentTagIds("14-11-14-x-0--3.5-9")).toEqual([9, 11, 14]);
        expect(normalizeRecruitmentTagIds([14, "11", 11])).toEqual([11, 14]);
    });

    it("keeps at most five tags, the game's own limit", () => {
        expect(normalizeRecruitmentTagIds("7-6-5-4-3-2-1")).toEqual([1, 2, 3, 4, 5]);
    });

    it("is order-free and carries the server only off the default", () => {
        expect(recruitmentOgId([14, 11])).toBe("11-14");
        expect(recruitmentOgId([11, 14, 14], "en")).toBe("11-14");
        expect(recruitmentOgId([14, 11], "jp")).toBe("jp:11-14");
        expect(parseRecruitmentOgId("jp:14-11")).toEqual({ server: "jp", tagIds: [11, 14] });
        expect(parseRecruitmentOgId("11-14")).toEqual({ server: "en", tagIds: [11, 14] });
        // An unknown prefix is no server: the whole id is read as tags, and the junk drops out.
        expect(parseRecruitmentOgId("zz:11")).toEqual({ server: "en", tagIds: [] });
    });
});

describe("buildRecruitmentOgData", () => {
    it("drops ids the server does not offer or the tool hides, and is null when none is left", () => {
        expect(buildRecruitmentOgData([9999, 1012], DATA, "en")).toBeNull();
        const card = buildRecruitmentOgData([9999, 9, 1012], DATA, "en");
        expect(card?.tags).toEqual([{ id: 9, name: "Melee" }]);
    });

    it("picks the combination with the highest guaranteed floor", () => {
        const card = buildRecruitmentOgData([1, 11, 9], DATA, "en");
        expect(card?.best?.floor).toBe(6);
        expect(card?.best?.tagIds).toContain(11);
        expect(card?.best?.operators.map((o) => o.id)).toEqual(["six"]);
    });

    it("hashes the same for the same selection in any order, and differently for another", () => {
        const a = buildRecruitmentOgData([14, 9], DATA, "en");
        const b = buildRecruitmentOgData([9, 14, 14], DATA, "en");
        const c = buildRecruitmentOgData([9], DATA, "en");
        if (!a || !b || !c) throw new Error("expected cards");
        expect(ogHashers.recruitment.hash(a)).toBe(ogHashers.recruitment.hash(b));
        expect(ogHashers.recruitment.hash(a)).not.toBe(ogHashers.recruitment.hash(c));
    });
});
