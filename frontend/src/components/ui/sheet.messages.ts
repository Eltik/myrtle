import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "sheet.close": {
        text: "Close",
        description: "Accessible name of the x button in a sheet's top corner. A verb: it dismisses the sheet.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
