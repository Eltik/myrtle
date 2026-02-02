import type { ApplyWay, DamageType, EnemyLevel } from "~/types/api";

// Sort options with display names
export const SORT_OPTIONS = [
    { value: "name", label: "Name" },
    { value: "level", label: "Level" },
    { value: "index", label: "Index" },
    { value: "hp", label: "HP" },
    { value: "atk", label: "ATK" },
    { value: "def", label: "DEF" },
    { value: "res", label: "RES" },
    { value: "move_speed", label: "Movement Speed" },
    { value: "aspd", label: "ASPD" },
    { value: "weight", label: "Weight" },
] as const;

// Pagination
export const ITEMS_PER_PAGE = 48;

// Hover delay for enemy cards
export const HOVER_DELAY = 500;

// Animation transitions
export const TOGGLE_TRANSITION = {
    duration: 0.2,
    ease: [0.4, 0, 0.2, 1] as const,
};

export const CONTAINER_TRANSITION = {
    duration: 0.2,
    ease: [0.4, 0, 0.2, 1] as const,
};

// Display name mappings for enemy levels
export const ENEMY_LEVEL_DISPLAY: Record<EnemyLevel, string> = {
    NORMAL: "Normal",
    ELITE: "Elite",
    BOSS: "Boss",
};

// Color schemes for enemy level buttons
export const ENEMY_LEVEL_COLORS: Record<EnemyLevel, { selected: string; unselected: string }> = {
    NORMAL: {
        selected: "border-zinc-500 bg-zinc-500/20 text-zinc-300",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-zinc-500/50 hover:text-zinc-400",
    },
    ELITE: {
        selected: "border-amber-500 bg-amber-500/20 text-amber-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-amber-500/50 hover:text-amber-400",
    },
    BOSS: {
        selected: "border-red-500 bg-red-500/20 text-red-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-red-500/50 hover:text-red-400",
    },
};

// Hex color mappings for enemy levels (for inline styles)
export const LEVEL_BAR_COLORS: Record<EnemyLevel, string> = {
    NORMAL: "#71717a", // zinc-500
    ELITE: "#f59e0b", // amber-500
    BOSS: "#ef4444", // red-500
};

// Blur color mappings for enemy levels
export const LEVEL_BLUR_COLORS: Record<EnemyLevel, string> = {
    NORMAL: "#a1a1aa", // zinc-400
    ELITE: "#fbbf24", // amber-400
    BOSS: "#f87171", // red-400
};

// Display name mappings for damage types
export const DAMAGE_TYPE_DISPLAY: Record<DamageType, string> = {
    PHYSIC: "Physical",
    MAGIC: "Arts",
    NO_DAMAGE: "No Damage",
    HEAL: "Heal",
};

// Color schemes for damage type buttons
export const DAMAGE_TYPE_COLORS: Record<DamageType, { selected: string; unselected: string }> = {
    PHYSIC: {
        selected: "border-orange-500 bg-orange-500/20 text-orange-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-orange-500/50 hover:text-orange-400",
    },
    MAGIC: {
        selected: "border-blue-500 bg-blue-500/20 text-blue-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-blue-500/50 hover:text-blue-400",
    },
    NO_DAMAGE: {
        selected: "border-gray-500 bg-gray-500/20 text-gray-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-gray-500/50 hover:text-gray-400",
    },
    HEAL: {
        selected: "border-green-500 bg-green-500/20 text-green-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-green-500/50 hover:text-green-400",
    },
};

// Display name mappings for attack/apply way
export const APPLY_WAY_DISPLAY: Record<NonNullable<ApplyWay>, string> = {
    MELEE: "Melee",
    RANGED: "Ranged",
    NONE: "None",
};

// Color schemes for attack type buttons
export const APPLY_WAY_COLORS: Record<NonNullable<ApplyWay>, { selected: string; unselected: string }> = {
    MELEE: {
        selected: "border-rose-500 bg-rose-500/20 text-rose-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-rose-500/50 hover:text-rose-400",
    },
    RANGED: {
        selected: "border-cyan-500 bg-cyan-500/20 text-cyan-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-cyan-500/50 hover:text-cyan-400",
    },
    NONE: {
        selected: "border-gray-500 bg-gray-500/20 text-gray-400",
        unselected: "border-border bg-secondary/50 text-muted-foreground hover:border-gray-500/50 hover:text-gray-400",
    },
};
