import { describe, expect, it } from "vitest";
import { createInstance } from "#/components/tools/shared/instance";
import type { IDpsRankRow } from "#/lib/api/dps";
import { hasRetention, normaliseRankSettings, rankRequest, rankRowKey, ratioToTop, rowToBuild, selectRankRows } from "./rank";
import { DEFAULT_ENEMY, DEFAULT_RANK, DEFAULT_TEAM, instanceToRequest } from "./state";
import { EMPTY_ZONES } from "./zones";

function row(patch: Partial<IDpsRankRow>): IDpsRankRow {
    return { operatorId: "char_x", name: "X", rarity: 6, profession: "SNIPER", subProfessionId: "fastshot", damageType: "physical", skillIndex: 1, moduleIndex: 0, skillDps: 100, totalDamage: 1000, averageDps: 50, retention: null, ...patch };
}

const ROWS: IDpsRankRow[] = [
    row({ operatorId: "char_a", name: "Exusiai", skillIndex: 3, skillName: "Overloading Mode", skillDps: 3000, averageDps: 900, totalDamage: 45000 }),
    row({ operatorId: "char_b", name: "Eyjafjalla", profession: "CASTER", damageType: "arts", skillIndex: 3, skillName: "Volcano", skillDps: 2500, averageDps: 1100, totalDamage: 60000, retention: 0.62 }),
    row({ operatorId: "char_c", name: "Kroos", rarity: 3, skillIndex: 1, skillName: "Double Tap", skillDps: 400, averageDps: 380, totalDamage: 800 }),
    row({ operatorId: "char_a", name: "Exusiai", skillIndex: 3, moduleIndex: 1, moduleName: "Fast-Shot X", skillName: "Overloading Mode", skillDps: 3000, averageDps: 950, totalDamage: 45000 }),
];

describe("selectRankRows", () => {
    it("sorts descending by the chosen metric, ties broken by name, skill, module", () => {
        expect(selectRankRows(ROWS, { ...DEFAULT_RANK, sortBy: "skillDps" }, "").map(rankRowKey)).toEqual(["char_a|3|0", "char_a|3|1", "char_b|3|0", "char_c|1|0"]);
        expect(selectRankRows(ROWS, { ...DEFAULT_RANK, sortBy: "averageDps" }, "").map((r) => r.averageDps)).toEqual([1100, 950, 900, 380]);
        expect(selectRankRows(ROWS, { ...DEFAULT_RANK, sortBy: "totalDamage" }, "")[0].name).toBe("Eyjafjalla");
    });

    it("puts rows without a retention last when sorting by it", () => {
        expect(selectRankRows(ROWS, { ...DEFAULT_RANK, sortBy: "retention" }, "")[0].name).toBe("Eyjafjalla");
        expect(hasRetention(ROWS)).toBe(true);
        expect(hasRetention(ROWS.filter((r) => r.retention === null))).toBe(false);
    });

    it("filters by class, rarity and damage type, each empty meaning all", () => {
        expect(selectRankRows(ROWS, { ...DEFAULT_RANK, professions: ["CASTER"] }, "").map((r) => r.name)).toEqual(["Eyjafjalla"]);
        expect(selectRankRows(ROWS, { ...DEFAULT_RANK, rarities: [3] }, "").map((r) => r.name)).toEqual(["Kroos"]);
        expect(selectRankRows(ROWS, { ...DEFAULT_RANK, damageTypes: ["physical"], rarities: [6] }, "")).toHaveLength(2);
        expect(selectRankRows(ROWS, { ...DEFAULT_RANK, professions: ["MEDIC"] }, "")).toEqual([]);
    });

    it("narrows by fuzzy search over name, skill and module without reordering past the metric", () => {
        expect(selectRankRows(ROWS, DEFAULT_RANK, "exu").map(rankRowKey)).toEqual(["char_a|3|0", "char_a|3|1"]);
        expect(selectRankRows(ROWS, DEFAULT_RANK, "volcano").map((r) => r.name)).toEqual(["Eyjafjalla"]);
        expect(selectRankRows(ROWS, DEFAULT_RANK, "fast-shot").map(rankRowKey)).toEqual(["char_a|3|1"]);
    });

    it("does not mutate its input", () => {
        const before = ROWS.map(rankRowKey);
        selectRankRows(ROWS, { ...DEFAULT_RANK, sortBy: "averageDps" }, "");
        expect(ROWS.map(rankRowKey)).toEqual(before);
    });
});

describe("ratioToTop", () => {
    it("is the share of the top value, null where it is undefined", () => {
        expect(ratioToTop(1500, 3000)).toBe(0.5);
        expect(ratioToTop(1500, 0)).toBeNull();
        expect(ratioToTop(1500, undefined)).toBeNull();
        expect(ratioToTop(Number.NaN, 3000)).toBeNull();
    });
});

