import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "calc.share.copyLink": {
        text: "Copy link",
        description: "Tooltip and accessible name of the icon button beside the PNG download that copies a link reproducing the whole calculator setup (cards, enemy or buffs, chart controls).",
    },
    "calc.share.copied": {
        text: "Link copied",
        description: "Toast after the calculator's setup link was copied to the clipboard.",
    },
    "calc.share.copyFailed": {
        text: "Couldn't copy the link",
        description: "Toast when the browser refused clipboard access for the calculator's setup link.",
    },
    "calc.share.invalid": {
        text: "This link's setup couldn't be read, so your saved setup is shown instead",
        description: "Warning toast when a shared calculator link is damaged or comes from an incompatible version of the page.",
    },
    "calc.share.skipped": {
        text: "{count, plural, one {# card from the link was skipped: its operator isn't available} other {# cards from the link were skipped: their operators aren't available}}",
        description: "Warning toast after opening a shared calculator link when some of its operator cards could not be restored (operator no longer supported, or the entry was damaged).",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
