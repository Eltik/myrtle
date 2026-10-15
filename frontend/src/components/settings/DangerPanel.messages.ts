import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "danger.delete.title": {
        text: "Delete your data",
        description: "Card title of the account-deletion section. Rendered in the destructive colour.",
    },
    "danger.delete.desc": {
        text: "Wipe your Myrtle account and everything we've stored for it. It happens right away and can't be undone.",
        description: "Card description warning what account deletion does.",
    },
    "danger.delete.action": {
        text: "Delete my data",
        description: "Button that opens the confirmation dialog for deleting the account and all its data.",
    },
    "danger.delete.note": {
        text: "Can't sign in anymore? Email {email} from the address on your Yostar account and we'll delete it for you.",
        description: "Note under the delete button, the fallback for someone who can't sign in. {email} is the deletion address, shown in monospace; move it wherever the sentence needs it.",
    },
    "danger.delete.noteEmail": {
        text: "privacy@myrtle.moe",
        description: "The deletion address substituted into danger.delete.note as {email}, shown in monospace. Leave it exactly as it is.",
    },
    "danger.dialog.title": {
        text: "Delete everything?",
        description: "Title of the confirmation dialog for deleting the account and all its data.",
    },
    "danger.dialog.body": {
        text: "This deletes your profile, synced game data, scores, gacha history, planner plans, grids, the tier lists you made, your settings, and the saved Yostar login. There's no undo.",
        description: "Dialog body listing what account deletion removes.",
    },
    "danger.dialog.kept": {
        text: "Official tier lists and translations you worked on stay up, just without your name. You can sign in again later, but you'll start from scratch.",
        description: "Dialog note on what survives deletion (shared content, anonymized) and what signing in again means.",
    },
    "danger.dialog.confirmLabel": {
        text: "Type {word} to confirm",
        description: "Label above the confirmation text field. {word} is danger.dialog.confirmWord, shown in monospace; move it wherever the sentence needs it.",
    },
    "danger.dialog.confirmWord": {
        text: "delete",
        description: "The word the user must type to enable the delete button. Translate it as one short lowercase word that is easy to type; it is compared ignoring case and surrounding spaces.",
    },
    "danger.dialog.cancel": {
        text: "Cancel",
        description: "Closes the deletion dialog without deleting anything.",
    },
    "danger.dialog.submit": {
        text: "Delete my data",
        description: "Confirms the deletion. Enabled only once the confirmation word is typed.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
