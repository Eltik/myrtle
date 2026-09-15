import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.row.editTier": {
        text: 'Edit tier "{name}"',
        description: "Accessible name of a tier's label button in the editor, which opens its settings. {name} is the author's own tier label; keep the quotes.",
    },
    "edit.row.dropArea": {
        text: "Operators in tier {name}",
        description: "Accessible name of the area holding a tier's operators, which is also the drop target. {name} is the author's own tier label.",
    },
    "edit.row.actions": {
        text: "Tier {name} actions",
        description: "Accessible name of the toolbar beside a tier row. {name} is the author's own tier label.",
    },
    "edit.row.moveUp": {
        text: "Move tier up",
        description: "Accessible name of the button that moves this tier one place up the board.",
    },
    "edit.row.settings": {
        text: "Tier settings",
        description: "Accessible name of the button that opens this tier's settings dialog.",
    },
    "edit.row.moveDown": {
        text: "Move tier down",
        description: "Accessible name of the button that moves this tier one place down the board.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
