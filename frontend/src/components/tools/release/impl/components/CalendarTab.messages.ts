import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Weekday names are NOT here: they come from `Intl`, which is what keeps them
 * right for a locale this catalog has never seen.
 */
export const namespace = "tools";

export const messages = {
    "release.calendar.prev": {
        text: "Previous month",
        description: "Accessible name of the back arrow above the calendar.",
    },
    "release.calendar.next": {
        text: "Next month",
        description: "Accessible name of the forward arrow above the calendar.",
    },
    "release.calendar.today": {
        text: "Today",
        description: "Button that jumps the calendar back to the current month.",
    },
    "release.calendar.empty.title": {
        text: "Nothing dated",
        description: "Empty-state heading when no row has an English date at all.",
    },
    "release.calendar.empty.desc": {
        text: "The backend returned no rows with an EN date.",
        description: "Empty-state body when no row has an English date. 'EN' is the English game server.",
    },
    "release.calendar.pillTitle": {
        text: "{name}\n{dates}",
        description: "Tooltip on a calendar pill: the row's name, then its dates on a second line. Keep the line break.",
    },
    "release.calendar.more": {
        text: "+{count} more",
        description: "Link standing in for the rows that did not fit in a calendar week, e.g. '+3 more'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
