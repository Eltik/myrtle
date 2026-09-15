import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Labels for the leaderboard's option lists. Plain `.ts` constants, so
 * `constants.ts` carries message KEYS and whichever component renders an
 * option resolves it with `t()`.
 *
 * Server codes (EN, JP, CN, KR, TW) are deliberately absent: they are the
 * game's own publisher regions, which belong to the game-data layer.
 */
export const namespace = "user";

export const messages = {
    "leaderboard.sort.total": {
        text: "Total",
        description: "Sort option: the combined score across every category.",
    },
    "leaderboard.sort.operators": {
        text: "Operators",
        description: "Sort option: the roster subscore.",
    },
    "leaderboard.sort.stages": {
        text: "Stages",
        description: "Sort option: the stage-clear subscore.",
    },
    "leaderboard.sort.roguelike": {
        text: "Roguelike",
        description: "Sort option: the Integrated Strategies subscore.",
    },
    "leaderboard.sort.sandbox": {
        text: "Sandbox",
        description: "Sort option: the Reclamation Algorithm subscore.",
    },
    "leaderboard.sort.medals": {
        text: "Medals",
        description: "Sort option: the medal-collection subscore.",
    },
    "leaderboard.sort.base": {
        text: "Base",
        description: "Sort option: the base-building subscore.",
    },
    "leaderboard.sort.skins": {
        text: "Skins",
        description: "Sort option: the outfit-collection subscore.",
    },
    "leaderboard.interval.day.label": {
        text: "Today",
        description: "Movement-window option in the interval menu: rank changes over the last day.",
    },
    "leaderboard.interval.week.label": {
        text: "Past 7 days",
        description: "Movement-window option in the interval menu.",
    },
    "leaderboard.interval.month.label": {
        text: "Past 30 days",
        description: "Movement-window option in the interval menu.",
    },
    "leaderboard.interval.day.short": {
        text: "1d",
        description: "The one-day movement window, abbreviated on the interval button. Two characters at most.",
    },
    "leaderboard.interval.week.short": {
        text: "7d",
        description: "The seven-day movement window, abbreviated on the interval button. Three characters at most.",
    },
    "leaderboard.interval.month.short": {
        text: "30d",
        description: "The thirty-day movement window, abbreviated on the interval button. Three characters at most.",
    },
    "leaderboard.interval.day.subtitle": {
        text: "today",
        description: "The one-day movement window as it reads after 'Top movers · '. Lowercase on purpose.",
    },
    "leaderboard.interval.week.subtitle": {
        text: "7 days",
        description: "The seven-day movement window as it reads after 'Top movers · '. Lowercase on purpose.",
    },
    "leaderboard.interval.month.subtitle": {
        text: "30 days",
        description: "The thirty-day movement window as it reads after 'Top movers · '. Lowercase on purpose.",
    },
    "leaderboard.interval.day.since": {
        text: "since yesterday",
        description: "The one-day movement window as it reads at the end of a sentence, e.g. 'Climbed 3 ranks since yesterday'.",
    },
    "leaderboard.interval.week.since": {
        text: "in the past 7 days",
        description: "The seven-day movement window as it reads at the end of a sentence, e.g. 'Climbed 3 ranks in the past 7 days'.",
    },
    "leaderboard.interval.month.since": {
        text: "in the past 30 days",
        description: "The thirty-day movement window as it reads at the end of a sentence, e.g. 'Climbed 3 ranks in the past 30 days'.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a registry/constants entry and resolved
// by the consuming component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
