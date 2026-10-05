import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "edit.breadcrumbMine": {
        text: "My grids",
        description: "Breadcrumb link from the grid editor back to the signed-in user's grids.",
    },
    "edit.breadcrumbUntitled": {
        text: "Untitled",
        description: "Breadcrumb stand-in for a grid whose title is empty.",
    },
    "edit.breadcrumbEdit": {
        text: "Edit",
        description: "Last breadcrumb in the grid editor: the current page.",
    },
    "edit.kicker": {
        text: "Editing grid",
        description: "Small uppercase label above the grid editor's title field.",
    },
    "edit.titleLabel": {
        text: "Grid title",
        description: "Accessible label of the grid editor's title field.",
    },
    "edit.titlePlaceholder": {
        text: "Name your grid",
        description: "Placeholder of the grid editor's title field.",
    },
    "edit.titleRequired": {
        text: "A grid needs a title before it can be saved.",
        description: "Shown in the save panel while the title is empty.",
    },
    "edit.descriptionLabel": {
        text: "Description",
        description: "Accessible label of the grid editor's description field.",
    },
    "edit.descriptionPlaceholder": {
        text: "What is this grid about? (optional)",
        description: "Placeholder of the grid editor's description field.",
    },
    "edit.rows": {
        text: "Rows",
        description: "Label of the stepper that sets how many rows the grid has (1 to 10). Rendered uppercase.",
    },
    "edit.cols": {
        text: "Columns",
        description: "Label of the stepper that sets how many columns the grid has (1 to 10). Rendered uppercase.",
    },
    "edit.listed": {
        text: "Listed",
        description: "Label of the switch that shows the grid in the public browse list.",
    },
    "edit.listedOn": {
        text: "Shown in Browse grids",
        description: "Hint under the Listed switch when it is on.",
    },
    "edit.listedOff": {
        text: "Only people with the link can see it",
        description: "Hint under the Listed switch when it is off: the grid stays reachable by its link.",
    },
    "edit.unsaved": {
        text: "Unsaved changes",
        description: "Status line in the editor's save panel while there are changes to save. Rendered uppercase.",
    },
    "edit.unsavedBody": {
        text: "Save to publish your changes.",
        description: "Line under 'Unsaved changes' in the save panel.",
    },
    "edit.allSaved": {
        text: "All saved",
        description: "Status line in the editor's save panel when nothing has changed. Rendered uppercase.",
    },
    "edit.upToDate": {
        text: "Your grid is up to date.",
        description: "Line under 'All saved' in the save panel.",
    },
    "edit.save": {
        text: "Save",
        description: "Button that saves the grid.",
    },
    "edit.discard": {
        text: "Discard changes",
        description: "Accessible name and tooltip of the icon button that drops every unsaved change.",
    },
    "edit.openPublic": {
        text: "Open public page",
        description: "Accessible name and tooltip of the icon button that opens the grid's public page in a new tab.",
    },
    "edit.hint": {
        text: "Click a picture to choose what goes in it, click a label to rename it, drag a cell onto another to swap them.",
        description: "Help line under the board in the grid editor.",
    },
    "edit.shrink.title": {
        text: "Remove filled cells?",
        description: "Title of the dialog shown when making the grid smaller would delete cells that have a label or a pick.",
    },
    "edit.shrink.body": {
        text: "{count, plural, one {# cell with a label or pick falls outside} other {# cells with a label or pick fall outside}} a {rows} by {cols} grid and will be removed.",
        description: "Body of the shrink dialog. {count} is how many filled cells are lost; {rows} and {cols} are the new size.",
    },
    "edit.shrink.confirm": {
        text: "Remove and resize",
        description: "Confirms the shrink, deleting the cells that fall outside.",
    },
    "edit.shrink.cancel": {
        text: "Keep size",
        description: "Cancels the shrink; the grid keeps its current size.",
    },
    "edit.leave.title": {
        text: "Leave without saving?",
        description: "Title of the dialog shown when leaving the grid editor with unsaved changes.",
    },
    "edit.leave.body": {
        text: "Your unsaved changes to this grid will be lost.",
        description: "Body of the leave dialog.",
    },
    "edit.leave.confirm": {
        text: "Leave",
        description: "Leaves the editor, dropping unsaved changes.",
    },
    "edit.leave.cancel": {
        text: "Stay",
        description: "Closes the leave dialog and stays in the editor.",
    },
    "edit.toast.savedTitle": {
        text: "Grid saved",
        description: "Toast title after a successful save.",
    },
    "edit.toast.savedBody": {
        text: "Your changes are live.",
        description: "Toast body after a successful save.",
    },
    "edit.toast.saveFailedTitle": {
        text: "Could not save",
        description: "Toast title when saving the grid failed; the body is the error.",
    },
    "edit.missing.title": {
        text: "Grid not found",
        description: "Heading of the editor when the grid does not exist.",
    },
    "edit.missing.body": {
        text: "It may have been deleted, or the link is wrong.",
        description: "Body of the editor's not-found notice.",
    },
    "edit.missing.action": {
        text: "Back to my grids",
        description: "Button from the not-found notice to the user's grids.",
    },
    "edit.forbidden.title": {
        text: "You can't edit this grid",
        description: "Heading of the editor when the viewer does not own the grid.",
    },
    "edit.forbidden.body": {
        text: "Only its owner can change it. You can still use it as a template.",
        description: "Body of the editor's forbidden notice.",
    },
    "edit.forbidden.action": {
        text: "View the grid",
        description: "Button from the forbidden notice to the grid's public page.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
