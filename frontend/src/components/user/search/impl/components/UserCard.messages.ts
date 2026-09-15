import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "search.card.level": {
        text: "Lv {level}",
        description: "Account level on a search-result card. Abbreviated on purpose; the tile is narrow.",
    },
    "search.card.points": {
        text: "pts",
        description: "Unit after the total score on a search-result card, short for points. The number is rendered just before it.",
    },
    "search.card.operators": {
        text: "ops",
        description: "Unit after the owned-operator count on a search-result card, short for operators. The number is rendered just before it.",
    },
    "search.card.skins": {
        text: "skins",
        description: "Unit after the owned-skin count on a search-result card. The number is rendered just before it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
