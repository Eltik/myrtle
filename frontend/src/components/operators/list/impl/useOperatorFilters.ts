import { useCallback, useMemo } from "react";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { compactForSearch } from "#/lib/search/fuzzy";
import { rarityToNumber } from "#/lib/utils";
import { CLASS_SORT_ORDER } from "./constants";
import type { ArrayFilterKey, IFilterOptions, IFilterState, IOperatorView, IUseOperatorFiltersReturn } from "./types";

const initialState: IFilterState = {
    searchQuery: "",
    classes: [],
    subclasses: [],
    rarities: [],
    genders: [],
    nations: [],
    factions: [],
    races: [],
    birthPlaces: [],
    artists: [],
    voiceActors: [],
    hasNotes: "any",
    availability: "global",
    sortBy: "rarity",
    sortOrder: "desc",
};

export function useOperatorFilters(data: IOperatorView[]): IUseOperatorFiltersReturn {
    const [filters, setFilters] = useLocalStorageState<IFilterState>("operators:filters", initialState);

    const set = useCallback(<K extends keyof IFilterState>(key: K, value: IFilterState[K]) => setFilters((prev) => ({ ...prev, [key]: value })), [setFilters]);

    const removeFrom = useCallback(
        (key: ArrayFilterKey, value: string) =>
            setFilters((prev) => ({
                ...prev,
                [key]: (prev[key] as string[]).filter((v) => v !== value),
            })),
        [setFilters],
    );

    // Build the option-lists for the sidebar from the full dataset.
    const filterOptions = useMemo<IFilterOptions>(() => {
        const subclasses = new Set<string>();
        const nations = new Set<string>();
        const factions = new Set<string>();
        const races = new Set<string>();
        const birthPlaces = new Set<string>();
        const artists = new Set<string>();
        const voiceActors = new Set<string>();

        for (const op of data) {
            const { race, placeOfBirth } = op.profile?.basicInfo ?? {};
            if (op.subProfessionId) subclasses.add(op.subProfessionId);
            if (op.nationId) nations.add(op.nationId);
            if (op.groupId) factions.add(op.groupId);
            if (op.teamId) factions.add(op.teamId);
            if (race && race !== "Unknown") races.add(race);
            if (placeOfBirth) birthPlaces.add(placeOfBirth);
            for (const a of op.artists) artists.add(a);
            for (const v of op.voiceActors) voiceActors.add(v);
        }

        return {
            subclasses: [...subclasses].sort(),
            nations: [...nations].sort(),
            factions: [...factions].sort(),
            races: [...races].sort(),
            birthPlaces: [...birthPlaces].sort(),
            artists: [...artists].sort(),
            voiceActors: [...voiceActors].sort(),
        };
    }, [data]);

    // Filter first, then sort - separating the two memos means changing sort
    // doesn't rerun the (more expensive) filter pass.
    const filtered = useMemo(() => {
        const query = compactForSearch(filters.searchQuery.trim());
        const sets = {
            classes: new Set(filters.classes),
            subclasses: new Set(filters.subclasses),
            rarities: new Set(filters.rarities),
            genders: new Set(filters.genders),
            nations: new Set(filters.nations),
            factions: new Set(filters.factions),
            races: new Set(filters.races),
            birthPlaces: new Set(filters.birthPlaces),
            artists: new Set(filters.artists),
            voiceActors: new Set(filters.voiceActors),
        };

        return data.filter((op) => {
            if (query) {
                const haystack = compactForSearch(`${op.name} ${op.appellation ?? ""} ${op.subProfessionId}`);
                if (!haystack.includes(query)) return false;
            }
            if (sets.classes.size && !sets.classes.has(op.profession)) return false;
            if (sets.subclasses.size && !sets.subclasses.has(op.subProfessionId)) return false;
            if (sets.rarities.size && !sets.rarities.has(op.rarity)) return false;
            if (sets.genders.size && (!op.gender || !sets.genders.has(op.gender))) return false;
            if (sets.nations.size && (!op.nationId || !sets.nations.has(op.nationId))) return false;
            if (sets.factions.size) {
                const hit = (op.groupId && sets.factions.has(op.groupId)) || (op.teamId && sets.factions.has(op.teamId));
                if (!hit) return false;
            }
            if (sets.races.size && (!op.race || !sets.races.has(op.race))) return false;
            if (sets.birthPlaces.size && (!op.placeOfBirth || !sets.birthPlaces.has(op.placeOfBirth))) return false;
            if (sets.artists.size && !op.artists.some((a) => sets.artists.has(a))) return false;
            if (sets.voiceActors.size && !op.voiceActors.some((v) => sets.voiceActors.has(v))) return false;
            if (filters.hasNotes === "yes" && !op.hasNotes) return false;
            if (filters.hasNotes === "no" && op.hasNotes) return false;
            return true;
        });
    }, [data, filters.searchQuery, filters.classes, filters.subclasses, filters.rarities, filters.genders, filters.nations, filters.factions, filters.races, filters.birthPlaces, filters.artists, filters.voiceActors, filters.hasNotes]);

    const filteredOperators = useMemo(() => {
        const { sortBy, sortOrder } = filters;
        const dir = sortOrder === "asc" ? 1 : -1;
        // Primary sort honors `dir`; name tiebreaker is always A→Z so listings stay readable in either direction.
        const nameTiebreak = (a: IOperatorView, b: IOperatorView) => a.name.localeCompare(b.name);
        const cmp = (a: IOperatorView, b: IOperatorView): number => {
            switch (sortBy) {
                case "name":
                    return a.name.localeCompare(b.name) * dir;
                case "class":
                    return ((CLASS_SORT_ORDER[a.profession] ?? 99) - (CLASS_SORT_ORDER[b.profession] ?? 99)) * dir || nameTiebreak(a, b);
                case "hp":
                    return ((a.stats?.hp ?? 0) - (b.stats?.hp ?? 0)) * dir || nameTiebreak(a, b);
                case "atk":
                    return ((a.stats?.atk ?? 0) - (b.stats?.atk ?? 0)) * dir || nameTiebreak(a, b);
                case "def":
                    return ((a.stats?.def ?? 0) - (b.stats?.def ?? 0)) * dir || nameTiebreak(a, b);
                case "res":
                    return ((a.stats?.res ?? 0) - (b.stats?.res ?? 0)) * dir || nameTiebreak(a, b);
                case "cost":
                    return ((a.stats?.cost ?? 0) - (b.stats?.cost ?? 0)) * dir || nameTiebreak(a, b);
                case "block":
                    return ((a.stats?.block ?? 0) - (b.stats?.block ?? 0)) * dir || nameTiebreak(a, b);
                default:
                    return (rarityToNumber(a.rarity) - rarityToNumber(b.rarity)) * dir || (CLASS_SORT_ORDER[a.profession] ?? 99) - (CLASS_SORT_ORDER[b.profession] ?? 99) || nameTiebreak(a, b);
            }
        };
        return [...filtered].sort(cmp);
    }, [filtered, filters.sortBy, filters.sortOrder, filters]);

    const clearFilters = useCallback(() => setFilters(initialState), [setFilters]);

    const activeFilterCount =
        filters.classes.length +
        filters.subclasses.length +
        filters.rarities.length +
        filters.genders.length +
        filters.nations.length +
        filters.factions.length +
        filters.races.length +
        filters.birthPlaces.length +
        filters.artists.length +
        filters.voiceActors.length +
        (filters.searchQuery ? 1 : 0) +
        (filters.hasNotes !== "any" ? 1 : 0);

    const setters = useMemo(
        () => ({
            setSearchQuery: (v: string) => set("searchQuery", v),
            setClasses: (v: string[]) => set("classes", v),
            setSubclasses: (v: string[]) => set("subclasses", v),
            setRarities: (v: IFilterState["rarities"]) => set("rarities", v),
            setGenders: (v: string[]) => set("genders", v),
            setNations: (v: string[]) => set("nations", v),
            setFactions: (v: string[]) => set("factions", v),
            setRaces: (v: string[]) => set("races", v),
            setBirthPlaces: (v: string[]) => set("birthPlaces", v),
            setArtists: (v: string[]) => set("artists", v),
            setVoiceActors: (v: string[]) => set("voiceActors", v),
            setHasNotes: (v: IFilterState["hasNotes"]) => set("hasNotes", v),
            setAvailability: (v: IFilterState["availability"]) => set("availability", v),
            setSortBy: (v: IFilterState["sortBy"]) => setFilters((prev) => ({ ...prev, sortBy: v, sortOrder: v === "name" || v === "class" ? "asc" : "desc" })),
            setSortOrder: (v: IFilterState["sortOrder"]) => set("sortOrder", v),
        }),
        [set, setFilters],
    );

    return {
        filters,
        filterOptions,
        filteredOperators,
        ...setters,
        removeFrom,
        clearFilters,
        activeFilterCount,
        hasActiveFilters: activeFilterCount > 0,
    };
}
