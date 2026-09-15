import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The three promotion tiers. The rows are a plain constant, so they hold
 * message KEYS and the card resolves them with `t()`. 'Elite' is the game's own
 * promotion tier - a translation should follow the game's wording.
 */
export const namespace = "user";

export const messages = {
    "profile.stats.elite.title": {
        text: "Elite Promotion",
        description: "Heading of the card breaking the roster down by promotion tier.",
    },
    "profile.stats.elite.e2": {
        text: "Elite 2",
        description: "Row label: operators at the game's second Elite promotion.",
    },
    "profile.stats.elite.e1": {
        text: "Elite 1",
        description: "Row label: operators at the game's first Elite promotion.",
    },
    "profile.stats.elite.e0": {
        text: "Elite 0",
        description: "Row label: operators not yet promoted.",
    },
} satisfies MessageMap;

// `dynamic`: the row labels are stored in the ELITE_ROWS table and resolved as
// `t(row.labelKey)`, so the extractor has no literal call site to match them
// against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
