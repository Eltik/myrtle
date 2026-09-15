import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "placeholder.title": {
        text: "Enemy placeholder",
        description: "Accessible name (SVG <title>) of the stand-in drawing shown when an enemy has no portrait. Never seen, only announced.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
