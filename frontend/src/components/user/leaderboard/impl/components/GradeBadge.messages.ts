import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.grade.tooltip": {
        text: "Grade {grade}: composite score {from}-{to}%, weighted across operators, base, Integrated Strategies, medals, stages and Reclamation Algorithm.",
        description: "Hover explanation on a grade badge. {grade} is the letter (S+, S, A, B, C, D, F); {from} and {to} are the percentage bounds of that band, already formatted. 'Integrated Strategies' and 'Reclamation Algorithm' are in-game modes.",
    },
    "leaderboard.grade.unknown": {
        text: "Not graded yet - this account has not been scored.",
        description: "Hover explanation on the placeholder shown in place of a grade badge when a user has no grade.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
