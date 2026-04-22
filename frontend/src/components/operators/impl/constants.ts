import { formatNationId, formatProfession, formatSubProfession } from "#/lib/utils";
import type { OperatorRarityTier } from "#/types/operators";
import type { ArrayFilterKey, SortOption, ViewMode } from "./types";

export const RARITIES: readonly OperatorRarityTier[] = ["TIER_6", "TIER_5", "TIER_4", "TIER_3", "TIER_2", "TIER_1"] as const;
export const CLASSES = ["WARRIOR", "SNIPER", "TANK", "MEDIC", "SUPPORT", "CASTER", "SPECIAL", "PIONEER"] as const;
export const GENDERS = ["Male", "Female", "Conviction"] as const;
export type Gender = (typeof GENDERS)[number];

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

export const VIEW_MODE_KEY = "operators:view-mode";
export const FILTERS_VISIBLE_KEY = "operators:filters-visible";
export const ITEMS_PER_PAGE = 30;

export const VIEW_MODES: ReadonlySet<ViewMode> = new Set(["grid", "compact", "list"]);

export const CHIP_CONFIG: { key: ArrayFilterKey; prefix: string; label: (v: string) => string }[] = [
    { key: "rarities", prefix: "rarity", label: (v) => `${v}★` },
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
