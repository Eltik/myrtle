import { useCallback, useMemo, useState } from "react";
import type { ApplyWay, DamageType, Enemy, EnemyLevel } from "~/types/api";
import { enemyLevelToNumber } from "../../enemy-list/impl/helpers";

export type SortOption = "name" | "level" | "index" | "hp" | "atk" | "def" | "res" | "move_speed" | "aspd" | "weight";

export interface FilterState {
    searchQuery: string;
    selectedEnemyLevels: EnemyLevel[];
    selectedDamageTypes: DamageType[];
    selectedEnemyTypes: ApplyWay[];
    sortBy: SortOption;
    sortOrder: "asc" | "desc";
}

export interface FilterOptions {
    enemyLevels: EnemyLevel[];
    damageTypes: DamageType[];
    enemyTypes: ApplyWay[];
}

export interface UseEnemyFiltersReturn {
    // State
    filters: FilterState;
    filterOptions: FilterOptions;
    filteredEnemies: Enemy[];

    // Setters
    setSearchQuery: (query: string) => void;
    setSelectedEnemyLevels: (levels: EnemyLevel[]) => void;
    setSelectedDamageTypes: (types: DamageType[]) => void;
    setSelectedEnemyTypes: (types: ApplyWay[]) => void;

    setSortBy: (sortBy: SortOption) => void;
    setSortOrder: (order: "asc" | "desc") => void;

    // Actions
    clearFilters: () => void;

    // Computed
    activeFilterCount: number;
    hasActiveFilters: boolean;
}

