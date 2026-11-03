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
        description: "Screen-reader text on a flat rank indicator (the visible part is a dash and 0). {interval} is a phrase like 'since yesterday' or 'in the past 7 days'.",
    },
    "leaderboard.table.movement.up": {
        text: "Climbed {count, plural, one {# rank} other {# ranks}} {interval}",
        description: "Screen-reader text on an upward rank indicator (the visible part is an arrow and the number). {interval} is a phrase like 'since yesterday' or 'in the past 7 days'.",
    },
    "leaderboard.table.movement.down": {
        text: "Fell {count, plural, one {# rank} other {# ranks}} {interval}",
        description: "Screen-reader text on a downward rank indicator (the visible part is an arrow and the number). {interval} is a phrase like 'since yesterday' or 'in the past 7 days'.",
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
        description: "Screen-reader text after a row's quantity while ranked by an item, describing its bar. {pct} is the player's quantity as a percentage of the largest holding, 0 to 100.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
