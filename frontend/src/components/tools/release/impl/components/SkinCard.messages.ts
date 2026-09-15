import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.card.obtain.eventReward": {
        text: "Event",
        description: "Very short stand-in for 'Event Reward' on the outfit card's price badge. The badge is only a few characters wide.",
    },
    "release.card.obtain.eventGift": {
        text: "Gift",
        description: "Very short stand-in for 'Event Gift' on the outfit card's price badge.",
    },
    "release.card.obtain.is": {
        text: "IS",
        description: "Very short stand-in for the game mode 'Integrated Strategies' on the outfit card's price badge. Its own abbreviation, normally unchanged.",
    },
    "release.card.obtain.ra": {
        text: "RA",
        description: "Very short stand-in for the game mode 'Reclamation Algorithm' on the outfit card's price badge. Its own abbreviation, normally unchanged.",
    },
    "release.card.obtain.quest": {
        text: "Quest",
        description: "Very short stand-in for 'Quest Reward' on the outfit card's price badge.",
    },
    "release.card.obtain.pack": {
        text: "Pack",
        description: "Very short stand-in for 'Obtain from Special Pack' on the outfit card's price badge.",
    },
    "release.card.obtain.code": {
        text: "Code",
        description: "Very short stand-in for any redemption-code source on the outfit card's price badge.",
    },
    "release.card.price.store": {
        text: "{price} Originite Prime in the Outfit Store",
        description: "Tooltip on a purchasable outfit. 'Originite Prime' is the game's premium currency and 'Outfit Store' its shop; both keep the game's own names.",
    },
    "release.card.price.none": {
        text: "{obtain}: no Originite Prime",
        description: "Tooltip on an outfit that is not sold; {obtain} says how it is obtained instead.",
    },
    "release.card.price.notSold": {
        text: "Not sold",
        description: "Stands in for the obtain method in the tooltip when the game data gives none.",
    },
    "release.card.add": {
        text: "Add {operator}: {skin}",
        description: "Accessible name of an unselected outfit card, which adds it to the plan. Both names come from the game data.",
    },
    "release.card.remove": {
        text: "Remove {operator}: {skin}",
        description: "Accessible name of a selected outfit card, which drops it from the plan.",
    },
    "release.card.title": {
        text: "{operator}: {skin}",
        description: "Native tooltip on an outfit card: the operator's name then the outfit's. Both come from the game data.",
    },
    "release.card.inspect": {
        text: "Inspect {skin}",
        description: "Accessible name of the magnifier that opens the outfit's full art.",
    },
    "release.card.selected": {
        text: "Selected",
        description: "Overlay across an outfit card that is in the plan.",
    },
} satisfies MessageMap;

// `dynamic`: the obtain-method keys are stored in a lookup table and resolved as
// `t(OBTAIN_SHORT_KEYS[obtain])`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
