import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.hero.breadcrumbMine": {
        text: "My Lists",
        description: "Breadcrumb link back to the player's own tier lists.",
    },
    "edit.hero.breadcrumbUntitled": {
        text: "Untitled",
        description: "Breadcrumb stand-in for a list whose title is still empty.",
    },
    "edit.hero.breadcrumbEdit": {
        text: "Edit",
        description: "Last breadcrumb crumb, naming the page the visitor is on.",
    },
    "edit.hero.kicker": {
        text: "Editing",
        description: "Small uppercase label above the title field of the editor.",
    },
    "edit.hero.titleLabel": {
        text: "List title",
        description: "Screen-reader-only label of the list title field.",
    },
    "edit.hero.titlePlaceholder": {
        text: "Untitled tier list",
        description: "Placeholder in the list title field.",
    },
    "edit.hero.descriptionLabel": {
        text: "Description",
        description: "Screen-reader-only label of the list description editor.",
    },
    "edit.hero.descriptionPlaceholder": {
        text: "Add a short description so viewers know what this list is about.",
        description: "Placeholder in the list description editor.",
    },
    "edit.hero.titleCounter": {
        text: "/ {max} title",
        description: "Follows the typed character count under the fields, e.g. '12 / 80 title'. The count itself is rendered just before it. Rendered uppercase.",
    },
    "edit.hero.descriptionCounter": {
        text: "/ {max} description",
        description: "Follows the typed character count under the fields, e.g. '340 / 4000 description'. The count itself is rendered just before it. Rendered uppercase.",
    },
    "edit.hero.unsaved": {
        text: "Unsaved changes",
        description: "Heading of the panel listing edits not yet written to the server. Rendered uppercase.",
    },
    "edit.hero.allSaved": {
        text: "All saved",
        description: "Heading of the same panel when there is nothing left to save. Rendered uppercase.",
    },
    "edit.hero.moreChanges": {
        text: "+{count} more",
        description: "Last line of the pending-change list when there are more changes than it shows.",
    },
    "edit.hero.upToDate": {
        text: "Your list is up to date with the server.",
        description: "Shown in place of the pending-change list when nothing has been edited.",
    },
    "edit.hero.save": {
        text: "Save changes",
        description: "Button that writes every pending edit to the server.",
    },
    "edit.hero.discard": {
        text: "Discard unsaved changes",
        description: "Accessible name of the button that throws away every pending edit.",
    },
    "edit.hero.addTier": {
        text: "Add tier",
        description: "Button that appends a new tier to the board.",
    },
    "edit.hero.openPublic": {
        text: "Open public view in new tab",
        description: "Accessible name of the button that opens the list's public page in another tab.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
