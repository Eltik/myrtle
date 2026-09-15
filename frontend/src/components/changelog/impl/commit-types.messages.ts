import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "commitType.feature": {
        text: "Feature",
        description: "Badge on a commit in the changelog feed: the commit added something new. A noun. Rendered uppercase in a small pill.",
    },
    "commitType.fix": {
        text: "Fix",
        description: "Badge on a commit in the changelog feed: the commit repaired a bug. A noun. Rendered uppercase in a small pill.",
    },
    "commitType.perf": {
        text: "Perf",
        description: "Badge on a commit in the changelog feed: a performance improvement. Short for 'performance'; keep it short, the pill is narrow. Rendered uppercase.",
    },
    "commitType.refactor": {
        text: "Refactor",
        description: "Badge on a commit in the changelog feed: internal restructuring with no behaviour change. A noun. Rendered uppercase.",
    },
    "commitType.docs": {
        text: "Docs",
        description: "Badge on a commit in the changelog feed: a documentation change. Short for 'documentation'. Rendered uppercase.",
    },
    "commitType.style": {
        text: "Style",
        description: "Badge on a commit in the changelog feed: formatting or whitespace only. A noun. Rendered uppercase.",
    },
    "commitType.test": {
        text: "Test",
        description: "Badge on a commit in the changelog feed: a change to the automated tests. A noun. Rendered uppercase.",
    },
    "commitType.chore": {
        text: "Chore",
        description: "Badge on a commit in the changelog feed: routine maintenance. A noun. Rendered uppercase.",
    },
    "commitType.revert": {
        text: "Revert",
        description: "Badge on a commit in the changelog feed: the commit undid an earlier one. A noun. Rendered uppercase.",
    },
    "commitType.other": {
        text: "Update",
        description: "Badge on a commit in the changelog feed that fits none of the other categories. A noun. Rendered uppercase.",
    },
} satisfies MessageMap;

// `dynamic`: these labels are stored in the COMMIT_TYPE_STYLES table and
// resolved as `t(style.labelKey)`, so the extractor has no literal call site to
// match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
