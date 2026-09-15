import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.sub.details.expand": {
        text: "{label} details - expand",
        description: "Accessible name of a collapsed section card. {label} is the section name.",
    },
    "score.sub.details.collapse": {
        text: "{label} details - collapse",
        description: "Accessible name of an expanded section card. {label} is the section name.",
    },
    "score.sub.shareBadge": {
        text: "{share}% of grade",
        description: "Badge in the corner of a section card saying how much of the overall grade this section can be worth.",
    },
    "score.sub.shareTooltip": {
        text: "Worth {share}% of your overall grade - at {pct}%, this section contributes {contribution} of those {share} points",
        description: "Tooltip on the share badge. {share} is this section's maximum share, {pct} how complete the section is, {contribution} the points it currently adds.",
    },
    "score.sub.progress": {
        text: "Progress",
        description: "Label over the section's completion bar.",
    },
    "score.sub.hideBreakdown": {
        text: "Hide breakdown",
        description: "Footer of an expanded section card, inviting a click to close it.",
    },
    "score.sub.whatToImprove": {
        text: "What can I improve?",
        description: "Footer of a collapsed section card, inviting a click to open the suggestions.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
