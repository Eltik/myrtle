import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "card.unknownOperator": {
        text: "Unknown",
        description: "Stand-in shown on a compact card when the game data carries no name for the operator.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
