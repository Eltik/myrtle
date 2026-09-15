import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.row.classNation": {
        text: "{class} · {nation}",
        description: "Second line of a detailed operator row: class then nation. Both halves are game vocabulary and come from the game data; only the middle dot is ours.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
