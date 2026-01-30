import type { DamageType, EnemyLevel } from "~/types/api/impl/enemy";
import type { ImmunityType } from "./types";

// Enemy level types
export const ENEMY_LEVELS = ["NORMAL", "ELITE", "BOSS"] as const;

// Damage type options
export const DAMAGE_TYPES = ["PHYSIC", "MAGIC", "HEAL", "NO_DAMAGE"] as const;

// Immunity types
export const IMMUNITY_TYPES: ImmunityType[] = ["stun", "silence", "sleep", "frozen", "levitate"];

// Immunity display names
export const IMMUNITY_DISPLAY: Record<ImmunityType, string> = {
    stun: "Stun",
    silence: "Silence",
    sleep: "Sleep",
    frozen: "Freeze",
    levitate: "Levitate",
};

// Color type definitions
type LevelColorConfig = { bg: string; text: string; border: string };
type DamageTypeColorConfig = { bg: string; text: string };

// Enemy level display names
export const LEVEL_DISPLAY: Record<EnemyLevel, string> = {
    NORMAL: "Normal",
    ELITE: "Elite",
    BOSS: "Boss",
};

// Enemy level colors (for badges/indicators)
export const LEVEL_COLORS: Record<EnemyLevel, LevelColorConfig> = {
    NORMAL: {
        bg: "bg-secondary/50",
        text: "text-muted-foreground",
        border: "border-border",
    },
    ELITE: {
        bg: "bg-purple-500/15",
        text: "text-purple-400",
        border: "border-purple-500/30",
    },
    BOSS: {
        bg: "bg-amber-500/15",
        text: "text-amber-400",
        border: "border-amber-500/30",
    },
};

// Damage type display names
export const DAMAGE_TYPE_DISPLAY: Record<DamageType, string> = {
    PHYSIC: "Physical",
    MAGIC: "Arts",
    HEAL: "Heal",
    NO_DAMAGE: "No Damage",
};

// Damage type colors
export const DAMAGE_TYPE_COLORS: Record<DamageType, DamageTypeColorConfig> = {
    PHYSIC: {
        bg: "bg-orange-500/15",
        text: "text-orange-400",
    },
    MAGIC: {
        bg: "bg-sky-500/15",
        text: "text-sky-400",
    },
    HEAL: {
        bg: "bg-emerald-500/15",
        text: "text-emerald-400",
    },
    NO_DAMAGE: {
        bg: "bg-secondary/50",
        text: "text-muted-foreground",
    },
};

// Sort options
export const SORT_OPTIONS = [
    { value: "sortId", label: "Default" },
    { value: "name", label: "Name" },
    { value: "level", label: "Level" },
    { value: "hp", label: "HP" },
    { value: "atk", label: "ATK" },
    { value: "def", label: "DEF" },
    { value: "res", label: "RES" },
] as const;

// Pagination
export const ITEMS_PER_PAGE = 48;

// Hover delay for enemy cards
export const HOVER_DELAY = 500;

// Animation transitions
export const TOGGLE_TRANSITION = {
    type: "spring" as const,
    bounce: 0.15,
    duration: 0.3,
};

export const CONTAINER_TRANSITION = {
    duration: 0.2,
    ease: [0.4, 0, 0.2, 1] as const,
};
