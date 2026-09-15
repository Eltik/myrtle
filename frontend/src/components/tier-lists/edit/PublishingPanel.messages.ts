import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.publishing.heading": {
        text: "Publishing",
        description: "Heading of the editor panel holding the flair, visibility and publish controls. Rendered uppercase.",
    },
    "edit.publishing.flair": {
        text: "Flair",
        description: "Label of the topic-tag picker. A flair is the tag shown on the list's browse card.",
    },
    "edit.publishing.noFlair": {
        text: "No flair",
        description: "The picker's value, and its first menu option, when the list carries no topic tag.",
    },
    "edit.publishing.flairHint": {
        text: "Tags your list with a topic shown on the browse page.",
        description: "Explains the flair picker, under it.",
    },
    "edit.publishing.visibility": {
        text: "Visibility",
        description: "Label of the switch controlling whether the list appears on the browse page.",
    },
    "edit.publishing.listed": {
        text: "Listed publicly",
        description: "The visibility switch's state when the list appears on the browse page.",
    },
    "edit.publishing.hidden": {
        text: "Hidden from browse",
        description: "The visibility switch's state when the list is reachable only by its direct link.",
    },
    "edit.publishing.listedHint": {
        text: "Anyone can find this list from /tier-lists.",
        description: "Explains the visibility switch while the list is public. '/tier-lists' is the site's own path and stays as-is.",
    },
    "edit.publishing.hiddenHint": {
        text: "Only people with the direct link can open it.",
        description: "Explains the visibility switch while the list is hidden from the browse page.",
    },
    "edit.publishing.publish": {
        text: "Publish version",
        description: "Button that opens the dialog for snapshotting the list as a new version.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
