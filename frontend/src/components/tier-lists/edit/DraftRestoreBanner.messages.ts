import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.draft.title": {
        text: "Restore unsaved changes?",
        description: "Heading of the banner offering edits this browser kept from an editing session that ended before they were saved.",
    },
    "edit.draft.body": {
        text: "This browser kept edits to this list that were never saved. Last edit: {when}.",
        description: "Banner body. {when} is when the edits were last made, as a relative time such as '5 min ago' or a date.",
    },
    "edit.draft.diverged": {
        text: "The list has been saved since then. Restoring puts your older edits back, and saving them would overwrite what changed.",
        description: "Extra warning in the banner when the list on the server changed after these edits were kept.",
    },
    "edit.draft.restore": {
        text: "Restore",
        description: "Button that loads the kept edits back into the editor. Nothing is saved until the author saves.",
    },
    "edit.draft.discard": {
        text: "Discard",
        description: "Button that throws the kept edits away.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
