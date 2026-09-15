import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "browse.card.hot": {
        text: "Hot",
        description: "Corner ribbon on a tier-list card that is trending. Rendered uppercase in a very small ribbon.",
    },
    "browse.card.official": {
        text: "Official",
        description: "Corner badge marking a tier list maintained by the site's own team. Rendered uppercase.",
    },
    "browse.card.emptyDraft": {
        text: "Empty draft",
        description: "Stand-in on a card whose tier list has no operators placed yet.",
    },
    "browse.card.tier": {
        text: "Tier {name}",
        description: "Tooltip on a tier's pill in the card thumbnail. {name} is the tier's own label, written by the list's author.",
    },
    "browse.card.views": {
        text: "{count} views",
        description: "Tooltip on a card's view count.",
    },
    "browse.card.favorites": {
        text: "{count} favorites",
        description: "Tooltip on a card's favourite count.",
    },
    "browse.card.views24h": {
        text: "{count} views in the last 24h",
        description: "Tooltip on a trending card's recent view count.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
