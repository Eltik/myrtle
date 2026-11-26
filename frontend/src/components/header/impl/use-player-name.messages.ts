import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** Shared by the header's account menu, the mobile drawer and Settings, so it sits in `common`. */
export const namespace = "common";

export const messages = {
    playerFallback: {
        text: "Player",
        description: "Stands in for a signed-in visitor's nickname when the game sent none, in the header account menu, the mobile drawer and Settings. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
