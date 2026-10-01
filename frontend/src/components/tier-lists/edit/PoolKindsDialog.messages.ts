import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.kinds.title": {
        text: "What this list ranks",
        description: "Title of the dialog where a list's owner picks which kinds of things (operators, enemies, events...) the editor's pool offers.",
    },
    "edit.kinds.description": {
        text: "Pick what the pool offers. A list can mix kinds. Changes save with your other edits.",
        description: "Subtitle of the kinds dialog. 'Save' refers to the editor's Save button; nothing is stored until then.",
    },
    "edit.kinds.groupLabel": {
        text: "Kinds this list offers",
        description: "Accessible name of the group of checkboxes, one per kind, in the kinds dialog.",
    },
    "edit.kinds.placed": {
        text: "{count} placed",
        description: "Small count beside a kind in the kinds dialog: how many of that kind are on the board now.",
    },
    "edit.kinds.atLeastOne": {
        text: "A list offers at least one kind.",
        description: "Shown when the owner tries to untick the last remaining kind in the kinds dialog.",
    },
    "edit.kinds.cancel": {
        text: "Cancel",
        description: "Button that closes the kinds dialog without changing anything.",
    },
    "edit.kinds.apply": {
        text: "Apply",
        description: "Button that applies the ticked kinds to the editor. They are stored when the list is saved.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
