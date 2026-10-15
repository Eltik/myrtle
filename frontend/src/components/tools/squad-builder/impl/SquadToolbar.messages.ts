import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.toolbar.name": {
        text: "Squad name",
        description: "Accessible name and placeholder of the squad name field.",
    },
    "squad.toolbar.count": {
        text: "{count} of {total}",
        description: "How many of the squad's seats are filled, e.g. '9 of 12'; {total} is the seats in a squad.",
    },
    "squad.toolbar.plusSupport": {
        text: "plus support",
        description: "After the seat count when a support is picked, e.g. '9 of 12 plus support'.",
    },
    "squad.toolbar.minimize": {
        text: "Minimize",
        description: "Button that shrinks the seat cards to small squares.",
    },
    "squad.toolbar.expand": {
        text: "Expand",
        description: "Button that grows the seat cards back to full cards.",
    },
    "squad.toolbar.revert": {
        text: "Revert",
        description: "Button that throws away unsaved changes and goes back to the saved squad.",
    },
    "squad.toolbar.clear": {
        text: "Clear squad",
        description: "Button that empties every seat.",
    },
    "squad.toolbar.clearTitle": {
        text: "Clear the squad?",
        description: "Title of the confirm dialog before emptying every seat.",
    },
    "squad.toolbar.clearDesc": {
        text: "Every seat and the support are emptied. A saved slot keeps its copy until you save.",
        description: "Body of the clear-squad confirm dialog.",
    },
    "squad.toolbar.cancel": {
        text: "Cancel",
        description: "Button that closes a confirm dialog without doing anything.",
    },
    "squad.toolbar.share": {
        text: "Share",
        description: "Button that opens the share dialog.",
    },
    "squad.toolbar.save": {
        text: "Save",
        description: "Button that saves the squad to its slot.",
    },
    "squad.toolbar.signInToSave": {
        text: "Sign in to save",
        description: "Save button label when signed out.",
    },
    "squad.toolbar.more": {
        text: "More",
        description: "Accessible name of the menu button with less common squad actions.",
    },
    "squad.toolbar.delete": {
        text: "Delete slot {slot}",
        description: "Menu item that deletes the saved squad in this slot. {slot} is 1 to 4.",
    },
    "squad.toolbar.deleteTitle": {
        text: "Delete slot {slot}?",
        description: "Title of the confirm dialog before deleting a saved squad.",
    },
    "squad.toolbar.deleteDesc": {
        text: "The saved squad is gone for good. What's on screen stays as an unsaved squad.",
        description: "Body of the delete-slot confirm dialog.",
    },
    "squad.toolbar.deleteConfirm": {
        text: "Delete",
        description: "Confirm button in the delete-slot dialog.",
    },
    "squad.toolbar.scratch": {
        text: "Not saved to a slot",
        description: "Shown next to the squad name when the working squad does not belong to any saved slot.",
    },
    "squad.toolbar.inSlot": {
        text: "Slot {slot}",
        description: "Shown next to the squad name: which saved slot this squad belongs to.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
