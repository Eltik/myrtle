import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "frameStepper.play": {
        text: "Play",
        description: "Accessible name of the button that resumes a paused animation.",
    },
    "frameStepper.pause": {
        text: "Pause",
        description: "Accessible name of the button that pauses a playing animation.",
    },
    "frameStepper.back": {
        text: "Back one frame (,)",
        description: "Accessible name and tooltip of the button that steps a paused animation back by one frame. The comma in brackets is the keyboard shortcut and stays as-is.",
    },
    "frameStepper.forward": {
        text: "Forward one frame (.)",
        description: "Accessible name and tooltip of the button that steps a paused animation forward by one frame. The full stop in brackets is the keyboard shortcut and stays as-is.",
    },
    "frameStepper.readout": {
        text: "{frame} / {total} F · {seconds} / {duration} s",
        description:
            "Live position readout beside the playback buttons, e.g. '55 / 180 F · 1.83 / 6.00 s'. {frame} is the current frame counting from 1, {total} the frames in the clip, {seconds} the time into the clip and {duration} its length, both with two decimals. 'F' abbreviates frames and 's' seconds; use this language's short forms.",
    },
    "frameStepper.scrub": {
        text: "Frame",
        description: "Accessible name of the slider under the animation that drags playback to any frame of the clip.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
