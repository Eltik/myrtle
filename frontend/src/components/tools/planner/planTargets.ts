import type { IOperatorPlanResponse } from "#/lib/api/planner";
import type { IRosterEntry } from "#/lib/api/user";
import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem, IOperatorModule } from "#/types/operators";

/**
 * The plan dialog's rules: what a target may be for an operator at a given
 * promotion and level, where a new target starts, and how a target is written
 * back to the planner API. Pure functions, so the dialog and its tests share
 * one copy.
 *
 * A skill target is a single number on one axis: 1-7 are the shared skill
 * levels, and 8-10 are Mastery 1-3 of that one skill. Levels 1-7 are shared by
 * every skill of the operator, which is why lowering one skill below 7 lowers
 * them all, and raising one into mastery lifts the others to 7.
 */

/** The last shared skill level; anything above it is a mastery of one skill. */
export const MAX_SKILL_LEVEL = 7;

/** Module stages a target can name; 0 means the module is not planned. */
export const MODULE_STAGES = [0, 1, 2, 3] as const;

/** Every promotion the dialog can offer; `maxEliteFor` trims it per rarity. */
export const ELITE_PHASES = [0, 1, 2] as const;

/**
 * Raidian's upgrades follow Integrated Strategies 6 progression rather than
 * the usual promotion, skill and module costs, so the dialog shows a notice in
 * place of the target fields and refuses to save.
 */
export const UNPLANNABLE_OPERATOR_ID = "char_4195_radian";

/**
 * The prefix every planner mutation invalidates. It is a prefix on purpose:
 * `plansQueryOptions` appends the active-plan filter, and every filtered copy
 * of the plan list has to refetch.
 */
export const PLANS_QUERY_PREFIX = ["user", "plans"] as const;

const PROMOTION_PHASES: readonly string[] = ["PHASE_0", "PHASE_1", "PHASE_2"];

export type SkillTargets = Record<number, number>;
export type ModuleTargets = Record<string, number>;

export type IOperatorSkill = IOperatorListItem["skills"][number];

/** The mastery rank a skill target names, or 0 below mastery. */
export function masteryOf(skillTarget: number): number {
    return skillTarget - MAX_SKILL_LEVEL;
}

export function getMaxLevel(rarity: number, elite: number): number {
    if (rarity <= 2) return 30;
    if (rarity === 3) return elite === 0 ? 40 : 55;
    if (rarity === 4) return elite === 0 ? 45 : elite === 1 ? 60 : 70;
    if (rarity === 5) return elite === 0 ? 50 : elite === 1 ? 70 : 80;
    return elite === 0 ? 50 : elite === 1 ? 80 : 90;
}

export function maxEliteFor(rarity: number): number {
    if (rarity <= 2) return 0;
    if (rarity === 3) return 1;
    return 2;
}

/** The Elite digit a locked-step tooltip names for a `PHASE_N` requirement. */
export function phaseLabel(phase: string): string {
    if (phase === "PHASE_0") return "0";
    if (phase === "PHASE_1") return "1";
    return "2";
}

/**
 * Whether an operator at `elite`/`level` meets an unlock requirement. A
 * missing requirement is met. An unrecognised required phase ranks below
 * Elite 0, so it is met too; an out-of-range `elite` counts as Elite 0.
 */
function meetsUnlock(cond: { phase: string; level: number } | null | undefined, elite: number, level: number): boolean {
    if (!cond) return true;
    const required = PROMOTION_PHASES.indexOf(cond.phase);
    const current = PROMOTION_PHASES[elite] ? elite : 0;
    if (current < required) return false;
    if (current === required && level < cond.level) return false;
    return true;
}

/**
 * The upgrade entry that raises skill `skillIdx` to `skillTarget`: the shared
 * `allSkillLevelUp` table for levels 2-7, the skill's own mastery table above.
 */
function skillUpgrade(operator: IOperatorListItem, skillIdx: number, skillTarget: number) {
    if (skillTarget <= MAX_SKILL_LEVEL) return operator.allSkillLevelUp?.[skillTarget - 2];
    return operator.skills?.[skillIdx]?.levelUpCostCond?.[skillTarget - 8];
}

