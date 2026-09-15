import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "common";

export const messages = {
    "localeSwitcher.language": {
        text: "Language",
        description: "A noun naming what the language controls select. Used as the screen-reader-only group label in the footer switcher, the heading of the header's globe menu and of the mobile drawer's language group, and the title of the Language card in appearance settings.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
