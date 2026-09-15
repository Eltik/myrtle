import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.day.title": {
        text: "{month} {day}",
        description: "Dialog title naming the clicked calendar day, e.g. 'October 26'. {month} is a month name from Intl.",
    },
    "birthdays.day.meta": {
        text: "{weekday} · {count, plural, one {# operator} other {# operators}}",
        description: "Dialog subtitle: which weekday it is and how many operators celebrate. {weekday} comes from Intl.",
    },
    "birthdays.day.metaToday": {
        text: "{weekday} · today · {count, plural, one {# operator} other {# operators}}",
        description: "The same subtitle when the clicked day is today. 'today' is lowercase because the line is set in small caps.",
    },
    "birthdays.day.empty.title": {
        text: "No birthdays",
        description: "Empty-state heading in the day dialog when nobody in the current filter celebrates that day.",
    },
    "birthdays.day.empty.desc": {
        text: "No operators in your current filter celebrate on this day.",
        description: "Empty-state body in the day dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
