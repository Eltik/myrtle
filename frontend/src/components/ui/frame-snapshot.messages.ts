import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "frameSnapshot.save": {
        text: "Save frame as PNG",
        description: "Accessible name and tooltip of the button beside the playback controls that saves the animation frame on screen as an image. 'PNG' is a file format and stays as-is.",
    },
    "frameSnapshot.heading": {
        text: "Transparent PNG",
        description: "Heading of the menu that picks the saved frame's size. The image has no background. 'PNG' is a file format and stays as-is.",
    },
    "frameSnapshot.option": {
        text: "{scale}x · {width} × {height}",
        description: "One size option in the save-frame menu, e.g. '2x · 640 × 480'. {scale} is the zoom factor, {width} and {height} the image size in pixels.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
