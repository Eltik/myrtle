import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.pick.description": {
        text: "Choose a tier, then add an optional note. You can also drag operators directly onto a tier.",
        description: "Explains the dialog for placing one operator on a tier.",
    },
    "edit.pick.placement": {
        text: "Placement",
        description: "Small uppercase label above the list of tiers to place the operator on.",
    },
    "edit.pick.current": {
        text: "Current",
        description: "Badge on the tier the operator already sits in. Rendered uppercase.",
    },
    "edit.pick.noteLabel": {
        text: "Description",
        description: "Label of the field holding the note shown with this operator's placement.",
    },
    "edit.pick.notePlaceholder": {
        text: "Why does this operator land here?",
        description: "Placeholder in the placement note field.",
    },
    "edit.pick.noteHintPlaced": {
        text: "Optional. Shown to viewers on this operator's tile.",
        description: "Explains the note field once the operator is on a tier.",
    },
    "edit.pick.noteHintUnplaced": {
        text: "Pick a tier above to save this note with the placement.",
        description: "Explains the note field while the operator is not on any tier, so the note has nothing to attach to.",
    },
    "edit.pick.unplace": {
        text: "Unplace",
        description: "Button that takes the operator off its tier and back into the pool.",
    },
    "edit.pick.done": {
        text: "Done",
        description: "Button that closes the placement dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
