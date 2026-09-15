import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * 'Mastery' is the game's own skill-rank system and M3 / M6 / M9 are the
 * community's shorthand for one, two and three skills at Mastery 3.
 */
export const namespace = "user";

export const messages = {
    "profile.stats.mastery.title": {
        text: "Skill Mastery",
        description: "Heading of the card covering skill masteries.",
    },
    "profile.stats.mastery.m3.tooltip": {
        text: "Operators with at least one Mastery 3 skill",
        description: "Tooltip on the M3 tile.",
    },
    "profile.stats.mastery.m6.tooltip": {
        text: "Operators with 2 skills at Mastery 3",
        description: "Tooltip on the M6 tile.",
    },
    "profile.stats.mastery.m9.tooltip": {
        text: "Operators with all 3 skills at Mastery 3",
        description: "Tooltip on the M9 tile.",
    },
    "profile.stats.mastery.totalLevels": {
        text: "Total Mastery Levels",
        description: "Label over the bar summing every mastery rank earned.",
    },
    "profile.stats.mastery.ofMax": {
        text: "{pct}% of max possible (E2 only)",
        description: "Caption under that bar. Only Elite 2 operators can hold masteries, so they are the only ones counted; 'E2' is the game's second Elite promotion.",
    },
    "profile.stats.mastery.gap.levels": {
        text: "levels",
        description: "Remaining-work pill: mastery levels still to earn. Follows a count.",
    },
    "profile.stats.mastery.gap.levels.tooltip": {
        text: "Skill levels remaining to reach Mastery 3 across all E2 operators",
        description: "Tooltip on that pill. 'E2' is the game's second Elite promotion.",
    },
    "profile.stats.mastery.gap.m3": {
        text: "pending M3",
        description: "Remaining-work pill: operators with no skill at Mastery 3 yet. Follows a count.",
    },
    "profile.stats.mastery.gap.m3.tooltip": {
        text: "Click to view E2 operators without any skill at Mastery 3 yet",
        description: "Tooltip on the 'pending M3' pill.",
    },
    "profile.stats.mastery.gap.m6": {
        text: "pending M6",
        description: "Remaining-work pill: operators with one skill at Mastery 3. Follows a count.",
    },
    "profile.stats.mastery.gap.m6.tooltip": {
        text: "Click to view operators with 1 skill at M3 still needing a second",
        description: "Tooltip on the 'pending M6' pill.",
    },
    "profile.stats.mastery.gap.m9": {
        text: "pending M9",
        description: "Remaining-work pill: operators with two skills at Mastery 3. Follows a count.",
    },
    "profile.stats.mastery.gap.m9.tooltip": {
        text: "Click to view operators with 2 skills at M3 still needing a third",
        description: "Tooltip on the 'pending M9' pill.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
