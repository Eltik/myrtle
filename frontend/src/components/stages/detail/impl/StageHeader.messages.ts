import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "header.boss": {
        text: "Boss",
        description: "Pill beside the stage code marking an operation that contains a boss enemy.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
