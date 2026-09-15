import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "summons.title": {
        text: "Summons",
        description: "Heading of the panel listing the tokens and drones an operator can deploy. The summons' own names come from the game data.",
    },
    "summons.attackRange": {
        text: "Attack Range",
        description: "Heading over the grid showing which tiles the summon can hit.",
    },
    "summons.talents": {
        text: "Talents",
        description: "Heading over the summon's talents. The talent names and text come from the game data.",
    },
    "summons.position.melee": {
        text: "Melee",
        description: "Badge naming where the summon can be placed: on melee tiles. The game's own term.",
    },
    "summons.position.ranged": {
        text: "Ranged",
        description: "Badge naming where the summon can be placed: on ranged tiles. The game's own term.",
    },
    "summons.position.all": {
        text: "Melee/Ranged",
        description: "Badge for a summon that can go on either tile type. The game's own terms, joined by a slash.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
