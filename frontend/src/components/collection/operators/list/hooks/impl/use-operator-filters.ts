import { useCallback, useMemo, useState } from "react";
import { rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api";
import { CLASS_SORT_ORDER } from "../../constants";

export type SortOption = "name" | "rarity" | "class" | "hp" | "atk" | "def" | "res" | "cost" | "block" | "aspd";

export interface FilterState {
    searchQuery: string;
    selectedClasses: string[];
    selectedSubclasses: string[];
    selectedRarities: number[];
    selectedBirthPlaces: string[];
    selectedNations: string[];
    selectedFactions: string[];
    selectedGenders: string[];
    selectedRaces: string[];
    selectedArtists: string[];
    sortBy: SortOption;
    sortOrder: "asc" | "desc";
}

export interface FilterOptions {
    subclasses: string[];
    birthPlaces: string[];
    nations: string[];
    factions: string[];
    races: string[];
    artists: string[];
}

export interface UseOperatorFiltersReturn {
    // State
    filters: FilterState;
    filterOptions: FilterOptions;
    filteredOperators: OperatorFromList[];

    // Setters
    setSearchQuery: (query: string) => void;
    setSelectedClasses: (classes: string[]) => void;
    setSelectedSubclasses: (subclasses: string[]) => void;
    setSelectedRarities: (rarities: number[]) => void;
    setSelectedBirthPlaces: (places: string[]) => void;
    setSelectedNations: (nations: string[]) => void;
    setSelectedFactions: (factions: string[]) => void;
    setSelectedGenders: (genders: string[]) => void;
    setSelectedRaces: (races: string[]) => void;
    setSelectedArtists: (artists: string[]) => void;
    setSortBy: (sortBy: SortOption) => void;
    setSortOrder: (order: "asc" | "desc") => void;

    // Actions
    clearFilters: () => void;

    // Computed
    activeFilterCount: number;
    hasActiveFilters: boolean;
}

export function useOperatorFilters(data: OperatorFromList[]): UseOperatorFiltersReturn {
    const [searchQuery, setSearchQuery] = useState("");
    const [selectedClasses, setSelectedClasses] = useState<string[]>([]);
    const [selectedSubclasses, setSelectedSubclasses] = useState<string[]>([]);
    const [selectedRarities, setSelectedRarities] = useState<number[]>([]);
    const [selectedBirthPlaces, setSelectedBirthPlaces] = useState<string[]>([]);
    const [selectedNations, setSelectedNations] = useState<string[]>([]);
    const [selectedFactions, setSelectedFactions] = useState<string[]>([]);
    const [selectedGenders, setSelectedGenders] = useState<string[]>([]);
    const [selectedRaces, setSelectedRaces] = useState<string[]>([]);
    const [selectedArtists, setSelectedArtists] = useState<string[]>([]);
    const [sortBy, setSortBy] = useState<SortOption>("rarity");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");

    const filterOptions = useMemo(() => {
        const subclasses = new Set<string>();
        const birthPlaces = new Set<string>();
        const nations = new Set<string>();
        const factions = new Set<string>();
        const races = new Set<string>();
        const artists = new Set<string>();

        for (const op of data) {
            if (op.subProfessionId) {
                subclasses.add(op.subProfessionId.toLowerCase());
            }
            if (op.profile?.basicInfo?.placeOfBirth && op.profile.basicInfo.placeOfBirth !== "Unknown" && op.profile.basicInfo.placeOfBirth !== "Undisclosed") {
                birthPlaces.add(op.profile.basicInfo.placeOfBirth);
            }
            if (op.nationId) {
                nations.add(op.nationId);
            }
            if (op.groupId) {
                factions.add(op.groupId);
            }
            if (op.teamId) {
                factions.add(op.teamId);
            }
            if (op.profile?.basicInfo?.race && op.profile.basicInfo.race !== "Unknown" && op.profile.basicInfo.race !== "Undisclosed") {
                races.add(op.profile.basicInfo.race);
            }
            if (op.artists) {
                for (const artist of op.artists) {
                    if (artist) artists.add(artist);
                }
            }
        }

        return {
            subclasses: Array.from(subclasses).sort(),
            birthPlaces: Array.from(birthPlaces).sort(),
            nations: Array.from(nations).sort(),
            factions: Array.from(factions).sort(),
            races: Array.from(races).sort(),
            artists: Array.from(artists).sort(),
        };
    }, [data]);

    const filterSets = useMemo(
        () => ({
            classes: new Set(selectedClasses),
            subclasses: new Set(selectedSubclasses),
            rarities: new Set(selectedRarities),
            birthPlaces: new Set(selectedBirthPlaces),
            nations: new Set(selectedNations),
            factions: new Set(selectedFactions),
            genders: new Set(selectedGenders),
            races: new Set(selectedRaces),
            artists: new Set(selectedArtists),
        }),
        [selectedClasses, selectedSubclasses, selectedRarities, selectedBirthPlaces, selectedNations, selectedFactions, selectedGenders, selectedRaces, selectedArtists],
    );

    const lowercaseQuery = useMemo(() => searchQuery.toLowerCase(), [searchQuery]);

    const hasFilters = useMemo(
        () => searchQuery !== "" || selectedClasses.length > 0 || selectedSubclasses.length > 0 || selectedRarities.length > 0 || selectedBirthPlaces.length > 0 || selectedNations.length > 0 || selectedFactions.length > 0 || selectedGenders.length > 0 || selectedRaces.length > 0 || selectedArtists.length > 0,
        [searchQuery, selectedClasses, selectedSubclasses, selectedRarities, selectedBirthPlaces, selectedNations, selectedFactions, selectedGenders, selectedRaces, selectedArtists],
    );

    const filteredOperators = useMemo(() => {
        let result: OperatorFromList[];

        if (!hasFilters) {
            result = [...data];
        } else {
            result = data.filter((op) => {
                if (lowercaseQuery && !op.name.toLowerCase().includes(lowercaseQuery) && !op.profession.toLowerCase().includes(lowercaseQuery) && !op.subProfessionId.toLowerCase().includes(lowercaseQuery)) {
                    return false;
                }

                if (filterSets.classes.size > 0 && !filterSets.classes.has(op.profession)) {
                    return false;
                }

                if (filterSets.subclasses.size > 0 && !filterSets.subclasses.has(op.subProfessionId.toLowerCase())) {
                    return false;
                }

                if (filterSets.rarities.size > 0 && !filterSets.rarities.has(rarityToNumber(op.rarity))) {
                    return false;
                }

                if (filterSets.birthPlaces.size > 0 && (!op.profile?.basicInfo?.placeOfBirth || !filterSets.birthPlaces.has(op.profile.basicInfo.placeOfBirth))) {
                    return false;
                }

                if (filterSets.nations.size > 0 && (!op.nationId || !filterSets.nations.has(op.nationId))) {
                    return false;
                }

                if (filterSets.factions.size > 0) {
                    const hasGroup = op.groupId && filterSets.factions.has(op.groupId);
                    const hasTeam = op.teamId && filterSets.factions.has(op.teamId);
                    if (!hasGroup && !hasTeam) {
                        return false;
                    }
                }

                if (filterSets.genders.size > 0 && (!op.profile?.basicInfo?.gender || !filterSets.genders.has(op.profile.basicInfo.gender))) {
                    return false;
                }

                if (filterSets.races.size > 0 && (!op.profile?.basicInfo?.race || !filterSets.races.has(op.profile.basicInfo.race))) {
                    return false;
                }

                if (filterSets.artists.size > 0 && (!op.artists || !op.artists.some((artist) => filterSets.artists.has(artist)))) {
                    return false;
                }

                return true;
            });
        }

        const getMaxStats = (op: OperatorFromList) => {
            const lastPhase = op.phases?.[op.phases.length - 1];
            const lastFrame = lastPhase?.AttributesKeyFrames?.[lastPhase.AttributesKeyFrames.length - 1];
            return lastFrame?.Data;
        };

        result.sort((a, b) => {
            let comparison = 0;

            switch (sortBy) {
                case "name":
                    comparison = a.name.localeCompare(b.name);
                    break;

                case "class": {
                    const aClassOrder = CLASS_SORT_ORDER[a.profession] ?? 99;
                    const bClassOrder = CLASS_SORT_ORDER[b.profession] ?? 99;
                    comparison = aClassOrder - bClassOrder;
                    if (comparison === 0) {
                        return a.name.localeCompare(b.name);
                    }
                    break;
                }

                case "hp": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.MaxHp ?? 0) - (aStats?.MaxHp ?? 0);
                    break;
                }

                case "atk": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.Atk ?? 0) - (aStats?.Atk ?? 0);
                    break;
                }

                case "def": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.Def ?? 0) - (aStats?.Def ?? 0);
                    break;
                }

                case "res": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.MagicResistance ?? 0) - (aStats?.MagicResistance ?? 0);
                    break;
                }

                case "cost": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.Cost ?? 0) - (aStats?.Cost ?? 0);
                    break;
                }

                case "block": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.BlockCnt ?? 0) - (aStats?.BlockCnt ?? 0);
                    break;
                }

                case "aspd": {
                    const aStats = getMaxStats(a);
                    const bStats = getMaxStats(b);
                    comparison = (bStats?.AttackSpeed ?? 0) - (aStats?.AttackSpeed ?? 0);
                    break;
                }

                default: {
                    const aRarity = rarityToNumber(a.rarity);
                    const bRarity = rarityToNumber(b.rarity);
                    comparison = bRarity - aRarity;

                    if (comparison === 0) {
                        const aClassOrder = CLASS_SORT_ORDER[a.profession] ?? 99;
                        const bClassOrder = CLASS_SORT_ORDER[b.profession] ?? 99;
                        const classComparison = aClassOrder - bClassOrder;

                        if (classComparison !== 0) {
                            return classComparison;
                        }

                        return a.name.localeCompare(b.name);
                    }
                    break;
                }
            }

            return sortOrder === "asc" ? -comparison : comparison;
        });

        return result;
    }, [data, hasFilters, lowercaseQuery, filterSets, sortBy, sortOrder]);

    const clearFilters = useCallback(() => {
        setSelectedClasses([]);
        setSelectedSubclasses([]);
        setSelectedRarities([]);
        setSelectedBirthPlaces([]);
        setSelectedNations([]);
        setSelectedFactions([]);
        setSelectedGenders([]);
        setSelectedRaces([]);
        setSelectedArtists([]);
        setSearchQuery("");
    }, []);

    const activeFilterCount = useMemo(
        () => selectedClasses.length + selectedSubclasses.length + selectedRarities.length + selectedBirthPlaces.length + selectedNations.length + selectedFactions.length + selectedGenders.length + selectedRaces.length + selectedArtists.length + (searchQuery ? 1 : 0),
        [selectedClasses.length, selectedSubclasses.length, selectedRarities.length, selectedBirthPlaces.length, selectedNations.length, selectedFactions.length, selectedGenders.length, selectedRaces.length, selectedArtists.length, searchQuery],
    );

    const hasActiveFilters = hasFilters;

    return {
        filters: {
            searchQuery,
            selectedClasses,
            selectedSubclasses,
            selectedRarities,
            selectedBirthPlaces,
            selectedNations,
            selectedFactions,
            selectedGenders,
            selectedRaces,
            selectedArtists,
            sortBy,
            sortOrder,
        },
        filterOptions,
        filteredOperators,
        setSearchQuery,
        setSelectedClasses,
        setSelectedSubclasses,
        setSelectedRarities,
        setSelectedBirthPlaces,
        setSelectedNations,
        setSelectedFactions,
        setSelectedGenders,
        setSelectedRaces,
        setSelectedArtists,
        setSortBy,
        setSortOrder,
        clearFilters,
        activeFilterCount,
        hasActiveFilters,
    };
}
