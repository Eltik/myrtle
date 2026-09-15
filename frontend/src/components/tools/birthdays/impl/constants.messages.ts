import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The calendar zoom levels in `constants.ts` are plain data in a module with
 * no React, so they carry message KEYS and `CalendarToolbar` resolves them
 * with `t()`.
 *
 * Month and weekday names are NOT here: they come from `Intl` through the
 * helpers in `helpers.ts`, which is what keeps them correct for a locale this
 * catalog has never seen. Class and nation labels are game vocabulary.
 */
export const namespace = "tools";

export const messages = {
    "birthdays.scale.day": {
        text: "Day",
        description: "Calendar zoom level showing a single day. Tab label, so very short.",
    },
    "birthdays.scale.3day": {
        text: "3 Day",
        description: "Calendar zoom level showing three days side by side. Tab label, so very short.",
    },
    "birthdays.scale.week": {
        text: "Week",
        description: "Calendar zoom level showing one week. Tab label, so very short.",
    },
    "birthdays.scale.month": {
        text: "Month",
        description: "Calendar zoom level showing a whole month as a grid. Tab label, so very short.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a constants entry and resolved by the
// consuming component as `t(item.labelKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
