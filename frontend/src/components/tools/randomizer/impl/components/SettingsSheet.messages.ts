import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.settings.title": {
        text: "Settings",
        description: "Title of the panel that constrains what the randomizer may draw.",
    },
    "randomizer.settings.desc": {
        text: "Constrain the randomizer - by class, rarity, owned operators, or stage availability.",
        description: "Caption under the Settings title. 'Class' and 'rarity' are the game's own operator attributes; the dash is a plain hyphen.",
    },
    "randomizer.settings.tab.operators": {
        text: "Operators",
        description: "Tab holding the class, rarity and squad-size settings.",
    },
    "randomizer.settings.tab.stages": {
        text: "Stages",
        description: "Tab holding the stage-pool settings.",
    },
    "randomizer.settings.tab.roster": {
        text: "Roster",
        description: "Tab where the player picks exactly which operators may be drawn.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
