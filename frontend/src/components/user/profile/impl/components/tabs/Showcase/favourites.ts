import { EMPTY_SHARED_FILTERS, matchesSharedFilters, toFilterSets } from "#/components/operators/list/impl/shared-filters";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorIndexEntry } from "#/types/operators";

// What a clicked favourite tile shows: whose roster the viewer may read, and
// which operators a faction holds.

/**
 * Whether the viewer may read this player's roster. `private`: the owner hid the
 * Roster tab (the profile never asks), or the backend refused or found none
 * (`null`); `loading`: asked, not answered; `ready`: the rows are in hand.
 */
export type RosterAccess = { state: "private" } | { state: "loading" } | { state: "ready"; roster: readonly IRosterEntry[] };

export function rosterAccess(canReadRoster: boolean, roster: readonly IRosterEntry[] | null | undefined): RosterAccess {
    if (!canReadRoster || roster === null) return { state: "private" };
    if (roster === undefined) return { state: "loading" };
    return { state: "ready", roster };
}

/** A faction favourite: its `handbook_team_table` id and level. */
export interface IFactionRef {
    id: string;
    powerLevel: "nation" | "group" | "team";
}

/**
 * Every operator of `faction` in `index`, rarest first, then by name. A nation
 * matches on `nationId`, a group or team on `groupId` or `teamId`, through the
 * /operators Nation and Faction filters themselves, so the count is the count
 * those filters give.
 */
export function factionMembers(index: readonly IOperatorIndexEntry[], faction: IFactionRef): IOperatorIndexEntry[] {
    const sets = toFilterSets(faction.powerLevel === "nation" ? { ...EMPTY_SHARED_FILTERS, nations: [faction.id] } : { ...EMPTY_SHARED_FILTERS, factions: [faction.id] });
    return index.filter((op) => matchesSharedFilters(op, sets)).sort((a, b) => b.rarity - a.rarity || a.name.localeCompare(b.name) || a.id.localeCompare(b.id));
}

/** The player's row for each operator they own, by operator id. */
export function rosterById(roster: readonly IRosterEntry[]): ReadonlyMap<string, IRosterEntry> {
    return new Map(roster.map((r) => [r.operator_id, r]));
}
