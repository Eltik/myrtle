import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "chibi.title": {
        text: "Chibi Preview",
        description: "Heading of the animated sprite viewer. 'Chibi' is the fan term for the small in-battle sprite, used as-is in English.",
    },
    "chibi.view": {
        text: "View",
        description: "Prompt in the select that picks which sprite set is shown, before a choice is made.",
    },
    "chibi.view.front": {
        text: "Front",
        description: "Sprite set: the operator seen from the front, as when facing the player.",
    },
    "chibi.view.back": {
        text: "Back",
        description: "Sprite set: the operator seen from behind.",
    },
    "chibi.view.dorm": {
        text: "Dorm",
        description: "Sprite set: the base/dormitory sprite, which idles rather than fights. 'Dorm' is short for dormitory.",
    },
    "chibi.view.down": {
        text: "Down",
        description: "Sprite set: the knocked-out sprite.",
    },
    "chibi.animation": {
        text: "Animation",
        description: "Prompt in the select that picks which clip plays. The clip names come from the animation files and are not translated.",
    },
    "chibi.error.noData": {
        text: "No spine data available",
        description: "Shown in the viewer when the chosen sprite set ships no animation files. 'Spine' is the animation format's name and stays as-is.",
    },
    "chibi.error.failed": {
        text: "Failed to load chibi",
        description: "Shown in the viewer when the animation files could not be fetched or parsed.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
