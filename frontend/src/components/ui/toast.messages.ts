import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "toast.close": {
        text: "Dismiss notification",
        description: "Accessible name of the x button on a pop-up notification (toast). It closes that one notification.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
