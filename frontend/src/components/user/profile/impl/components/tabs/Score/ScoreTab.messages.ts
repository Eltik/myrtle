import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.beta.tag": {
        text: "Beta",
        description: "Small uppercase tag marking the scoring feature as unfinished.",
    },
    "score.beta.note": {
        text: 'Scoring is subject to change and in the very early stages of development. Score is based off of potential completion, NOT how "meta" your account is.',
        description: "Notice above the score cards. 'Meta' is community slang for the strongest current picks, hence the quotation marks; the capitalised NOT is emphasis and should stay emphatic.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
