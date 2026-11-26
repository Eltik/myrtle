import { describe, expect, it } from "vitest";
import { calculateResults } from "./calculator";
import { MAX_TAGS_PER_RECRUIT } from "./constants";
import { buildPotentials, type IRecruitmentSourceOperator } from "./derive";
import type { IRecruitableOperator } from "./types";

const MELEE = { id: 9, name: "Melee" };
const GUARD = { id: 1, name: "Guard" };

function op(id: string, rarity: number, extra: Partial<IRecruitableOperator> = {}): IRecruitableOperator {
    return { id, name: id, rarity, profession: "WARRIOR", professionName: "Guard", position: "MELEE", tagList: ["Melee", "Guard"], potentials: [], ...extra };
}

describe("potential-asc operator sort", () => {
    // Four 4★ guards the roster holds at P6, P1, unowned, P3: the recruit that
    // gains the most sits first, the maxed one last.
    const pool = [op("maxed", 4), op("p1", 4), op("unowned", 4), op("p3", 4)];
    const potentialByOperator = new Map([
        ["maxed", 5],
        ["p1", 0],
        ["p3", 2],
    ]);

    it("orders unowned, then lowest potential, then maxed", () => {
        const [result] = calculateResults([MELEE], pool, { operatorSortMode: "potential-asc", potentialByOperator });
        expect(result?.operators.map((o) => o.id)).toEqual(["unowned", "p1", "p3", "maxed"]);
    });

    it("breaks potential ties on rarity, highest first", () => {
        const tied = [op("three", 3), op("five", 5), op("four", 4)];
        const [result] = calculateResults([MELEE], tied, { operatorSortMode: "potential-asc", potentialByOperator: new Map() });
        expect(result?.operators.map((o) => o.id)).toEqual(["five", "four", "three"]);
    });

    it("is the rarity-desc order when no roster is supplied", () => {
        const byRarity = calculateResults([MELEE], pool, { operatorSortMode: "rarity-desc" })[0]?.operators.map((o) => o.id);
        const byPotential = calculateResults([MELEE], pool, { operatorSortMode: "potential-asc" })[0]?.operators.map((o) => o.id);
        expect(byPotential).toEqual(byRarity);
    });

    it("never reorders the combinations themselves", () => {
        // Guard-only pool floors at 3★ with a 5★ in it; Melee+Guard floors at 4★.
        // The floor ranks first in every mode; the roster must not change that.
        const mixed = [op("floor3", 3, { tagList: ["Guard"], position: "RANGED" }), op("five", 5, { tagList: ["Guard"], position: "RANGED" }), op("four", 4)];
        const ranks = (mode: "rarity-desc" | "potential-asc") => calculateResults([MELEE, GUARD], mixed, { operatorSortMode: mode, potentialByOperator: new Map([["four", 5]]) }).map((r) => r.tagNames.join("+"));
        expect(ranks("potential-asc")).toEqual(ranks("rarity-desc"));
        expect(ranks("potential-asc").at(-1)).toBe("Guard");
    });

    it("carries each operator's potentials into the result", () => {
        const potentials = [{ kind: "stat" as const, attribute: "COST", value: -1 }];
        const [result] = calculateResults([MELEE], [op("a", 4, { potentials })], {});
        expect(result?.operators[0]?.potentials).toBe(potentials);
    });
});

describe("combination size", () => {
    // One operator matching all five offered tags: every subset has a non-empty
    // pool, so only the size cap decides which combinations come back.
    const TAGS = [MELEE, GUARD, { id: 100, name: "DPS" }, { id: 101, name: "Defense" }, { id: 102, name: "Survival" }];
    const pool = [op("all", 4, { tagList: TAGS.map((t) => t.name) })];

    it("never uses more tags than a recruit can take", () => {
        const sizes = calculateResults(TAGS, pool).map((r) => r.tags.length);
        expect(Math.max(...sizes)).toBe(MAX_TAGS_PER_RECRUIT);
    });

    it("still returns every one-, two- and three-tag combination of five offered tags", () => {
        // C(5,1) + C(5,2) + C(5,3)
        expect(calculateResults(TAGS, pool)).toHaveLength(5 + 10 + 10);
    });
});

describe("buildPotentials", () => {
    // The shape `/operators/recruitment` serves.
    const stat = (attributeType: string, value: number) => ({ description: "", modifier: { attributeType, value } });
    const custom = (description: string) => ({ description, modifier: null });

    it("reads BUFF ranks off the modifier and CUSTOM ranks off the talent candidates", () => {
        const operator = {
            potentialRanks: [stat("COST", -1), custom("Improves Second Talent"), stat("ATK", 28), custom("Improves First Talent"), stat("COST", -1)],
            talentPotentialRanks: [
                [0, 4],
                [0, 2],
            ],
        } satisfies Partial<IRecruitmentSourceOperator>;
        expect(buildPotentials(operator)).toEqual([
            { kind: "stat", attribute: "COST", value: -1 },
            { kind: "talent", index: 1, of: 2 },
            { kind: "stat", attribute: "ATK", value: 28 },
            { kind: "talent", index: 0, of: 2 },
            { kind: "stat", attribute: "COST", value: -1 },
        ]);
    });

    it("keeps the description when neither a modifier nor a talent candidate explains the rank", () => {
        const operator = { potentialRanks: [custom("Something new")], talentPotentialRanks: [[0]] } satisfies Partial<IRecruitmentSourceOperator>;
        expect(buildPotentials(operator)).toEqual([{ kind: "text", text: "Something new" }]);
    });

    it("is empty for an operator without ranks", () => {
        expect(buildPotentials({})).toEqual([]);
    });
});
