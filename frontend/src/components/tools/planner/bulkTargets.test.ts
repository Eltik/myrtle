import { describe, expect, it } from "vitest";

import type { IRosterEntry } from "#/lib/api/user";
import type { PresetTarget } from "#/types/generated/PresetTarget";
import type { IOperatorListItem } from "#/types/operators";
import { clampPresetForOperator, normalizePresetTarget, raiseToRoster, withPresetChange } from "./bulkTargets";

const cond = (phase: string, level: number) => ({ unlockCond: { phase, level } });

/** Skill levels 2-4 need E0, 5-7 need E1, the way every skilled operator's table reads. */
const SHARED_LEVELS = [cond("PHASE_0", 1), cond("PHASE_0", 1), cond("PHASE_0", 1), cond("PHASE_1", 1), cond("PHASE_1", 1), cond("PHASE_1", 1)];
const MASTERIES = [cond("PHASE_2", 1), cond("PHASE_2", 1), cond("PHASE_2", 1)];
const ORIGINAL = { uniEquipId: "original", typeName1: "ORIGINAL", unlockEvolvePhase: "PHASE_0", unlockLevel: 1 };

function operator(rarity: number, skills: { masteries: boolean }[], modules: { id: string; level: number }[] = []): IOperatorListItem {
    return {
        rarity,
        allSkillLevelUp: SHARED_LEVELS,
        skills: skills.map((s, idx) => ({ skillId: `s${idx}`, levelUpCostCond: s.masteries ? MASTERIES : [] })),
        modules: [ORIGINAL, ...modules.map((m) => ({ uniEquipId: m.id, typeName1: "SUM", unlockEvolvePhase: "PHASE_2", unlockLevel: m.level }))],
    } as unknown as IOperatorListItem;
}

const FULL: PresetTarget = { elite: 2, level: 90, skill_level: 7, masteries: [3, 3, 3], module_stage: 3 };

const threeStar = operator(3, [{ masteries: false }]);
const fourStar = operator(4, [{ masteries: true }, { masteries: true }], [{ id: "mod_4", level: 30 }]);
const sixStar = operator(
    6,
    [{ masteries: true }, { masteries: true }, { masteries: true }],
    [
        { id: "mod_x", level: 60 },
        { id: "mod_y", level: 60 },
    ],
);

function roster(entry: Partial<IRosterEntry>): IRosterEntry {
    return { elite: 0, level: 1, skill_level: 1, masteries: [], modules: [], ...entry } as IRosterEntry;
}

describe("clampPresetForOperator", () => {
    it("stops a 3-star at E1 Lv55 with no masteries or modules", () => {
        expect(clampPresetForOperator(FULL, threeStar)).toEqual({
            targetElite: 1,
            targetLevel: 55,
            targetSkillLevel: 7,
            targetSkills: [{ skill_index: 0, mastery_level: 0 }],
            targetModules: [],
        });
    });

    it("lands a 3-star aimed at E2 Lv1 on its E1 cap, not E1 Lv1", () => {
        expect(clampPresetForOperator({ ...FULL, level: 1 }, threeStar).targetLevel).toBe(55);
    });

    it("caps a 4-star at E2 Lv70 and keeps its masteries and module", () => {
        expect(clampPresetForOperator(FULL, fourStar)).toEqual({
            targetElite: 2,
            targetLevel: 70,
            targetSkillLevel: 7,
            targetSkills: [
                { skill_index: 0, mastery_level: 3 },
                { skill_index: 1, mastery_level: 3 },
            ],
            targetModules: [{ module_id: "mod_4", module_stage: 3 }],
        });
    });

    it("gives a 6-star the whole preset", () => {
        const target = clampPresetForOperator(FULL, sixStar);
        expect(target.targetElite).toBe(2);
        expect(target.targetLevel).toBe(90);
        expect(target.targetSkills).toEqual([0, 1, 2].map((i) => ({ skill_index: i, mastery_level: 3 })));
        expect(target.targetModules).toEqual([
            { module_id: "mod_x", module_stage: 3 },
            { module_id: "mod_y", module_stage: 3 },
        ]);
    });

    it("ignores the third mastery on a two-skill operator", () => {
        const target = clampPresetForOperator({ ...FULL, masteries: [0, 2, 3] }, fourStar);
        expect(target.targetSkills).toEqual([
            { skill_index: 0, mastery_level: 0 },
            { skill_index: 1, mastery_level: 2 },
        ]);
    });

    it("plans no module for an operator without one, and drops a module the level does not reach", () => {
        expect(clampPresetForOperator(FULL, operator(6, [{ masteries: true }])).targetModules).toEqual([]);
        expect(clampPresetForOperator({ ...FULL, level: 59 }, sixStar).targetModules.every((m) => m.module_stage === 0)).toBe(true);
    });

    it("reads a null level as the cap at the clamped promotion", () => {
        const preset = { ...FULL, level: null };
        expect(clampPresetForOperator(preset, sixStar).targetLevel).toBe(90);
        expect(clampPresetForOperator(preset, fourStar).targetLevel).toBe(70);
        expect(clampPresetForOperator({ ...preset, elite: 1, masteries: [0, 0, 0], module_stage: 0 }, sixStar).targetLevel).toBe(80);
    });

    it("leaves masteries out below skill level 7", () => {
        const target = clampPresetForOperator({ elite: 1, level: null, skill_level: 6, masteries: [3, 3, 3], module_stage: 0 }, sixStar);
        expect(target.targetSkillLevel).toBe(6);
        expect(target.targetSkills.every((s) => s.mastery_level === 0)).toBe(true);
    });
});

