import { describe, expect, it } from "vitest";
import type { IOperatorListItem } from "#/types/operators";
import { calculateResults } from "./calculator";
import { buildPotentials } from "./derive";
import type { IRecruitableOperator } from "./types";

const MELEE = { id: 9, name: "Melee" };
const GUARD = { id: 1, name: "Guard" };

function op(id: string, rarity: number, extra: Partial<IRecruitableOperator> = {}): IRecruitableOperator {
    return { id, name: id, rarity, profession: "WARRIOR", position: "MELEE", tagList: ["Melee", "Guard"], potentials: [], ...extra };
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

describe("buildPotentials", () => {
    // The shape `/static/operators` hands the server fn after deepCamelize.
    const stat = (attributeType: string, value: number) => ({ type_: "BUFF", description: "", buff: { attributes: { attributeModifiers: [{ attributeType, value }] } } });
    const custom = (description: string) => ({ type_: "CUSTOM", description, buff: null });
    const talent = (ranks: number[]) => ({ candidates: ranks.map((requiredPotentialRank) => ({ requiredPotentialRank })) });

    it("reads BUFF ranks off the modifier and CUSTOM ranks off the talent candidates", () => {
        const operator = {
            potentialRanks: [stat("COST", -1), custom("Improves Second Talent"), stat("ATK", 28), custom("Improves First Talent"), stat("COST", -1)],
            talents: [talent([0, 4]), talent([0, 2])],
        } as unknown as IOperatorListItem;
        expect(buildPotentials(operator)).toEqual([
            { kind: "stat", attribute: "COST", value: -1 },
            { kind: "talent", index: 1, of: 2 },
            { kind: "stat", attribute: "ATK", value: 28 },
            { kind: "talent", index: 0, of: 2 },
            { kind: "stat", attribute: "COST", value: -1 },
        ]);
    });

    it("keeps the description when neither a modifier nor a talent candidate explains the rank", () => {
        const operator = { potentialRanks: [custom("Something new")], talents: [talent([0])] } as unknown as IOperatorListItem;
        expect(buildPotentials(operator)).toEqual([{ kind: "text", text: "Something new" }]);
    });

    it("is empty for an operator without ranks", () => {
        expect(buildPotentials({} as IOperatorListItem)).toEqual([]);
    });
});
