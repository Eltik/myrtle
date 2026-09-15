import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "filters.dropdown.selected": {
        text: "{count, plural, one {# selected} other {# selected}}",
        description: "Placeholder shown in a multi-select once options are chosen, replacing the field's own prompt. English reads the same for one and many; other languages may not.",
    },
    "filters.dropdown.noMatches": {
        text: "No matches",
        description: "Shown inside an open multi-select when the typed text matches no option.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
