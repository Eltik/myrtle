import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Chrome shared by every improvement panel. `shared.tsx` itself takes its
 * labels as props, so these keys are rendered by the panels that reach for
 * `ShowMoreButton` - they live here because all of them render the same two.
 */
export const namespace = "user";

export const messages = {
    "score.improvements.showMore": {
        text: "Show {n} more",
        description: "Button under a truncated list in an improvement panel. {n} is how many entries are still hidden.",
    },
    "score.improvements.showLess": {
        text: "Show less",
        description: "Button that re-truncates a fully expanded list in an improvement panel.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
