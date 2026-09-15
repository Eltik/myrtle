import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.sheet.trigger": {
        text: "Filters",
        description: "Button that opens the filter panel on narrow screens.",
    },
    "birthdays.sheet.title": {
        text: "Filters",
        description: "Title of the filter panel on narrow screens.",
    },
    "birthdays.sheet.matchCount": {
        text: "{matched} of {total} match",
        description: "How many operators the current filters keep, out of all with a known birthday, e.g. '42 of 312 match'.",
    },
    "birthdays.sheet.reset": {
        text: "Reset",
        description: "Button that clears every filter.",
    },
    "birthdays.sheet.show": {
        text: "{count, plural, one {Show # result} other {Show # results}}",
        description: "Button that closes the filter panel and returns to the results.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
