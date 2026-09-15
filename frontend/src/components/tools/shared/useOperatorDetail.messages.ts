import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The build-summary labels `useOperatorDetail` derives. Skill, module and
 * potential NAMES come from the game data and are never translated here; only
 * the chrome around them - the "no skill selected" wording, the designator
 * fallbacks, and the separator that joins a designator to a name - lives here.
 */
export const namespace = "tools";

export const messages = {
    "calc.detail.basicAttack": {
        text: "Basic attack",
        description: "Shown where a skill name would go when the operator is set to use no skill at all.",
    },
    "calc.detail.designatorWithName": {
        text: "{label} · {name}",
        description: "Joins a short designator (S2, SUM-X, P3) to the game-data name that follows it. Both halves come from the game data; only the separator is ours.",
    },
    "calc.detail.noModule": {
        text: "No module",
        description: "Module selector option and summary text for a build with no module equipped. 'Module' is the game's own equipment system.",
    },
    "calc.detail.noModuleLower": {
        text: "no module",
        description: "Same as the 'No module' label but mid-sentence in a one-line build summary, so it is lowercase.",
    },
    "calc.detail.moduleFallback": {
        text: "Mod {index}",
        description: "Last-resort module label when the in-game designator is unknown. 'Mod' abbreviates 'module'; {index} is the engine's own module number.",
    },
    "calc.detail.moduleFallbackShort": {
        text: "Mod{index}",
        description: "The 'Mod {index}' fallback without the space, for the compact chart-export legend.",
    },
    "calc.detail.potentialBase": {
        text: "Base",
        description: "Stands in for a potential-rank description at potential 1, which grants no bonus.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
