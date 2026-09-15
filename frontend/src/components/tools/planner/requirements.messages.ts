import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The category and status tables in `requirements.ts` are plain data in a
 * module with no React, so they carry message KEYS and `RequirementsPanel`
 * resolves them with `t()`. The two summary helpers in that module take an
 * optional `t` and fall back to the bundled source catalog.
 *
 * Material and item NAMES come from the game data and are never translated
 * here.
 */
export const namespace = "tools";

export const messages = {
    "planner.category.expLmd": {
        text: "EXP & LMD",
        description: "Requirement category: experience cards and money. 'EXP' and 'LMD' are the game's own names for them and normally stay as-is.",
    },
    "planner.category.skills": {
        text: "Skill Summaries",
        description: "Requirement category: the Skill Summary items used to level skills. The game's own item family name.",
    },
    "planner.category.chips": {
        text: "Chips",
        description: "Requirement category: the class chips used to promote operators. The game's own item family name.",
    },
    "planner.category.modules": {
        text: "Module Data",
        description: "Requirement category: the Module Data items used to unlock modules. The game's own item family name.",
    },
    "planner.category.materials": {
        text: "Materials",
        description: "Requirement category: the tiered crafting materials.",
    },
    "planner.status.missing": {
        text: "Missing",
        description: "Requirement status: short, and cannot be crafted from what the player has.",
    },
    "planner.status.craft": {
        text: "Craft needed",
        description: "Requirement status: short, but craftable from what the player has.",
    },
    "planner.status.complete": {
        text: "Complete",
        description: "Requirement status: the player already has enough.",
    },
    "planner.subtotal.items": {
        text: "{count, plural, one {# item} other {# items}}",
        description: "First part of a requirement group's summary line, counting distinct items.",
    },
    "planner.subtotal.missing": {
        text: "{count} missing",
        description: "Part of a requirement group's summary line: how many items are short and uncraftable.",
    },
    "planner.subtotal.craft": {
        text: "{count} to craft",
        description: "Part of a requirement group's summary line: how many items must be crafted.",
    },
    "planner.subtotal.separator": {
        text: " · ",
        description: "Joins the parts of a requirement group's summary line, e.g. '18 items · 3 missing · 2 to craft'. Keep the spaces around the middle dot.",
    },
    "planner.target.level": {
        text: "E{elite} Lv{level}",
        description: "First part of a plan's target summary: the Elite promotion and level aimed for, e.g. 'E2 Lv90'. 'E' and 'Lv' are the game's own abbreviations.",
    },
    "planner.target.mastery": {
        text: "S{skill} M{mastery}",
        description: "One skill's mastery target, e.g. 'S2 M3'. 'S' numbers the skill and 'M' the mastery rank; both are the game's own notation.",
    },
    "planner.target.skillLevel": {
        text: "SL{level}",
        description: "A pre-mastery skill-level target, e.g. 'SL7'. 'SL' abbreviates skill level.",
    },
    "planner.target.modules": {
        text: "{count, plural, one {# module} other {# modules}}",
        description: "How many modules a plan aims to unlock. 'Module' is the game's own equipment system.",
    },
    "planner.target.separator": {
        text: " · ",
        description: "Joins the parts of a plan's target summary, e.g. 'E2 Lv90 · S2 M3 · 2 modules'. Keep the spaces around the middle dot.",
    },
} satisfies MessageMap;

// `dynamic`: the category and status keys are stored on constants entries and
// resolved by the consuming component as `t(def.labelKey)`, so the extractor
// has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
