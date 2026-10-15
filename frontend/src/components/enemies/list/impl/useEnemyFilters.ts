import { useCallback, useMemo } from "react";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { FILTERS_KEY, LEVEL_ORDER } from "./constants";
import type { IEnemyLocationIndex, IEnemyView, IFilterState, IUseEnemyFiltersReturn } from "./types";

const initialState: IFilterState = {
    q: "",
    levels: [],
    damageTypes: [],
    attackTypes: [],
    races: [],
    appearsIn: [],
    immunities: [],
    sortBy: "index",
    sortOrder: "asc",
};

const EMPTY_LOCATIONS: IEnemyLocationIndex = { zonesByEnemy: new Map(), stagesByEnemy: new Map() };

/**
 * The keys that narrow the result set, as opposed to reordering it. Changing one
 * invalidates whatever page you were on, so it fires `onNarrow`; changing a sort
 * key leaves the row count alone and does not.
 */
const NARROWING_KEYS: ReadonlySet<keyof IFilterState> = new Set(["q", "levels", "damageTypes", "attackTypes", "races", "appearsIn", "immunities"]);

/**
 * `onNarrow` fires only from the setters, never from the mount-time hydration of
 * stored filters, so rewinding the page here can't discard a page just restored
 * from the URL.
 */
export function useEnemyFilters(data: IEnemyView[], locations: IEnemyLocationIndex = EMPTY_LOCATIONS, onNarrow?: () => void): IUseEnemyFiltersReturn {
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

    const set = useCallback(
        <K extends keyof IFilterState>(key: K, value: IFilterState[K]) => {
            setFilters((prev) => ({ ...prev, [key]: value }));
            if (NARROWING_KEYS.has(key)) onNarrow?.();
        },
        [setFilters, onNarrow],
    );

    const filtered = useMemo(() => {
        const query = filters.q.trim().toLowerCase();
        const levels = new Set(filters.levels);
        const damageTypes = new Set(filters.damageTypes);
        const attackTypes = new Set(filters.attackTypes);
        const races = new Set(filters.races);
        // Location tokens are zone ids or stage ids; an enemy matches if any
        // selected token is in its zone set or stage set.
        const wantLocations = filters.appearsIn;
        const hasLocationFilter = wantLocations.length > 0;
        // Immunities are AND, unlike the OR filters above: "immune to stun and
        // sleep" means both, so every selected status must be immune.
        const wantImmunities = filters.immunities;

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
            if (hasLocationFilter) {
                const zones = locations.zonesByEnemy.get(e.enemyId);
                const stages = locations.stagesByEnemy.get(e.enemyId);
                const hit = wantLocations.some((t) => zones?.has(t) || stages?.has(t));
                if (!hit) return false;
            }
            if (wantImmunities.length && !wantImmunities.every((k) => e.immunities[k])) return false;
            if (query) {
                const haystack = `${e.name} ${e.enemyIndex} ${e.race ?? ""} ${e.description ?? ""}`.toLowerCase();
                if (!haystack.includes(query)) return false;
            }
            return true;
        });
    }, [data, locations, filters.q, filters.levels, filters.damageTypes, filters.attackTypes, filters.races, filters.appearsIn, filters.immunities]);

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

    const clearFilters = useCallback(() => {
        setFilters((prev) => ({ ...prev, levels: [], damageTypes: [], attackTypes: [], races: [], appearsIn: [], immunities: [], q: "" }));
        onNarrow?.();
    }, [setFilters, onNarrow]);

    const activeFilterCount = filters.levels.length + filters.damageTypes.length + filters.attackTypes.length + filters.races.length + filters.appearsIn.length + filters.immunities.length + (filters.q ? 1 : 0);

    const setters = useMemo(
        () => ({
            setSearchQuery: (v: string) => set("q", v),
            setLevels: (v: IFilterState["levels"]) => set("levels", v),
            setDamageTypes: (v: IFilterState["damageTypes"]) => set("damageTypes", v),
            setAttackTypes: (v: IFilterState["attackTypes"]) => set("attackTypes", v),
            setRaces: (v: IFilterState["races"]) => set("races", v),
            setAppearsIn: (v: IFilterState["appearsIn"]) => set("appearsIn", v),
            setImmunities: (v: IFilterState["immunities"]) => set("immunities", v),
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
