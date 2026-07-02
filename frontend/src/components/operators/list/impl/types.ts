import type { IOperatorIndexEntry, OperatorRarityTier } from "#/types/operators";

export type ViewMode = "grid" | "compact" | "list";
export type SortOption = "rarity" | "name" | "class" | "hp" | "atk" | "def" | "res" | "cost" | "block" | "ownership";
export type SortOrder = "asc" | "desc";
export type HasNotesFilter = "any" | "yes" | "no";
export type AvailabilityFilter = "global" | "upcoming";

/** Share of sharing players that own an operator. `pct` is a fraction in
 *  [0, 1]; `null` only when the population denominator is unknown. */
export interface IOperatorOwnershipInfo {
    pct: number | null;
    owners: number;
}

/**
 * View-model consumed by operator list UI. Built from the slim operators index
 * (which already carries flattened `gender`/`race`/`placeOfBirth`/`stats`) and
 * augmented with voice-actor data from `/api/static/voices`, per-user notes, and
 * population ownership.
 */
export interface IOperatorView extends IOperatorIndexEntry {
    voiceActors: string[];
    hasNotes: boolean;
    /** Population-level ownership, or `null` when ownership data is unavailable
     *  (still loading) or the operator has no recorded owners. */
    ownership: IOperatorOwnershipInfo | null;
}

/**
 * An `IOperatorView` augmented with the full-table-only fields the export dialog
 * offers. These are lazily merged in from `/static/operators` when the dialog
 * opens; they are `null` until (and unless) the full data has loaded.
 */
export interface IOperatorExportRow extends IOperatorView {
    displayNumber: string | null;
    description: string | null;
    itemUsage: string | null;
    itemDesc: string | null;
    itemObtainApproach: string | null;
    isSpChar: boolean | null;
    maxPotentialLevel: number | null;
    skin: string | null;
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
