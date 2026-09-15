import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.preview.rarityLabel": {
        text: "{rarity}-star",
        description: "Accessible name of the row of stars in an operator's hover card, e.g. '6-star'.",
    },
    "detail.preview.position.melee": {
        text: "Melee",
        description: "An operator who is deployed on the ground. The game's own word for the position.",
    },
    "detail.preview.position.ranged": {
        text: "Ranged",
        description: "An operator who is deployed on the raised tiles. The game's own word for the position.",
    },
    "detail.preview.nation": {
        text: "Nation",
        description: "Field label in an operator's hover card; its value is a place name from the game data.",
    },
    "detail.preview.position": {
        text: "Position",
        description: "Field label in an operator's hover card; its value is Melee or Ranged.",
    },
    "detail.preview.hint": {
        text: "Click to view operator",
        description: "Footer of an operator's hover card, saying the card is a link to that operator's page.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
