import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "cnUpdates.tag": {
        text: "CN",
        description: "Small tag on a module, skill, value or text on an operator's page that comes from the Chinese server's data. The server code, which normally stays as-is.",
    },
    "cnUpdates.tag.title": {
        text: "From CN's data, not on this server yet. Names and descriptions are CN's own text.",
        description: "Hover text of the CN tag. 'CN's own text' means untranslated Chinese. 'CN' is the Chinese game server, which receives content months before the others.",
    },
    "cnUpdates.value.title": {
        text: "{value} on CN",
        description: "Hover text and accessible name of the small pill beside a number on an operator's page, giving the Chinese server's value where it differs. {value} is the number with its unit, such as '70s'.",
    },
    "cnUpdates.text.title": {
        text: "This text with CN's numbers: struck through is this server's, highlighted is CN's.",
        description: "Hover text on a CN tag in front of a description repeated with the Chinese server's numbers, the changed ones marked.",
    },
    "cnUpdates.text.own.title": {
        text: "CN's own text: this server's no longer matches CN's numbers, so it is shown as CN has it.",
        description: "Hover text on a CN tag in front of a description shown in Chinese, because the description on this server could not be filled with the Chinese server's numbers. 'CN' is the Chinese game server.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
