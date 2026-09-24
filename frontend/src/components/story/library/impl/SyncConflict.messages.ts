import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "progress.sync.conflict.title": {
        text: "This browser and your account disagree",
        description: "Heading of the panel shown when local and account reading progress differ and neither has been written.",
    },
    "progress.sync.conflict.blurb": {
        text: "Nothing has been changed yet. Choose which reading progress to keep.",
        description: "Explanation under the conflict heading, telling the reader that no document has been written.",
    },
    "progress.sync.conflict.browser": {
        text: "This browser",
        description: "Column heading for the reading progress stored in the current browser.",
    },
    "progress.sync.conflict.account": {
        text: "Your account",
        description: "Column heading for the reading progress stored on the account.",
    },
    "progress.sync.conflict.newer": {
        text: "newer",
        description: "Badge marking the side of the comparison that carries the more recent reading activity.",
    },
    "progress.sync.conflict.marks": {
        text: "Finished stories",
        description: "Row label in the conflict comparison, counting stories marked read.",
    },
    "progress.sync.conflict.positions": {
        text: "Saved positions",
        description: "Row label in the conflict comparison, counting stories with a saved reading position.",
    },
    "progress.sync.conflict.last": {
        text: "Last read",
        description: "Row label in the conflict comparison, naming the story each side was read last.",
    },
    "progress.sync.conflict.lastNone": {
        text: "None",
        description: "Value of the last-read row when the side names no story.",
    },
    "progress.sync.conflict.merge": {
        text: "Merge both",
        description: "Primary button that keeps everything from both sides.",
    },
    "progress.sync.conflict.mergeHint": {
        text: "Keeps everything from both: {marks} finished stories and {positions} saved positions.",
        description: "Subtitle under the merge button, quoting what the merged document would hold.",
    },
    "progress.sync.conflict.useAccount": {
        text: "Use account",
        description: "Button that replaces this browser's reading progress with the account's.",
    },
    "progress.sync.conflict.useAccountHint": {
        text: "Replaces this browser: {marks} finished stories and {positions} saved positions are dropped.",
        description: "Subtitle under the use-account button, quoting what this browser loses.",
    },
    "progress.sync.conflict.keepBrowser": {
        text: "Keep browser",
        description: "Button that replaces the account's reading progress with this browser's.",
    },
    "progress.sync.conflict.keepBrowserHint": {
        text: "Replaces your account: {marks} finished stories and {positions} saved positions are dropped.",
        description: "Subtitle under the keep-browser button, quoting what the account loses.",
    },
    "progress.sync.policy": {
        text: "When they differ next time",
        description: "Label of the select that decides what happens the next time the two sides disagree.",
    },
    "progress.sync.policy.ask": {
        text: "Ask me",
        description: "Policy option: stop and show the comparison every time the two sides disagree.",
    },
    "progress.sync.policy.merge": {
        text: "Merge",
        description: "Policy option: always keep everything from both sides without asking.",
    },
    "progress.sync.policy.account": {
        text: "Account wins",
        description: "Policy option: always replace this browser's reading progress with the account's.",
    },
    "progress.sync.policy.browser": {
        text: "Browser wins",
        description: "Policy option: always replace the account's reading progress with this browser's.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
