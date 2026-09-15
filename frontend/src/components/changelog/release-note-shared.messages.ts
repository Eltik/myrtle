import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "note.kind.new": {
        text: "New",
        description: "Inline label in front of a release-note bullet describing something newly added. An adjective.",
    },
    "note.kind.improved": {
        text: "Improved",
        description: "Inline label in front of a release-note bullet describing something made better. A past participle.",
    },
    "note.kind.fixed": {
        text: "Fixed",
        description: "Inline label in front of a release-note bullet describing a repaired bug. A past participle.",
    },
    "note.takeALook": {
        text: "Take a look",
        description: "Default label of a release note's call to action, used when the entry names none of its own. A verb phrase.",
    },
} satisfies MessageMap;

// `dynamic`: the three bullet-category labels are stored in KIND_LABEL_KEYS and
// resolved as `t(KIND_LABEL_KEYS[item.kind])`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
