import type { FilterSets } from "#/components/operators/list/impl/shared-filters";
import { hasAnySharedFilter, matchesSharedFilters } from "#/components/operators/list/impl/shared-filters";
import { compactForSearch } from "#/lib/search/fuzzy";
import { isMaxed } from "./helpers.card";
import type { IDisplayEntry, SortKey, SortOrder } from "./types";

export function filterEntries(entries: IDisplayEntry[], search: string, sets: FilterSets): IDisplayEntry[] {
    const q = compactForSearch(search);
    const active = hasAnySharedFilter(sets);
    if (!q && !active) return entries;

    return entries.filter((e) => {
        if (q && !compactForSearch(e.name).includes(q)) return false;
        if (!active) return true;
        // An operator missing from the index cannot be shown as matching any attribute filter.
        if (!e.meta) return false;
        return matchesSharedFilters({ ...e.meta, voiceActors: e.voiceActors }, sets);
    });
}

function levelKey(e: IDisplayEntry): number {
    return e.isOwned ? e.elite * 1000 + e.level : -1;
}

function obtainedKey(e: IDisplayEntry): number {
    return e.isOwned ? (e.obtained_at ?? 0) : -1;
}

function potentialKey(e: IDisplayEntry): number {
    return e.isOwned ? e.potential : -1;
}

function trustKey(e: IDisplayEntry): number {
    return e.isOwned ? e.favor_point : -1;
}

function maxedKey(e: IDisplayEntry): number {
    return e.isOwned && isMaxed(e) ? 1 : 0;
}

function cmpByKey(a: IDisplayEntry, b: IDisplayEntry, key: SortKey): number {
    switch (key) {
        case "rarity":
            return a.rarity - b.rarity;
        case "level": {
            const diff = levelKey(a) - levelKey(b);
            return diff !== 0 ? diff : a.rarity - b.rarity;
        }
        case "obtained": {
            const diff = obtainedKey(a) - obtainedKey(b);
            return diff !== 0 ? diff : a.rarity - b.rarity;
        }
        case "potential": {
            const diff = potentialKey(a) - potentialKey(b);
            return diff !== 0 ? diff : a.rarity - b.rarity;
        }
        case "trust": {
            const diff = trustKey(a) - trustKey(b);
            return diff !== 0 ? diff : a.rarity - b.rarity;
        }
        case "maxed": {
            const diff = maxedKey(a) - maxedKey(b);
            if (diff !== 0) return diff;
            const lvl = levelKey(a) - levelKey(b);
            return lvl !== 0 ? lvl : a.rarity - b.rarity;
        }
    }
}

export function sortEntries(entries: IDisplayEntry[], key: SortKey, order: SortOrder): IDisplayEntry[] {
    const dir = order === "asc" ? 1 : -1;
    return [...entries].sort((a, b) => {
        const c = cmpByKey(a, b, key) * dir;
        return c !== 0 ? c : a.name.localeCompare(b.name);
    });
}
