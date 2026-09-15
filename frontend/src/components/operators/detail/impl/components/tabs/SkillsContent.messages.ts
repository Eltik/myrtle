import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "skills.title": {
        text: "Skills",
        description: "Heading of the Skills tab.",
    },
    "skills.subtitle": {
        text: "Skill details, mastery information, and skill comparisons.",
        description: "Blurb under the Skills tab heading. 'Mastery' is the game's name for skill ranks M1 to M3.",
    },
    "skills.empty.icon": {
        text: "No skills",
        description: "Accessible name of the decorative icon in the no-skills empty state.",
    },
    "skills.empty.title": {
        text: "No Skills",
        description: "Empty-state heading for an operator with no active skills.",
    },
    "skills.empty.body": {
        text: "This operator has no active skills. They may rely solely on their talent or passive abilities.",
        description: "Empty-state body for an operator with no active skills.",
    },
    "skills.fallbackName": {
        text: "Skill {index}",
        description: "Stand-in name for a skill the game data does not name. {index} counts from 1.",
    },
    "skills.imageAlt": {
        text: "Skill",
        description: "Alt text of a skill icon when the game data carries no skill name to use instead.",
    },
    "skills.cohort": {
        text: "E2 owners use this skill",
        description: "Names the group behind the share pill on a skill button, completing 'N of M ...'. 'E2' is the game's second promotion.",
    },
    "skills.mastery.title": {
        text: "Community mastery",
        description: "Heading of the bar chart showing how far owners have levelled this skill's mastery.",
    },
    "skills.mastery.none": {
        text: "No mastery",
        description: "Bar label for owners who have not raised this skill past level 7. Spelled out rather than 'M0', which would read as a rank someone reached.",
    },
    "skills.mastery.m1": {
        text: "M1",
        description: "Bar label for mastery rank 1. The game's own shorthand, which normally stays as-is.",
    },
    "skills.mastery.m2": {
        text: "M2",
        description: "Bar label for mastery rank 2. The game's own shorthand, which normally stays as-is.",
    },
    "skills.mastery.m3": {
        text: "M3",
        description: "Bar label for mastery rank 3. The game's own shorthand, which normally stays as-is.",
    },
    "skills.mastery.summary": {
        text: "mastered this skill",
        description: "Completes the headline 'N% have ...' over the mastery bars.",
    },
    "skills.mastery.cohort": {
        text: "E2 owners",
        description: "Names the group the mastery bars are drawn from. 'E2' is the game's second promotion, the only stage at which mastery exists.",
    },
    "skills.compare.toggle": {
        text: "Compare Skill Levels",
        description: "Label of the switch that turns the level-by-level comparison table on.",
    },
    "skills.level": {
        text: "Skill Level",
        description: "Label beside the slider that picks which skill level is shown.",
    },
    "skills.compare.selectLevels": {
        text: "Select Levels",
        description: "Label over the buttons choosing which skill levels the comparison shows.",
    },
    "skills.compare.selectedCount": {
        text: "({count} selected)",
        description: "Count of chosen comparison levels, shown in brackets after the label.",
    },
    "skills.compare.showFull": {
        text: "Show Full",
        description: "Button that switches the comparison back to full descriptions.",
    },
    "skills.compare.showDiff": {
        text: "Show Diff",
        description: "Button that switches the comparison to only what changed between levels. 'Diff' is short for difference.",
    },
    "skills.spCost": {
        text: "SP Cost",
        description: "Tile label: skill points needed to fire the skill. 'SP' is the game's abbreviation and stays as-is.",
    },
    "skills.initialSp": {
        text: "Initial SP",
        description: "Tile label: skill points the skill starts with.",
    },
    "skills.duration": {
        text: "Duration",
        description: "Tile label: how long the skill lasts.",
    },
    "skills.durationSeconds": {
        text: "{value}s",
        description: "A duration in seconds, e.g. '20s'. Very tight tile; abbreviate the unit to a single letter if the language allows.",
    },
    "skills.range": {
        text: "Range",
        description: "Small uppercase heading over the tile grid the skill can hit.",
    },
    "skills.range.hideDiff": {
        text: "Hide Diff",
        description: "Button that stops showing the skill's range beside the operator's normal range.",
    },
    "skills.range.original": {
        text: "Original",
        description: "Caption over the operator's normal attack range in the side-by-side comparison.",
    },
    "skills.range.onSkill": {
        text: "On Skill",
        description: "Caption over the attack range while the skill is active, in the side-by-side comparison.",
    },
    "skills.column.level": {
        text: "Level",
        description: "Column header of the comparison table.",
    },
    "skills.column.description": {
        text: "Description / Changes",
        description: "Column header of the comparison table: the skill text, or only what changed in diff mode.",
    },
    "skills.column.sp": {
        text: "SP",
        description: "Narrow column header: skill-point cost. The game's own abbreviation; three characters at most.",
    },
    "skills.column.init": {
        text: "Init",
        description: "Narrow column header: initial skill points, short for 'initial'. Four characters at most.",
    },
    "skills.column.dur": {
        text: "Dur",
        description: "Narrow column header: duration, short for 'duration'. Four characters at most.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
