import { rarityToNumber } from "#/lib/utils";
import type { OperatorRarityTier } from "#/types/operators";
import type { IDisplayEntry, RarityFilter, SortKey, SortOrder } from "./types";

export function filterEntries(entries: IDisplayEntry[], rarity: RarityFilter, search: string): IDisplayEntry[] {
    const q = search.trim().toLowerCase();
    const tierNum = rarity === "all" ? null : rarityToNumber(rarity as OperatorRarityTier);
    if (!q && tierNum === null) return entries;

    return entries.filter((e) => {
        if (tierNum !== null && e.rarity !== tierNum) return false;
        if (q && !e.name.toLowerCase().includes(q)) return false;
        return true;
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
    }
}

export function sortEntries(entries: IDisplayEntry[], key: SortKey, order: SortOrder): IDisplayEntry[] {
    const dir = order === "asc" ? 1 : -1;
    return [...entries].sort((a, b) => {
        const c = cmpByKey(a, b, key) * dir;
        return c !== 0 ? c : a.name.localeCompare(b.name);
    });
}
