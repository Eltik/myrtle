import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "chibiExport.label": {
        text: "Export",
        description: "Label in front of the sprite viewer's row of save buttons.",
    },
    "chibiExport.png": {
        text: "PNG",
        description: "Button that saves the frame on screen as an image. A file format name; keep as-is.",
    },
    "chibiExport.webm": {
        text: "WebM",
        description: "Button that records the animation as a WebM video. A file format name; keep as-is.",
    },
    "chibiExport.gif": {
        text: "GIF",
        description: "Button that records the animation as a GIF. A file format name; keep as-is.",
    },
    "chibiExport.gif2x": {
        text: "GIF 2x",
        description: "Button that records the animation as a GIF at twice the size. '2x' means double size.",
    },
    "chibiExport.webmHint": {
        text: "Record the clip as a WebM video",
        description: "Tooltip of the WebM button.",
    },
    "chibiExport.gifHint": {
        text: "Record the clip as a GIF",
        description: "Tooltip of the GIF button.",
    },
    "chibiExport.gif2xHint": {
        text: "Record the clip as a GIF at double size",
        description: "Tooltip of the GIF 2x button.",
    },
    "chibiExport.settings": {
        text: "Export settings",
        description: "Accessible name and heading of the popover with the recording options.",
    },
    "chibiExport.frameRate": {
        text: "Frame Rate",
        description: "Label over the frames-per-second choices.",
    },
    "chibiExport.fps": {
        text: "{fps} FPS",
        description: "One frame-rate option, e.g. '30 FPS'. 'FPS' is frames per second; keep the usual abbreviation for this language.",
    },
    "chibiExport.gifFpsNote": {
        text: "GIFs play at {gifFps} FPS (format limit); videos use the full {fps} FPS.",
        description: "Caption warning that the GIF format cannot hit the chosen frame rate. 'GIF' is a file format and stays as-is.",
    },
    "chibiExport.loopCount": {
        text: "Video loops",
        description: "Label over the choices for how many times the clip repeats in a recorded video.",
    },
    "chibiExport.loops": {
        text: "{count}×",
        description: "One loop-count option, e.g. '3×' for three repeats.",
    },
    "chibiExport.backgroundNote": {
        text: "Recordings use the viewer's backdrop. On the checkerboard a GIF stays see-through and a video gets a dark fill. PNG is always see-through.",
        description: "Caption in the export settings explaining which background each format gets. 'GIF' and 'PNG' are file formats.",
    },
    "chibiExport.downloadMp4": {
        text: "Download MP4",
        description: "Button in the export settings that records the clip as an MP4 video, for players without WebM. 'MP4' is a file format.",
    },
    "chibiExport.cancel": {
        text: "Cancel recording",
        description: "Accessible name of the button that stops a recording in progress.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
