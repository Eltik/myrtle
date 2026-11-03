import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "planner.groupDialog.rename.title": {
        text: "Rename group",
        description: "Title of the dialog that renames a plan group.",
    },
    "planner.groupDialog.rename.label": {
        text: "Group name",
        description: "Label of the text field holding the group's new name.",
    },
    "planner.groupDialog.rename.save": {
        text: "Save",
        description: "Button that saves the group's new name.",
    },
    "planner.groupDialog.rename.failed": {
        text: "Couldn't rename the group. Try again.",
        description: "Error shown in the rename dialog when the request failed.",
    },
    "planner.groupDialog.delete.title": {
        text: 'Delete the group "{name}"?',
        description: "Title of the dialog confirming a group deletion; {name} is the group's own name, in quotes.",
    },
    "planner.groupDialog.delete.confirm": {
        text: "Delete group",
        description: "Button that carries out the group deletion.",
    },
    "planner.groupDialog.delete.failed": {
        text: "Couldn't delete the group. Try again.",
        description: "Error shown in the delete dialog when the request failed.",
    },
    "planner.groupDialog.cancel": {
        text: "Cancel",
        description: "Button that closes the rename or delete dialog without changing the group.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
