import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The two graded components of the Base subscore. The table is a plain
 * constant, so it holds message KEYS and the panel resolves them with `t()`.
 */
export const namespace = "user";

export const messages = {
    "score.improvements.base.empty": {
        text: "Component breakdown appears after the next score refresh.",
        description: "Shown in the Base panel for an account whose stored score predates the component split.",
    },
    "score.improvements.base.title": {
        text: "Score breakdown",
        description: "Heading over the Base subscore's two components.",
    },
    "score.improvements.base.weight": {
        text: "{pct}% of this score",
        description: "Small caption after a component's name saying how much of the Base score it carries.",
    },
    "score.improvements.base.utilization.label": {
        text: "Stationing",
        description: "Name of the Base component covering which operators are assigned to which rooms.",
    },
    "score.improvements.base.utilization.desc": {
        text: "Sustained daily yield of your base as stationed, against the optimizer's best staffing of your own roster on the same rooms.",
        description: "Blurb under the Stationing component. 'Base' is the player's in-game home installation.",
    },
    "score.improvements.base.infrastructure.label": {
        text: "Upgrades",
        description: "Name of the Base component covering how far the rooms themselves are built up.",
    },
    "score.improvements.base.infrastructure.desc": {
        text: "What your rooms can achieve as built, against the same rooms at max level.",
        description: "Blurb under the Upgrades component.",
    },
    "score.improvements.base.optimizerHint": {
        text: "Plan restaffing and see the full room-by-room comparison in the Optimizer tab.",
        description: "Footer of the Base panel pointing at the profile's Optimizer tab, whose name is translated on the tab strip.",
    },
} satisfies MessageMap;

// `dynamic`: the component labels and blurbs are stored in the COMPONENTS table
// and resolved as `t(c.labelKey)`, so the extractor has no literal call site to
// match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
