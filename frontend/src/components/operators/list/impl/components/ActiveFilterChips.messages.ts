import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "chips.active": {
        text: "Active:",
        description: "Label before the row of currently applied filter chips. Keep the colon.",
    },
    "chips.remove": {
        text: "Remove {label}",
        description: "Accessible name of the small x on one filter chip. {label} is the chip's own text, e.g. a class or nation name from the game data.",
    },
    "chips.clearAll": {
        text: "Clear all",
        description: "Link at the end of the chip row that removes every filter.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
