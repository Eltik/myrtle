import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorBaseSkill, IOperatorListItem } from "#/types/operators";

/** A base skill plus whether this operator's promotion actually grants it. */
export interface IRosterSkill extends IOperatorBaseSkill {
    unlocked: boolean;
    /** The tier the game actually runs: the highest unlocked tier of its slot
     *  (the top tier under ignore-promotion). An unlocked-but-not-live tier is
     *  one a promotion has REPLACED - the backend never prices it. */
    live: boolean;
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

/** Tier order within a slot: later promotions supersede earlier ones. */
function tierRank(skill: IOperatorBaseSkill): number {
    return skill.unlockElite * 1000 + skill.unlockLevel;
}

/**
 * `ignorePromotion` keeps the skills the operator has not promoted far enough
 * to use, flagged `unlocked: false` so the planner can show them greyed out
 * rather than pretending the player already has them.
 *
 * Within a slot only ONE tier is live - the highest the operator has unlocked
 * (or the top tier under ignore-promotion), mirroring the backend's
 * `OperatorBaseProfile::build`. Lower unlocked tiers stay listed, flagged
 * `live: false`, so the board can label them "replaced" without needing a
 * ledger line to infer it from.
 */
export function toRosterOptions(roster: IRosterEntry[], operatorsStatic: IOperatorListItem[], ignorePromotion = false): IRosterOption[] {
    const staticById = new Map<string, IOperatorListItem>();
    for (const op of operatorsStatic) {
        if (op.id) staticById.set(op.id, op);
    }

    return roster
        .map((entry) => {
            const op = staticById.get(entry.operator_id);
            const all = op?.baseSkills ?? [];
            // The live tier per slot: highest-ranked tier that counts.
            const liveBySlot = new Map<number, string>();
            for (const skill of all) {
                if (!(ignorePromotion || isUnlocked(skill, entry))) continue;
                const cur = all.find((s) => s.buffId === liveBySlot.get(skill.slot));
                // `>=`: on equal unlock conditions the LATER tier wins, matching the
                // backend's `rfind` over the slot.
                if (!cur || tierRank(skill) >= tierRank(cur)) liveBySlot.set(skill.slot, skill.buffId);
            }
            const skills: IRosterSkill[] = [];
            for (const skill of all) {
                const unlocked = isUnlocked(skill, entry);
                if (unlocked || ignorePromotion) skills.push({ ...skill, unlocked, live: liveBySlot.get(skill.slot) === skill.buffId });
            }
            return { id: entry.operator_id, name: op?.name ?? entry.operator_id, baseSkills: skills };
        })
        .sort((a, b) => a.name.localeCompare(b.name));
}
