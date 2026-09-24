import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** Reading style preset names, resolved from a `ReadingStyle` value (`settings.style.${style}`). */
export const namespace = "story";

export const messages = {
    "settings.style.default": {
        text: "Default",
        description: "Reading style preset.",
    },
    "settings.style.comfortable": {
        text: "Comfortable",
        description: "Reading style preset: a serif face with a taller line.",
    },
    "settings.style.large": {
        text: "Large print",
        description: "Reading style preset: bigger text.",
    },
    "settings.style.dyslexia": {
        text: "Dyslexia friendly",
        description: "Reading style preset: the OpenDyslexic typeface, wider tracking and a taller line.",
    },
    "settings.style.custom": {
        text: "Custom",
        description: "Shown in the reading style select once one of the settings a preset writes has been changed by hand. Not a preset a reader can pick.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
