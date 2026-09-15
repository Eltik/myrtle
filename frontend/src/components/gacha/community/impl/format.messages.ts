import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Labels for the derived values `format.ts` produces. That module has no
 * React, so its label helpers take a `t` from the component that renders
 * them.
 */
export const namespace = "gacha";

export const messages = {
    "community.compare.onTarget": {
        text: "on target",
        description: "An observed drop rate that matches its baseline within one percent. Lowercase; it sits in a row of small stats.",
    },
    "community.compare.ratioOfExpected": {
        text: "{percent}% of expected",
        description: "An observed rate as a share of the expected one, e.g. '61% of expected' for a rate well under, '247% of expected' for one well over.",
    },
    "community.compare.ratioBaseline": {
        text: "{ratio}× baseline",
        description: "Used instead of a percentage once the observed rate is more than ten times the baseline, e.g. '12.4× baseline'. The × is a multiplication sign.",
    },
    "community.compare.deviation": {
        text: "{direction, select, above {above by {percent}%} other {below by {percent}%}}",
        description: "How far an observed rate sits from its baseline, scaled to that baseline: 'above by 147%', 'below by 39%'. Lowercase; it sits in a row of small stats.",
    },
    "community.relative.moments": {
        text: "moments ago",
        description: "How long ago the community figures were recomputed, for anything under a minute. Lowercase; it follows the word 'updated'.",
    },
    "community.relative.minutes": {
        text: "{count}m ago",
        description: "How long ago the community figures were recomputed, in minutes. Abbreviated to a single letter - the chip it sits in is very narrow.",
    },
    "community.relative.hours": {
        text: "{count}h ago",
        description: "How long ago the community figures were recomputed, in hours. Abbreviated to a single letter - the chip it sits in is very narrow.",
    },
    "community.relative.days": {
        text: "{count}d ago",
        description: "How long ago the community figures were recomputed, in days. Abbreviated to a single letter - the chip it sits in is very narrow.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
