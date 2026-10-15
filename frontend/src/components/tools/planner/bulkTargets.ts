import type { IPresetTarget, IUpsertPlanInput } from "#/lib/api/planner";
import type { IRosterEntry } from "#/lib/api/user";
import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { clampModuleTargets, clampSkillTargets, getMaxLevel, higherPromotion, isPromotionBelow, MAX_SKILL_LEVEL, type ModuleTargets, maxEliteFor, ownedMastery, ownedModuleStage, plannableModules, planTargetPayload, type SkillTargets } from "./planTargets";

/**
 * The bulk-add rules: one rarity-free target (a preset) applied to many
 * operators. Each operator gets the preset clamped to what it can reach,
 * using the same unlock rules as the single plan dialog, then raised to
 * wherever the player's roster already is, so a plan never asks to go down.
 * Pure functions, so the dialog and its tests share one copy.
 */

/** Level caps a preset may name at Elite 0, 1 and 2, the highest any rarity reaches. */
const PRESET_LEVEL_CAPS = [50, 80, 90] as const;

export function presetLevelCap(elite: number): number {
    return PRESET_LEVEL_CAPS[elite] ?? 90;
}

/** Skill levels past this one need Elite 1, whatever the operator. */
const PRESET_E0_SKILL_CAP = 4;

/** Where the bulk form starts: Elite 2 at its cap, skill level 7, nothing mastered or modded. */
export const DEFAULT_PRESET_TARGET: IPresetTarget = {
    elite: 2,
    level: null,
    skill_level: MAX_SKILL_LEVEL,
    masteries: [0, 0, 0],
    module_stage: 0,
    display_on_profile: false,
};

export type IBulkPlanTarget = Pick<IUpsertPlanInput, "targetElite" | "targetLevel" | "targetSkillLevel" | "targetSkills" | "targetModules">;

/**
 * Lowers whatever the preset's own fields no longer allow, matching the
 * server's preset rules: the level cap per promotion, skill levels past 4
 * from Elite 1, masteries at Elite 2 and skill level 7, modules at Elite 2.
 */
export function normalizePresetTarget(target: IPresetTarget): IPresetTarget {
    const elite = Math.min(Math.max(0, target.elite), 2);
    const cap = presetLevelCap(elite);
    const level = target.level === null ? null : Math.min(Math.max(1, target.level), cap);
    let skillLevel = Math.min(Math.max(1, target.skill_level), MAX_SKILL_LEVEL);
    if (elite < 1) skillLevel = Math.min(skillLevel, PRESET_E0_SKILL_CAP);
    const canMaster = elite === 2 && skillLevel === MAX_SKILL_LEVEL;
    const masteries = target.masteries.map((m) => (canMaster ? Math.min(Math.max(0, m), 3) : 0)) as IPresetTarget["masteries"];
    const moduleStage = elite === 2 ? Math.min(Math.max(0, target.module_stage), 3) : 0;
    return { elite, level, skill_level: skillLevel, masteries, module_stage: moduleStage, display_on_profile: target.display_on_profile ?? false };
}

type PresetChange = { field: "elite"; value: number } | { field: "level"; value: number | null } | { field: "skill_level"; value: number } | { field: "mastery"; index: number; value: number } | { field: "module_stage"; value: number };

/**
 * Applies one form change. Raising a field lifts what it needs (a mastery
 * lifts promotion to Elite 2 and skill level to 7, a module lifts promotion
 * to Elite 2, a skill level past 4 lifts promotion to Elite 1); lowering one
 * drops whatever depended on it.
 */
export function withPresetChange(prev: IPresetTarget, change: PresetChange): IPresetTarget {
    const next: IPresetTarget = { ...prev, masteries: [...prev.masteries] };
    switch (change.field) {
        case "elite":
            next.elite = change.value;
            break;
        case "level":
            next.level = change.value;
            break;
        case "skill_level":
            next.skill_level = change.value;
            if (change.value > PRESET_E0_SKILL_CAP) next.elite = Math.max(next.elite, 1);
            break;
        case "mastery":
            next.masteries[change.index] = change.value;
            if (change.value > 0) {
                next.elite = 2;
                next.skill_level = MAX_SKILL_LEVEL;
            }
            break;
        case "module_stage":
            next.module_stage = change.value;
            if (change.value > 0) next.elite = 2;
            break;
    }
    return normalizePresetTarget(next);
}

