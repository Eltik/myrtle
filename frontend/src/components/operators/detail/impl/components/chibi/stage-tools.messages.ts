import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "chibiStage.flip": {
        text: "Flip horizontally",
        description: "Toggle button on the sprite viewer that mirrors the character left to right. Exports follow it.",
    },
    "chibiStage.grid": {
        text: "Show grid",
        description: "Toggle button on the sprite viewer that draws guide lines behind the character, with the ground and centre lines stronger.",
    },
    "chibiStage.fit": {
        text: "Fit to view",
        description: "Button on the sprite viewer that zooms and centres so the whole animation fills the stage.",
    },
    "chibiStage.fullscreen": {
        text: "Fullscreen",
        description: "Button on the sprite viewer that fills the screen with it.",
    },
    "chibiStage.exitFullscreen": {
        text: "Exit fullscreen",
        description: "The same button while the viewer fills the screen.",
    },
    "chibiStage.zoom": {
        text: "{percent}%",
        description: "Zoom readout under the viewer's tool buttons, e.g. '150%'.",
    },
    "chibiStage.hint": {
        text: "Drag to pan · Ctrl + scroll to zoom · Double-click to reset",
        description: "Small help line in the corner of the sprite viewer. 'Ctrl' is the keyboard key; keep the usual name for it in this language.",
    },
    "chibiStage.background": {
        text: "Background",
        description: "Accessible name of the group of colour swatches that set the viewer's backdrop. It only changes the view: saved PNGs stay see-through.",
    },
    "chibiStage.bg.checker": {
        text: "Transparent",
        description: "Backdrop swatch: a checkerboard that shows where the image is see-through.",
    },
    "chibiStage.bg.black": {
        text: "Black",
        description: "Backdrop swatch: solid black.",
    },
    "chibiStage.bg.white": {
        text: "White",
        description: "Backdrop swatch: solid white.",
    },
    "chibiStage.bg.custom": {
        text: "Custom colour",
        description: "Backdrop swatch that opens a colour picker.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
