import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "search.scope.class": {
        text: "Class",
        description: "Legend over the row of eight class icons in the class/archetype picker.",
    },
    "search.scope.archetype": {
        text: "Archetype",
        description: "Legend over the archetype chips that appear once a class is chosen.",
    },
    "search.scope.anyArchetype": {
        text: "Whole class",
        description: "First chip in the archetype row: the whole class counts, not one archetype of it.",
    },
    "search.scope.loading": {
        text: "Loading archetypes…",
        description: "Shown in place of the archetype chips while the operator index is still loading. Keep the single-character ellipsis.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
