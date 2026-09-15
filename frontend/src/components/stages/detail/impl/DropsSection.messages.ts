import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "drops.title": {
        text: "Drops",
        description: "Kicker over the stage's drop cards; the count of drops sits beside it.",
    },
    "drops.rateTitle": {
        text: "Drop rate: {rate}",
        description: "Hover text on the 5-bar drop-rate meter. {rate} is one of the drop-rate tier labels ('Guaranteed', 'Common', ...).",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
