import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.pick.description": {
        text: "Choose a tier, then add an optional note. You can also drag tiles straight onto a tier.",
        description: "Explains the dialog for placing one entry (an operator, enemy, event or any other ranked thing) on a tier.",
    },
    "edit.pick.descriptionNoTiers": {
        text: "This list has no tiers yet, so there is nowhere to place this.",
        description: "Replaces the dialog's explanation when the list being edited has no tiers at all.",
    },
    "edit.pick.noTiers": {
        text: "No tiers yet. Close this dialog and add a tier to the board first.",
        description: "Shown in place of the list of tiers when the list being edited has none. 'Add a tier' refers to the editor's own button for creating a tier.",
    },
    "edit.pick.noteHintNoTiers": {
        text: "A note is saved with a placement, so add a tier first.",
        description: "Explains the note field while the list has no tiers, so the note has nothing to attach to.",
    },
    "edit.pick.placement": {
        text: "Placement",
        description: "Small uppercase label above the list of tiers to place the entry on.",
    },
    "edit.pick.current": {
        text: "Current",
        description: "Badge on the tier the entry already sits in. Rendered uppercase.",
    },
    "edit.pick.noteLabel": {
        text: "Description",
        description: "Label of the field holding the note shown with this entry's placement.",
    },
    "edit.pick.notePlaceholder": {
        text: "Why does this land here?",
        description: "Placeholder in the placement note field.",
    },
    "edit.pick.noteHintPlaced": {
        text: "Optional. Shown to viewers on this tile.",
        description: "Explains the note field once the entry is on a tier.",
    },
    "edit.pick.noteHintUnplaced": {
        text: "Pick a tier above to save this note with the placement.",
        description: "Explains the note field while the entry is not on any tier, so the note has nothing to attach to.",
    },
    "edit.pick.unplace": {
        text: "Unplace",
        description: "Button that takes the entry off its tier and back into the pool.",
    },
    "edit.pick.owner": {
        text: "Operator: {name}",
        description: "Second line under the name of a skin, module or skill in the placement dialog's title: the operator it belongs to. The name comes from the game data.",
    },
    "edit.pick.done": {
        text: "Done",
        description: "Button that closes the placement dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
