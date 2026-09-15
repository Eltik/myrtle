import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "card.upcoming.portraitAlt": {
        text: "{name} portrait",
        description: "Alt text of the art on a not-yet-released operator's card. {name} is the operator's name from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
