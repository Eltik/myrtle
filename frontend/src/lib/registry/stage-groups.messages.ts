import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Blurbs for the stage-group taxonomy. Plain `.ts`, so
 * `STAGE_GROUP_DESCRIPTION_KEY` holds keys and the dialog resolves them.
 *
 * The group LABELS are deliberately absent: "Annihilation", "Integrated
 * Strategies", "Contingency Contract" and the rest are official in-game mode
 * names, which belong to the game-data layer (each region ships its own) and
 * not to this catalog. Only this site's own prose about them lives here.
 */
export const namespace = "nav";

export const messages = {
    "stageGroup.story.desc": {
        text: "The mainline campaign - Rhodes Island's canonical operations, told episode by episode across Terra.",
        description: "Blurb for the Main Story group. 'Rhodes Island' and 'Terra' are in-game proper nouns; keep the game's own names for the region.",
    },
    "stageGroup.events.desc": {
        text: "Limited-time side stories, intermezzi and reruns, each with its own maps, mechanics and rewards.",
        description: "Blurb for the Events group. 'Side story', 'intermezzi' and 'rerun' name in-game event categories.",
    },
    "stageGroup.annihilation.desc": {
        text: "Endless-wave defense operations scored by total kills - the weekly source of Orundum.",
        description: "Blurb for the Annihilation game mode. 'Orundum' is an in-game currency; keep the game's own name.",
    },
    "stageGroup.is.desc": {
        text: "Roguelike expeditions: assemble a squad node by node across randomized floors, one run at a time.",
        description: "Blurb for the Integrated Strategies game mode.",
    },
    "stageGroup.ra.desc": {
        text: "A survival sandbox - scout the wilds, gather resources, build defenses and endure each cycle.",
        description: "Blurb for the Reclamation Algorithm game mode.",
    },
    "stageGroup.sss.desc": {
        text: "Deck-building defense runs against escalating security directives at fixed installations.",
        description: "Blurb for the Stationary Security Service game mode.",
    },
    "stageGroup.paradox.desc": {
        text: "Operator-specific challenge simulations that stress-test a single unit's kit.",
        description: "Blurb for the Paradox Simulation game mode. A 'kit' is an operator's set of skills and talents.",
    },
    "stageGroup.cc.desc": {
        text: "Score-attack contracts - stack risk modifiers on a fixed map for higher rewards.",
        description: "Blurb for the Contingency Contract game mode. 'Risk' is the in-game difficulty score.",
    },
    "stageGroup.supplies.desc": {
        text: "Daily and weekly resource runs for LMD, battle records, skill summaries and chips.",
        description: "Blurb for the Supplies group. LMD, battle records, skill summaries and chips are in-game item names.",
    },
    "stageGroup.other.desc": {
        text: "Special and uncategorized operations that sit outside the regular rotation.",
        description: "Blurb for the catch-all stage group.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a registry/constants entry and resolved
// by the consuming component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
