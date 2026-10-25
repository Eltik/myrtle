import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "header.boss": {
        text: "Boss",
        description: "Pill beside the stage code marking an operation that contains a boss enemy.",
    },
    "header.variants": {
        text: "Stage versions",
        description: "Accessible name of the switcher between a stage's Standard, Adverse and Challenge Mode versions.",
    },
    "header.variant.standard": {
        text: "Standard",
        description: "Switcher option for the regular version of an operation. Use the game's own term.",
    },
    "header.variant.adverse": {
        text: "Adverse",
        description: "Switcher option for the Adverse version of a main story operation, a separate harder stage. Use the game's own term.",
    },
    "header.variant.cm": {
        text: "Challenge Mode",
        description: "Switcher option for the Challenge Mode version of an operation: the same map with an extra condition. Use the game's own term.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
