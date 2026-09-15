import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The tag-group headings in `constants.ts` are plain data in a module with no
 * React, so they carry message KEYS and `TagSelector` resolves them with
 * `t()`.
 *
 * Only the group headings live here. The tags themselves - Guard, Melee, Top
 * Operator, DPS - are recruitment game data, and so are the profession labels
 * in `PROFESSION_LABELS`, so both are deliberately absent.
 */
export const namespace = "tools";

export const messages = {
    "recruit.group.qualification": {
        text: "Qualification",
        description: "Heading over the recruitment tags that describe an operator's standing (Top Operator, Senior Operator, Starter, Robot).",
    },
    "recruit.group.position": {
        text: "Position",
        description: "Heading over the recruitment tags for where an operator is deployed (Melee, Ranged).",
    },
    "recruit.group.class": {
        text: "Class",
        description: "Heading over the recruitment tags naming an operator's class (Guard, Sniper, ...).",
    },
    "recruit.group.affix": {
        text: "Affix",
        description: "Heading over the remaining recruitment tags, which describe what an operator does (DPS, Healing, Slow, ...). 'Affix' is this site's own name for that leftover group.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a constants entry and resolved by the
// consuming component as `t(item.labelKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
