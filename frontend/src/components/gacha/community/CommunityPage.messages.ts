import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.error.load": {
        text: "Couldn’t load community stats.",
        description: "Bold lead-in of the error banner on the community gacha page; the server's own message follows it on the same line.",
    },
    "community.error.unknown": {
        text: "Unknown error.",
        description: "Stand-in for the server's error message when the failed request carried none.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
