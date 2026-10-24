import { describe, expect, it } from "vitest";
import { mergeWithDefaults } from "./persist";
import { DEFAULT_TEAM, INITIAL_STATE } from "./state";
import { EMPTY_ZONES } from "./zones";

const op = { id: "char_a", name: "A", availableSkills: [1, 2, 3], availableModules: [0, 1], availableModuleIds: ["", "uniequip_x"], defaultSkill: 3, defaultModule: 1, conditionals: [] };

/** A state as phase 3a saved it: per-card buffs, SP boost and shred on `enemy`, no `team`. */
const V1_STORED = {
    instances: [{ uid: "u1", op, color: "oklch(0.65 0.20 30)", visible: true, collapsed: false, config: { potential: 1, trust: 100, skillIndex: 3, skillRank: 10, moduleIndex: 1, moduleLevel: 3, buffs: { atk: 0.4, flatAtk: 0, aspd: 12 }, conditionals: {}, allCond: true } }],
    enemy: { defense: 500, res: 30, targets: 2, spBoost: 0.3, shred: { def: 20, defFlat: 0, res: 0, resFlat: 5 } },
    xAxis: "res",
    yMetric: "average_dps",
    sweep: { defense: { min: 0, max: 2000, steps: 21 }, res: { min: 0, max: 80, steps: 17 } },
    markers: false,
};

describe("stored DPS state migration", () => {
    it("moves a phase 3a state's card buffs and enemy shred / SP boost into zones", () => {
        const s = mergeWithDefaults(JSON.parse(JSON.stringify(V1_STORED)));
        expect(s).not.toBeNull();
        if (!s) return;
        expect(s.instances[0].config.buffs).toEqual({});
        expect(s.instances[0].config.zones).toEqual({ ...EMPTY_ZONES, atkPercent: 0.4, aspd: 12 });
        expect(s.enemy).toEqual({ defense: 500, res: 30, targets: 2, secondary: { enabled: false, def: 0, res: 0, count: 1 }, cycleModel: "legacy" });
        expect(s.team).toEqual({ ...DEFAULT_TEAM, zones: { ...EMPTY_ZONES, spRecovery: 0.3, defPercent: 0.2, resFlat: 5 } });
        expect(s.xAxis).toBe("res");
        expect(s.view).toBe("compare");
        expect(s.rank).toEqual(INITIAL_STATE.rank);
    });

    it("keeps a current state as it is", () => {
        const current = { ...INITIAL_STATE, team: { ...DEFAULT_TEAM, zones: { ...EMPTY_ZONES, inspire: 200 }, supporters: ["sora_s2"] }, view: "rank" };
        const s = mergeWithDefaults(JSON.parse(JSON.stringify(current)));
        expect(s).toEqual(current);
    });

    it("validates a stored rank block instead of spreading it", () => {
        const s = mergeWithDefaults({ ...INITIAL_STATE, rank: { preset: "nope", sortBy: "skillDps", professions: [{}, "CASTER"], rarities: "6", damageTypes: ["arts"], extra: 1 } });
        expect(s?.rank).toEqual({ preset: "p1Max", sortBy: "skillDps", professions: ["CASTER"], rarities: [], damageTypes: ["arts"] });
    });

    it("rejects a value that is not a stored state", () => {
        expect(mergeWithDefaults(null)).toBeNull();
        expect(mergeWithDefaults({ instances: "x" })).toBeNull();
    });
});
