import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.loadFailed.retry": {
        text: "Try again",
        description: "Button under the failure line on a profile tab (Stats, Lookback, Plans) that reloads what failed to load.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
