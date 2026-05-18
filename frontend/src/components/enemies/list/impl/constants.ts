import type { IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";
import type { ApplyWay, SortOption, ViewMode } from "./types";

export const VIEW_MODE_KEY = "enemies:view-mode";
export const ITEMS_PER_PAGE_KEY = "enemies:items-per-page";
export const FILTERS_KEY = "enemies:filters";

export const ITEMS_PER_PAGE = 48;
export const ITEMS_PER_PAGE_OPTIONS = [24, 48, 96, 144] as const;
export type ItemsPerPage = (typeof ITEMS_PER_PAGE_OPTIONS)[number];

export const VIEW_MODES: ReadonlySet<ViewMode> = new Set(["grid", "list"]);

export const SORT_OPTIONS: { value: SortOption; label: string }[] = [
    { value: "index", label: "Index" },
    { value: "name", label: "Name" },
    { value: "level", label: "Threat" },
    { value: "hp", label: "HP" },
    { value: "atk", label: "ATK" },
    { value: "def", label: "DEF" },
    { value: "res", label: "RES" },
    { value: "weight", label: "Weight" },
];

export const ENEMY_LEVELS: readonly IEnemyLevel[] = ["NORMAL", "ELITE", "BOSS"] as const;
export const DAMAGE_TYPES: readonly IEnemyDamageType[] = ["PHYSIC", "MAGIC", "HEAL", "NO_DAMAGE"] as const;
export const APPLY_WAYS: readonly ApplyWay[] = ["MELEE", "RANGED", "NONE"] as const;

export const ENEMY_LEVEL_DISPLAY: Record<IEnemyLevel, string> = {
    NORMAL: "Normal",
    ELITE: "Elite",
    BOSS: "Boss",
};

export const DAMAGE_TYPE_DISPLAY: Record<IEnemyDamageType, string> = {
    PHYSIC: "Physical",
    MAGIC: "Arts",
    HEAL: "Heal",
    NO_DAMAGE: "No Damage",
};

export const APPLY_WAY_DISPLAY: Record<ApplyWay, string> = {
    MELEE: "Melee",
    RANGED: "Ranged",
    NONE: "None",
};

export const LIST_GRID_COLS = "56px 1fr 90px 144px 110px";

export const LEVEL_ORDER: Record<IEnemyLevel, number> = {
    NORMAL: 0,
    ELITE: 1,
    BOSS: 2,
};
