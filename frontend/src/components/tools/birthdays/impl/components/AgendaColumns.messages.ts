import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.agenda.empty.title": {
        text: "No birthdays",
        description: "Empty-state heading in the single-day view when nobody in the current filter has a birthday that day.",
    },
    "birthdays.agenda.empty.desc": {
        text: "No operators in your current filter celebrate on this day.",
        description: "Empty-state body in the single-day view.",
    },
    "birthdays.agenda.overflow": {
        text: "+{count} more",
        description: "Line standing in for the operators that did not fit in a day column, e.g. '+3 more'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
