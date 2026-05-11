import type { IDefaultOgData } from "./templates/Default";

export const DEFAULT_OG_PRESETS = {
    _root: {
        title: "Operator data, rosters, and tier lists.",
    },
    home: {
        title: "Operator data, rosters, and tier lists.",
        subtitle: "Track your roster, scout community pulls, and explore every operator.",
        activeTag: "Home",
    },
    login: {
        title: "Log in to Myrtle",
        subtitle: "Sign in to track your roster, gacha history, and tier lists.",
        activeTag: "About",
    },
    settings: {
        title: "Settings",
        subtitle: "Manage your account, profile, and preferences.",
        activeTag: "About",
    },
    "gacha-community": {
        title: "Gacha · Community",
        subtitle: "Community-wide pull rates, top operators, and pull timing.",
        activeTag: "Tools",
    },
    "user-leaderboard": {
        title: "Doctor Leaderboard",
        subtitle: "Top Doctors ranked by score across servers.",
        activeTag: "Tools",
    },
    "user-search": {
        title: "Search Doctors",
        subtitle: "Find and browse user profiles.",
        activeTag: "Tools",
    },
    "tools-dps": {
        title: "DPS Calculator",
        subtitle: "Compare Arknights operator DPS.",
        activeTag: "Tools",
    },
    "tools-recruitment": {
        title: "Recruitment Calculator",
        subtitle: "Calculate recruitment combos.",
        activeTag: "Tools",
    },
    "tools-randomizer": {
        title: "Randomizer",
        subtitle: "Roll a random stage, squad, and modifier for your next run.",
        activeTag: "Tools",
    },
    operators: {
        title: "Operators",
        subtitle: "Every operator released in Arknights.",
        activeTag: "Operators",
    },
} as const satisfies Record<string, IDefaultOgData>;

export type DefaultOgPresetSlug = keyof typeof DEFAULT_OG_PRESETS;
