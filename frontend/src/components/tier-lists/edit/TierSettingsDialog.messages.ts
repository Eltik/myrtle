import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.tierSettings.title": {
        text: "Tier settings",
        description: "Title of the dialog for editing one tier.",
    },
    "edit.tierSettings.description": {
        text: "Customize the label, color, and description for this tier. Changes take effect when you save the list.",
        description: "Explains the tier settings dialog.",
    },
    "edit.tierSettings.label": {
        text: "Label",
        description: "Field label for the tier's own short name, such as S or A.",
    },
    "edit.tierSettings.labelPlaceholder": {
        text: "S, A, Pick-One, etc.",
        description: "Placeholder in the tier label field, giving examples of short tier names.",
    },
    "edit.tierSettings.labelHint": {
        text: 'Short labels like "S" or "A" read clearest; up to {max} characters.',
        description: "Explains the tier label field and gives its character limit. The quoted letters are example tier names.",
    },
    "edit.tierSettings.color": {
        text: "Color",
        description: "Field label for the tier's colour.",
    },
    "edit.tierSettings.descriptionLabel": {
        text: "Description",
        description: "Field label for the tier's description.",
    },
    "edit.tierSettings.descriptionPlaceholder": {
        text: "When should an operator land here?",
        description: "Placeholder in the tier description field.",
    },
    "edit.tierSettings.descriptionHint": {
        text: "Optional. Shown to viewers in the tier hover/detail.",
        description: "Explains where the tier description appears: in the hover card and the tier details dialog.",
    },
    "edit.tierSettings.operatorCount": {
        text: "{count, plural, one {operator} other {operators}} in this tier",
        description: "Follows the operator count at the foot of the dialog; the number itself is rendered just before it.",
    },
    "edit.tierSettings.clearConfirmPrompt": {
        text: "Remove all?",
        description: "Asks the author to confirm emptying the tier of every operator.",
    },
    "edit.tierSettings.clear": {
        text: "Clear operators",
        description: "Button that takes every operator out of this tier.",
    },
    "edit.tierSettings.deleteConfirmPrompt": {
        text: "Delete this tier?",
        description: "Asks the author to confirm deleting the tier.",
    },
    "edit.tierSettings.delete": {
        text: "Delete tier",
        description: "Button that deletes this tier from the list.",
    },
    "edit.tierSettings.lastTier": {
        text: "Lists must have at least one tier.",
        description: "Shown in place of the delete button when this is the list's only tier.",
    },
    "edit.tierSettings.confirm": {
        text: "Confirm",
        description: "Button confirming a destructive action the author just asked for.",
    },
    "edit.tierSettings.cancel": {
        text: "Cancel",
        description: "Button that backs out of a confirmation, or closes the dialog without saving.",
    },
    "edit.tierSettings.save": {
        text: "Save tier",
        description: "Button that applies the edited label, colour and description to the tier.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
