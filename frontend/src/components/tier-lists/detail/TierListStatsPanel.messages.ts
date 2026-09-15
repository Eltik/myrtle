import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.stats.panelLabel": {
        text: "Tier list stats",
        description: "Accessible name of the sidebar holding the list's view and favourite counts.",
    },
    "detail.stats.title": {
        text: "Stats",
        description: "Heading of the stats sidebar.",
    },
    "detail.stats.trending": {
        text: "Trending now",
        description: "Badge in the stats sidebar marking a list with unusual recent traffic. Rendered uppercase.",
    },
    "detail.stats.empty": {
        text: "No stats yet.",
        description: "Shown in the stats sidebar when the list has no recorded traffic.",
    },
    "detail.stats.views": {
        text: "All-time views",
        description: "Row label for the list's total view count.",
    },
    "detail.stats.views.title": {
        text: "{count} total views",
        description: "Tooltip on the all-time views row, with the exact number.",
    },
    "detail.stats.uniqueViews": {
        text: "Unique viewers",
        description: "Row label for how many distinct visitors opened the list.",
    },
    "detail.stats.uniqueViews.title": {
        text: "{count} unique viewers",
        description: "Tooltip on the unique viewers row, with the exact number.",
    },
    "detail.stats.favorites": {
        text: "Favorites",
        description: "Row label for how many visitors saved the list.",
    },
    "detail.stats.favorites.title": {
        text: "{count} favorites",
        description: "Tooltip on the favorites row, with the exact number.",
    },
    "detail.stats.shares": {
        text: "Shares",
        description: "Row label for how many times the list's link was copied or shared.",
    },
    "detail.stats.shares.title": {
        text: "{count} shares",
        description: "Tooltip on the shares row, with the exact number.",
    },
    "detail.stats.views24h": {
        text: "Views · 24h",
        description: "Row label for views in the last day. Keep the middle dot; the row is narrow.",
    },
    "detail.stats.views24h.title": {
        text: "{count} views in the last 24h",
        description: "Tooltip on the 24-hour views row, with the exact number.",
    },
    "detail.stats.created": {
        text: "Created",
        description: "Footer label in the stats sidebar, before the date the list was made. Rendered uppercase.",
    },
    "detail.stats.updated": {
        text: "Updated",
        description: "Footer label in the stats sidebar, before the date the list was last edited. Rendered uppercase.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
