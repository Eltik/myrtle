import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorBaseSkill, IOperatorListItem } from "#/types/operators";

/** A base skill plus whether this operator's promotion actually grants it. */
export interface IRosterSkill extends IOperatorBaseSkill {
    unlocked: boolean;
}

export interface IRosterOption {
    id: string;
    name: string;
    baseSkills: IRosterSkill[];
}

function isUnlocked(skill: IOperatorBaseSkill, entry: IRosterEntry): boolean {
    if (entry.elite !== skill.unlockElite) return entry.elite > skill.unlockElite;
    return entry.level >= skill.unlockLevel;
}

/**
 * `ignorePromotion` keeps the skills the operator has not promoted far enough
 * to use, flagged `unlocked: false` so the planner can show them greyed out
 * rather than pretending the player already has them.
 */
export function toRosterOptions(roster: IRosterEntry[], operatorsStatic: IOperatorListItem[], ignorePromotion = false): IRosterOption[] {
    const staticById = new Map<string, IOperatorListItem>();
    for (const op of operatorsStatic) {
        if (op.id) staticById.set(op.id, op);
    }

    return roster
        .map((entry) => {
            const op = staticById.get(entry.operator_id);
            const skills: IRosterSkill[] = [];
            for (const skill of op?.baseSkills ?? []) {
                const unlocked = isUnlocked(skill, entry);
                if (unlocked || ignorePromotion) skills.push({ ...skill, unlocked });
            }
            return { id: entry.operator_id, name: op?.name ?? entry.operator_id, baseSkills: skills };
        })
        .sort((a, b) => a.name.localeCompare(b.name));
}
