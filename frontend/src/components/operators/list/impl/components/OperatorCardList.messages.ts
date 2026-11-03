import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "card.rarityStars": {
        text: "{rarity} star",
        description: "Accessible name of the star row on a list-view card. Always singular in English on purpose: it reads as a rating name ('4 star'), not a count of stars.",
    },
    "card.noOwnership": {
        text: "No community data",
        description: "Screen-reader text for the dash in the Owned column of the list view, where no imported roster has this operator or the counts have not loaded.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
