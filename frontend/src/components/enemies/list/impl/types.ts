import type { IEnemy, IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";

export type ViewMode = "grid" | "list";
export type SortOption = "index" | "name" | "level" | "hp" | "atk" | "def" | "res" | "weight";
export type SortOrder = "asc" | "desc";
export type ApplyWay = "MELEE" | "RANGED" | "NONE";

export interface IEnemyFlatStats {
    maxHp: number;
    atk: number;
    def: number;
    res: number;
    aspd: number;
    ms: number;
    weight: number;
    baseAttackTime: number;
    hpRecoveryPerSec: number;
}

export interface IEnemyImmunities {
    stun: boolean;
    silence: boolean;
    sleep: boolean;
    frozen: boolean;
    levitate: boolean;
}

export interface IEnemyView extends IEnemy {
    stats: IEnemy["stats"];
    flatStats: IEnemyFlatStats;
    immunities: IEnemyImmunities;
    applyWay: ApplyWay | null;
    race: string | null;
}

export interface IEnemyStatMax {
    hp: number;
    atk: number;
    def: number;
}

export type IEnemyStatMaxByLevel = Record<IEnemyLevel, IEnemyStatMax>;

export interface IFilterState {
    q: string;
    levels: IEnemyLevel[];
    damageTypes: IEnemyDamageType[];
    attackTypes: ApplyWay[];
    races: string[];
    sortBy: SortOption;
    sortOrder: SortOrder;
}

export interface IUseEnemyFiltersReturn {
    filters: IFilterState;
    filteredEnemies: IEnemyView[];
    setSearchQuery: (q: string) => void;
    setLevels: (v: IEnemyLevel[]) => void;
    setDamageTypes: (v: IEnemyDamageType[]) => void;
    setAttackTypes: (v: ApplyWay[]) => void;
    setRaces: (v: string[]) => void;
    setSortBy: (v: SortOption) => void;
    setSortOrder: (v: SortOrder) => void;
    clearFilters: () => void;
    activeFilterCount: number;
    hasActiveFilters: boolean;
}
