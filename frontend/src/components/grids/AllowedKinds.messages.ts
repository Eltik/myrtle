import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "kinds.label": {
        text: "Allowed types",
        description: "Label over the set of entity types (operators, skins, enemies...) a grid's cells may hold. Shown in the create dialog, the editor and the grid page.",
    },
    "kinds.hint": {
        text: "People filling in this grid can only pick these.",
        description: "Hint under the allowed-types checkboxes in the create-grid dialog and the editor's types dialog.",
    },
    "kinds.atLeastOne": {
        text: "A grid allows at least one type.",
        description: "Shown when the user tries to untick the last allowed type.",
    },
    "kinds.placed": {
        text: "{count, plural, one {# pick} other {# picks}}",
        description: "Small count beside a type in the editor's types dialog: how many cells hold a pick of that type now.",
    },
    "kinds.more": {
        text: "+{count}",
        description: "Last chip of a shortened list of allowed types on a grid card: how many more types are allowed. {count} is a number.",
    },
    "kinds.moreTitle": {
        text: "Also allows: {kinds}",
        description: "Tooltip on the '+N' chip of a grid card. {kinds} is a comma-separated list of type names.",
    },
    "kinds.change": {
        text: "Change",
        description: "Button in the grid editor that opens the dialog for changing the allowed types.",
    },
    "kinds.locked": {
        text: "Set by the template this grid was made from.",
        description: "Note in the grid editor when the allowed types cannot be changed because the grid is a copy of a template.",
    },
    "kinds.dialog.title": {
        text: "Allowed types",
        description: "Title of the editor dialog where a grid's owner picks which entity types its cells may hold.",
    },
    "kinds.dialog.description": {
        text: "People filling in this grid can only pick these types. Changes save with your other edits.",
        description: "Subtitle of the allowed-types dialog. 'Save' refers to the editor's Save button; nothing is stored until then.",
    },
    "kinds.dialog.apply": {
        text: "Apply",
        description: "Applies the ticked types in the allowed-types dialog.",
    },
    "kinds.dialog.cancel": {
        text: "Cancel",
        description: "Closes the allowed-types dialog without changing anything.",
    },
    "kinds.clear.title": {
        text: "Clear picks?",
        description: "Title of the confirmation shown when removing an allowed type that some cells hold.",
    },
    "kinds.clear.body": {
        text: "{count, plural, one {# pick is of a type you removed and will be cleared.} other {# picks are of types you removed and will be cleared.}} Labels stay.",
        description: "Body of the confirmation shown when removing allowed types. {count} is how many cells lose their pick; their labels are kept.",
    },
    "kinds.clear.confirm": {
        text: "Clear picks",
        description: "Confirms removing the types and clearing the picks of those types.",
    },
    "kinds.clear.cancel": {
        text: "Keep types",
        description: "Cancels removing the types; nothing changes.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
