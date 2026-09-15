import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.items.dialog.usage": {
        text: "Usage. ",
        description: "Introduces the item's in-game usage note, which follows on the same line. Keep the full stop and the trailing space.",
    },
    "profile.items.dialog.onHand": {
        text: "On hand",
        description: "Tile label: how many of the item the account holds. Rendered uppercase by CSS.",
    },
    "profile.items.dialog.rarity": {
        text: "Rarity",
        description: "Tile label: the item's star rating. Rendered uppercase by CSS.",
    },
    "profile.items.dialog.expValue": {
        text: "EXP value",
        description: "Tile label: how much experience the item grants. 'EXP' is the game's own abbreviation.",
    },
    "profile.items.dialog.type": {
        text: "Type",
        description: "Tile label: the item's type as the API reports it. Rendered uppercase by CSS.",
    },
    "profile.items.dialog.category": {
        text: "Category",
        description: "Tile label: which inventory category the item falls in. Rendered uppercase by CSS.",
    },
    "profile.items.dialog.craftedIn": {
        text: "Crafted in",
        description: "Heading over the base rooms that can produce the item. Rendered uppercase by CSS.",
    },
    "profile.items.dialog.voucherExchange": {
        text: "Voucher exchange",
        description: "Heading over the vouchers the item can be exchanged for. Rendered uppercase by CSS.",
    },
    "profile.items.dialog.obtainChannels": {
        text: "Obtain channels",
        description: "Heading over where the item can be obtained, listed from the game data. Rendered uppercase by CSS.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
