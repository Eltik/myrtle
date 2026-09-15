import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "chibi.error": {
        text: "Failed to load chibi data.",
        description: "Shown in the Chibi tab when the chibi index request fails. 'Chibi' is the game's small animated sprite.",
    },
    "chibi.none": {
        text: "No chibi available for this enemy.",
        description: "Shown in the Chibi tab when the game data ships no animated sprite for this enemy.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
