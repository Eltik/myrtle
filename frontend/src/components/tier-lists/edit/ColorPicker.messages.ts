import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.color.presets": {
        text: "Presets",
        description: "Small uppercase label above the grid of ready-made tier colours.",
    },
    "edit.color.useColor": {
        text: "Use color {hex}",
        description: "Accessible name of one preset swatch. {hex} is the colour's hex code.",
    },
    "edit.color.custom": {
        text: "Custom",
        description: "Label of the field where a colour can be typed or picked by hand.",
    },
    "edit.color.nativePicker": {
        text: "Open native color picker",
        description: "Accessible name of the swatch that opens the browser's own colour picker.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
