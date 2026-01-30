import type { Enemy } from "~/types/api/impl/enemy";
import type { EnemyFilterState, EnemySortOption, SortOrder } from "./types";

// Level sort order
const LEVEL_ORDER: Record<string, number> = {
    NORMAL: 0,
    ELITE: 1,
    BOSS: 2,
};

/**
 * Get the max level stats for an enemy
 */
export function getEnemyMaxStats(enemy: Enemy) {
    const levels = enemy.stats?.levels;
    if (!levels || levels.length === 0) {
        return null;
    }
    // Get the highest level stats
    const firstLevel = levels[0];
    if (!firstLevel) {
        return null;
    }
    const maxLevelStats = levels.reduce((max, current) => (current.level > max.level ? current : max), firstLevel);
    return maxLevelStats.attributes;
}

/**
 * Filter enemies based on search query and selected filters
 */
export function filterEnemies(enemies: Enemy[], filters: EnemyFilterState): Enemy[] {
    return enemies.filter((enemy) => {
        // Search filter
        if (filters.searchQuery) {
            const query = filters.searchQuery.toLowerCase();
            const matchesName = enemy.name.toLowerCase().includes(query);
            const matchesId = enemy.enemyId.toLowerCase().includes(query);
            const matchesDescription = enemy.description?.toLowerCase().includes(query) ?? false;
            if (!matchesName && !matchesId && !matchesDescription) {
                return false;
            }
        }

        // Level filter
        if (filters.selectedLevels.length > 0) {
            if (!filters.selectedLevels.includes(enemy.enemyLevel)) {
                return false;
            }
        }

        // Damage type filter
        if (filters.selectedDamageTypes.length > 0) {
            const enemyDamageTypes = enemy.damageType ?? [];
            const hasMatchingDamageType = filters.selectedDamageTypes.some((dt) => enemyDamageTypes.includes(dt as "PHYSIC" | "MAGIC" | "NO_DAMAGE" | "HEAL"));
            if (!hasMatchingDamageType) {
                return false;
            }
        }

        // Race filter - uses enemyTags which contain race IDs
        if (filters.selectedRaces.length > 0) {
            const enemyTags = enemy.enemyTags ?? [];
            const hasMatchingRace = filters.selectedRaces.some((race) => enemyTags.includes(race));
            if (!hasMatchingRace) {
                return false;
            }
        }

        // Immunity filter - enemy must have ALL selected immunities
        if (filters.selectedImmunities && filters.selectedImmunities.length > 0) {
            const stats = getEnemyMaxStats(enemy);
            if (!stats) {
                return false;
            }
            const immunityMap: Record<string, boolean> = {
                stun: stats.stunImmune,
                silence: stats.silenceImmune,
                sleep: stats.sleepImmune,
                frozen: stats.frozenImmune,
                levitate: stats.levitateImmune,
            };
            const hasAllImmunities = filters.selectedImmunities.every((immunity) => immunityMap[immunity]);
            if (!hasAllImmunities) {
                return false;
            }
        }

        // Stat range filters
        if (filters.statFilters) {
            const stats = getEnemyMaxStats(enemy);
            if (!stats) {
                // If no stats available but filters are set, exclude the enemy
                const hasStatFilter =
                    filters.statFilters.hp.min !== null ||
                    filters.statFilters.hp.max !== null ||
                    filters.statFilters.atk.min !== null ||
                    filters.statFilters.atk.max !== null ||
                    filters.statFilters.def.min !== null ||
                    filters.statFilters.def.max !== null ||
                    filters.statFilters.res.min !== null ||
                    filters.statFilters.res.max !== null;
                if (hasStatFilter) {
                    return false;
                }
            } else {
                // HP filter
                if (filters.statFilters.hp.min !== null && stats.maxHp < filters.statFilters.hp.min) {
                    return false;
                }
                if (filters.statFilters.hp.max !== null && stats.maxHp > filters.statFilters.hp.max) {
                    return false;
                }
                // ATK filter
                if (filters.statFilters.atk.min !== null && stats.atk < filters.statFilters.atk.min) {
                    return false;
                }
                if (filters.statFilters.atk.max !== null && stats.atk > filters.statFilters.atk.max) {
                    return false;
                }
                // DEF filter
                if (filters.statFilters.def.min !== null && stats.def < filters.statFilters.def.min) {
                    return false;
                }
                if (filters.statFilters.def.max !== null && stats.def > filters.statFilters.def.max) {
                    return false;
                }
                // RES filter
                if (filters.statFilters.res.min !== null && stats.magicResistance < filters.statFilters.res.min) {
                    return false;
                }
                if (filters.statFilters.res.max !== null && stats.magicResistance > filters.statFilters.res.max) {
                    return false;
                }
            }
        }

        return true;
    });
}

/**
 * Sort enemies based on sort option and order
 */
export function sortEnemies(enemies: Enemy[], sortBy: EnemySortOption, sortOrder: SortOrder): Enemy[] {
    const sorted = [...enemies].sort((a, b) => {
        let comparison = 0;

        switch (sortBy) {
            case "sortId":
                comparison = a.sortId - b.sortId;
                break;
            case "name":
                comparison = a.name.localeCompare(b.name);
                break;
            case "level":
                comparison = (LEVEL_ORDER[a.enemyLevel] ?? 0) - (LEVEL_ORDER[b.enemyLevel] ?? 0);
                break;
            case "hp": {
                const aStats = getEnemyMaxStats(a);
                const bStats = getEnemyMaxStats(b);
                comparison = (aStats?.maxHp ?? 0) - (bStats?.maxHp ?? 0);
                break;
            }
            case "atk": {
                const aStats = getEnemyMaxStats(a);
                const bStats = getEnemyMaxStats(b);
                comparison = (aStats?.atk ?? 0) - (bStats?.atk ?? 0);
                break;
            }
            case "def": {
                const aStats = getEnemyMaxStats(a);
                const bStats = getEnemyMaxStats(b);
                comparison = (aStats?.def ?? 0) - (bStats?.def ?? 0);
                break;
            }
            case "res": {
                const aStats = getEnemyMaxStats(a);
                const bStats = getEnemyMaxStats(b);
                comparison = (aStats?.magicResistance ?? 0) - (bStats?.magicResistance ?? 0);
                break;
            }
            default:
                comparison = 0;
        }

        return sortOrder === "asc" ? comparison : -comparison;
    });

    return sorted;
}

/**
 * Format large numbers for display (e.g., 12500 -> 12.5k)
 */
export function formatStatNumber(num: number): string {
    if (num >= 1000000) {
        return `${(num / 1000000).toFixed(1)}M`;
    }
    if (num >= 10000) {
        return `${(num / 1000).toFixed(1)}k`;
    }
    if (num >= 1000) {
        return `${(num / 1000).toFixed(1)}k`;
    }
    return num.toString();
}
