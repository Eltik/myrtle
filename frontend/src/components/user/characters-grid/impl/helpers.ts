import { getRarityStarCount } from "~/lib/utils";
import type { DisplayCharacter, EnrichedRosterEntry } from "~/types/api/impl/user";
import type { RarityFilter, SortBy, SortOrder } from "~/types/frontend/impl/user";

function getCharName(char: DisplayCharacter): string {
    if (!char.isOwned) return char.name;
    return (char.static?.name ?? "").toLowerCase();
}

function getCharRarity(char: DisplayCharacter): string {
    if (!char.isOwned) return char.rarity;
    return char.static?.rarity ?? "TIER_1";
}

export function filterCharacters(chars: EnrichedRosterEntry[], filterRarity: RarityFilter, searchTerm: string): EnrichedRosterEntry[] {
    return chars.filter((char) => {
        const charRarity = (char.static?.rarity ?? "TIER_1") as string;
        const charName = (char.static?.name ?? "").toLowerCase();
        const matchesRarity = filterRarity === "all" || charRarity === filterRarity;
        const matchesSearch = charName.includes(searchTerm.toLowerCase());
        return matchesRarity && matchesSearch;
    });
}

export function sortCharacters(chars: EnrichedRosterEntry[], sortBy: SortBy, sortOrder: SortOrder): EnrichedRosterEntry[] {
    return [...chars].sort((a, b) => {
        let comparison = 0;
        switch (sortBy) {
            case "level":
                if (b.elite !== a.elite) {
                    comparison = b.elite - a.elite;
                } else {
                    comparison = b.level - a.level;
                }
                break;
            case "rarity":
                comparison = getRarityStarCount(b.static?.rarity ?? "TIER_1") - getRarityStarCount(a.static?.rarity ?? "TIER_1");
                break;
            case "obtained":
                comparison = (b.obtained_at ?? 0) - (a.obtained_at ?? 0);
                break;
            case "potential":
                comparison = (b.potential ?? 0) - (a.potential ?? 0);
                break;
        }
        return sortOrder === "asc" ? -comparison : comparison;
    });
}

export function filterAndSortCharacters(chars: EnrichedRosterEntry[], sortBy: SortBy, sortOrder: SortOrder, filterRarity: RarityFilter, searchTerm: string): EnrichedRosterEntry[] {
    const filtered = filterCharacters(chars, filterRarity, searchTerm);
    return sortCharacters(filtered, sortBy, sortOrder);
}

export function filterDisplayCharacters(chars: DisplayCharacter[], filterRarity: RarityFilter, searchTerm: string): DisplayCharacter[] {
    const lowerSearch = searchTerm.toLowerCase();
    return chars.filter((char) => {
        const rarity = getCharRarity(char);
        const name = getCharName(char).toLowerCase();
        const matchesRarity = filterRarity === "all" || rarity === filterRarity;
        const matchesSearch = name.includes(lowerSearch);
        return matchesRarity && matchesSearch;
    });
}

export function sortDisplayCharacters(chars: DisplayCharacter[], sortBy: SortBy, sortOrder: SortOrder): DisplayCharacter[] {
    return [...chars].sort((a, b) => {
        // For owned-only sorts (level, obtained, potential), push unowned to the bottom
        const aOwned = a.isOwned;
        const bOwned = b.isOwned;

        if (sortBy === "level" || sortBy === "obtained" || sortBy === "potential") {
            if (aOwned && !bOwned) return -1;
            if (!aOwned && bOwned) return 1;
            // Both unowned: sort by rarity
            if (!aOwned && !bOwned) {
                const cmp = getRarityStarCount(b.rarity) - getRarityStarCount(a.rarity);
                return sortOrder === "asc" ? -cmp : cmp;
            }
        }

        let comparison = 0;
        switch (sortBy) {
            case "level":
                if (aOwned && bOwned) {
                    if (b.elite !== a.elite) {
                        comparison = b.elite - a.elite;
                    } else {
                        comparison = b.level - a.level;
                    }
                }
                break;
            case "rarity":
                comparison = getRarityStarCount(getCharRarity(b)) - getRarityStarCount(getCharRarity(a));
                break;
            case "obtained":
                if (aOwned && bOwned) {
                    comparison = (b.obtained_at ?? 0) - (a.obtained_at ?? 0);
                }
                break;
            case "potential":
                if (aOwned && bOwned) {
                    comparison = (b.potential ?? 0) - (a.potential ?? 0);
                }
                break;
        }
        return sortOrder === "asc" ? -comparison : comparison;
    });
}

export function filterAndSortDisplayCharacters(chars: DisplayCharacter[], sortBy: SortBy, sortOrder: SortOrder, filterRarity: RarityFilter, searchTerm: string): DisplayCharacter[] {
    const filtered = filterDisplayCharacters(chars, filterRarity, searchTerm);
    return sortDisplayCharacters(filtered, sortBy, sortOrder);
}
