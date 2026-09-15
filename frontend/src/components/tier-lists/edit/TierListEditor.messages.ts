import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.board.label": {
        text: "Edit board for {title}",
        description: "Accessible name of the editable tier board. {title} is the list's own title, written by its author.",
    },
    "edit.board.emptyTitle": {
        text: "No tiers yet.",
        description: "Shown in place of the board when the list has no tiers.",
    },
    "edit.board.emptyBody": {
        text: "Create your first tier to start ranking operators.",
        description: "Second line of the empty board state in the editor.",
    },
    "edit.board.addTier": {
        text: "Add tier",
        description: "Button in the empty board state that creates the list's first tier.",
    },
    "edit.toast.savedTitle": {
        text: "Saved",
        description: "Toast title after the editor's pending changes were written.",
    },
    "edit.toast.savedBody": {
        text: "Your changes are live.",
        description: "Toast body after the editor's pending changes were written.",
    },
    "edit.toast.saveFailedTitle": {
        text: "Save failed",
        description: "Toast title when writing the editor's changes failed.",
    },
    "edit.toast.saveFailedBody": {
        text: "Couldn't save changes.",
        description: "Stand-in message when the save failed and the server sent none of its own.",
    },
    "edit.toast.flairSetTitle": {
        text: "Flair updated",
        description: "Toast title after the list's topic tag was changed.",
    },
    "edit.toast.flairSetBody": {
        text: 'Tagged as "{label}".',
        description: "Toast body naming the new topic tag. {label} is the tag's own name; keep the quotes.",
    },
    "edit.toast.flairClearedTitle": {
        text: "Flair cleared",
        description: "Toast title after the list's topic tag was removed.",
    },
    "edit.toast.flairClearedBody": {
        text: "No flair on this list.",
        description: "Toast body after the list's topic tag was removed.",
    },
    "edit.toast.flairFailedTitle": {
        text: "Flair failed",
        description: "Toast title when changing the topic tag failed.",
    },
    "edit.toast.flairFailedBody": {
        text: "Couldn't update flair.",
        description: "Stand-in message when changing the topic tag failed and the server sent none of its own.",
    },
    "edit.toast.publicTitle": {
        text: "Now public",
        description: "Toast title after the list was made visible on the browse page.",
    },
    "edit.toast.publicBody": {
        text: "This list appears on /tier-lists.",
        description: "Toast body after the list was made public. '/tier-lists' is the site's own path and stays as-is.",
    },
    "edit.toast.hiddenTitle": {
        text: "Hidden from browse",
        description: "Toast title after the list was hidden from the browse page.",
    },
    "edit.toast.hiddenBody": {
        text: "Only people with the direct link can find it.",
        description: "Toast body after the list was hidden from the browse page.",
    },
    "edit.toast.visibilityFailedTitle": {
        text: "Visibility failed",
        description: "Toast title when changing the list's visibility failed.",
    },
    "edit.toast.visibilityFailedBody": {
        text: "Couldn't update visibility.",
        description: "Stand-in message when changing visibility failed and the server sent none of its own.",
    },
    "edit.toast.publishedTitle": {
        text: "Published v{version}",
        description: "Toast title after a version was published, naming it. Keep the 'v' before the number.",
    },
    "edit.toast.publishedWithChangelog": {
        text: "Your changelog is now live.",
        description: "Toast body after publishing a version that carried a changelog.",
    },
    "edit.toast.publishedNoChangelog": {
        text: "Snapshot saved.",
        description: "Toast body after publishing a version with no changelog.",
    },
    "edit.publishFailed": {
        text: "Couldn't publish version.",
        description: "Stand-in message when publishing failed and the server sent none of its own.",
    },
    "edit.publishBlocked": {
        text: "Save your changes before publishing a version.",
        description: "Explains why the publish button is disabled: a version snapshots what is saved, so pending edits have to be written first.",
    },
    "edit.missing.title": {
        text: "Tier list not found",
        description: "Heading shown when the list being edited does not exist.",
    },
    "edit.missing.body": {
        text: "It may have been removed, or the link could be wrong.",
        description: "Paragraph under the not-found heading in the editor.",
    },
    "edit.missing.action": {
        text: "Back to my lists",
        description: "Button from the editor's not-found page back to the player's own lists.",
    },
    "edit.forbidden.title": {
        text: "You can't edit this list",
        description: "Heading shown to a visitor without permission to edit this list.",
    },
    "edit.forbidden.body": {
        text: "You don't have permission to edit this tier list.",
        description: "Paragraph under the no-permission heading.",
    },
    "edit.forbidden.action": {
        text: "View public page",
        description: "Button that takes a visitor without edit rights to the list's public page.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
