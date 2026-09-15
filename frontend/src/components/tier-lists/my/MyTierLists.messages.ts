import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "my.toast.createdTitle": {
        text: "List created",
        description: "Toast title after a new tier list was created.",
    },
    "my.toast.createdBody": {
        text: '"{name}" is ready to edit.',
        description: "Toast body naming the list just created. {name} is the author's own title; keep the quotes.",
    },
    "my.toast.createFailed": {
        text: "Couldn't create list.",
        description: "Stand-in message when creating the list failed and the server sent none of its own.",
    },
    "my.toast.updatedTitle": {
        text: "List updated",
        description: "Toast title after the list's name or description was saved.",
    },
    "my.toast.updatedBody": {
        text: "Your changes are live.",
        description: "Toast body after the list's name or description was saved.",
    },
    "my.toast.updateFailed": {
        text: "Couldn't save changes.",
        description: "Stand-in message when saving the details failed and the server sent none of its own.",
    },
    "my.toast.deletedTitle": {
        text: "List deleted",
        description: "Toast title after a tier list was deleted.",
    },
    "my.toast.deletedBody": {
        text: '"{name}" has been removed.',
        description: "Toast body naming the list just deleted. {name} is the author's own title; keep the quotes.",
    },
    "my.toast.deletedFallbackName": {
        text: "List",
        description: "Stand-in for the deleted list's name in the toast when the name is no longer to hand.",
    },
    "my.toast.deleteFailed": {
        text: "Couldn't delete list.",
        description: "Stand-in message when deleting the list failed and the server sent none of its own.",
    },
    "my.toast.copiedTitle": {
        text: "Link copied",
        description: "Toast title after the list's share link was copied.",
    },
    "my.toast.copiedBody": {
        text: "The share link is on your clipboard.",
        description: "Toast body after the list's share link was copied.",
    },
    "my.toast.copyFailedTitle": {
        text: "Couldn't copy",
        description: "Toast title when copying the share link failed.",
    },
    "my.toast.copyFailedBody": {
        text: "Clipboard access was denied.",
        description: "Toast body when the browser refused clipboard access.",
    },
    "my.error": {
        text: "Couldn't load your tier lists.",
        description: "Shown in place of the grid when the player's own lists could not be fetched.",
    },
    "my.retry": {
        text: "Retry",
        description: "Button that re-runs the failed request for the player's lists.",
    },
    "my.empty.title": {
        text: "Your workshop is empty",
        description: "Empty state heading when the player has no tier lists at all.",
    },
    "my.empty.body": {
        text: "Create your first tier list to start ranking operators. You can publish it instantly and share it with anyone.",
        description: "Empty state body inviting the player to make their first list.",
    },
    "my.empty.action": {
        text: "Create your first list",
        description: "Button in the empty state that opens the create dialog.",
    },
    "my.filtered.emptyTitle": {
        text: "No lists match these filters.",
        description: "Empty state heading when the search or type filter leaves nothing.",
    },
    "my.filtered.emptyBody": {
        text: "Try clearing the search or switching the type filter.",
        description: "Empty state body suggesting how to get results back.",
    },
    "my.filtered.clear": {
        text: "Clear filters",
        description: "Button that empties the search box and switches the type tab back to All.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
