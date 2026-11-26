import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.history.title": {
        text: "Score history",
        description: "Card title over the chart of total score over time.",
    },
    "score.history.loading": {
        text: "Loading history…",
        description: "Placeholder while the score snapshots are being fetched. Keep the single-character ellipsis.",
    },
    "score.history.empty": {
        text: "History builds one point per leaderboard snapshot. Check back after the next one.",
        description: "Shown in place of the chart when there are fewer than two snapshots to plot.",
    },
    "score.history.trend": {
        text: "{delta} pts over {count, plural, one {# point} other {# points}}",
        description: "Chip beside the chart title summarising the change across the series; an up or down arrow precedes it. 'pts' is short for points of score, while {count} counts plotted data points.",
    },
    "score.history.loadFailed": {
        text: "Couldn't load the score history.",
        description: "Shown in the score history card, beside a Try again button, when the snapshot history failed to load. Distinct from the empty line, which means there are no snapshots yet.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
