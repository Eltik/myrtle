import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.popup.loading": {
        text: "Loading the skin record",
        description: "Shown in the outfit dialog while the skin's data is still being fetched.",
    },
    "release.popup.missing": {
        text: "This skin is not in either client's skin table.",
        description: "Shown in the outfit dialog when neither the Chinese nor the English client lists the skin.",
    },
    "release.popup.close": {
        text: "Back to the planner",
        description: "Button that closes the outfit dialog and returns to the release planner.",
    },
    "release.popup.tileAlt": {
        text: "{operator}, {skin}",
        description: "Alt text of an outfit thumbnail: the operator's name then the outfit's. Both come from the game data.",
    },
    "release.popup.loadFailed": {
        text: "Couldn't load this skin.",
        description: "Shown in the outfit popup, beside a Try again button, when the outfit index failed to load. Distinct from the 'missing' line, which means the outfit is not in the index.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
