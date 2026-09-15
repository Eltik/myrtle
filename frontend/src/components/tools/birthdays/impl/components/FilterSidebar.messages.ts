import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.sidebar.title": {
        text: "Filters",
        description: "Heading of the filter rail shown on wide screens.",
    },
    "birthdays.sidebar.count": {
        text: "({matched}/{total})",
        description: "How many operators the current filters keep, out of all with a known birthday, e.g. '(42/312)'. Keep the parentheses.",
    },
    "birthdays.sidebar.matched": {
        text: "matched",
        description: "Caption under the big number of kept operators. Lowercase; it is set in small caps.",
    },
    "birthdays.sidebar.reset": {
        text: "Reset filters",
        description: "Link that clears every filter.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
