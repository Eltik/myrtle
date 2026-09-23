import { describe, expect, it } from "vitest";

import type { IOperatorListItem } from "#/types/operators";
import { clampModuleTargets, clampSkillTargets, isSkillLevelAllowed, planTargetPayload, withSkillTarget } from "./planTargets";

const cond = (phase: string, level: number) => ({ unlockCond: { phase, level } });

/**
 * A 6-star with two skills and one module. Skill levels 2-4 need E0, 5-7
 * need E1, and every mastery needs E2 Lv1; the module needs E2 Lv40.
 */
const operator = {
    rarity: 6,
    allSkillLevelUp: [cond("PHASE_0", 1), cond("PHASE_0", 1), cond("PHASE_0", 1), cond("PHASE_1", 1), cond("PHASE_1", 1), cond("PHASE_1", 1)],
    skills: [
        { skillId: "s1", levelUpCostCond: [cond("PHASE_2", 1), cond("PHASE_2", 1), cond("PHASE_2", 1)] },
        { skillId: "s2", levelUpCostCond: [cond("PHASE_2", 1), cond("PHASE_2", 1), cond("PHASE_2", 1)] },
    ],
    modules: [
        { uniEquipId: "original", typeName1: "ORIGINAL", unlockEvolvePhase: "PHASE_0", unlockLevel: 1 },
        { uniEquipId: "mod_x", typeName1: "SUM", unlockEvolvePhase: "PHASE_2", unlockLevel: 40 },
    ],
} as unknown as IOperatorListItem;

describe("isSkillLevelAllowed", () => {
    it("gates shared levels and masteries on promotion and level", () => {
        expect(isSkillLevelAllowed(operator, 0, 1, 0, 1)).toBe(true);
        expect(isSkillLevelAllowed(operator, 0, 4, 0, 1)).toBe(true);
        expect(isSkillLevelAllowed(operator, 0, 5, 0, 50)).toBe(false);
        expect(isSkillLevelAllowed(operator, 0, 7, 1, 1)).toBe(true);
        expect(isSkillLevelAllowed(operator, 0, 8, 1, 80)).toBe(false);
        expect(isSkillLevelAllowed(operator, 1, 10, 2, 1)).toBe(true);
        expect(isSkillLevelAllowed(operator, 0, 11, 2, 90)).toBe(false);
    });
});

describe("withSkillTarget", () => {
    it("moves every skill for a shared level and lifts the others to 7 for a mastery", () => {
        expect(withSkillTarget({ 0: 10, 1: 7 }, 2, 0, 4)).toEqual({ 0: 4, 1: 4 });
        expect(withSkillTarget({ 0: 3, 1: 3 }, 2, 1, 9)).toEqual({ 0: 7, 1: 9 });
    });
});

describe("clampSkillTargets", () => {
    it("drops every skill to the lowest reachable shared level once one must fall below 7", () => {
        expect(clampSkillTargets({ 0: 10, 1: 7 }, operator, 0, 1)).toEqual({ 0: 4, 1: 4 });
    });

    it("drops a mastery back to 7 when the shared level still holds", () => {
        expect(clampSkillTargets({ 0: 10, 1: 7 }, operator, 1, 1)).toEqual({ 0: 7, 1: 7 });
    });

    it("returns a fresh object even when nothing moved", () => {
        const prev = { 0: 7, 1: 7 };
        const next = clampSkillTargets(prev, operator, 2, 1);
        expect(next).toEqual(prev);
        expect(next).not.toBe(prev);
    });
});

describe("clampModuleTargets", () => {
    it("unplans a module the promotion no longer reaches and keeps the object otherwise", () => {
        expect(clampModuleTargets({ mod_x: 3 }, operator, 2, 39)).toEqual({ mod_x: 0 });
        const prev = { mod_x: 3 };
        expect(clampModuleTargets(prev, operator, 2, 40)).toBe(prev);
    });
});

describe("planTargetPayload", () => {
    it("caps the shared level at 7 and splits masteries per skill", () => {
        expect(planTargetPayload({ 0: 10, 1: 7 }, { mod_x: 2 })).toEqual({
            targetSkillLevel: 7,
            targetSkills: [
                { skill_index: 0, mastery_level: 3 },
                { skill_index: 1, mastery_level: 0 },
            ],
            targetModules: [{ module_id: "mod_x", module_stage: 2 }],
        });
        expect(planTargetPayload({}, {}).targetSkillLevel).toBe(1);
    });
});
