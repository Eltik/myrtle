import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.table.empty": {
        text: "No players match these filters.",
        description: "Shown in place of the leaderboard rows when the filters exclude everyone.",
    },
    "leaderboard.table.th.rank": {
        text: "Rank",
        description: "Column header over the global rank number.",
    },
    "leaderboard.table.th.doctor": {
        text: "Player",
        description: "Column header over the player's avatar, nickname and UID. 'Player' is the site's term; the game itself says 'Doctor'.",
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
    "leaderboard.table.empty.item": {
        text: "Nobody with a public profile holds this item yet.",
        description: "Shown in place of the table when the ranked item has no visible holders.",
    },
    "leaderboard.table.share": {
        text: "{pct}% of the top holding",
        description: "Tooltip on a row's quantity bar while ranked by an item. {pct} is the player's quantity as a percentage of the largest holding, 0 to 100.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
