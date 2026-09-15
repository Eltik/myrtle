import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.serverSplit.title": {
        text: "Server share · top 250",
        description: "Heading of the card breaking the top 250 players down by game server.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
