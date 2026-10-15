import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "shell.kicker": {
        text: "Admin",
        description: "Small label at the start of the admin section bar, marking that this is the admin panel. Rendered in capitals.",
    },
    "shell.compact.sections": {
        text: "{count, plural, one {# section} other {# sections}}",
        description: "How many admin sections this person can open, shown on the narrow-screen section button.",
    },
    "shell.sheet.title": {
        text: "Admin",
        description: "Title of the side panel listing the admin sections on narrow screens.",
    },
    "shell.sheet.description": {
        text: "{nickname} · {role}",
        description: "Under the admin sections panel title: the signed-in person's nickname and their role name (e.g. 'Translator').",
    },
    "shell.nav.label": {
        text: "Admin sections",
        description: "Accessible name of the row of admin section tabs.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
