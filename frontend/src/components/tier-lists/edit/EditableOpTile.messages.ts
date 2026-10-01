import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.tile.label": {
        text: "{name} ({rarity}★){placed, select, true { - already placed} other {}}{noted, select, true { - has a description} other {}}",
        description: "Accessible name of a draggable operator tile: the operator's name and stars, then whether it is already on a tier and whether it carries a note. The name comes from the game data.",
    },
    "edit.tile.title": {
        text: "{name} ({rarity}★){noted, select, true { · has a description} other {}}",
        description: "Tooltip on a draggable operator tile: the operator's name and stars, then whether it carries a note. Keep the middle dot; the name comes from the game data.",
    },
    "edit.tile.entityLabel": {
        text: "{name}, {kind}{placed, select, true { - already placed} other {}}{noted, select, true { - has a description} other {}}",
        description: "Accessible name of a draggable tile that is not an operator (an enemy, an event, a faction...): its name from the game data and its kind, then whether it is already on a tier and whether it carries a note.",
    },
    "edit.tile.entityTitle": {
        text: "{name}{noted, select, true { · has a description} other {}}",
        description: "Tooltip on a draggable tile that is not an operator: its name from the game data, then whether it carries a note. Keep the middle dot.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