/**
 * The preset clamped to one operator. A promotion past the rarity's last one
 * stops at that last one at its level cap, since any Elite 2 level is beyond
 * every Elite 1 level (a 3-star aimed at E2 Lv1 lands on E1 Lv55). A null
 * level means the cap. Skills the operator lacks are ignored, and each
 * skill and module is then clamped by the same unlock rules the plan dialog
 * uses, so a mastery or module the operator cannot reach at that promotion
 * is dropped rather than rejected by the server.
 */
export function clampPresetForOperator(preset: IPresetTarget, operator: IOperatorListItem): IBulkPlanTarget {
    const rarity = rarityToNumber(operator.rarity);
    const maxElite = maxEliteFor(rarity);
    const elite = Math.min(preset.elite, maxElite);
    const levelCap = getMaxLevel(rarity, elite);
    const level = preset.level === null || preset.elite > maxElite ? levelCap : Math.min(Math.max(1, preset.level), levelCap);

    const wanted: SkillTargets = {};
    operator.skills.forEach((_, idx) => {
        const mastery = preset.skill_level >= MAX_SKILL_LEVEL ? (preset.masteries[idx] ?? 0) : 0;
        wanted[idx] = Math.min(preset.skill_level, MAX_SKILL_LEVEL) + mastery;
    });
    const skills = clampSkillTargets(wanted, operator, elite, level);

    const wantedModules: ModuleTargets = {};
    for (const mod of plannableModules(operator)) wantedModules[mod.uniEquipId] = preset.module_stage;
    const modules = clampModuleTargets(wantedModules, operator, elite, level);

    return { targetElite: elite, targetLevel: level, ...planTargetPayload(skills, modules) };
}

export interface IMergedPlanTarget {
    target: IBulkPlanTarget;
    /** True when the roster already meets or passes every field, so the plan would have nothing to do. */
    isReached: boolean;
}

/**
 * Raises a clamped target to the operator's roster state, field by field:
 * promotion and level as one pair, the shared skill level, each mastery and
 * each module stage. Taking the higher value per field (rather than skipping
 * the operator) keeps the parts of the preset still ahead of the player while
 * never planning a step down. Both inputs pass the unlock rules and every rule
 * only gets easier as promotion and level rise, so the merge passes too.
 *
 * Without a roster entry the target is returned as is.
 */
export function raiseToRoster(target: IBulkPlanTarget, entry: IRosterEntry | undefined): IMergedPlanTarget {
    if (!entry) return { target, isReached: false };

    const current = { elite: entry.elite, level: entry.level };
    const wanted = { elite: target.targetElite, level: target.targetLevel };
    const promotionReached = !isPromotionBelow(current, wanted);
    const promotion = higherPromotion(current, wanted);

    const skillReached = entry.skill_level >= target.targetSkillLevel;
    const mastery = (skillIndex: number) => ownedMastery(entry, skillIndex) ?? 0;
    const targetSkills = target.targetSkills.map((s) => ({ skill_index: s.skill_index, mastery_level: Math.max(mastery(s.skill_index), s.mastery_level) }));
    const masteriesReached = target.targetSkills.every((s) => mastery(s.skill_index) >= s.mastery_level);

    const targetModules = target.targetModules.map((m) => ({ module_id: m.module_id, module_stage: Math.max(ownedModuleStage(entry, m.module_id), m.module_stage) }));
    const modulesReached = target.targetModules.every((m) => ownedModuleStage(entry, m.module_id) >= m.module_stage);

    return {
        target: {
            targetElite: promotion.elite,
            targetLevel: promotion.level,
            targetSkillLevel: Math.max(entry.skill_level, target.targetSkillLevel),
            targetSkills,
            targetModules,
        },
        isReached: promotionReached && skillReached && masteriesReached && modulesReached,
    };
}
