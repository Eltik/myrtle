import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.overall.title": {
        text: "Overall Grade",
        description: "Card title over the account's composite letter grade.",
    },
    "score.overall.composite": {
        text: "composite",
        description: "Caption under the composite percentage beside the letter grade. Lowercase on purpose.",
    },
    "score.overall.calculated": {
        text: "Calculated · {date}",
        description: "When the grade was last recomputed. Keep the separator dot.",
    },
    "score.global": {
        text: "global",
        description: "Unit after a worldwide rank number, e.g. '#1,204 global'. Also used in the score-history tooltip.",
    },
    "score.standing.top": {
        text: "top {pct}%",
        description: "The account's percentile on the leaderboard, e.g. 'top 3%'. Lowercase on purpose: it sits in a row of small stats.",
    },
    "score.standing.thisWeek": {
        text: "this week",
        description: "Follows an up/down arrow and a number of ranks gained or lost, e.g. '▲12 this week'.",
    },
    "score.ladder.title": {
        text: "Grade ladder",
        description: "Label over the track showing every grade threshold and where the account sits.",
    },
    "score.ladder.top": {
        text: "Top of the ladder - nothing left to climb.",
        description: "Shown in place of the next-grade hint when the account already holds the highest grade.",
    },
    "score.next.points": {
        text: "{points} points",
        description: "How much composite score is still needed for the next grade; the grade name follows. Always plural: the value carries a decimal.",
    },
    "score.next.step": {
        text: "{points} to {grade}",
        description: "How far the account is from the next grade, e.g. '4.2 points to S'. {points} is the highlighted figure labelled by score.next.points and {grade} the highlighted grade name; both may move wherever the phrase needs them.",
    },
    "score.next.fastest": {
        text: "- fastest through {section}, worth {share}% of the grade with headroom left",
        description: "Hint about where to spend effort, appended to score.next.step. {section} is the bold section name and may move; a full stop is added after this. Keep the leading dash.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
