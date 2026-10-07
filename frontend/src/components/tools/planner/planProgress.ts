import type { IOperatorPlanResponse } from "#/lib/api/planner";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem, IOperatorModule } from "#/types/operators";
import { type IOperatorSkill, isPromotionBelow, ownedMastery, ownedModuleStage, plannableModules, skillTargetOf } from "./planTargets";

/**
 * Where a plan stands against the roster, field by field: what the operator
 * is now, what the plan targets, and whether the target is ahead. The planner's
 * plan cards and the profile's Plans tab render the same rows from this.
 */

interface IPromotion {
    elite: number;
    level: number;
}

export interface IProgressStep {
    current: number;
    target: number;
    /** True when the target is ahead of the roster. */
    isUpgraded: boolean;
}

export interface ISkillProgress extends IProgressStep {
    skill: IOperatorSkill;
    index: number;
}

export interface IModuleProgress extends IProgressStep {
    module: IOperatorModule;
}

export interface IPlanProgress {
    promotion: { current: IPromotion; target: IPromotion; isUpgraded: boolean };
    /** Skill values are one-axis skill targets (1-7, then 8-10 for M1-M3). */
    skills: ISkillProgress[];
    modules: IModuleProgress[];
}

function step(current: number, target: number): IProgressStep {
    return { current, target, isUpgraded: target > current };
}

/** Without a roster entry the operator reads as unowned: E0 Lv1, skill level 1, no modules. */
export function planProgress(operator: IOperatorListItem, plan: IOperatorPlanResponse, rosterEntry: IRosterEntry | undefined): IPlanProgress {
    const current = { elite: rosterEntry?.elite ?? 0, level: rosterEntry?.level ?? 1 };
    const target = { elite: plan.target_elite, level: plan.target_level };

    return {
        promotion: { current, target, isUpgraded: isPromotionBelow(current, target) },
        skills: operator.skills.map((skill, index) => {
            const currentSkill = rosterEntry ? skillTargetOf(rosterEntry.skill_level, ownedMastery(rosterEntry, index)) : 1;
            const targetSkill = skillTargetOf(plan.target_skill_level, plan.target_skills?.find((s) => s.skill_index === index)?.mastery_level);
            return { skill, index, ...step(currentSkill, targetSkill) };
        }),
        modules: plannableModules(operator).map((module) => {
            const currentStage = rosterEntry ? ownedModuleStage(rosterEntry, module.uniEquipId) : 0;
            const targetStage = plan.target_modules?.find((m) => m.module_id === module.uniEquipId)?.module_stage ?? 0;
            return { module, ...step(currentStage, targetStage) };
        }),
    };
}
