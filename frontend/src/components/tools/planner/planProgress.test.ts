import { describe, expect, it } from "vitest";

import type { IOperatorPlanResponse } from "#/lib/api/planner";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";
import { planProgress } from "./planProgress";

const operator = {
    skills: [{ skillId: "s1" }, { skillId: "s2" }],
    modules: [
        { uniEquipId: "original", typeName1: "ORIGINAL" },
        { uniEquipId: "mod_x", typeName1: "SUM" },
        { uniEquipId: "mod_y", typeName1: "SUM" },
    ],
} as unknown as IOperatorListItem;

const plan = {
    target_elite: 2,
    target_level: 60,
    target_skill_level: 7,
    target_skills: [{ skill_index: 1, mastery_level: 3 }],
    target_modules: [{ module_id: "mod_x", module_stage: 2 }],
} as unknown as IOperatorPlanResponse;

const steps = (rows: { current: number; target: number; isUpgraded: boolean }[]) => rows.map(({ current, target, isUpgraded }) => [current, target, isUpgraded]);

describe("planProgress", () => {
    it("reads an operator with no roster entry as E0 Lv1, skill level 1, no modules", () => {
        const progress = planProgress(operator, plan, undefined);
        expect(progress.promotion).toEqual({ current: { elite: 0, level: 1 }, target: { elite: 2, level: 60 }, isUpgraded: true });
        expect(steps(progress.skills)).toEqual([
            [1, 7, true],
            [1, 10, true],
        ]);
        expect(progress.modules.map((m) => m.module.uniEquipId)).toEqual(["mod_x", "mod_y"]);
        expect(steps(progress.modules)).toEqual([
            [0, 2, true],
            [0, 0, false],
        ]);
    });

    it("reads masteries per skill and a locked module as stage 0", () => {
        const entry = {
            elite: 2,
            level: 60,
            skill_level: 7,
            masteries: [{ index: 1, mastery: 3 }],
            modules: [
                { id: "mod_x", level: 1, locked: true },
                { id: "mod_y", level: 2, locked: false },
            ],
        } as unknown as IRosterEntry;
        const progress = planProgress(operator, plan, entry);
        expect(progress.promotion.isUpgraded).toBe(false);
        expect(steps(progress.skills)).toEqual([
            [7, 7, false],
            [10, 10, false],
        ]);
        expect(steps(progress.modules)).toEqual([
            [0, 2, true],
            [2, 0, false],
        ]);
    });
});
