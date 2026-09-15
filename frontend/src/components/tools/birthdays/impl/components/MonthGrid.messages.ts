import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.month.overflow": {
        text: "+{count}",
        description: "Chip standing in for the operators that did not fit in a month cell, e.g. '+4'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
