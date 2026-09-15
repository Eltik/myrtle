import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "chibiExport.button": {
        text: "Export",
        description: "Button that opens the sprite-recording options.",
    },
    "chibiExport.title": {
        text: "Export Settings",
        description: "Heading of the recording-options popover.",
    },
    "chibiExport.resolution": {
        text: "Resolution",
        description: "Field label over the output-size select. Its options are pixel dimensions.",
    },
    "chibiExport.frameRate": {
        text: "Frame Rate",
        description: "Field label over the frames-per-second select.",
    },
    "chibiExport.fps": {
        text: "{fps} FPS",
        description: "One frame-rate option, e.g. '30 FPS'. 'FPS' is frames per second; keep the usual abbreviation for this language.",
    },
    "chibiExport.gifFpsNote": {
        text: "GIFs play at {gifFps} FPS (format limit); MP4 uses the full {fps} FPS.",
        description: "Caption warning that the GIF format cannot hit the chosen frame rate. 'GIF' and 'MP4' are file formats and stay as-is.",
    },
    "chibiExport.gifOptions": {
        text: "GIF Options",
        description: "Divider over the settings that only affect GIF output.",
    },
    "chibiExport.transparentBg": {
        text: "Transparent Background",
        description: "Switch label: leave the background see-through instead of filling it.",
    },
    "chibiExport.mp4Options": {
        text: "MP4 Options",
        description: "Divider over the settings that only affect MP4 output.",
    },
    "chibiExport.loopCount": {
        text: "Loop Count",
        description: "Field label over the select for how many times the animation repeats in the recording.",
    },
    "chibiExport.loops": {
        text: "{count, plural, one {# loop} other {# loops}}",
        description: "One loop-count option, e.g. '3 loops'.",
    },
    "chibiExport.downloadGif": {
        text: "Download GIF",
        description: "Button that renders and saves the animation as a GIF.",
    },
    "chibiExport.downloadMp4": {
        text: "Download MP4",
        description: "Button that renders and saves the animation as an MP4 video.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
