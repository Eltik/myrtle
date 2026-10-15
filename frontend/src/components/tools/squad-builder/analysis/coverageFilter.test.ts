import { describe, expect, it } from "vitest";
import type { CoverageGroup } from "#/types/generated/CoverageGroup";
import type { IStageContext } from "../model";
import { filterCoverage, neededRows } from "./coverageFilter";

const myrtle = { seat: 0, char_id: "char_151_myrtle", support: false, why: { kind: "skill" as const, name: "Healing Wings", skill_index: 1 } };
const exusiai = { seat: 1, char_id: "char_103_angel", support: false, why: { kind: "trait" as const, name: null, skill_index: null } };

const groups: CoverageGroup[] = [
    {
        key: "getting_on_the_field",
        rows: [
            { key: "dp_gen", ops: [myrtle] },
            { key: "openers", ops: [myrtle, exusiai] },
            { key: "fast_redeploy", ops: [] },
        ],
    },
    {
        key: "damage",
        rows: [
            { key: "physical", ops: [exusiai] },
            { key: "anti_air", ops: [exusiai] },
            { key: "arts", ops: [] },
        ],
    },
    {
        key: "utility",
        rows: [
            { key: "invisible", ops: [] },
            { key: "summons", ops: [] },
        ],
    },
];

const stage: Pick<IStageContext, "threats"> = {
    threats: [
        { key: "aerial", rows: ["anti_air"], count: 12, enemies: [{ enemyId: "enemy_1106_byokai", name: "Drone", count: 12 }] },
        { key: "high_def", rows: ["arts", "true_elemental"], count: 4, enemies: [] },
        { key: "invisible", rows: ["invisible"], count: 2, enemies: [] },
        { key: "fast", rows: [], count: 6, enemies: [] },
    ],
};

const keys = (gs: CoverageGroup[]) => gs.map((g) => [g.key, g.rows.map((r) => r.key)]);

describe("coverage filter", () => {
    it("All keeps every row", () => {
        expect(filterCoverage(groups, "all", null)).toEqual(groups);
    });
    it("Covered keeps rows with an operator and drops empty groups", () => {
        expect(keys(filterCoverage(groups, "covered", null))).toEqual([
            ["getting_on_the_field", ["dp_gen", "openers"]],
            ["damage", ["physical", "anti_air"]],
        ]);
    });
    it("Not in squad keeps the empty rows", () => {
        expect(keys(filterCoverage(groups, "missing", stage))).toEqual([
            ["getting_on_the_field", ["fast_redeploy"]],
            ["damage", ["arts"]],
            ["utility", ["invisible", "summons"]],
        ]);
    });
    it("This stage keeps the rows a threat names, covered or not", () => {
        expect(keys(filterCoverage(groups, "stage", stage))).toEqual([
            ["damage", ["anti_air", "arts"]],
            ["utility", ["invisible"]],
        ]);
    });
    it("This stage without a stage keeps nothing", () => {
        expect(filterCoverage(groups, "stage", null)).toEqual([]);
        expect(filterCoverage(groups, "stage", { threats: [] })).toEqual([]);
    });
});

describe("needed rows", () => {
    it("maps each row to the threats that name it", () => {
        const needed = neededRows(stage);
        expect(needed.get("anti_air")?.map((t) => t.key)).toEqual(["aerial"]);
        expect(needed.get("true_elemental")?.map((t) => t.key)).toEqual(["high_def"]);
        expect(needed.has("physical")).toBe(false);
        expect(neededRows(null).size).toBe(0);
    });
    it("joins two threats on one row", () => {
        const both = neededRows({ threats: [...stage.threats, { key: "high_res", rows: ["arts"], count: 3, enemies: [] }] });
        expect(both.get("arts")?.map((t) => t.key)).toEqual(["high_def", "high_res"]);
    });
});
