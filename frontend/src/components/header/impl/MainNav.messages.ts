import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "nav";

export const messages = {
    "mainNav.press": {
        text: "Press",
        description: "First half of the palette hint in a nav dropdown's footer, read as 'Press ⌘K for all commands'. Two key caps follow it, so this fragment ends before them.",
    },
    "mainNav.forAllCommands": {
        text: "for all commands",
        description: "Second half of the palette hint in a nav dropdown's footer, read after the ⌘ and K key caps: 'Press ⌘K for all commands'.",
    },
    "mainNav.openPalette": {
        text: "Open palette →",
        description: "Link in a nav dropdown's footer that opens the command palette. Ends with a rightwards arrow character.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
