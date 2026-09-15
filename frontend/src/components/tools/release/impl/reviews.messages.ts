import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The one label `reviews.ts` derives. The review's own name is game content -
 * the Chinese and English titles both live in that module as data - so only
 * the year heading is here.
 */
export const namespace = "tools";

export const messages = {
    "release.review.yearGroup": {
        text: "Year {gameYear} ({calendarYear})",
        description: "Heading grouping outfits by which year of the game they are from, e.g. 'Year 3 (2021)'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
