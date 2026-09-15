import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.table.empty": {
        text: "No Doctors match these filters.",
        description: "Shown in place of the leaderboard rows when the filters exclude everyone.",
    },
    "leaderboard.table.th.rank": {
        text: "Rank",
        description: "Column header over the global rank number.",
    },
    "leaderboard.table.th.doctor": {
        text: "Doctor",
        description: "Column header over the player's avatar, nickname and UID. 'Doctor' is what Arknights calls the player.",
    },
    "leaderboard.table.th.server": {
        text: "Server",
        description: "Column header over the game server a player is on.",
    },
    "leaderboard.table.th.grade": {
        text: "Grade",
        description: "Column header over the letter grade badge (S+, SS, A…).",
    },
    "leaderboard.table.th.level": {
        text: "Lv",
        description: "Column header over the account level, abbreviated: the column is 60px wide.",
    },
    "leaderboard.table.sortLabel": {
        text: "Change sort category",
        description: "Accessible name of the menu button that picks which score the leaderboard is ranked by.",
    },
    "leaderboard.table.sortHeading": {
        text: "Sort",
        description: "Small uppercase label beside the sort menu on narrow screens.",
    },
    "leaderboard.table.movement.none": {
        text: "No change in rank {interval}",
        description: "Tooltip on a flat rank indicator. {interval} is a phrase like 'since yesterday' or 'in the past 7 days'.",
    },
    "leaderboard.table.movement.up": {
        text: "Climbed {count, plural, one {# rank} other {# ranks}} {interval}",
        description: "Tooltip on an upward rank indicator. {interval} is a phrase like 'since yesterday' or 'in the past 7 days'.",
    },
    "leaderboard.table.movement.down": {
        text: "Fell {count, plural, one {# rank} other {# ranks}} {interval}",
        description: "Tooltip on a downward rank indicator. {interval} is a phrase like 'since yesterday' or 'in the past 7 days'.",
    },
    "leaderboard.table.viewProfile": {
        text: "View {nickname} profile",
        description: "Accessible name of a leaderboard row, which links to that player's profile.",
    },
    "leaderboard.table.levelInline": {
        text: "· Lv {level}",
        description: "Account level on the narrow-screen row, after the server tag. Keep the leading separator dot and the abbreviation.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
