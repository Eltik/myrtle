import type { ToolIconName } from "#/lib/registry/tools";

export interface IPage {
    id: string;
    href: string;
    label: string;
    desc: string;
    icon: ToolIconName;
    /** Words a user might type that aren't in label/desc. */
    keywords: string[];
}

export const PAGES: IPage[] = [
    {
        id: "operators",
        href: "/operators",
        label: "Operators",
        desc: "Browse every operator · stats, skills, modules",
        icon: "pack",
        keywords: ["operator", "roster", "characters", "list", "browse"],
    },
    {
        id: "stages",
        href: "/stages",
        label: "Stages",
        desc: "Every stage, mapped with an enemy-pathing simulator",
        icon: "map",
        keywords: ["stage", "stages", "map", "level", "mission", "operation", "pathing", "simulator"],
    },
    {
        id: "tier-lists",
        href: "/tier-lists",
        label: "Tier Lists",
        desc: "Official and community tier lists for every operator",
        icon: "tiers",
        keywords: ["tier", "tierlist", "ranking", "meta", "rank", "community", "official"],
    },
    {
        id: "players-search",
        href: "/user/search",
        label: "Search Doctors",
        desc: "Find Doctor profiles by nickname or UID",
        icon: "search",
        keywords: ["players", "doctors", "users", "find", "profile", "nickname", "uid"],
    },
    {
        id: "players-leaderboard",
        href: "/user/leaderboard",
        label: "Leaderboard",
        desc: "Top Doctors ranked by score",
        icon: "trophy",
        keywords: ["leaderboard", "ranking", "top", "score", "ranks", "players"],
    },
    {
        id: "gacha-community",
        href: "/gacha/community",
        label: "Gacha Community",
        desc: "Pull rates, top operators, and timing across opted-in doctors",
        icon: "users",
        keywords: ["gacha", "community", "pulls", "rates", "stats"],
    },
    {
        id: "gacha-history",
        href: "/gacha/history",
        label: "Gacha History",
        desc: "Your synced pulls, rarity splits, and pity counters",
        icon: "history",
        keywords: ["gacha", "history", "pulls", "pity", "rolls", "tracker"],
    },
];
