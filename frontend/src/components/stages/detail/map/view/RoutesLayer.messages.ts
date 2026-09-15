import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "routes.svgTitle": {
        text: "enemy route",
        description: "Accessible name (SVG <title>) of the drawn path one enemy walks. Lowercase on purpose; it is only ever announced, never seen.",
    },
    "routes.svgTitle.shadow": {
        text: "enemy route shadow",
        description: "Accessible name (SVG <title>) of the drop-shadow copy of an enemy's route.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
