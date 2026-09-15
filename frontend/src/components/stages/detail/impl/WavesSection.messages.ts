import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "waves.title": {
        text: "Waves · {count}",
        description: "Kicker over the wave cards, with how many waves the stage has.",
    },
    "waves.wave": {
        text: "Wave {index}",
        description: "Heading of one wave card. {index} counts from 1.",
    },
    "waves.spawns": {
        text: "{count, plural, one {# spawn} other {# spawns}}",
        description: "How many enemies one wave releases in total, on the right of the wave card's heading row.",
    },
    "waves.preDelay": {
        text: "Pre-Delay",
        description: "Wave card field label: seconds before the wave starts.",
    },
    "waves.postDelay": {
        text: "Post-Delay",
        description: "Wave card field label: seconds after the wave ends before the next begins.",
    },
    "waves.fragments": {
        text: "Fragments",
        description: "Wave card field label: how many timing fragments the wave is split into. 'Fragment' is the level file's own term.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
