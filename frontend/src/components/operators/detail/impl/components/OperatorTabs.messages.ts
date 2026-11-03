import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "tabs.aria": {
        text: "Operator sections",
        description: "Accessible name of the tab rail on an operator's page.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
