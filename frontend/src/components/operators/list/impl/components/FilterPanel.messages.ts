import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "filters.panel.title": {
        text: "Filters",
        description: "Heading of the filter sidebar, and the title of the sheet it becomes on phones.",
    },
    "filters.panel.clearAll": {
        text: "Clear all",
        description: "Button in the filter panel header that resets every filter.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
