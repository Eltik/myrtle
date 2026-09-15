import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.upcoming.empty.title": {
        text: "No results",
        description: "Empty-state heading in the upcoming view when the filters keep nothing.",
    },
    "birthdays.upcoming.empty.desc": {
        text: "No operators match your filters. Try clearing rarity or class chips, or reset everything.",
        description: "Empty-state body in the upcoming view; 'chips' are the small filter buttons in the rail.",
    },
    "birthdays.upcoming.today": {
        text: "Today",
        description: "Kicker on an upcoming card whose date is today.",
    },
    "birthdays.upcoming.tomorrow": {
        text: "Tomorrow",
        description: "Kicker on an upcoming card whose date is tomorrow.",
    },
    "birthdays.upcoming.inDays": {
        text: "{count, plural, one {In # day} other {In # days}}",
        description: "Kicker on an upcoming card, counting whole days ahead. Today and tomorrow have their own wording, so this starts at two.",
    },
    "birthdays.upcoming.date": {
        text: "{month} {day}",
        description: "Date on an upcoming card, e.g. 'Nov 3'. {month} is an abbreviated month name from Intl.",
    },
    "birthdays.upcoming.rarity": {
        text: "{rarity}★",
        description: "Star rating at the end of an operator line, e.g. '4★'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