/** The requirement that gates `skillTarget`, for the locked-step tooltip. */
export function skillUnlockCond(operator: IOperatorListItem, skillIdx: number, skillTarget: number) {
    return skillUpgrade(operator, skillIdx, skillTarget)?.unlockCond;
}

/** How many steps a skill's target picker offers: level 1, the shared levels, then its masteries. */
export function skillStepCount(operator: IOperatorListItem, skill: IOperatorSkill): number {
    return 1 + (operator.allSkillLevelUp?.length ?? 0) + (skill.levelUpCostCond?.length ?? 0);
}

export function isSkillLevelAllowed(operator: IOperatorListItem, skillIdx: number, skillTarget: number, elite: number, level: number): boolean {
    if (skillTarget <= 1) return true;
    const upgrade = skillUpgrade(operator, skillIdx, skillTarget);
    if (!upgrade) return false;
    return meetsUnlock(upgrade.unlockCond, elite, level);
}

export function isModuleAllowed(mod: IOperatorModule, elite: number, level: number): boolean {
    return meetsUnlock({ phase: mod.unlockEvolvePhase, level: mod.unlockLevel }, elite, level);
}

/** The modules a plan can target; every operator's `ORIGINAL` entry is the unmodded base, not a module. */
export function plannableModules(operator: IOperatorListItem): IOperatorModule[] {
    return operator.modules.filter((m) => m.typeName1 !== "ORIGINAL");
}

/**
 * The skill target a stored state names for skill `idx`: the shared level
 * below 7, else 7 plus that skill's mastery (a missing mastery row is M0).
 */
function seedSkillTargets(operator: IOperatorListItem, skillLevel: number, masteryAt: (idx: number) => number | undefined): SkillTargets {
    const targets: SkillTargets = {};
    operator.skills.forEach((_, idx) => {
        if (skillLevel < MAX_SKILL_LEVEL) {
            targets[idx] = skillLevel;
            return;
        }
        const mastery = masteryAt(idx) ?? 0;
        targets[idx] = mastery > 0 ? MAX_SKILL_LEVEL + mastery : MAX_SKILL_LEVEL;
    });
    return targets;
}

function seedModuleTargets(operator: IOperatorListItem, stageOf: (mod: IOperatorModule) => number): ModuleTargets {
    const targets: ModuleTargets = {};
    for (const mod of plannableModules(operator)) {
        targets[mod.uniEquipId] = stageOf(mod);
    }
    return targets;
}

export interface IInitialTargets {
    elite: number;
    level: number;
    skills: SkillTargets;
    modules: ModuleTargets;
    displayOnProfile: boolean;
    groups: string[];
}

/**
 * Where the dialog's targets start for `operator`: the saved plan when there
 * is one, else the player's current roster state (so a new plan starts from
 * what they already have), else a fresh E0 Lv1 operator. The promotion and
 * level are clamped to what the operator's rarity allows.
 */
export function initialTargets(operator: IOperatorListItem, plan: IOperatorPlanResponse | undefined, rosterEntry: IRosterEntry | undefined): IInitialTargets {
    const rarity = rarityToNumber(operator.rarity);
    const clampElite = (elite: number) => Math.min(elite, maxEliteFor(rarity));

    if (plan) {
        const elite = clampElite(plan.target_elite);
        return {
            elite,
            level: Math.min(plan.target_level, getMaxLevel(rarity, elite)),
            skills: seedSkillTargets(operator, plan.target_skill_level, (idx) => plan.target_skills?.find((s) => s.skill_index === idx)?.mastery_level),
            modules: seedModuleTargets(operator, (mod) => plan.target_modules?.find((m) => m.module_id === mod.uniEquipId)?.module_stage ?? 0),
            displayOnProfile: plan.display_on_profile,
            groups: plan.groups ?? [],
        };
    }

    if (rosterEntry) {
        const elite = clampElite(rosterEntry.elite);
        return {
            elite,
            level: Math.min(rosterEntry.level, getMaxLevel(rarity, elite)),
            skills: seedSkillTargets(operator, rosterEntry.skill_level, (idx) => rosterEntry.masteries?.find((m) => m.index === idx)?.mastery),
            modules: seedModuleTargets(operator, (mod) => {
                const owned = rosterEntry.modules?.find((m) => m.id === mod.uniEquipId);
                return owned && !owned.locked ? owned.level : 0;
            }),
            displayOnProfile: false,
            groups: [],
        };
    }

    return {
        elite: 0,
        level: 1,
        skills: seedSkillTargets(operator, 1, () => undefined),
        modules: seedModuleTargets(operator, () => 0),
        displayOnProfile: false,
        groups: [],
    };
}

