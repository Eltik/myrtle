import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "nav";

export const messages = {
    "languageToggle.trigger": {
        text: "Language: {language}",
        description: "Tooltip on the globe button in the header. {language} is the active language's own name, e.g. 日本語.",
    },
    "languageToggle.triggerAria": {
        text: "{label}. Choose a language.",
        description: "Accessible name of the globe button. {label} is the tooltip text above.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
