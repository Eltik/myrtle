import { formatNationId, formatProfession, formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { OperatorRarityTier } from "#/types/operators";
import type { ArrayFilterKey, HasNotesFilter, SortOption, ViewMode } from "./types";

export { RARITY_HEX as RARITY_COLORS } from "#/lib/utils";

export const RARITIES: readonly OperatorRarityTier[] = ["TIER_6", "TIER_5", "TIER_4", "TIER_3", "TIER_2", "TIER_1"] as const;
export const CLASSES = ["WARRIOR", "SNIPER", "TANK", "MEDIC", "SUPPORT", "CASTER", "SPECIAL", "PIONEER"] as const;
export const GENDERS = ["Male", "Female", "Conviction"] as const;
export type Gender = (typeof GENDERS)[number];

export const HAS_NOTES_OPTIONS: { value: HasNotesFilter; label: string }[] = [
    { value: "any", label: "Any" },
    { value: "yes", label: "Has notes" },
    { value: "no", label: "No notes" },
];

export const HAS_NOTES_LABELS: Record<HasNotesFilter, string> = {
    any: "Any",
    yes: "Has notes",
    no: "No notes",
};

export const SORT_OPTIONS: { value: SortOption; label: string }[] = [
    { value: "rarity", label: "Rarity" },
    { value: "name", label: "Name" },
    { value: "class", label: "Class" },
    { value: "hp", label: "HP" },
    { value: "atk", label: "ATK" },
    { value: "def", label: "DEF" },
    { value: "res", label: "RES" },
    { value: "cost", label: "Cost" },
    { value: "block", label: "Block" },
];

export const CLASS_SORT_ORDER: Record<string, number> = {
    PIONEER: 0,
    WARRIOR: 1,
    TANK: 2,
    SNIPER: 3,
    CASTER: 4,
    SUPPORT: 5,
    MEDIC: 6,
    SPECIAL: 7,
};

export const PROFESSION_ORDER = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "SUPPORT", "MEDIC", "SPECIAL"] as const;

export const VIEW_MODE_KEY = "operators:view-mode";
export const FILTERS_VISIBLE_KEY = "operators:filters-visible";
export const ITEMS_PER_PAGE_KEY = "operators:items-per-page";
export const ITEMS_PER_PAGE = 30;
export const ITEMS_PER_PAGE_OPTIONS = [12, 24, 30, 48, 60, 100] as const;
export type ItemsPerPage = (typeof ITEMS_PER_PAGE_OPTIONS)[number];

export const LIST_GRID_COLS = "52px 1fr 96px 128px 160px 32px";

export const VIEW_MODES: ReadonlySet<ViewMode> = new Set(["grid", "compact", "list"]);

export const CHIP_CONFIG: { key: ArrayFilterKey; prefix: string; label: (v: string) => string }[] = [
    { key: "rarities", prefix: "rarity", label: (v) => `${rarityToNumber(v as OperatorRarityTier)}★` },
    { key: "classes", prefix: "class", label: formatProfession },
    { key: "subclasses", prefix: "sub", label: formatSubProfession },
    { key: "nations", prefix: "nation", label: formatNationId },
    { key: "factions", prefix: "fac", label: formatNationId },
    { key: "genders", prefix: "gen", label: (v) => v },
    { key: "races", prefix: "race", label: (v) => v },
    { key: "birthPlaces", prefix: "birth", label: (v) => v },
    { key: "artists", prefix: "artist", label: (v) => v },
    { key: "voiceActors", prefix: "va", label: (v) => v },
];

export const RARITY_BLUR_COLORS: Record<number, string> = {
    6: "#cc9b6a",
    5: "#d6c474",
    4: "#9e87c7",
    3: "#62a2bd",
    2: "#57ab72",
    1: "#aaaaaa",
};