/**
 * Sets skill `skillIdx` to `value`. Below 7 the value is a shared level, so it
 * moves every skill; at 7 and above only this skill masters, and the others
 * are lifted to 7 because a mastery needs the shared level maxed.
 */
export function withSkillTarget(prev: SkillTargets, skillCount: number, skillIdx: number, value: number): SkillTargets {
    const next = { ...prev };
    if (value < MAX_SKILL_LEVEL) {
        for (let idx = 0; idx < skillCount; idx++) next[idx] = value;
        return next;
    }
    next[skillIdx] = value;
    for (let idx = 0; idx < skillCount; idx++) {
        if (idx !== skillIdx && (next[idx] ?? 1) < MAX_SKILL_LEVEL) next[idx] = MAX_SKILL_LEVEL;
    }
    return next;
}

/**
 * Lowers each skill target to the highest step the promotion and level
 * unlock. If any skill had to drop below 7, the shared level drops for every
 * skill to the lowest of them, since levels 1-7 are one value.
 *
 * Always returns a fresh object, even when nothing moved.
 */
export function clampSkillTargets(prev: SkillTargets, operator: IOperatorListItem, elite: number, level: number): SkillTargets {
    const next = { ...prev };
    let changed = false;
    operator.skills.forEach((_, idx) => {
        const current = next[idx] ?? 1;
        let allowed = current;
        while (allowed > 1 && !isSkillLevelAllowed(operator, idx, allowed, elite, level)) allowed--;
        if (allowed !== current) {
            next[idx] = allowed;
            changed = true;
        }
    });
    if (!changed) return next;

    const lowest = Math.min(MAX_SKILL_LEVEL, ...operator.skills.map((_, idx) => next[idx] ?? 1));
    if (lowest < MAX_SKILL_LEVEL) {
        operator.skills.forEach((_, idx) => {
            next[idx] = lowest;
        });
    }
    return next;
}

/** Unplans every module the promotion and level no longer unlock; returns `prev` when nothing moved. */
export function clampModuleTargets(prev: ModuleTargets, operator: IOperatorListItem, elite: number, level: number): ModuleTargets {
    const next = { ...prev };
    let changed = false;
    for (const mod of plannableModules(operator)) {
        if ((next[mod.uniEquipId] ?? 0) > 0 && !isModuleAllowed(mod, elite, level)) {
            next[mod.uniEquipId] = 0;
            changed = true;
        }
    }
    return changed ? next : prev;
}

/**
 * The skill and module fields of the upsert payload. The API stores one shared
 * skill level (capped at 7) plus a mastery rank per skill.
 */
export function planTargetPayload(skills: SkillTargets, modules: ModuleTargets) {
    const values = Object.values(skills);
    return {
        targetSkillLevel: values.length > 0 ? Math.min(MAX_SKILL_LEVEL, Math.max(1, ...values)) : 1,
        targetSkills: Object.entries(skills).map(([idx, value]) => ({
            skill_index: Number.parseInt(idx, 10),
            mastery_level: value > MAX_SKILL_LEVEL ? masteryOf(value) : 0,
        })),
        targetModules: Object.entries(modules).map(([moduleId, stage]) => ({
            module_id: moduleId,
            module_stage: stage,
        })),
    };
}
