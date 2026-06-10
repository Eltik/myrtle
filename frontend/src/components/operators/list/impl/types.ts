import type { IOperatorListItem, OperatorRarityTier } from "#/types/operators";

export type ViewMode = "grid" | "compact" | "list";
export type SortOption = "rarity" | "name" | "class" | "hp" | "atk" | "def" | "res" | "cost" | "block";
export type SortOrder = "asc" | "desc";
export type HasNotesFilter = "any" | "yes" | "no";
export type AvailabilityFilter = "global" | "upcoming";

export interface IOperatorStats {
    hp: number;
    atk: number;
    def: number;
    res: number;
    cost: number;
    block: number;
}

/**
 * View-model consumed by operator list UI. Flattens the nested backend shape
 * (`profile.basicInfo.*`, last-phase attribute keyframe, `voiceLangDict`) into
 * top-level fields and merges voice-actor data from `/api/static/voices`.
 */
export interface IOperatorView extends IOperatorListItem {
    gender: string | null;
    race: string | null;
    placeOfBirth: string | null;
    voiceActors: string[];
    stats: IOperatorStats | null;
    hasNotes: boolean;
}

export interface IFilterState {
    searchQuery: string;
    classes: string[];
    subclasses: string[];
    rarities: OperatorRarityTier[];
    genders: string[];
    nations: string[];
    factions: string[];
    races: string[];
    birthPlaces: string[];
    artists: string[];
    voiceActors: string[];
    hasNotes: HasNotesFilter;
    /** "upcoming" swaps the grid to CN operators not yet on Global. */
    availability: AvailabilityFilter;
    sortBy: SortOption;
    sortOrder: SortOrder;
}

export type ArrayFilterKey = "classes" | "subclasses" | "rarities" | "genders" | "nations" | "factions" | "races" | "birthPlaces" | "artists" | "voiceActors";

export interface IFilterOptions {
    subclasses: string[];
    nations: string[];
    factions: string[];
    races: string[];
    birthPlaces: string[];
    artists: string[];
    voiceActors: string[];
}

export interface IUseOperatorFiltersReturn {
    filters: IFilterState;
    filterOptions: IFilterOptions;
    filteredOperators: IOperatorView[];
    setSearchQuery: (q: string) => void;
    setClasses: (v: string[]) => void;
    setSubclasses: (v: string[]) => void;
    setRarities: (v: OperatorRarityTier[]) => void;
    setGenders: (v: string[]) => void;
    setNations: (v: string[]) => void;
    setFactions: (v: string[]) => void;
    setRaces: (v: string[]) => void;
    setBirthPlaces: (v: string[]) => void;
    setArtists: (v: string[]) => void;
    setVoiceActors: (v: string[]) => void;
    setHasNotes: (v: HasNotesFilter) => void;
    setAvailability: (v: AvailabilityFilter) => void;
    setSortBy: (v: SortOption) => void;
    setSortOrder: (v: SortOrder) => void;
    removeFrom: (key: ArrayFilterKey, value: string) => void;
    clearFilters: () => void;
    activeFilterCount: number;
    hasActiveFilters: boolean;
}
