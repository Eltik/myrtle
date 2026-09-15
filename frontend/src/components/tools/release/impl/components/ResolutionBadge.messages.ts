import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.badge.independent": {
        text: "EN schedules separately",
        description: "Badge on a row whose EN date cannot be projected from the CN date. 'EN' is the English game server and stays as-is.",
    },
    "release.badge.independent.title": {
        text: "EN schedules this banner kind on its own calendar; the CN date says nothing about the EN date.",
        description: "Tooltip explaining the 'EN schedules separately' badge. 'CN' and 'EN' are the Chinese and English game servers.",
    },
    "release.badge.noEstimate": {
        text: "No estimate",
        description: "Badge on a row this tool cannot model at all.",
    },
    "release.badge.confirmed": {
        text: "Confirmed",
        description: "Badge on a row whose EN date is in the English game data already.",
    },
    "release.badge.announced": {
        text: "Announced",
        description: "Badge on a row whose EN date comes from an announcement rather than the game data.",
    },
    "release.badge.estimated": {
        text: "Estimated",
        description: "Badge on a row whose EN date is projected from how far behind EN usually runs.",
    },
    "release.badge.overrideTitle": {
        text: "{source}: {note}",
        description: "Tooltip on an announced row: where the date came from, then the note left with it.",
    },
    "release.badge.per": {
        text: "per {source}",
        description: "Attribution line under an announced date, e.g. 'per the official EN stream'.",
    },
    "release.badge.range": {
        text: "{lo} to {hi}",
        description: "The window an estimated date could fall in. Both halves are already formatted dates.",
    },
    "release.badge.since": {
        text: "since {date}",
        description: "Shown in place of a date range for a pool that opened once and never closed.",
    },
    "release.badge.standing": {
        text: "standing pool",
        description: "Shown in place of a relative day for a banner that is permanently available. 'Standing' means always open.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
