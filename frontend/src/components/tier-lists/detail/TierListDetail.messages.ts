import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.notFound.title": {
        text: "Tier list not found",
        description: "Heading shown when the requested tier list does not exist.",
    },
    "detail.notFound.body": {
        text: "It may have been removed or the link could be wrong.",
        description: "Paragraph under the not-found heading.",
    },
    "detail.notFound.action": {
        text: "Browse all lists",
        description: "Link from the not-found page back to the tier-list browse page.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
