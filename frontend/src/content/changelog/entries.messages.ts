import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The hand-written release notes' own prose. Keyed at entry granularity - one
 * key per title, one per lead, one per bullet - because this is document
 * content, not UI chrome: a translator works through a whole entry at once and
 * the entry is what goes stale when it is rewritten.
 *
 * Every new entry in `entries.ts` needs its keys added here.
 */
export const namespace = "changelog";

export const messages = {
    "note.2026-09-16.title": {
        text: "Release & event planner",
        description: "Title of the 2026-09-16 release note. The planner forecasts when CN content (operators, events, skins) reaches the global server.",
    },
    "note.2026-09-16.lead": {
        text: "A new release and event planner forecasts when upcoming operators, events, and skins arrive on the global server, so you can plan your pulls ahead of time. Translations have also started rolling out.",
        description: "Lead paragraph of the 2026-09-16 release note, rendered as Markdown. 'Pulls' is the community's word for gacha rolls.",
    },
    "note.2026-09-16.hrefLabel": {
        text: "Open the planner",
        description: "Label of the 2026-09-16 release note's call to action, which opens the release and event planner.",
    },
    "note.2026-09-16.item.1": {
        text: "Added release & event planner feature.",
        description: "Bullet in the 2026-09-16 release note, filed under 'New'.",
    },
    "note.2026-09-16.item.2": {
        text: "Added translation support.",
        description: "Bullet in the 2026-09-16 release note, filed under 'New'.",
    },
    "note.2026-09-16.item.3": {
        text: "Improved/fixed base optimizer (daily LMD, control center, perception).",
        description: "Bullet in the 2026-09-16 release note, filed under 'Improved'. 'LMD' is the game's currency; 'control center' and 'perception' are base-building terms.",
    },
    "note.2026-09-16.item.4": {
        text: "Improved dynamic illustration.",
        description: "Bullet in the 2026-09-16 release note, filed under 'Improved'. 'Dynamic illustration' is the game's name for animated operator art.",
    },
    "note.2026-09-16.item.5": {
        text: "Workflow and security fixes.",
        description: "Bullet in the 2026-09-16 release note, filed under 'Fixed'.",
    },
    "note.2026-09-10.title": {
        text: "Build statistics",
        description: "Title of the 2026-09-10 release note. 'Build' here means how players level and equip an operator.",
    },
    "note.2026-09-10.lead": {
        text: "Based on community statistics, operators now display what users build for masteries and modules. More bug fixes and grading have been improved.",
        description: "Lead paragraph of the 2026-09-10 release note, rendered as Markdown. 'Mastery' and 'module' are the game's own names for skill and equipment upgrades.",
    },
    "note.2026-09-10.hrefLabel": {
        text: "Browse operators",
        description: "Label of the 2026-09-10 release note's call to action, which opens the operator index.",
    },
    "note.2026-09-10.item.1": {
        text: "Added changelog notifications.",
        description: "Bullet in the 2026-09-10 release note, filed under 'New'.",
    },
    "note.2026-09-10.item.2": {
        text: "Added build statistics.",
        description: "Bullet in the 2026-09-10 release note, filed under 'New'.",
    },
    "note.2026-09-10.item.3": {
        text: "Display breakpoints and percentages for mastery/module levels.",
        description: "Bullet in the 2026-09-10 release note, filed under 'New'. A breakpoint is the level at which a skill's numbers jump.",
    },
    "note.2026-09-10.item.4": {
        text: "Operator skills/modules default to what is most used.",
        description: "Bullet in the 2026-09-10 release note, filed under 'Improved'.",
    },
    "note.2026-09-10.item.5": {
        text: "Operator grades were computed from the wrong baseline. Scores across the roster have shifted.",
        description: "Bullet in the 2026-09-10 release note, filed under 'Improved'. A grade is this site's own letter rating of an operator.",
    },
    "note.2026-09-10.item.6": {
        text: "Profile rosters no longer play E2 dynamic art over an operator who has not reached E2.",
        description: "Bullet in the 2026-09-10 release note, filed under 'Fixed'. 'E2' is the game's second promotion tier and stays as-is.",
    },
    "note.2026-09-10.item.7": {
        text: "Check-in dates and Reclamation Algorithm scoring.",
        description: "Bullet in the 2026-09-10 release note, filed under 'Fixed'. 'Reclamation Algorithm' is a game mode name and is never translated.",
    },
} satisfies MessageMap;

// `dynamic`: every key here is reached through a field on a RELEASE_NOTES entry
// (`t(note.titleKey)`, `t(item.textKey)`), never as a literal, so the extractor
// has no call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
