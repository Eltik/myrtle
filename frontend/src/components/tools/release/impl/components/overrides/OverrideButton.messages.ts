import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.override.button.add": {
        text: "Override release date for {name}",
        description: "Accessible name of the admin pencil button beside a row's date. {name} is the event or banner name.",
    },
    "release.override.button.edit": {
        text: "Edit override for {name}",
        description: "Accessible name of the same button when the row already carries a hand-set date. {name} is the event or banner name.",
    },
    "release.override.button.addTitle": {
        text: "Override release date",
        description: "Tooltip on the admin pencil button beside a row's date.",
    },
    "release.override.button.editTitle": {
        text: "Edit override",
        description: "Tooltip on the same button when the row already carries a hand-set date.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
