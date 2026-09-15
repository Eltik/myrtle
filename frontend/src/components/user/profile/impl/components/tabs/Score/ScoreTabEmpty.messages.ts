import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.empty.title": {
        text: "No score yet",
        description: "Empty-state title on the Score tab when this account has never been graded.",
    },
    "score.empty.desc": {
        text: "This Doctor's grade hasn't been calculated. Scores are computed periodically once enough data is on file.",
        description: "Empty-state body on the Score tab. 'Doctor' is what Arknights calls the player; keep the apostrophes.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
