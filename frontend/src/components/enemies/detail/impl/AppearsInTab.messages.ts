import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "appears.category.stages": {
        text: "Story stages",
        description: "Heading of the bucket holding mainline campaign operations.",
    },
    "appears.category.events": {
        text: "Events",
        description: "Heading of the bucket holding limited-time events and their reruns.",
    },
    "appears.category.modes": {
        text: "Permanent game modes",
        description: "Heading of the bucket holding the game's always-available modes (Annihilation, Integrated Strategies and so on).",
    },
    "appears.sub.story": {
        text: "Story",
        description: "Sub-heading inside an event, over its normal operations (as opposed to its EX ones).",
    },
    "appears.sub.ex": {
        text: "EX Stages",
        description: "Sub-heading inside an event, over its EX operations. 'EX' is the game's own prefix for the harder event stages; keep it.",
    },
    "appears.zoneType.main": {
        text: "Main",
        description: "Badge on a zone row: part of the mainline campaign. Short badge, so keep it brief.",
    },
    "appears.zoneType.sideStory": {
        text: "Side Story",
        description: "Badge on a zone row: a Side Story event. The game's own category name.",
    },
    "appears.zoneType.branch": {
        text: "Branch",
        description: "Badge on a zone row: a Vignette / branchline event. Short badge, so keep it brief.",
    },
    "appears.zoneType.event": {
        text: "Event",
        description: "Badge on a zone row: a limited-time event.",
    },
    "appears.zoneType.weekly": {
        text: "Weekly",
        description: "Badge on a zone row: a weekly resource-farming zone.",
    },
    "appears.zoneType.annihilation": {
        text: "Annihilation",
        description: "Badge on a zone row: the Annihilation game mode. An in-game mode name; keep the game's own name for it.",
    },
    "appears.zoneType.sss": {
        text: "S.S.S.",
        description: "Badge on a zone row: the Stationary Security Service game mode, abbreviated. An in-game mode name; keep the game's own abbreviation.",
    },
    "appears.zoneType.is": {
        text: "I.S.",
        description: "Badge on a zone row: the Integrated Strategies game mode, abbreviated. An in-game mode name; keep the game's own abbreviation.",
    },
    "appears.zoneType.guide": {
        text: "Guide",
        description: "Badge on a zone row: a tutorial zone.",
    },
    "appears.zoneType.special": {
        text: "Special",
        description: "Badge on a zone row: a special zone outside the usual categories.",
    },
    "appears.empty.title": {
        text: "No appearances recorded",
        description: "Empty-state heading in the Appears In tab when no level file mentions this enemy.",
    },
    "appears.empty.body": {
        text: "This enemy isn't in any stage we have data for yet.",
        description: "Empty-state paragraph in the Appears In tab, explaining the gap is this site's coverage rather than the game's.",
    },
    "appears.found": {
        text: "Found in {locations} {locationCount, plural, one {location} other {locations}} across {zones} {zoneCount, plural, one {zone} other {zones}}.",
        description: "Sentence above the appearance list, e.g. 'Found in 12 locations across 4 zones.'. {locations} and {zones} are the two bold counts and may move anywhere the sentence needs them; {locationCount} and {zoneCount} are the same numbers again, used only to pick the plural form.",
    },
    "appears.chip.spawns": {
        text: "{count, plural, one {# spawn} other {# spawns}}",
        description: "Read by screen readers on a stage chip (sighted users see '×N'): how many of this enemy the operation sends.",
    },
    "appears.chip.conditional": {
        text: "summoned / conditional",
        description: "Shown on a stage chip in place of a spawn count, when the enemy only appears if something summons it or a condition is met.",
    },
    "appears.chip.adverse": {
        text: "Adverse",
        description: "Small badge on a stage chip marking the stage's harder variant. Use the game's own word for 'Adverse' (the Adverse Environment variant) in this language.",
    },
    "appears.failed": {
        text: "Couldn't load where this enemy appears.",
        description: "Error line in an enemy's Appears In tab when the stage list for that enemy fails to load.",
    },
} satisfies MessageMap;

// `dynamic`: the category, sub-section and zone-type keys live on module-level
// tables and are resolved as `t(entry.labelKey)`, so the extractor has no
// literal call site for them.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
