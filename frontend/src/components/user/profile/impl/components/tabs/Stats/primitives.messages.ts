import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.stats.gaps.toGo": {
        text: "To Go",
        description: "Default label in front of the row of remaining-work pills on a stats card.",
    },
    "profile.stats.gaps.allComplete": {
        text: "All Complete",
        description: "Replaces the remaining-work pills when nothing is left to do on that card.",
    },
    "profile.stats.gaps.showAria": {
        text: "Show {value} {label}",
        description: "Accessible name of a remaining-work pill. {value} is a count and {label} the thing counted, e.g. 'pending M3'.",
    },
    "profile.stats.gaps.total": {
        text: "total",
        description: "Follows the count in the header of an expanded remaining-work list.",
    },
    "profile.stats.gaps.overflow": {
        text: "+ {n} more (not rendered)",
        description: "Last row of a long remaining-work list, standing in for the entries beyond the render cap.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
