import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.items.card.dropStages": {
        text: "{count, plural, one {# drop stage} other {# drop stages}}",
        description: "Native tooltip on the drop count of an item card: how many operations drop this item. A handful at most, so the number is inline.",
    },
    "profile.items.card.drops": {
        text: "drops",
        description: "Caption after the drop count on an item card. Always in this one form, whatever the count; very little room.",
    },
    "profile.items.card.recipe": {
        text: "Has crafting recipe",
        description: "Native tooltip on the anvil mark of an item that can be crafted in the base.",
    },
    "profile.items.card.quantity": {
        text: "Quantity",
        description: "Label over how many of the item the account holds. Rendered uppercase by CSS.",
    },
    "profile.items.card.obtain": {
        text: "Obtain",
        description: "Label in front of the ways the item can be obtained, which come from the game data. Rendered uppercase by CSS.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
