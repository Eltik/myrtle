// Muted rarity color mappings - more monochrome with subtle tints
export const RARITY_COLORS: Record<string, { bg: string; text: string; border: string }> = {
    TIER_1: { bg: "bg-neutral-100 dark:bg-neutral-800", text: "text-neutral-600 dark:text-neutral-300", border: "border-neutral-200 dark:border-neutral-700" },
    TIER_2: { bg: "bg-neutral-100 dark:bg-neutral-800", text: "text-neutral-600 dark:text-neutral-300", border: "border-neutral-200 dark:border-neutral-700" },
    TIER_3: { bg: "bg-neutral-100/80 dark:bg-neutral-800/80", text: "text-neutral-700 dark:text-neutral-200", border: "border-neutral-300 dark:border-neutral-600" },
    TIER_4: { bg: "bg-neutral-200/60 dark:bg-neutral-700/60", text: "text-neutral-700 dark:text-neutral-200", border: "border-neutral-300 dark:border-neutral-600" },
    TIER_5: { bg: "bg-neutral-200/80 dark:bg-neutral-700/80", text: "text-neutral-800 dark:text-neutral-100", border: "border-neutral-400 dark:border-neutral-500" },
    TIER_6: { bg: "bg-neutral-300/60 dark:bg-neutral-600/60", text: "text-neutral-900 dark:text-neutral-50", border: "border-neutral-400 dark:border-neutral-500" },
};

// Rarity glow effects for item icons (muted to match monochrome theme)
export const RARITY_GLOW: Record<string, string> = {
    TIER_1: "",
    TIER_2: "drop-shadow-[0_0_8px_rgba(134,239,172,0.15)]",
    TIER_3: "drop-shadow-[0_0_10px_rgba(125,211,252,0.2)]",
    TIER_4: "drop-shadow-[0_0_12px_rgba(201,184,240,0.25)]",
    TIER_5: "drop-shadow-[0_0_15px_rgba(255,230,109,0.3)]",
    TIER_6: "drop-shadow-[0_0_18px_rgba(255,154,74,0.35)]",
};

export const RARITY_LABELS: Record<string, string> = {
    TIER_1: "Tier 1",
    TIER_2: "Tier 2",
    TIER_3: "Tier 3",
    TIER_4: "Tier 4",
    TIER_5: "Tier 5",
    TIER_6: "Tier 6",
};

export const OCC_PER_LABELS: Record<string, string> = {
    USUAL: "Usual",
    ALMOST: "Almost",
    ALWAYS: "Always",
    SOMETIMES: "Sometimes",
    OFTEN: "Often",
};
