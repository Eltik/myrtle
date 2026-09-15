import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.kpi.totalPulls": {
        text: "Total pulls",
        description: "Label over the player's total pull count. Rendered uppercase.",
    },
    "history.kpi.bannerTypes": {
        text: "across {count, plural, one {# banner type} other {# banner types}}",
        description: "Under the total pull count: how many of the four banner buckets the player has pulled on. Lowercase; it starts a fragment, not a sentence.",
    },
    "history.kpi.orundum": {
        text: "≈ {count} Orundum",
        description: "Under the total pull count: what those pulls would have cost. Orundum is the game's currency and keeps its name; keep the approximation sign.",
    },
    "history.kpi.orundum.title": {
        text: "{pulls} pulls x {rate} Orundum; free and discounted pulls make this an upper estimate",
        description: "Tooltip explaining the Orundum estimate. Orundum is the game's currency and keeps its name; the x is a plain multiplication.",
    },
    "history.kpi.sixStars": {
        text: "6★ operators",
        description: "Label over how many 6-star operators the player pulled. Rendered uppercase.",
    },
    "history.kpi.fiveStars": {
        text: "5★ operators",
        description: "Label over how many 5-star operators the player pulled. Rendered uppercase.",
    },
    "history.kpi.other": {
        text: "Other (4★ / 3★)",
        description: "Label over the count of everything below 5 stars. Rendered uppercase.",
    },
    "history.kpi.pullsUnit": {
        text: "pulls",
        description: "Unit printed small after a rarity's pull count.",
    },
    "history.kpi.rate": {
        text: "{value}% rate",
        description: "Under a rarity's count: that rarity's share of the player's pulls, already formatted as a number.",
    },
    "history.kpi.shareOfPulls": {
        text: "{value}% of pulls",
        description: "Under the 'Other' count: its share of the player's pulls, already formatted as a number.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
