import { useMemo, useState } from "react";
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

    // Apply filters and sorting
    const filteredOperators = useMemo(() => {
        let result = [...data];

        // Search filter
        if (searchQuery) {
            const query = searchQuery.toLowerCase();
            result = result.filter((op) => op.name.toLowerCase().includes(query) || op.profession.toLowerCase().includes(query) || op.subProfessionId.toLowerCase().includes(query));
        }

        // Class filter
        if (selectedClasses.length > 0) {
            result = result.filter((op) => selectedClasses.includes(op.profession));
        }

        // Subclass filter
        if (selectedSubclasses.length > 0) {
            result = result.filter((op) => selectedSubclasses.includes(op.subProfessionId.toLowerCase()));
        }

        // Rarity filter
        if (selectedRarities.length > 0) {
            result = result.filter((op) => selectedRarities.includes(rarityToNumber(op.rarity)));
        }

        // Birth place filter
        if (selectedBirthPlaces.length > 0) {
            result = result.filter((op) => op.profile?.basicInfo?.placeOfBirth && selectedBirthPlaces.includes(op.profile.basicInfo.placeOfBirth));
        }

        // Nation filter
        if (selectedNations.length > 0) {
            result = result.filter((op) => op.nationId && selectedNations.includes(op.nationId));
        }

        // Faction filter
        if (selectedFactions.length > 0) {
            result = result.filter((op) => (op.groupId && selectedFactions.includes(op.groupId)) || (op.teamId && selectedFactions.includes(op.teamId)));
        }

        // Gender filter
        if (selectedGenders.length > 0) {
            result = result.filter((op) => op.profile?.basicInfo?.gender && selectedGenders.includes(op.profile.basicInfo.gender));
        }

        // Race filter
        if (selectedRaces.length > 0) {
            result = result.filter((op) => op.profile?.basicInfo?.race && selectedRaces.includes(op.profile.basicInfo.race));
        }

        // Artists filter
        if (selectedArtists.length > 0) {
            result = result.filter((op) => op.artists?.some((artist) => selectedArtists.includes(artist)));
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
    }, [searchQuery, selectedClasses, selectedSubclasses, selectedRarities, selectedBirthPlaces, selectedNations, selectedFactions, selectedGenders, selectedRaces, selectedArtists, sortBy, sortOrder, data]);

    // Clear all filters
    const clearFilters = () => {
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
    };

    // Computed values
    const activeFilterCount = selectedClasses.length + selectedSubclasses.length + selectedRarities.length + selectedBirthPlaces.length + selectedNations.length + selectedFactions.length + selectedGenders.length + selectedRaces.length + selectedArtists.length + (searchQuery ? 1 : 0);

    const hasActiveFilters = activeFilterCount > 0;

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
