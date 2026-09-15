import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "nav";

export const messages = {
    "themeToggle.trigger": {
        text: "Theme: {mode, select, dark {Dark} light {Light} other {System}}",
        description: "Tooltip on the header's theme button, naming the current setting. 'System' is the branch for following the OS preference.",
    },
    "themeToggle.triggerAria": {
        text: "{label}. Open appearance settings.",
        description: "Accessible name of the header's theme button. {label} is the themeToggle.trigger text, e.g. 'Theme: Dark'.",
    },
    "themeToggle.appearance": {
        text: "Appearance",
        description: "Heading of the theme popover in the header.",
    },
    "themeToggle.light": {
        text: "Light",
        description: "Theme option: the light colour scheme. Sits in a narrow third-of-a-row button, so keep it short.",
    },
    "themeToggle.dark": {
        text: "Dark",
        description: "Theme option: the dark colour scheme. Sits in a narrow third-of-a-row button, so keep it short.",
    },
    "themeToggle.auto": {
        text: "Auto",
        description: "Theme option: follow the operating system. Sits in a narrow third-of-a-row button, so keep it short.",
    },
    "themeToggle.accent": {
        text: "Accent",
        description: "Heading over the accent-colour swatches in the theme popover.",
    },
    "themeToggle.resetAccentAria": {
        text: "Reset accent color to default",
        description: "Accessible name of the small reset link beside the Accent heading.",
    },
    "themeToggle.reset": {
        text: "Reset",
        description: "Visible label of the reset link beside the Accent heading. Very tight space.",
    },
    "themeToggle.setAccent": {
        text: "Set accent color to {name}",
        description: "Accessible name of one accent-colour swatch. {name} is the preset's colour name.",
    },
    "themeToggle.customColor": {
        text: "Custom color",
        description: "Label of the row that opens a colour picker for an arbitrary accent.",
    },
    "themeToggle.chooseCustomColor": {
        text: "Choose custom accent color",
        description: "Accessible name of the hidden colour input behind the Custom color row.",
    },
    "themeToggle.dynamicArt": {
        text: "Dynamic art",
        description: "Label of the switch that enables animated operator artwork.",
    },
    "themeToggle.dynamicArtAria": {
        text: "Animate dynamic (L2D) operator art",
        description: "Accessible name of the dynamic-art switch. 'L2D' is Live2D, the animation format, and stays as-is.",
    },
    "themeToggle.dynamicArtNote": {
        text: "Animate L2D art on operator and profile pages. Heavier to load.",
        description: "Caption under the dynamic-art switch warning that the animations are large downloads.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
