import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "cardList.aria": {
        text: "{name} ({index})",
        description: "Accessible name of an enemy row in the list view. {name} is the enemy's name and {index} its in-game handbook callsign, both game data; only the punctuation around them is yours.",
    },
    "cardList.portraitAlt": {
        text: "{name} portrait",
        description: "Alt text of an enemy's portrait image on a list row. {name} is the enemy's name from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
