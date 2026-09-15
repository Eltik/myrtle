import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Step labels for the save progress bar. `save.ts` has no React, so
 * `saveEdits` takes a `t` from the component that renders its progress.
 */
export const namespace = "tierLists";

export const messages = {
    "edit.save.listDetails": {
        text: "Saving list details",
        description: "Save step: writing the list's title and description.",
    },
    "edit.save.reordering": {
        text: 'Reordering "{name}"',
        description: "Save step: moving a tier to its new position. {name} is the author's own tier label; keep the quotes.",
    },
    "edit.save.deletingTier": {
        text: 'Deleting tier "{name}"',
        description: "Save step: removing a tier. {name} is the author's own tier label; keep the quotes.",
    },
    "edit.save.creatingTier": {
        text: 'Creating tier "{name}"',
        description: "Save step: adding a new tier. {name} is the author's own tier label; keep the quotes.",
    },
    "edit.save.settling": {
        text: 'Settling "{name}"',
        description: "Save step: putting a reordered tier into its final position after the others have moved. {name} is the author's own tier label; keep the quotes.",
    },
    "edit.save.updatingTier": {
        text: 'Updating "{name}"',
        description: "Save step: writing a tier's edited label, colour or description. {name} is the author's own tier label; keep the quotes.",
    },
    "edit.save.removingOperator": {
        text: "Removing operator",
        description: "Save step: taking one operator off the board.",
    },
    "edit.save.movingOperator": {
        text: "Moving operator",
        description: "Save step: moving one operator to another tier or another position.",
    },
    "edit.save.placingOperator": {
        text: "Placing operator",
        description: "Save step: adding one operator to a tier.",
    },
    "edit.save.updatingDescription": {
        text: "Updating description",
        description: "Save step: writing the note attached to one placed operator.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
