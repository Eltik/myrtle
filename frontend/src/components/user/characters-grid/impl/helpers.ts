import { getRarityStarCount } from "~/lib/utils";
import type { CharacterData } from "~/types/api/impl/user";
import type { RarityFilter, SortBy, SortOrder } from "~/types/frontend/impl/user";

type CharacterWithStatic = CharacterData & { static?: { name?: string; rarity?: string } };

export function filterCharacters(chars: CharacterWithStatic[], filterRarity: RarityFilter, searchTerm: string): CharacterWithStatic[] {
    return chars.filter((char) => {
        const charRarity = (char.static?.rarity ?? "TIER_1") as string;
        const charName = (char.static?.name ?? "").toLowerCase();
        const matchesRarity = filterRarity === "all" || charRarity === filterRarity;
        const matchesSearch = charName.includes(searchTerm.toLowerCase());
        return matchesRarity && matchesSearch;
    });
}

export function sortCharacters(chars: CharacterWithStatic[], sortBy: SortBy, sortOrder: SortOrder): CharacterWithStatic[] {
    return [...chars].sort((a, b) => {
        let comparison = 0;
        switch (sortBy) {
            case "level":
                if (b.evolvePhase !== a.evolvePhase) {
                    comparison = b.evolvePhase - a.evolvePhase;
                } else {
                    comparison = b.level - a.level;
                }
                break;
            case "rarity":
                comparison = getRarityStarCount(b.static?.rarity ?? "TIER_1") - getRarityStarCount(a.static?.rarity ?? "TIER_1");
                break;
            case "obtained":
                comparison = b.gainTime - a.gainTime;
                break;
            case "potential":
                comparison = (b.potentialRank ?? 0) - (a.potentialRank ?? 0);
                break;
        }
        return sortOrder === "asc" ? -comparison : comparison;
    });
}

export function filterAndSortCharacters(chars: CharacterWithStatic[], sortBy: SortBy, sortOrder: SortOrder, filterRarity: RarityFilter, searchTerm: string): CharacterWithStatic[] {
    const filtered = filterCharacters(chars, filterRarity, searchTerm);
    return sortCharacters(filtered, sortBy, sortOrder);
}
