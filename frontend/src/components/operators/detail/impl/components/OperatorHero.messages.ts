import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "hero.breadcrumb.operators": {
        text: "Operators",
        description: "Breadcrumb link back to the operator list.",
    },
    "hero.position.ranged": {
        text: "Ranged",
        description: "Badge naming where the operator can be placed: on ranged tiles. The game's own term for the placement type.",
    },
    "hero.position.melee": {
        text: "Melee",
        description: "Badge naming where the operator can be placed: on melee tiles. The game's own term for the placement type.",
    },
    "hero.hiddenFaction.tooltip": {
        text: "Hidden faction: the game never shows this one on the operator's card. You're seeing it because spoilers are on in Settings.",
        description: "Tooltip on a badge naming a faction the operator secretly belongs to. Only shown when the reader turned on spoilers in Settings.",
    },
    "hero.hiddenFaction.srLabel": {
        text: "Hidden faction (spoiler):",
        description: "Screen-reader prefix read before the name of a faction the operator secretly belongs to.",
    },
    "hero.dataSaver.load": {
        text: "Load artwork",
        description: "Button in place of the operator's splash art while data saver is on; pressing it downloads the full art.",
    },
    "hero.dataSaver.note": {
        text: "Data saver is on",
        description: "Small caption under the Load artwork button, saying why the splash art is not shown. 'Data saver' is the setting in Settings, Appearance.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
