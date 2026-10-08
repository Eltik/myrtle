import type { ITierOperator } from "#/lib/api/tier-entities";
import { formatNationId, nationLabel } from "#/lib/utils";

// The operator pool's Nation, Faction and Race filters: which values each one
// offers and what an operator answers to. The ids and the server's names are
// the ones /operators filters on (`buildFilterOptions`), so the same faction
// reads the same in both places.

export interface IOperatorFacetOption {
    value: string;
    label: string;
}

/** The faction an operator answers to under the Faction filter: its group, else its team. No operator on EN 2026-10-08 has both (77 groups, 54 teams, 0 both). */
export function operatorFaction(op: Pick<ITierOperator, "groupId" | "teamId">): string | null {
    return op.groupId ?? op.teamId;
}

function byLabel(options: Map<string, string>): IOperatorFacetOption[] {
    return [...options].map(([value, label]) => ({ value, label })).sort((a, b) => a.label.localeCompare(b.label) || a.value.localeCompare(b.value));
}

/** Every nation the catalogue holds, by its server name (the English id spelling where the server names none), sorted by label. */
export function nationOptions(catalogue: readonly ITierOperator[]): IOperatorFacetOption[] {
    const options = new Map<string, string>();
    for (const op of catalogue) {
        if (!op.nationId) continue;
        const label = nationLabel(op) ?? op.nationId;
        // A name from any operator beats the id fallback another one fell to.
        if (!options.has(op.nationId) || op.nationName) options.set(op.nationId, label);
    }
    return byLabel(options);
}

/** Every group and team the catalogue holds, named as /operators names them, sorted by label. */
export function factionOptions(catalogue: readonly ITierOperator[]): IOperatorFacetOption[] {
    const options = new Map<string, string>();
    for (const op of catalogue) {
        for (const [id, name] of [
            [op.groupId, op.groupName],
            [op.teamId, op.teamName],
        ] as const) {
            if (!id) continue;
            if (name) options.set(id, name);
            else if (!options.has(id)) options.set(id, formatNationId(id));
        }
    }
    return byLabel(options);
}

/** Every race the catalogue holds (an `Unknown` profile sends none), through `label`, sorted by label. */
export function raceOptions(catalogue: readonly ITierOperator[], label: (race: string) => string): IOperatorFacetOption[] {
    const options = new Map<string, string>();
    for (const op of catalogue) if (op.race && !options.has(op.race)) options.set(op.race, label(op.race));
    return byLabel(options);
}
