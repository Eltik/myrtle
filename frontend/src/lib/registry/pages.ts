import type { messages as pageMessages } from "#/lib/registry/pages.messages";
import type { ToolIconName } from "#/lib/registry/tools";

/** A key in `pages.messages.ts`; resolved by the component that renders the entry. */
export type PageMessageKey = keyof typeof pageMessages & string;

export interface IPage {
    id: string;
    href: string;
    labelKey: PageMessageKey;
    descKey: PageMessageKey;
    icon: ToolIconName;
    /** Words a user might type that aren't in label/desc. */
    keywords: string[];
}

export const PAGES: IPage[] = [
    {
        id: "operators",
        href: "/operators",
        labelKey: "page.operators.label",
        descKey: "page.operators.desc",
        icon: "pack",
        keywords: ["operator", "roster", "characters", "list", "browse"],
    },
    {
        id: "stages",
        href: "/stages",
        labelKey: "page.stages.label",
        descKey: "page.stages.desc",
        icon: "map",
        keywords: ["stage", "stages", "map", "level", "mission", "operation", "pathing", "simulator"],
    },
    {
        id: "tier-lists",
        href: "/tier-lists",
        labelKey: "page.tierLists.label",
        descKey: "page.tierLists.desc",
        icon: "tiers",
        keywords: ["tier", "tierlist", "ranking", "meta", "rank", "community", "official"],
    },
    {
        id: "players-search",
        href: "/user/search",
        labelKey: "page.playersSearch.label",
        descKey: "page.playersSearch.desc",
        icon: "search",
        keywords: ["players", "doctors", "users", "find", "profile", "nickname", "uid"],
    },
    {
        id: "players-leaderboard",
        href: "/user/leaderboard",
        labelKey: "page.playersLeaderboard.label",
        descKey: "page.playersLeaderboard.desc",
        icon: "trophy",
        keywords: ["leaderboard", "ranking", "top", "score", "ranks", "players"],
    },
    {
        id: "gacha-community",
        href: "/gacha/community",
        labelKey: "page.gachaCommunity.label",
        descKey: "page.gachaCommunity.desc",
        icon: "users",
        keywords: ["gacha", "community", "pulls", "rates", "stats"],
    },
    {
        id: "gacha-history",
        href: "/gacha/history",
        labelKey: "page.gachaHistory.label",
        descKey: "page.gachaHistory.desc",
        icon: "history",
        keywords: ["gacha", "history", "pulls", "pity", "rolls", "tracker"],
    },
];
