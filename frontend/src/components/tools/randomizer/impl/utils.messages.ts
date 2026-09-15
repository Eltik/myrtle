import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Labels the stage grouping in `utils.ts` derives. That module has no React,
 * so `buildStageGroups` takes an optional `t` and falls back to the bundled
 * source catalog. Chapter, event and zone NAMES come from the game data and
 * are never translated here.
 */
export const namespace = "tools";

export const messages = {
    "randomizer.stages.chapter": {
        text: "Chapter {number} - {name}",
        description: "Label of one main-story chapter group, e.g. 'Chapter 8 - Roaring Flare'. {name} is the chapter's own name from the game data. The dash is a plain hyphen.",
    },
    "randomizer.stages.permanent": {
        text: "Permanent",
        description: "Sub-label marking an event whose stages are always available. Sits under the event name in small caps.",
    },
    "randomizer.stages.rerun": {
        text: "Rerun",
        description: "Sub-label marking a repeat showing of a past event. Sits under the event name in small caps.",
    },
    "randomizer.section.main": {
        text: "Main Story",
        description: "Heading over the main-story chapters in the stage picker.",
    },
    "randomizer.section.event": {
        text: "Events",
        description: "Heading over the full side stories in the stage picker.",
    },
    "randomizer.section.other": {
        text: "Other",
        description: "Heading over the leftover content in the stage picker - mini events, sandbox modes, festivals.",
    },
} satisfies MessageMap;

// `dynamic`: the section keys are stored in a lookup table and resolved by the
// consuming component as `t(STAGE_SECTION_LABEL_KEYS[section])`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
