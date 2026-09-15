import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "drawer.close": {
        text: "Close",
        description: "Accessible name of the x button in a drawer's top corner. A verb: it dismisses the drawer.",
    },
    "drawer.checked": {
        text: "Checked",
        description: "Accessible name of the checkmark drawn beside a ticked drawer menu checkbox item. A state, not a verb.",
    },
    "drawer.selected": {
        text: "Selected",
        description: "Accessible name of the checkmark drawn beside the chosen drawer menu radio item. A state, not a verb.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
