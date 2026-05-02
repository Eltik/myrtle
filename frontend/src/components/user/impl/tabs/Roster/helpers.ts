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

function cmpByKey(a: IDisplayEntry, b: IDisplayEntry, key: SortKey): number {
    if (key === "rarity") return a.rarity - b.rarity;

    if (!a.isOwned && !b.isOwned) return a.rarity - b.rarity;
    if (!a.isOwned) return 1;
    if (!b.isOwned) return -1;

    switch (key) {
        case "level":
            return a.elite !== b.elite ? a.elite - b.elite : a.level - b.level;
        case "obtained":
            return (a.obtained_at ?? 0) - (b.obtained_at ?? 0);
        case "potential":
            return (a.potential ?? 0) - (b.potential ?? 0);
    }
}

export function sortEntries(entries: IDisplayEntry[], key: SortKey, order: SortOrder): IDisplayEntry[] {
    const dir = order === "asc" ? 1 : -1;
    return [...entries].sort((a, b) => {
        const c = cmpByKey(a, b, key) * dir;
        return c !== 0 ? c : a.name.localeCompare(b.name);
    });
}
