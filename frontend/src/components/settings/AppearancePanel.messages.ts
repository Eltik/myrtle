import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "appearance.theme.title": {
        text: "Theme",
        description: "Card title over the light/dark/auto choice.",
    },
    "appearance.theme.desc": {
        text: "Pick light, dark, or follow your system. The mobile picker in the header controls the same setting.",
        description: "Card description under the Theme title.",
    },
    "appearance.mode.light": {
        text: "Light",
        description: "Theme option: the light colour scheme.",
    },
    "appearance.mode.dark": {
        text: "Dark",
        description: "Theme option: the dark colour scheme.",
    },
    "appearance.mode.auto": {
        text: "Auto",
        description: "Theme option: follow the operating system's setting.",
    },
    "appearance.accent.title": {
        text: "Accent color",
        description: "Card title over the accent-colour picker.",
    },
    "appearance.accent.desc": {
        text: "Tints buttons, links, focus rings, and the active state across the whole app.",
        description: "Card description under the Accent color title.",
    },
    "appearance.accent.presetHue": {
        text: "Preset hue",
        description: "Small uppercase label over the row of ready-made accent swatches.",
    },
    "appearance.accent.setTo": {
        text: "Set accent color to {name}",
        description: "Accessible name of one accent swatch. {name} is the preset's colour name.",
    },
    "appearance.accent.customColor": {
        text: "Custom color",
        description: "Small uppercase label over the free colour picker.",
    },
    "appearance.accent.pickCustomHex": {
        text: "Pick custom hex",
        description: "Label on the custom-colour button before a custom colour has been chosen. 'Hex' is a hexadecimal colour code.",
    },
    "appearance.accent.chooseCustom": {
        text: "Choose custom accent color",
        description: "Accessible name of the hidden colour input behind the custom-colour button.",
    },
    "appearance.accent.reset": {
        text: "Reset to default",
        description: "Button that restores the site's original accent colour.",
    },
    "appearance.language.desc": {
        text: "Sets the language of the site's own text. Choosing one reloads the page so the new language is there from the first paint.",
        description: "Card description under the Language title in appearance settings. The reload is deliberate and worth naming, so the reload does not look like a fault.",
    },
    "appearance.language.displayLanguage": {
        text: "Display language",
        description: "Small uppercase label over the language picker. 'Display' distinguishes the site's own text from the game data, which follows the language's own region.",
    },
    "appearance.language.choose": {
        text: "Choose a language",
        description: "Accessible name of the language picker in appearance settings.",
    },
    "appearance.language.onlyOne": {
        text: "{language} is the only language this site is available in right now.",
        description: "Shown in place of the language picker when just one language is enabled. {language} is that language's own name for itself, e.g. 'English'.",
    },
    "appearance.dynamicArt.title": {
        text: "Dynamic art",
        description: "Card title over the animated-artwork option.",
    },
    "appearance.dynamicArt.desc": {
        text: "Animate L2D (dynamic) operator illustrations across operator and profile pages.",
        description: "Card description under the Dynamic art title. 'L2D' is Live2D, the animation format, and stays as-is.",
    },
    "appearance.dynamicArt.rowTitle": {
        text: "Animate dynamic art",
        description: "Label of the switch that turns animated artwork on, and the switch's accessible name.",
    },
    "appearance.dynamicArt.rowDesc": {
        text: "Plays multi-megabyte Spine animations in place of static art. On by default; turn it off to save bandwidth.",
        description: "Caption under the animated-artwork switch. 'Spine' is the animation runtime's name and stays as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
