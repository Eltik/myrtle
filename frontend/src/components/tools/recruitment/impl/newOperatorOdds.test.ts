import { describe, expect, it } from "vitest";
import { calculateResults } from "./calculator";
import { newOddsPercent, newOperatorOdds } from "./newOperatorOdds";
import type { IRecruitableOperator } from "./types";

const TOP_OPERATOR = { id: 11, name: "Top Operator" };
const SUPPORTER = { id: 1012, name: "Supporter" };

function op(id: string, rarity: number, tagList: string[] = []): IRecruitableOperator {
    return { id, name: id, rarity, profession: "SUPPORT", professionName: "Supporter", position: "RANGED", tagList, potentials: [] };
}

const POOL = [op("suzuran", 6, ["Supporter"]), op("magallan", 6, ["Supporter"]), op("angelina", 6, []), op("pramanix", 5, ["Supporter"]), op("istina", 5, ["Supporter"]), op("orchid", 4, ["Supporter"]), op("earthspirit", 4, ["Supporter"]), op("podenco", 4, ["Supporter"]), op("ansel", 3, [])];

const OWNS_SUZURAN = new Map([["suzuran", 0]]);

describe("newOperatorOdds", () => {
    it("gives one exact figure for a single-rarity combination", () => {
        const result = calculateResults([TOP_OPERATOR, SUPPORTER], POOL).find((r) => r.tags.length === 2);
        const odds = newOperatorOdds(result?.operators ?? [], OWNS_SUZURAN);
        expect(odds).toEqual([{ rarity: 6, unowned: 1, total: 2 }]);
        expect(odds?.map(newOddsPercent)).toEqual([50]);
    });

    it("keeps a mixed-rarity combination split by rarity, never combined", () => {
        const roster = new Map([
            ["istina", 2],
            ["orchid", 5],
            ["podenco", 0],
        ]);
        const [result] = calculateResults([SUPPORTER], POOL);
        const odds = newOperatorOdds(result?.operators ?? [], roster);
        expect(odds).toEqual([
            { rarity: 5, unowned: 1, total: 2 },
            { rarity: 4, unowned: 1, total: 3 },
        ]);
        expect(odds?.map(newOddsPercent)).toEqual([50, 33]);
    });

    it("reads 0 when every operator is owned", () => {
        const roster = new Map(POOL.map((o) => [o.id, 0]));
        const odds = newOperatorOdds([op("pramanix", 5), op("istina", 5)], roster);
        expect(odds).toEqual([{ rarity: 5, unowned: 0, total: 2 }]);
        expect(odds?.map(newOddsPercent)).toEqual([0]);
    });

    it("reads 100 when nothing is owned", () => {
        const odds = newOperatorOdds([op("a", 4), op("b", 4), op("c", 3)], new Map());
        expect(odds?.map(newOddsPercent)).toEqual([100, 100]);
    });

    it("is null when nobody is signed in", () => {
        expect(newOperatorOdds([op("a", 4)], undefined)).toBeNull();
        expect(newOperatorOdds([op("a", 4)], null)).toBeNull();
    });

    it("never rounds a partial share to a certainty", () => {
        expect(newOddsPercent({ rarity: 4, unowned: 199, total: 200 })).toBe(99);
        expect(newOddsPercent({ rarity: 4, unowned: 1, total: 200 })).toBe(1);
    });
});
