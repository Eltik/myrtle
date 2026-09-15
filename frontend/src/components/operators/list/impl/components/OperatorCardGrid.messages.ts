import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "card.portraitAlt": {
        text: "{name} portrait",
        description: "Alt text of the full-body art on an operator card. {name} is the operator's name from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
