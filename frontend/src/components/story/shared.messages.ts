import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** Story vocabulary shared by the reader and the library: category labels resolved from a `StoryCategory` value. */
export const namespace = "story";

export const messages = {
    "category.main": {
        text: "Main story",
        description: "Story category: the mainline chapters.",
    },
    "category.side": {
        text: "Side stories",
        description: "Story category: event stories (side stories and intermezzi).",
    },
    "category.vignette": {
        text: "Vignettes",
        description: "Story category: the small seasonal event stories the game calls Vignettes.",
    },
    "category.record": {
        text: "Operator records",
        description: "Story category: per-operator record stories from the handbook.",
    },
    "category.is": {
        text: "Integrated Strategies",
        description: "Story category: the roguelike mode's stories. Game mode name, keep the official translation.",
    },
    "category.reclamation": {
        text: "Reclamation Algorithm",
        description: "Story category: the survival mode's stories. Game mode name, keep the official translation.",
    },
    "category.sideContent": {
        text: "Other",
        description: "Story category for anything the derivation could not place.",
    },
} satisfies MessageMap;

// `dynamic`: the key is built from a `StoryCategory` value (`category.${c}`).
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
