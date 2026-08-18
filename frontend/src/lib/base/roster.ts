import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorBaseSkill, IOperatorListItem } from "#/types/operators";

export interface IRosterOption {
    id: string;
    name: string;
    baseSkills: IOperatorBaseSkill[];
}

function isUnlocked(skill: IOperatorBaseSkill, entry: IRosterEntry): boolean {
    if (entry.elite !== skill.unlockElite) return entry.elite > skill.unlockElite;
    return entry.level >= skill.unlockLevel;
}

export function toRosterOptions(roster: IRosterEntry[], operatorsStatic: IOperatorListItem[]): IRosterOption[] {
    const staticById = new Map<string, IOperatorListItem>();
    for (const op of operatorsStatic) {
        if (op.id) staticById.set(op.id, op);
    }

    return roster
        .map((entry) => {
            const op = staticById.get(entry.operator_id);
            return {
                id: entry.operator_id,
                name: op?.name ?? entry.operator_id,
                baseSkills: (op?.baseSkills ?? []).filter((skill) => isUnlocked(skill, entry)),
            };
        })
        .sort((a, b) => a.name.localeCompare(b.name));
}
