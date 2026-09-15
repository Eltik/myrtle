import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The calendar range title `formatRangeTitle` assembles. Month names come from
 * `Intl`; these messages only decide the order of the pieces and the
 * punctuation between them, which is exactly the part that differs by locale.
 */
export const namespace = "tools";

export const messages = {
    "birthdays.range.month": {
        text: "{month} {year}",
        description: "Calendar header for a whole month, e.g. 'October 2026'. {month} is a month name from Intl.",
    },
    "birthdays.range.day": {
        text: "{month} {day}, {year}",
        description: "Condensed calendar header for a single day, e.g. 'Oct 26, 2026'. {month} is an abbreviated month name from Intl.",
    },
    "birthdays.range.crossYear": {
        text: "{startMonth} {startDay}, {startYear} - {endMonth} {endDay}, {endYear}",
        description: "Calendar header for a range that crosses new year, e.g. 'Dec 28, 2026 - Jan 3, 2027'. The dash is a plain hyphen.",
    },
    "birthdays.range.crossMonth": {
        text: "{startMonth} {startDay} - {endMonth} {endDay}, {year}",
        description: "Calendar header for a range that crosses a month boundary, e.g. 'Oct 26 - Nov 1, 2026'. The dash is a plain hyphen.",
    },
    "birthdays.range.sameMonth": {
        text: "{month} {startDay} - {endDay}, {year}",
        description: "Calendar header for a range inside one month, e.g. 'October 4 - 10, 2026'. The dash is a plain hyphen.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
