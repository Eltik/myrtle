import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "breakdown.cohortCount": {
        text: "{total} {cohort}",
        description: "Size of the group the bars are drawn from, e.g. '3,551 E2 owners'. {total} is a formatted count and {cohort} names the group.",
    },
    "breakdown.headline": {
        text: "{pct} have {summary}",
        description: "Headline above the level rows, e.g. '26% have mastered this skill'. {pct} is the bold share and {summary} the phrase the caller supplies, such as 'mastered this skill'; both may move wherever the sentence needs them.",
    },
    "breakdown.levelFallback": {
        text: "L{level}",
        description: "Stand-in row label when the caller supplied no name for a level. Very tight space.",
    },
    "breakdown.you": {
        text: "you",
        description: "Tiny uppercase marker on the row the signed-in reader is at.",
    },
    "breakdown.rowTooltip": {
        text: "{users} of {total} {cohort} ({pct}%){own, select, yes { - where you are} other {}}",
        description: "Hover text on one bar, e.g. '912 of 3,551 E2 owners (25.7%)'. {users} and {total} are formatted counts, {pct} is the share, and the trailing clause only appears on the reader's own row.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
