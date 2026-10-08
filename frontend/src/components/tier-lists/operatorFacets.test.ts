import { describe, expect, it } from "vitest";
import type { ITierOperator } from "#/lib/api/tier-entities";
import { factionOptions, nationOptions, operatorFaction, raceOptions } from "./operatorFacets";

function op(fields: Partial<ITierOperator>): ITierOperator {
    return { nationId: null, nationName: null, groupId: null, groupName: null, teamId: null, teamName: null, race: null, ...fields } as ITierOperator;
}

const CATALOGUE = [
    op({ nationId: "laterano", nationName: "Laterano", race: "Sankta" }),
    op({ nationId: "kjerag", nationName: "Kjerag", groupId: "karlan", groupName: "Karlan Trade", race: "Feline" }),
    op({ nationId: "rhodes", race: "Lupo" }),
    op({ nationId: "rim", teamId: "rainbow", teamName: "Team Rainbow", race: "Sankta" }),
    op({ teamId: "followers" }),
];

describe("operator pool facets", () => {
    it("offers each nation once, by its server name, else the English id spelling, sorted by label", () => {
        expect(nationOptions(CATALOGUE)).toEqual([
            { value: "kjerag", label: "Kjerag" },
            { value: "laterano", label: "Laterano" },
            { value: "rhodes", label: "Rhodes Island" },
            { value: "rim", label: "Rim Billiton" },
        ]);
    });

    it("offers groups and teams together, as /operators does", () => {
        const options = factionOptions(CATALOGUE);
        expect(options.map((o) => o.value).sort()).toEqual(["followers", "karlan", "rainbow"]);
        const labels = options.map((o) => o.label);
        expect(labels).toEqual([...labels].sort((a, b) => a.localeCompare(b)));
        expect(factionOptions(CATALOGUE).find((o) => o.value === "karlan")?.label).toBe("Karlan Trade");
    });

    it("answers the Faction filter with the group, else the team", () => {
        expect(operatorFaction(CATALOGUE[1] as ITierOperator)).toBe("karlan");
        expect(operatorFaction(CATALOGUE[3] as ITierOperator)).toBe("rainbow");
        expect(operatorFaction(CATALOGUE[0] as ITierOperator)).toBeNull();
    });

    it("offers only the races that occur, labelled and sorted by label", () => {
        const label = (r: string) => ({ Sankta: "Sankta", Feline: "Feline", Lupo: "Lupo" })[r] ?? r;
        expect(raceOptions(CATALOGUE, label)).toEqual([
            { value: "Feline", label: "Feline" },
            { value: "Lupo", label: "Lupo" },
            { value: "Sankta", label: "Sankta" },
        ]);
        // The label decides the order, not the wire value.
        expect(raceOptions(CATALOGUE, (r) => (r === "Sankta" ? "Aaa" : r))[0]?.value).toBe("Sankta");
    });
});
