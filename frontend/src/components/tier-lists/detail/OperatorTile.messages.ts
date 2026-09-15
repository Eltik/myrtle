import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.operatorTile.label": {
        text: "{name} ({rarity}★)",
        description: "Accessible name of an operator tile on the board: the operator's name, then their star rating. The name comes from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
