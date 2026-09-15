import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "baseSkills.title": {
        text: "Base Skills",
        description: "Heading of the panel listing the operator's base (infrastructure) skills. The skill names and effects themselves come from the game data.",
    },
    "baseSkills.eliteAlt": {
        text: "Elite {elite}",
        description: "Alt text of the promotion badge on a base-skill icon. 'Elite' is the game's promotion tier and {elite} is 0, 1 or 2.",
    },
    "baseSkills.unlockTooltip": {
        text: "Unlocks at Elite {elite}, Lv {level}",
        description: "Hover text on the promotion badge, naming when the base skill becomes available. 'Lv' is the game's abbreviation for level.",
    },
    "baseSkills.unlockLine": {
        text: "Elite {elite} · Lv {level}{hasTargets, select, yes { · {targets}} other {}}",
        description: "Caption under a base skill, e.g. 'Elite 2 · Lv 1 · Gold'. Keep the middle dot separators. {targets} is a comma-joined list of what the skill affects, derived from the game data.",
    },
    "baseSkills.room.control": {
        text: "Control Center",
        description: "Base facility a skill works in. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.manufacture": {
        text: "Factory",
        description: "Base facility a skill works in. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.trading": {
        text: "Trading Post",
        description: "Base facility a skill works in. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.power": {
        text: "Power Plant",
        description: "Base facility a skill works in. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.dormitory": {
        text: "Dormitory",
        description: "Base facility a skill works in. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.hire": {
        text: "Office",
        description: "Base facility a skill works in - the recruitment office. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.meeting": {
        text: "Reception Room",
        description: "Base facility a skill works in - where clues are gathered. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.training": {
        text: "Training Room",
        description: "Base facility a skill works in. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.workshop": {
        text: "Workshop",
        description: "Base facility a skill works in. Use the game's own name for the facility in this language.",
    },
    "baseSkills.room.other": {
        text: "Base",
        description: "Stand-in badge when the game data names a facility this site does not know. Refers to the player's base as a whole.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
