import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.modifier.kicker": {
        text: "RULE",
        description: "Vertical label down the side of the challenge panel. Set in capitals with wide letter spacing, so keep it very short.",
    },
    "randomizer.modifier.kind.restriction": {
        text: "Restriction",
        description: "Category of a challenge that forbids something.",
    },
    "randomizer.modifier.kind.modifier": {
        text: "Modifier",
        description: "Category of a challenge that changes how the stage is played.",
    },
    "randomizer.modifier.kind.objective": {
        text: "Objective",
        description: "Category of a challenge that sets an extra goal.",
    },
    "randomizer.modifier.rerollAria": {
        text: "Reroll modifier",
        description: "Accessible name of the button that draws a new challenge.",
    },
    "randomizer.modifier.reroll": {
        text: "Reroll",
        description: "Visible label of the button that draws a new challenge.",
    },
} satisfies MessageMap;

// `dynamic`: the challenge-category keys are stored in a lookup table and resolved
// as `t(KIND_LABEL_KEY[kind])`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
