import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "marks.markRead": {
        text: "Mark {story} read",
        description: "Accessible name of an unfilled tick beside a story, which marks it read when clicked.",
    },
    "marks.markUnread": {
        text: "Mark {story} unread",
        description: "Accessible name of a filled tick beside a story, which withdraws the read mark when clicked.",
    },
    "marks.source.own": {
        text: "You marked this read",
        description: "Tooltip on a tick the reader filled themselves.",
    },
    "marks.source.game": {
        text: "Read in game",
        description: "Tooltip on a tick filled by the Arknights client's own record rather than by the reader here.",
    },
    "marks.source.cleared": {
        text: "Cleared by you",
        description: "Tooltip on an empty tick whose story the game says was played, which the reader marked unread here.",
    },
    "marks.markAll": {
        text: "Mark all read",
        description: "Chapter sheet action that marks every story in the chapter read.",
    },
    "marks.clearAll": {
        text: "Clear all",
        description: "Chapter sheet action that withdraws every read mark in the chapter.",
    },
    "marks.clearAll.confirm": {
        text: "Clear {count} marks? Cleared stories stay unread even if the game says you read them.",
        description: "Inline confirmation shown in place of the chapter actions after Clear all is pressed. {count} is how many stories count as read. The second sentence says that a hand clear outranks the game's verdict.",
    },
    "marks.clearAll.yes": {
        text: "Confirm",
        description: "Button that confirms clearing a chapter's read marks.",
    },
    "marks.clearAll.no": {
        text: "Cancel",
        description: "Button that dismisses the inline clear-marks confirmation.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
