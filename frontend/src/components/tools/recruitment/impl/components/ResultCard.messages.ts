import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "recruit.result.robotFloor": {
        text: "Robot",
        description: "Stands in for '1★' as the guaranteed floor, because the one-star tier is the game's Robot tier and a bare star would read as an ordinary 1★.",
    },
    "recruit.result.starFloor": {
        text: "{rarity}★",
        description: "The guaranteed floor as a star rating, e.g. '4★'.",
    },
    "recruit.result.guaranteed": {
        text: "Guaranteed {rarity}★",
        description: "Filled badge on a tag combination that cannot roll below five stars.",
    },
    "recruit.result.minTitle": {
        text: "Guaranteed minimum: {floor}",
        description: "Native tooltip on the quiet floor label. {floor} is either a star rating or the word Robot.",
    },
    "recruit.result.min": {
        text: "Min",
        description: "Label before the guaranteed floor on cards below a five-star lock. Abbreviation of 'minimum'; the space is very narrow.",
    },
    "recruit.result.noTags": {
        text: "No tags",
        description: "Shown in place of an operator's recruitment tags when it has none. Rendered in italics.",
    },
    "recruit.result.tags": {
        text: "Tags",
        description: "Heading over one operator's recruitment tags, in the hover card and in the expanded mobile row.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
