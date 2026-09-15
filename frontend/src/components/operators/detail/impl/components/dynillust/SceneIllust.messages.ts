import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "scene.interact": {
        text: "Play the interact animation",
        description: "Accessible name of the canvas holding the animated (L2D) illustration: activating it plays the operator's tap reaction.",
    },
    "scene.loading": {
        text: "Loading animation",
        description: "Badge in the corner of the canvas while the animated illustration downloads.",
    },
    "scene.error": {
        text: "Failed to load animation",
        description: "Badge in the corner of the canvas when the animated illustration could not be loaded.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
