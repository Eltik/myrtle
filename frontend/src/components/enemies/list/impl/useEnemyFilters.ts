import { useCallback, useMemo } from "react";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { FILTERS_KEY, LEVEL_ORDER } from "./constants";
import type { IEnemyView, IFilterState, IUseEnemyFiltersReturn } from "./types";

const initialState: IFilterState = {
    q: "",
    levels: [],
    damageTypes: [],
    attackTypes: [],
    races: [],
    sortBy: "index",
    sortOrder: "asc",
};

export function useEnemyFilters(data: IEnemyView[]): IUseEnemyFiltersReturn {
    const [filters, setFilters] = useLocalStorageState<IFilterState>(FILTERS_KEY, initialState, {
        parse: (raw) => {
            try {
                const parsed = JSON.parse(raw) as Partial<IFilterState>;
                return { ...initialState, ...parsed };
            } catch {
                return undefined;
            }
        },
    });

    const set = useCallback(<K extends keyof IFilterState>(key: K, value: IFilterState[K]) => setFilters((prev) => ({ ...prev, [key]: value })), [setFilters]);

    const filtered = useMemo(() => {
        const query = filters.q.trim().toLowerCase();
        const levels = new Set(filters.levels);
        const damageTypes = new Set(filters.damageTypes);
        const attackTypes = new Set(filters.attackTypes);
        const races = new Set(filters.races);

        return data.filter((e) => {
            if (e.hideInHandbook) return false;
            if (levels.size && !levels.has(e.enemyLevel)) return false;
            if (damageTypes.size && !e.damageType.some((d) => damageTypes.has(d))) return false;
            if (attackTypes.size) {
                if (!e.applyWay || !attackTypes.has(e.applyWay)) return false;
            }
            if (races.size) {
                const tags = e.enemyTags;
                if (!tags || !tags.some((t) => races.has(t))) return false;
            }
            if (query) {
                const haystack = `${e.name} ${e.enemyIndex} ${e.race ?? ""} ${e.description ?? ""}`.toLowerCase();
                if (!haystack.includes(query)) return false;
            }
            return true;
        });
    }, [data, filters.q, filters.levels, filters.damageTypes, filters.attackTypes, filters.races]);

    const { sortBy, sortOrder } = filters;
    const filteredEnemies = useMemo(() => {
        const dir = sortOrder === "asc" ? 1 : -1;
        const cmp = (a: IEnemyView, b: IEnemyView): number => {
            switch (sortBy) {
                case "name":
                    return a.name.localeCompare(b.name);
                case "level":
                    return (LEVEL_ORDER[a.enemyLevel] ?? 0) - (LEVEL_ORDER[b.enemyLevel] ?? 0) || a.sortId - b.sortId;
                case "hp":
                    return a.flatStats.maxHp - b.flatStats.maxHp;
                case "atk":
                    return a.flatStats.atk - b.flatStats.atk;
                case "def":
                    return a.flatStats.def - b.flatStats.def;
                case "res":
                    return a.flatStats.res - b.flatStats.res;
                case "weight":
                    return a.flatStats.weight - b.flatStats.weight;
                default:
                    return a.sortId - b.sortId;
            }
        };
        return [...filtered].sort((a, b) => cmp(a, b) * dir);
    }, [filtered, sortBy, sortOrder]);

    const clearFilters = useCallback(() => setFilters((prev) => ({ ...prev, levels: [], damageTypes: [], attackTypes: [], races: [], q: "" })), [setFilters]);

    const activeFilterCount = filters.levels.length + filters.damageTypes.length + filters.attackTypes.length + filters.races.length + (filters.q ? 1 : 0);

    const setters = useMemo(
        () => ({
            setSearchQuery: (v: string) => set("q", v),
            setLevels: (v: IFilterState["levels"]) => set("levels", v),
            setDamageTypes: (v: IFilterState["damageTypes"]) => set("damageTypes", v),
            setAttackTypes: (v: IFilterState["attackTypes"]) => set("attackTypes", v),
            setRaces: (v: IFilterState["races"]) => set("races", v),
            setSortBy: (v: IFilterState["sortBy"]) => set("sortBy", v),
            setSortOrder: (v: IFilterState["sortOrder"]) => set("sortOrder", v),
        }),
        [set],
    );

    return {
        filters,
        filteredEnemies,
        ...setters,
        clearFilters,
        activeFilterCount,
        hasActiveFilters: activeFilterCount > 0,
    };
}
