import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";

/** An operator the player owns, reduced to what the planner UI needs. */
export interface RosterOption {
    id: string;
    name: string;
}

/**
 * The profile's owned operators, named.
 *
 * Ownership is a fact about the roster, not a toggle - the backend already
 * builds its candidate pool from exactly this list, so the picker showing
 * anything else would offer operators the optimizer can never seat.
 */
export function toRosterOptions(roster: IRosterEntry[], operatorsStatic: IOperatorListItem[]): RosterOption[] {
    const nameById = new Map<string, string>();
    for (const op of operatorsStatic) {
        if (op.id) nameById.set(op.id, op.name);
    }
    return roster.map((entry) => ({ id: entry.operator_id, name: nameById.get(entry.operator_id) ?? entry.operator_id })).sort((a, b) => a.name.localeCompare(b.name));
}
