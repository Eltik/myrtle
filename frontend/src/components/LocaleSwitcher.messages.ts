import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "common";

export const messages = {
    "localeSwitcher.language": {
        text: "Language",
        description: "A noun naming what the language controls select. Used as the label of the language group in the header's language menu and the mobile drawer, and the title of the Language card in appearance settings.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
