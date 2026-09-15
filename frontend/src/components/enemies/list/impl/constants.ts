import type { IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";
import type { messages as listConstantsMessages } from "./constants.messages";
import type { ApplyWay, SortOption, ViewMode } from "./types";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type EnemyListMessageKey = keyof typeof listConstantsMessages & string;

export const VIEW_MODE_KEY = "enemies:view-mode";
export const ITEMS_PER_PAGE_KEY = "enemies:items-per-page";
export const PAGE_KEY = "enemies:page";
export const FILTERS_KEY = "enemies:filters";

export const ITEMS_PER_PAGE = 48;
/** `"all"` collapses the list to a single page; it resolves to the filtered row count at render time. */
export const ITEMS_PER_PAGE_OPTIONS = [24, 48, 96, 144, "all"] as const;
export type ItemsPerPage = (typeof ITEMS_PER_PAGE_OPTIONS)[number];

export const VIEW_MODES: ReadonlySet<ViewMode> = new Set(["grid", "list"]);

export const SORT_OPTIONS: { value: SortOption; labelKey: EnemyListMessageKey }[] = [
    { value: "index", labelKey: "sort.index" },
    { value: "name", labelKey: "sort.name" },
    { value: "level", labelKey: "sort.level" },
    { value: "hp", labelKey: "sort.hp" },
    { value: "atk", labelKey: "sort.atk" },
    { value: "def", labelKey: "sort.def" },
    { value: "res", labelKey: "sort.res" },
    { value: "weight", labelKey: "sort.weight" },
];

export const ENEMY_LEVELS: readonly IEnemyLevel[] = ["NORMAL", "ELITE", "BOSS"] as const;
export const DAMAGE_TYPES: readonly IEnemyDamageType[] = ["PHYSIC", "MAGIC", "HEAL", "NO_DAMAGE"] as const;
export const APPLY_WAYS: readonly ApplyWay[] = ["MELEE", "RANGED", "NONE"] as const;

export const ENEMY_LEVEL_LABEL_KEY: Record<IEnemyLevel, EnemyListMessageKey> = {
    NORMAL: "level.NORMAL",
    ELITE: "level.ELITE",
    BOSS: "level.BOSS",
};

export const DAMAGE_TYPE_LABEL_KEY: Record<IEnemyDamageType, EnemyListMessageKey> = {
    PHYSIC: "damageType.PHYSIC",
    MAGIC: "damageType.MAGIC",
    HEAL: "damageType.HEAL",
    NO_DAMAGE: "damageType.NO_DAMAGE",
};

export const APPLY_WAY_LABEL_KEY: Record<ApplyWay, EnemyListMessageKey> = {
    MELEE: "applyWay.MELEE",
    RANGED: "applyWay.RANGED",
    NONE: "applyWay.NONE",
};

export const LIST_GRID_COLS = "56px 1fr 90px 144px 110px";

export const LEVEL_ORDER: Record<IEnemyLevel, number> = {
    NORMAL: 0,
    ELITE: 1,
    BOSS: 2,
};
