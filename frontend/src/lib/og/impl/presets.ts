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
        activeTag: "Players",
    },
    settings: {
        title: "Settings",
        subtitle: "Manage your account, profile, and preferences.",
        activeTag: "Players",
    },
    operators: {
        title: "Operators",
        subtitle: "Every operator released in Arknights.",
        activeTag: "Collection",
    },
    enemies: {
        title: "Enemy Database",
        subtitle: "Every enemy catalogued in Arknights.",
        activeTag: "Collection",
    },
    stats: {
        title: "Stats",
        subtitle: "A live count of every operator, tier list, and roster indexed on myrtle.moe.",
        activeTag: "Stats",
    },
    "user-search": {
        title: "Search Doctors",
        subtitle: "Find and browse user profiles.",
        activeTag: "Players",
    },
    "user-leaderboard": {
        title: "Doctor Leaderboard",
        subtitle: "Top Doctors ranked by score across servers.",
        activeTag: "Players",
    },
    "gacha-community": {
        title: "Gacha · Community",
        subtitle: "Community-wide pull rates, top operators, and pull timing.",
        activeTag: "Gacha",
    },
    "gacha-history": {
        title: "Gacha · My History",
        subtitle: "Your synced pulls, rarity splits, and pity counters.",
        activeTag: "Gacha",
    },
    "tools-dps": {
        title: "DPS Calculator",
        subtitle: "Compare Arknights operator DPS.",
        activeTag: "Tools",
    },
    "tools-hps": {
        title: "HPS Calculator",
        subtitle: "Compare Arknights healer HPS.",
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
    "tier-lists": {
        title: "Tier Lists",
        subtitle: "Browse official and community tier lists.",
        activeTag: "Home",
    },
    terms: {
        title: "Terms of Service",
        subtitle: "Legal agreement governing your use of Myrtle.",
        activeTag: "Home",
    },
    privacy: {
        title: "Privacy Policy",
        subtitle: "How we protect and handle your personal information.",
        activeTag: "Home",
    },
} as const satisfies Record<string, IDefaultOgData>;

export type DefaultOgPresetSlug = keyof typeof DEFAULT_OG_PRESETS;
