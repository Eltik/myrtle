import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Names and one-line blurbs for the six scored sections. Plain `.ts`
 * constants, so `helpers.ts` carries message KEYS and whichever card renders a
 * section resolves them with `t()`.
 *
 * The grade letters (F through S+) are absent on purpose: they are symbols, not
 * words.
 */
export const namespace = "user";

export const messages = {
    "score.section.operator.label": {
        text: "Operator",
        description: "Name of the scored section covering the account's roster.",
    },
    "score.section.operator.desc": {
        text: "Roster depth & investment",
        description: "Blurb under the Operator section's percentage. Very little room.",
    },
    "score.section.base.label": {
        text: "Base",
        description: "Name of the scored section covering the account's base layout.",
    },
    "score.section.base.desc": {
        text: "Yield vs. your optimal setup",
        description: "Blurb under the Base section's percentage. Very little room.",
    },
    "score.section.stage.label": {
        text: "Stages",
        description: "Name of the scored section covering cleared operations.",
    },
    "score.section.stage.desc": {
        text: "Story & event clears",
        description: "Blurb under the Stages section's percentage. Very little room.",
    },
    "score.section.roguelike.label": {
        text: "Roguelike",
        description: "Name of the scored section covering Integrated Strategies.",
    },
    "score.section.roguelike.desc": {
        text: "IS endings & relics",
        description: "Blurb under the Roguelike section's percentage. 'IS' abbreviates the game mode Integrated Strategies and 'relics' are its in-game collectibles.",
    },
    "score.section.sandbox.label": {
        text: "Sandbox",
        description: "Name of the scored section covering Reclamation Algorithm.",
    },
    "score.section.sandbox.desc": {
        text: "RA progress & nodes",
        description: "Blurb under the Sandbox section's percentage. 'RA' abbreviates the game mode Reclamation Algorithm; 'nodes' are its map points.",
    },
    "score.section.medal.label": {
        text: "Medals",
        description: "Name of the scored section covering collected medals.",
    },
    "score.section.medal.desc": {
        text: "Achievement collection",
        description: "Blurb under the Medals section's percentage. Very little room.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a registry/constants entry and resolved
// by the consuming component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
