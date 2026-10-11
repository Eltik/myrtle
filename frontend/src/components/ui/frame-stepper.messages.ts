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
        text: "Frame {frame}/{total} · {seconds}s",
        description: "Live position readout beside the playback buttons. {frame} is the current frame counting from 1, {total} the frames in the clip, {seconds} the time into the clip with two decimals. 's' is seconds.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
