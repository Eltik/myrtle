import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The `STAT_HEADERS` set in `LoreContent.tsx` is deliberately not here: those
 * strings are matched against the game's own handbook text to decide which
 * paragraphs are data rows, so they are parsing input, not labels.
 */
export const namespace = "operators";

export const messages = {
    "lore.title": {
        text: "Operator Files",
        description: "Heading of the Lore tab.",
    },
    "lore.subtitle": {
        text: "Personal records, archives, and classified documents.",
        description: "Blurb under the Lore tab heading, written in the voice of the in-game archive.",
    },
    "lore.archives": {
        text: "Archive Files",
        description: "Heading over the list of handbook sections. The section titles themselves come from the game data.",
    },
    "lore.empty.title": {
        text: "No Records Available",
        description: "Empty state heading when the game data carries no handbook text for this operator.",
    },
    "lore.empty.body": {
        text: "This operator''s files are classified or not yet documented.",
        description: "Empty-state body, in the voice of the in-game archive. The doubled apostrophe is ICU escaping and renders as one.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
