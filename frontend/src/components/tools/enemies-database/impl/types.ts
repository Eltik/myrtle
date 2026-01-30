export type EnemySortOption = "sortId" | "name" | "level" | "hp" | "atk" | "def" | "res";
export type SortOrder = "asc" | "desc";

export type ImmunityType = "stun" | "silence" | "sleep" | "frozen" | "levitate";

export interface StatRange {
    min: number | null;
    max: number | null;
}

export interface EnemyFilterState {
    searchQuery: string;
    selectedLevels: string[];
    selectedDamageTypes: string[];
    selectedRaces: string[];
    // Advanced filters
    selectedImmunities: ImmunityType[];
    statFilters: {
        hp: StatRange;
        atk: StatRange;
        def: StatRange;
        res: StatRange;
    };
}

export interface RaceOption {
    id: string;
    name: string;
}
