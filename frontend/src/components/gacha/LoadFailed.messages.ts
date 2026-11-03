import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "loadFailed.retry": {
        text: "Try again",
        description: "Button beside a failure line on the gacha history or community page that reloads what failed to load.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
