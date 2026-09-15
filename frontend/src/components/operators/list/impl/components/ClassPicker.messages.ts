import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "filters.class": {
        text: "Class",
        description: "Field label over the row of class icons in the filter panel. The class names themselves are game vocabulary and are not in this catalog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