describe("raiseToRoster", () => {
    const target = clampPresetForOperator({ elite: 2, level: 60, skill_level: 7, masteries: [1, 0, 0], module_stage: 1 }, sixStar);

    it("returns the target untouched for an operator not on the roster", () => {
        expect(raiseToRoster(target, undefined)).toEqual({ target, isReached: false });
    });

    it("keeps each field at whichever is higher, the roster or the preset", () => {
        const entry = roster({ elite: 2, level: 80, skill_level: 7, masteries: [{ index: 1, mastery: 3 }], modules: [{ id: "mod_y", level: 2, locked: false }] });
        const merged = raiseToRoster(target, entry);
        expect(merged.isReached).toBe(false);
        expect(merged.target.targetElite).toBe(2);
        expect(merged.target.targetLevel).toBe(80);
        expect(merged.target.targetSkills.map((s) => s.mastery_level)).toEqual([1, 3, 0]);
        expect(merged.target.targetModules).toEqual([
            { module_id: "mod_x", module_stage: 1 },
            { module_id: "mod_y", module_stage: 2 },
        ]);
    });

    it("counts a locked module as stage 0 and flags a roster already past the preset", () => {
        const ahead = roster({
            elite: 2,
            level: 90,
            skill_level: 7,
            masteries: [{ index: 0, mastery: 3 }],
            modules: [
                { id: "mod_x", level: 3, locked: false },
                { id: "mod_y", level: 3, locked: false },
            ],
        });
        expect(raiseToRoster(target, ahead).isReached).toBe(true);
        const locked = roster({ ...ahead, modules: [{ id: "mod_x", level: 3, locked: true }] });
        expect(raiseToRoster(target, locked).isReached).toBe(false);
    });

    it("treats a higher promotion as ahead even at a lower level", () => {
        const e1 = clampPresetForOperator({ elite: 1, level: 80, skill_level: 7, masteries: [0, 0, 0], module_stage: 0 }, sixStar);
        const merged = raiseToRoster(e1, roster({ elite: 2, level: 1, skill_level: 7 }));
        expect(merged.target.targetElite).toBe(2);
        expect(merged.target.targetLevel).toBe(1);
        expect(merged.isReached).toBe(true);
    });
});

describe("withPresetChange", () => {
    const base: PresetTarget = { elite: 0, level: null, skill_level: 1, masteries: [0, 0, 0], module_stage: 0 };

    it("lifts promotion and skill level for a mastery, and promotion for a module", () => {
        expect(withPresetChange(base, { field: "mastery", index: 1, value: 2 })).toEqual({ elite: 2, level: null, skill_level: 7, masteries: [0, 2, 0], module_stage: 0 });
        expect(withPresetChange(base, { field: "module_stage", value: 3 }).elite).toBe(2);
        expect(withPresetChange(base, { field: "skill_level", value: 5 }).elite).toBe(1);
    });

    it("drops masteries, modules and the level past the cap when promotion falls", () => {
        const full: PresetTarget = { elite: 2, level: 90, skill_level: 7, masteries: [3, 3, 3], module_stage: 3 };
        expect(withPresetChange(full, { field: "elite", value: 0 })).toEqual({ elite: 0, level: 50, skill_level: 4, masteries: [0, 0, 0], module_stage: 0 });
        expect(withPresetChange(full, { field: "skill_level", value: 6 }).masteries).toEqual([0, 0, 0]);
    });
});

describe("normalizePresetTarget", () => {
    it("passes a valid target through unchanged", () => {
        expect(normalizePresetTarget(FULL)).toEqual(FULL);
    });
});
