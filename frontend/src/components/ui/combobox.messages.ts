import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "combobox.removeChip": {
        text: "Remove",
        description: "Accessible name of the small x on a selected-value chip in a multi-select combobox. A verb: it deselects that one value.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
