import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Blurbs for the stage-group taxonomy. Plain `.ts`, so
 * `STAGE_GROUP_DESCRIPTION_KEY` holds keys and the dialog resolves them.
 *
 * The group LABELS live here too (`stageGroup.<key>.label`), because the game
 * data names only some modes, and not by a stable id: on 2026-10-11
 * `main_text.json` prints 6 of the 10 (story "Main Theme" as
 * `COMMON_STORYLINE_MAINLINE`, Integrated Strategies as
 * `ROGUE_COMMON_NOUN_ROGUE`; Annihilation, Reclamation Algorithm, Paradox
 * Simulation and Supplies only under hashed `&&...` keys) and none of
 * Events, Stationary Security Service, Contingency Contract or Other as a
 * mode name. Translators should use the game's own name in their language.
 */
export const namespace = "nav";

export const messages = {
    "stageGroup.story.label": {
        text: "Main Story",
        description: "Stage-group name: the mainline campaign.",
    },
    "stageGroup.events.label": {
        text: "Events",
        description: "Stage-group name: limited-time events and their reruns.",
    },
    "stageGroup.annihilation.label": {
        text: "Annihilation",
        description: "Stage-group name: the in-game mode. Use the game's own name for 'Annihilation' in this language.",
    },
    "stageGroup.is.label": {
        text: "Integrated Strategies",
        description: "Stage-group name: the in-game mode. Use the game's own name for 'Integrated Strategies' in this language.",
    },
    "stageGroup.ra.label": {
        text: "Reclamation Algorithm",
        description: "Stage-group name: the in-game mode. Use the game's own name for 'Reclamation Algorithm' in this language.",
    },
    "stageGroup.sss.label": {
        text: "Stationary Security Service",
        description: "Stage-group name: the in-game mode. Use the game's own name for 'Stationary Security Service' in this language.",
    },
    "stageGroup.paradox.label": {
        text: "Paradox Simulation",
        description: "Stage-group name: the in-game mode. Use the game's own name for 'Paradox Simulation' in this language.",
    },
    "stageGroup.cc.label": {
        text: "Contingency Contract",
        description: "Stage-group name: the in-game mode. Use the game's own name for 'Contingency Contract' in this language.",
    },
    "stageGroup.supplies.label": {
        text: "Supplies",
        description: "Stage-group name: daily and weekly resource stages.",
    },
    "stageGroup.other.label": {
        text: "Other Modes",
        description: "Stage-group name: the catch-all group.",
    },
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
