import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.improvements.unavailable": {
        text: "Improvements unavailable. Profile may be private or not yet synced.",
        description: "Shown inside an expanded section card when the suggestions could not be loaded for this account.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
