import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.kpi.sixStarRate": {
        text: "6★ rate · all-time",
        description: "Label over the share of pulls that produced a 6-star operator, across the whole community corpus. Rendered uppercase; keep the star and the middle dot.",
    },
    "community.kpi.sixStarRate.meta": {
        text: "{six} of {total} pulls",
        description: "Under the 6-star rate: how many 6-stars came out of how many pulls.",
    },
    "community.kpi.fiveStarRate": {
        text: "5★ rate · all-time",
        description: "Label over the share of pulls that produced a 5-star operator, across the whole community corpus. Rendered uppercase; keep the star and the middle dot.",
    },
    "community.kpi.fiveStarRate.meta": {
        text: "{count} pulls",
        description: "Under the 5-star rate: how many 5-stars the community pulled in total.",
    },
    "community.kpi.avgPullsSixStar": {
        text: "Avg pulls / 6★",
        description: "Label over the average number of pulls between 6-star operators. Rendered uppercase; 'Avg' is abbreviated because the tile is narrow.",
    },
    "community.kpi.avgPullsFiveStar.meta": {
        text: "avg pulls / 5★ · {value}",
        description: "Under the average pulls per 6-star: the same average for 5-stars. Lowercase; keep the middle dot.",
    },
    "community.kpi.totalPulls": {
        text: "Total pulls",
        description: "Label over the community's total pull count. Rendered uppercase.",
    },
    "community.kpi.totalPulls.meta": {
        text: "{count} contributing doctors",
        description: "Under the total pull count: how many players shared their records. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
