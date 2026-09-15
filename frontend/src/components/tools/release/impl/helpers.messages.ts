import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The date-range and relative-day wording `helpers.ts` produces. That module
 * has no React, so the helpers take an optional `t` and an optional locale and
 * fall back to the bundled source catalog and to English.
 */
export const namespace = "tools";

export const messages = {
    "release.dateRange": {
        text: "{start} to {end}",
        description: "A date range, e.g. 'Oct 26 to Nov 8, 2026'. Both halves are already formatted dates.",
    },
    "release.rel.today": {
        text: "today",
        description: "Relative day for the current date. Lowercase, because it follows a date on the same line.",
    },
    "release.rel.tomorrow": {
        text: "tomorrow",
        description: "Relative day for the next day. Lowercase, because it follows a date on the same line.",
    },
    "release.rel.yesterday": {
        text: "yesterday",
        description: "Relative day for the previous day. Lowercase, because it follows a date on the same line.",
    },
    "release.rel.inDays": {
        text: "{count, plural, one {in # day} other {in # days}}",
        description: "How far ahead a date is. Lowercase, because it follows a date on the same line. Today and tomorrow have their own wording, so this starts at two.",
    },
    "release.rel.daysAgo": {
        text: "{count, plural, one {# day ago} other {# days ago}}",
        description: "How far back a date is. Lowercase, because it follows a date on the same line. Yesterday has its own wording, so this starts at two.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