describe("rowToBuild", () => {
    it("mirrors the max presets, clamping the skill rank to the rarity's highest promotion", () => {
        expect(rowToBuild({ skillIndex: 3, moduleIndex: 1, rarity: 6 }, "p1Max")).toEqual({ skillIndex: 3, moduleIndex: 1, conditionals: {}, allCond: true, potential: 1, trust: 100, skillRank: 10, moduleLevel: 3 });
        expect(rowToBuild({ skillIndex: 3, moduleIndex: 1, rarity: 6 }, "p6Max").potential).toBe(6);
        expect(rowToBuild({ skillIndex: 1, moduleIndex: 0, rarity: 3 }, "p1Max").skillRank).toBe(7);
        expect(rowToBuild({ skillIndex: 2, moduleIndex: 0, rarity: 5 }, "default")).toEqual({ skillIndex: 2, moduleIndex: 0, conditionals: {}, allCond: true });
    });

    it("adds a card whose conditional flags resolve like the ranking's (none sent, all on), not to the conditionals' defaults", () => {
        // A conditional whose own default is OFF: a picked card would send it false; the ranked row ran it on.
        const op = { id: "char_t", name: "T", availableSkills: [1, 2, 3], availableModules: [0, 1], availableModuleIds: ["", "uniequip_t"], defaultSkill: 3, defaultModule: 1, conditionals: [{ conditionalType: "trait", name: "rangedAtk", default: false, skills: [], modules: [], inverted: true }] };
        const picked = instanceToRequest(createInstance(op, "x"), DEFAULT_ENEMY, DEFAULT_TEAM, []);
        expect(picked.conditionals).toEqual({ traitDamage: false });
        const added = instanceToRequest(createInstance(op, "x", rowToBuild({ skillIndex: 2, moduleIndex: 1, rarity: 6 }, "p1Max")), DEFAULT_ENEMY, DEFAULT_TEAM, []);
        expect(added.conditionals).toEqual({});
        expect(added.allCond).toBe(true);
        // The rest of the context matches the ranking request field for field.
        const rank = rankRequest(DEFAULT_ENEMY, DEFAULT_TEAM, [], "p1Max");
        expect({ defense: added.defense, res: added.res, targets: added.targets, zones: added.zones, cycleModel: added.cycleModel, secondary: added.secondary }).toEqual({ defense: rank.defense, res: rank.res, targets: rank.targets, zones: rank.zones, cycleModel: rank.cycleModel, secondary: rank.secondary });
        expect(added).toMatchObject({ skillIndex: 2, moduleIndex: 1, moduleLevel: 3, potential: 1, trust: 100 });
    });
});

describe("normaliseRankSettings", () => {
    it("keeps known values and drops unknown ones", () => {
        expect(normaliseRankSettings({ preset: "p6Max", sortBy: "retention", professions: ["SNIPER", "sniper", 3, "TOKEN"], rarities: [6, 0, 7, "5", 6], damageTypes: ["arts", "fire"] }, DEFAULT_RANK)).toEqual({ preset: "p6Max", sortBy: "retention", professions: ["SNIPER"], rarities: [6], damageTypes: ["arts"] });
    });

    it("falls back field by field, and wholly for a non-object", () => {
        expect(normaliseRankSettings({ preset: "max", sortBy: 1, professions: "SNIPER" }, DEFAULT_RANK)).toEqual(DEFAULT_RANK);
        expect(normaliseRankSettings(null, DEFAULT_RANK)).toBe(DEFAULT_RANK);
        expect(normaliseRankSettings([1, 2], DEFAULT_RANK)).toBe(DEFAULT_RANK);
    });
});

describe("rankRequest", () => {
    it("sends the page context and preset, and no zones or cycle flag when none are set", () => {
        expect(rankRequest(DEFAULT_ENEMY, DEFAULT_TEAM, [], "p1Max")).toEqual({ defense: 0, res: 0, targets: 1, preset: "p1Max" });
    });

    it("carries secondary targets (primary as one enemy), v2 and the team zones", () => {
        const enemy = { ...DEFAULT_ENEMY, defense: 500, res: 30, targets: 3, secondary: { enabled: true, def: 800, res: 50, count: 3 }, cycleModel: "v2" as const };
        const team = { ...DEFAULT_TEAM, zones: { ...EMPTY_ZONES, fragile: 0.3 } };
        expect(rankRequest(enemy, team, [], "p6Max")).toEqual({ defense: 500, res: 30, targets: 1, secondary: { def: 800, res: 50, count: 3 }, cycleModel: "v2", zones: { fragile: [0.3] }, preset: "p6Max" });
    });
});
