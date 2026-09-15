import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.filters.search": {
        text: "Search",
        description: "Heading over the name-search box in the birthday filter form.",
    },
    "birthdays.filters.search.placeholder": {
        text: "Operator name…",
        description: "Placeholder in the name-search box. Keep the single ellipsis character.",
    },
    "birthdays.filters.class": {
        text: "Class",
        description: "Heading over the class filter buttons. The class names themselves are game vocabulary.",
    },
    "birthdays.filters.rarity": {
        text: "Rarity",
        description: "Heading over the star-rating filter buttons.",
    },
    "birthdays.filters.rarityChip": {
        text: "{rarity}★",
        description: "One star-rating filter button, e.g. '5★'.",
    },
    "birthdays.filters.nation": {
        text: "Nation",
        description: "Heading over the nation filter. Nation names themselves are game vocabulary.",
    },
    "birthdays.filters.nation.placeholder": {
        text: "All nations",
        description: "Placeholder in the nation dropdown when no nation is selected.",
    },
    "birthdays.filters.clear": {
        text: "clear",
        description: "Link that empties one filter section. Lowercase on purpose - it sits inside a small-caps heading row but is set in normal case.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
