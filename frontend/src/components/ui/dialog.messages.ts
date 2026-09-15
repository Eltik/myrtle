import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "dialog.close": {
        text: "Close",
        description: "Accessible name of the x button in a dialog's top corner. A verb: it dismisses the dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
