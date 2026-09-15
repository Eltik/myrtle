import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.morale.sparkline": {
        text: "Morale across the simulated week",
        description: "Accessible title of the little morale line drawn for one operator.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
