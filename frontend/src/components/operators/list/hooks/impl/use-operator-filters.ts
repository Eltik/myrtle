import { useCallback, useMemo, useState } from "react";
import { rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";

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
    sortBy: "name" | "rarity";
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
    setSortBy: (sortBy: "name" | "rarity") => void;
    setSortOrder: (order: "asc" | "desc") => void;

    // Actions
    clearFilters: () => void;

    // Computed
    activeFilterCount: number;
    hasActiveFilters: boolean;
}

export function useOperatorFilters(data: OperatorFromList[]): UseOperatorFiltersReturn {
    // Filter state
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
    const [sortBy, setSortBy] = useState<"name" | "rarity">("rarity");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");

    // Compute available filter options from data
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

    // Apply filters and sorting
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

        // Sorting
        result.sort((a, b) => {
            if (sortBy === "name") {
                return sortOrder === "asc" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name);
            }

            const aRarity = rarityToNumber(a.rarity);
            const bRarity = rarityToNumber(b.rarity);
            return sortOrder === "asc" ? aRarity - bRarity : bRarity - aRarity;
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
