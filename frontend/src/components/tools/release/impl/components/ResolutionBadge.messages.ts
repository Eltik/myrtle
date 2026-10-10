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
    "release.badge.overdue": {
        text: "Due now, not in EN yet",
        description: "Badge on a row whose estimated EN date has passed while the English game data still does not carry it. 'EN' is the English game server and stays as-is.",
    },
    "release.badge.overdue.title": {
        text: "The estimate has passed and EN has not run it yet.",
        description: "Tooltip on the 'Due now, not in EN yet' badge. 'EN' is the English game server.",
    },
    "release.badge.overdue.was": {
        text: "est. {date}",
        description: "Next to the 'Due now' badge: the date the estimate gave, already formatted. 'est.' abbreviates 'estimated'.",
    },
    "release.badge.unlisted": {
        text: "Not in EN's shop",
        description: "Badge on an outfit sale the English server skipped: EN's shop has moved on to later sales without listing this one. 'EN' is the English game server.",
    },
    "release.badge.unlisted.title": {
        text: "EN moved past this sale without listing it in the outfit shop.",
        description: "Tooltip on the 'Not in EN's shop' badge. 'EN' is the English game server.",
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
    "release.badge.returning": {
        text: "Returning Doctors only",
        description: "Shown in place of a date for the Returning Headhunting banner, which the game offers only to players coming back after a long absence. 'Doctor' is how the game addresses the player.",
    },
    "release.badge.returning.title": {
        text: "Offered only to Doctors returning after a long absence. Each one gets their own window, so there is no calendar date.",
        description: "Tooltip on 'Returning Doctors only'. 'Doctor' is how the game addresses the player.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
