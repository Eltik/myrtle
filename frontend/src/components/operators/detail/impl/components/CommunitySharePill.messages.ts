import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "share.tooltip": {
        text: "{users} of {total} {cohort}{top, select, yes { - the most common choice} other {}}",
        description: "Hover text on the community share pill, e.g. '1,204 of 3,551 E2 owners use this skill - the most common choice'. {users} and {total} are formatted counts, {cohort} names the group in the reader's terms, and the trailing clause only appears for the most-picked option.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
