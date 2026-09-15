import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.stats.collection.title": {
        text: "Operator Collection",
        description: "Heading of the card counting how many operators the account owns.",
    },
    "profile.stats.collection.caption": {
        text: "operators collected",
        description: "Caption under the owned / available figure. Rendered uppercase by CSS.",
    },
    "profile.stats.collection.completion": {
        text: "Completion",
        description: "Label over the collection-completion bar.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
