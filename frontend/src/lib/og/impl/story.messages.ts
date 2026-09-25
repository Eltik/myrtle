import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "meta";

/**
 * The authored words on a story's social card. Everything else on it is game
 * text (the story's name, its chapter, its operation code and phase), which
 * the game data layer already serves per region, and the category label is
 * the library's own `story.category.*`, reused rather than restated.
 *
 * The card draws each label in Geist Mono at 10 to 13px, uppercased, in a row
 * that holds three stats and three chips beside a 640px title column, so each
 * one has to stay a word or two.
 *
 * `dynamic: true`: `story.ts` resolves these through its own lookup, which
 * falls back to the English below when the source catalog has not been
 * re-extracted yet, so the extractor's literal `t("...")` scan cannot see them.
 */
export const messages = {
    "og.story.position": {
        text: "Story",
        description: "Stat label on a story's social card, over a value like `3 / 24`: where this story sits in its chapter. Drawn uppercase at 10px, keep it to one short word.",
    },
    "og.story.words": {
        text: "Words",
        description: "Stat label on a story's social card, over the story's word count. Drawn uppercase at 10px, keep it to one short word.",
    },
    "og.story.reading": {
        text: "Read time",
        description: "Stat label on a story's social card, over an estimate like `~7m` at 225 words a minute. Drawn uppercase at 10px, keep it short.",
    },
    "og.story.cutscene": {
        text: "Cutscene",
        description: "Chip on a story's social card when the story plays an animated cutscene. Drawn uppercase at 13px beside a play glyph, keep it to one word.",
    },
    "og.story.noScript": {
        text: "No script yet",
        description: "Chip on a story's social card when the game lists the story but ships no script for it yet (unreleased or upcoming content). Drawn uppercase at 13px, keep it short.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
