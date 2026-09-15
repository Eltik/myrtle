import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.movers.title": {
        text: "Top movers · {interval}",
        description: "Heading of the biggest-climbers card. {interval} is a short window like 'today' or '7 days'.",
    },
    "leaderboard.deltaRank": {
        text: "Δ Rank",
        description: "Label over a change in rank, in the movers card and on the standing card. The delta sign is a symbol and stays as-is.",
    },
    "leaderboard.movers.empty": {
        text: "No movements yet.",
        description: "Shown in the movers card when nobody's rank has changed inside the chosen window.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
