import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.roster.unowned.badge": {
        text: "Not owned",
        description: "Badge across an operator the account does not have.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
