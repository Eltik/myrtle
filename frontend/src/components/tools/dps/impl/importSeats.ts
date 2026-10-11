/**
 * `?import=` from the Squad Builder: turn each seat into the DPS calculator's
 * build config. The Squad Builder counts potential and skills from 0; the
 * calculator counts both from 1, and folds skill level and mastery into one
 * 1-10 rank. Pure, so the offsets are tested.
 */

import { FIRST_MASTERY_RANK } from "#/components/tools/shared/skill";
import type { IBuildConfig, ICalcBuffs, IOperatorListEntry } from "#/components/tools/shared/types";
import type { IDpsImportSeat } from "#/components/tools/squad-builder/model";

/** Skill level 1-7 plus mastery 0-3 as the calculator's 1-10 rank. */
export function seatSkillRank(skillLevel: number, mastery: number): number {
    if (mastery > 0) return Math.min(10, FIRST_MASTERY_RANK - 1 + mastery);
    return Math.max(1, Math.min(7, skillLevel));
}

/** The calculator's module number for a module id, 0 when the engine does not know it. */
export function seatModuleIndex(op: Pick<IOperatorListEntry, "availableModules" | "availableModuleIds">, moduleId: string | null): number {
    if (!moduleId) return 0;
    const slot = op.availableModuleIds.indexOf(moduleId);
    if (slot < 0) return 0;
    return op.availableModules[slot] ?? 0;
}

export function importSeatToConfig(seat: IDpsImportSeat, op: Pick<IOperatorListEntry, "availableSkills" | "availableModules" | "availableModuleIds" | "defaultSkill">): Partial<IBuildConfig> {
    const skill = seat.skillIndex + 1;
    const moduleIndex = seatModuleIndex(op, seat.moduleId);
    const buffs: ICalcBuffs = {};
    if (seat.buffs.atk) buffs.atk = seat.buffs.atk;
    if (seat.buffs.flatAtk) buffs.flatAtk = seat.buffs.flatAtk;
    if (seat.buffs.aspd) buffs.aspd = seat.buffs.aspd;
    return {
        promotion: seat.phase,
        level: seat.level,
        // No upper caps here: the Squad Builder already clamped potential and
        // module rank to the operator's own `potentialRanks` and module
        // `rankCount` (`model.ts` `clampBuild`), and the engine's ranks start at 1.
        potential: Math.max(1, seat.potential + 1),
        skillIndex: op.availableSkills.includes(skill) ? skill : op.defaultSkill,
        skillRank: seatSkillRank(seat.skillLevel, seat.mastery),
        moduleIndex,
        buffs,
        // No module: leave the level to the calculator's own default.
        ...(moduleIndex > 0 ? { moduleLevel: Math.max(1, seat.moduleLevel || 1) } : {}),
    };
}
