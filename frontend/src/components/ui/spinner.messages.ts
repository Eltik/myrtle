import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "spinner.loading": {
        text: "Loading",
        description: "Accessible name of the spinning busy indicator, announced while something is still being fetched. No trailing ellipsis.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
