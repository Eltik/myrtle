import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.improvements.medal.empty": {
        text: "You've earned every medal that's currently reachable. Nice work.",
        description: "Empty state in the Medals panel when no reachable medal is missing.",
    },
    "score.improvements.medal.missingCount": {
        text: "{n} missing",
        description: "Count badge on a medal list heading. {n} is how many medals in that group the account is missing.",
    },
    "score.improvements.medal.event.title": {
        text: "Event medals - limited time",
        description: "Heading of the group of medals that can only be earned while their event runs.",
    },
    "score.improvements.medal.event.subtitle": {
        text: "Sorted by earliest ending. Hidden medals are still listed but their hint is suppressed.",
        description: "Blurb under the event-medal heading. A 'hidden' medal is one whose unlock condition the game does not show.",
    },
    "score.improvements.medal.permanent.title": {
        text: "Permanent medals",
        description: "Heading of the group of medals that are always obtainable.",
    },
    "score.improvements.medal.permanent.subtitle": {
        text: "Sorted by rarity desc. Highest-rarity gaps first.",
        description: "Blurb under the permanent-medal heading. 'desc' abbreviates descending.",
    },
    "score.improvements.medal.locked.title": {
        text: "Behind unavailable operators",
        description: "Heading of the group of medals that need an operator the account cannot currently obtain.",
    },
    "score.improvements.medal.locked.subtitle": {
        text: "These require a collab operator you don't own. Since collab operators aren't normally obtainable, these medals don't count toward your medal score. Earn the operator in a rerun and they'll count again. Shown for reference.",
        description: "Blurb under the 'Behind unavailable operators' heading. A 'collab' operator comes from a crossover event and a 'rerun' is its repeat run - both in-game terms.",
    },
    "score.improvements.medal.unobtainable.title": {
        text: "No longer obtainable",
        description: "Heading of the group of medals whose event window has closed for good.",
    },
    "score.improvements.medal.unobtainable.subtitle": {
        text: "These medals' windows have passed and won't reopen. Past event medals still count in your score with decaying weight rather than against the permanent pool; one-time modes and retired towers are excluded. Shown for reference.",
        description: "Blurb under the 'No longer obtainable' heading. 'Towers' are the in-game Stultifera Navis-style tower events.",
    },
    "score.improvements.medal.show": {
        text: "Show {n}",
        description: "Button that reveals a medal group that starts collapsed. {n} is how many medals it holds.",
    },
    "score.improvements.medal.hide": {
        text: "Hide",
        description: "Button that collapses a medal group again.",
    },
    "score.improvements.medal.hidden": {
        text: "Hidden",
        description: "Small tag on a medal whose unlock condition the game does not reveal.",
    },
    "score.improvements.medal.hidden.tooltip": {
        text: "Hidden medal - unlock condition isn't shown in-game.",
        description: "Tooltip on the 'Hidden' tag.",
    },
    "score.improvements.medal.ownedTooltip": {
        text: "Earned by {pct}% of synced players on your server who share stats - how rare this medal is in the community.",
        description: "Tooltip on the owners figure of a medal row. {pct} is the share of players who have it.",
    },
    "score.improvements.medal.endingNow": {
        text: "ending now",
        description: "Pill on an event medal whose window closes today. Lowercase on purpose.",
    },
    "score.improvements.medal.daysLeft": {
        text: "{days}d left",
        description: "Pill on an event medal counting down its window. {days} is a whole number of days and 'd' is the abbreviation for day.",
    },
    "score.improvements.medal.ended": {
        text: "Ended {date}",
        description: "Pill on a medal whose event window has closed. {date} is the closing date.",
    },
    "score.improvements.medal.lockTooltip": {
        text: "Requires {operator}, a {reason} operator you don't own. Excluded from your medal score until you have them.",
        description: "Tooltip on a locked medal. {operator} is the operator's name from the game data and {reason} the category the API reports, such as collab.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
