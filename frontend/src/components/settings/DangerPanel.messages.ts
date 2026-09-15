import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "danger.delete.title": {
        text: "Delete account",
        description: "Card title of the account-deletion section. Rendered in the destructive colour.",
    },
    "danger.delete.desc": {
        text: "Permanently remove your profile, synced game data, scores, and any saved configurations from our servers. This cannot be undone.",
        description: "Card description warning what account deletion removes.",
    },
    "danger.delete.action": {
        text: "Email privacy@myrtle.moe to delete",
        description: "Button that opens a pre-addressed deletion request. The address is literal and must not be changed.",
    },
    "danger.delete.note": {
        text: "Self-serve deletion isn't exposed via the API yet. Email {email} from the address linked to your Yostar account and we'll process it within 30 days, per the privacy policy.",
        description: "Note under the deletion button. {email} is the deletion address, shown in monospace; move it wherever the sentence needs it.",
    },
    "danger.delete.noteEmail": {
        text: "privacy@myrtle.moe",
        description: "The deletion address substituted into danger.delete.note as {email}, shown in monospace. Leave it exactly as it is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
