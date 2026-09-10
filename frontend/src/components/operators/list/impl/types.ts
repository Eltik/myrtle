import type { IOperatorIndexEntry, OperatorRarity, OperatorRarityTier } from "#/types/operators";

export type ViewMode = "grid" | "compact" | "list";
export type SortOption = "rarity" | "name" | "class" | "hp" | "atk" | "def" | "res" | "cost" | "block" | "ownership" | "e2";
export type SortOrder = "asc" | "desc";
export type HasNotesFilter = "any" | "yes" | "no";
export type AvailabilityFilter = "global" | "upcoming";

/** Which community number the card badge shows. Independent of `sortBy`, so
 *  sorting by E2 rate while displaying ownership is legal. */
export type StatMetric = "owned" | "e2";

/** Share of sharing players that own an operator, and how many of those owners
 *  promoted them to E2. */
export interface IOperatorOwnershipInfo {
    /** Owners as a fraction of the sharing population, in [0, 1]. `null` only
     *  when the population denominator is unknown. */
    pct: number | null;
    owners: number;
    e2Owners: number;
    /** E2 owners as a fraction of OWNERS, in [0, 1] - conversion, not
     *  popularity. This is the figure worth showing: measured across the 409
     *  operators with 50+ owners it has a standard deviation of 31.1 against
     *  15.5 for the share-of-population form, and it correlates -0.4256 with
     *  ownership, so it says something "Most owned" does not.
     *
     *  `null` when the operator has no owners, or when they cannot reach E2 at
     *  all - see `canE2`. */
    e2Pct: number | null;
    /** False for the 36 operators that cannot be promoted to E2 (one to three
     *  stars carry fewer than three phases). Their 0% is a fact about the game,
     *  not about the community, so the badge is suppressed and they are dropped
     *  from the E2 sort rather than ranked last. */
    canE2: boolean;
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

/** The array-valued filters every operator-list surface shares (the /operators page, the profile roster). Page-specific single selects (availability, notes, ownership) live beside these on each page's own state. */
export interface ISharedFilters {
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
}

export interface IFilterState extends ISharedFilters {
    searchQuery: string;
    hasNotes: HasNotesFilter;
    /** "upcoming" swaps the grid to CN operators not yet on Global. */
    availability: AvailabilityFilter;
    sortBy: SortOption;
    sortOrder: SortOrder;
}

export type ArrayFilterKey = keyof ISharedFilters;

export interface IFilterOptions {
    subclasses: string[];
    nations: string[];
    factions: string[];
    races: string[];
    birthPlaces: string[];
    artists: string[];
    voiceActors: string[];
}

/** The per-operator fields the shared predicate and option builder read. `IOperatorView` satisfies it directly; the roster builds one from an index entry plus the voices table. */
export interface IFilterSubject {
    profession: string;
    subProfessionId: string;
    rarity: OperatorRarity;
    gender: string;
    nationId: string;
    groupId: string | null;
    teamId: string | null;
    race: string;
    placeOfBirth: string;
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