export function useEnemyFilters(data: Enemy[]): UseEnemyFiltersReturn {
    // Filter state
    const [searchQuery, setSearchQuery] = useState("");
    const [selectedEnemyLevels, setSelectedEnemyLevels] = useState<EnemyLevel[]>([]);
    const [selectedDamageTypes, setSelectedDamageTypes] = useState<DamageType[]>([]);
    const [selectedEnemyTypes, setSelectedEnemyTypes] = useState<ApplyWay[]>([]);
    const [sortBy, setSortBy] = useState<SortOption>("name");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");

    // Compute available filter options from data
    const filterOptions = useMemo(() => {
        const enemyLevels = new Set<EnemyLevel>();
        const damageTypes = new Set<DamageType>();
        const enemyTypes = new Set<ApplyWay>();

        for (const enemy of data) {
            if (enemy.enemyLevel) {
                enemyLevels.add(enemy.enemyLevel);
            }
            if (enemy.damageType) {
                for (let i = 0; i < enemy.damageType.length; i++) {
                    damageTypes.add(enemy.damageType[i] ?? "NO_DAMAGE");
                }
            }
            if (enemy.stats) {
                for (let i = 0; i < (enemy.stats?.levels ?? []).length; i++) {
                    enemyTypes.add(enemy.stats.levels[i]?.applyWay ?? "NONE");
                }
            }
        }

        return {
            enemyLevels: Array.from(enemyLevels).sort(),
            damageTypes: Array.from(damageTypes).sort(),
            enemyTypes: Array.from(enemyTypes).sort(),
        };
    }, [data]);

    const filterSets = useMemo(
        () => ({
            enemyLevels: new Set(selectedEnemyLevels),
            damageTypes: new Set(selectedDamageTypes),
            enemyTypes: new Set(selectedEnemyTypes),
        }),
        [selectedEnemyLevels, selectedDamageTypes, selectedEnemyTypes],
    );

    const lowercaseQuery = useMemo(() => searchQuery.toLowerCase(), [searchQuery]);

    const hasFilters = useMemo(() => searchQuery !== "" || selectedEnemyLevels.length > 0 || selectedDamageTypes.length > 0 || selectedEnemyTypes.length > 0, [searchQuery, selectedEnemyLevels, selectedDamageTypes, selectedEnemyTypes]);

    // Apply filters and sorting
    const filteredEnemies = useMemo(() => {
        let result: Enemy[];

        if (!hasFilters) {
            result = [...data];
        } else {
            result = data.filter((enemy) => {
                if (lowercaseQuery && !enemy.name.toLowerCase().includes(lowercaseQuery)) {
                    return false;
                }

                if (filterSets.enemyLevels.size > 0 && !filterSets.enemyLevels.has(enemy.enemyLevel)) {
                    return false;
                }

                if (filterSets.damageTypes.size > 0 && !enemy.damageType.some((type) => filterSets.damageTypes.has(type))) {
                    return false;
                }

                if (filterSets.enemyTypes.size > 0 && enemy.stats && !enemy.stats?.levels.some((e) => filterSets.enemyTypes.has(e.applyWay))) {
                    return false;
                }
                return true;
            });
        }

        const getMaxStats = (enemy: Enemy) => {
            return enemy.stats?.levels[enemy.stats.levels.length - 1]?.attributes;
        };

        // Sorting
        result.sort((a, b) => {
            let comparison = 0;

            switch (sortBy) {
                case "name":
                    comparison = a.name.localeCompare(b.name);
                    break;

                case "index": {
                    comparison = a.enemyIndex.localeCompare(b.enemyIndex);
                    break;
                }

                case "level": {
                    comparison = enemyLevelToNumber(b.enemyLevel) - enemyLevelToNumber(a.enemyLevel);
                    break;
                }

                case "hp": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.maxHp ?? 0) - (aStats?.maxHp ?? 0);
                    break;
                }

                case "atk": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.atk ?? 0) - (aStats?.atk ?? 0);
                    break;
                }

                case "def": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.def ?? 0) - (aStats?.def ?? 0);
                    break;
                }

                case "res": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.magicResistance ?? 0) - (aStats?.magicResistance ?? 0);
                    break;
                }

                case "move_speed": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.moveSpeed ?? 0) - (aStats?.moveSpeed ?? 0);
                    break;
                }

                case "weight": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.massLevel ?? 0) - (aStats?.massLevel ?? 0);
                    break;
                }

                case "aspd": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.attackSpeed ?? 0) - (aStats?.attackSpeed ?? 0);
                    break;
                }

                default: {
                    // "level" and fallback
                    // Primary sort: Enemy level
                    const aLevel = enemyLevelToNumber(a.enemyLevel);
                    const bLevel = enemyLevelToNumber(b.enemyLevel);
                    comparison = bLevel - aLevel;

                    if (comparison === 0) {
                        // Tertiary sort: Name (when rarity and class are the same)
                        return a.name.localeCompare(b.name);
                    }
                    break;
                }
            }

            // Apply sort order
            return sortOrder === "asc" ? -comparison : comparison;
        });

        return result;
    }, [data, hasFilters, lowercaseQuery, filterSets, sortBy, sortOrder]);

    const clearFilters = useCallback(() => {
        setSelectedEnemyLevels([]);
        setSelectedDamageTypes([]);
        setSelectedEnemyTypes([]);
        setSearchQuery("");
    }, []);

    const activeFilterCount = useMemo(() => selectedEnemyLevels.length + selectedDamageTypes.length + selectedEnemyTypes.length + (searchQuery ? 1 : 0), [selectedEnemyLevels.length, selectedDamageTypes.length, selectedEnemyTypes.length, searchQuery]);

    const hasActiveFilters = hasFilters;

    return {
        filters: {
            searchQuery,
            selectedEnemyLevels,
            selectedDamageTypes,
            selectedEnemyTypes,
            sortBy,
            sortOrder,
        },
        filterOptions,
        filteredEnemies,
        setSearchQuery,
        setSelectedEnemyLevels,
        setSelectedDamageTypes,
        setSelectedEnemyTypes,
        setSortBy,
        setSortOrder,
        clearFilters,
        activeFilterCount,
        hasActiveFilters,
    };
}
