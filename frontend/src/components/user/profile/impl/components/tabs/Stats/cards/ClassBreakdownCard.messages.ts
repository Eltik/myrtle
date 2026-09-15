import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.stats.classes.title": {
        text: "Class Breakdown",
        description: "Heading of the card breaking the roster down by operator class. The class and archetype names themselves come from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
