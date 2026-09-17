import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The profile's enemy handbook. Enemy names and index codes come from the game
 * data; the tier filters are a plain table, so they hold message KEYS.
 */
export const namespace = "user";

export const messages = {
    "profile.enemies.aria": {
        text: "Enemy handbook",
        description: "Accessible name of the enemy-handbook section of a profile.",
    },
    "profile.enemies.kicker": {
        text: "Enemy Handbook",
        description: "Kicker over the handbook's progress figures and over its empty states. Rendered uppercase by CSS.",
    },
    "profile.enemies.unavailable.title": {
        text: "Enemy data unavailable",
        description: "Empty-state title when the handbook cannot be read.",
    },
    "profile.enemies.unavailable.body": {
        text: "This player's enemy handbook is private, or their profile could not be found.",
        description: "Empty-state body when the handbook cannot be read. 'Doctor' is what Arknights calls the player; keep the apostrophe.",
    },
    "profile.enemies.noMatch.title": {
        text: "No enemies match",
        description: "Empty-state title when the filters and search leave nothing.",
    },
    "profile.enemies.noMatch.body": {
        text: "Try a different filter or clear your search.",
        description: "Empty-state body when the filters and search leave nothing.",
    },
    "profile.enemies.search.placeholder": {
        text: "Search enemies...",
        description: "Prompt inside the empty handbook search box. Keep the three dots as written.",
    },
    "profile.enemies.status.all": {
        text: "All",
        description: "Filter chip: every enemy in the handbook.",
    },
    "profile.enemies.status.encountered": {
        text: "Encountered",
        description: "Filter chip: only enemies the account has met.",
    },
    "profile.enemies.status.missing": {
        text: "Missing",
        description: "Filter chip: only enemies the account has never met.",
    },
    "profile.enemies.level.all": {
        text: "All Tiers",
        description: "Filter chip: every enemy tier.",
    },
    "profile.enemies.level.normal": {
        text: "Normal",
        description: "Filter chip: the game's ordinary enemy tier.",
    },
    "profile.enemies.level.elite": {
        text: "Elite",
        description: "Filter chip: the game's tougher enemy tier.",
    },
    "profile.enemies.level.boss": {
        text: "Boss",
        description: "Filter chip: the game's boss enemy tier.",
    },
    "profile.enemies.noneYet": {
        text: "No enemies recorded yet. Encountered enemies populate on the next profile resync.",
        description: "Notice above the grid when the account has met nothing yet. A 'resync' re-imports the account from the game.",
    },
    "profile.enemies.encountered": {
        text: "encountered",
        description: "Follows the seen / total figures in the progress header. Lowercase.",
    },
    "profile.enemies.progressAria": {
        text: "{seen} of {total} enemies encountered, {pct}%",
        description: "Accessible name of the progress bar.",
    },
    "profile.enemies.progressTooltip": {
        text: "{seen} / {total} encountered",
        description: "The figures inside the progress bar's tooltip, beside the percentage.",
    },
    "profile.enemies.average": {
        text: "Community average: {value}",
        description: "Tooltip line: how many enemies the average synced account has met. A percentage may follow in brackets.",
    },
    "profile.enemies.averagePct": {
        text: " ({pct}%)",
        description: "The percentage in brackets after that figure. Keep the leading space.",
    },
    "profile.enemies.averageEncountered": {
        text: "Community average: {value} encountered",
        description: "Line under the progress bar: how many enemies the average synced account has met. {value} is the highlighted figure and may move wherever the sentence needs it. A percentage in brackets may follow.",
    },
    "profile.enemies.card.title": {
        text: "{name} ({index})",
        description: "Native tooltip on an enemy card: its name and its handbook code, both from the game data.",
    },
    "profile.enemies.card.titleMissing": {
        text: "{name} ({index}) - not encountered",
        description: "The same tooltip for an enemy the account has never met.",
    },
    "profile.enemies.card.portraitAlt": {
        text: "{name} portrait",
        description: "Alt text of an enemy's picture. {name} comes from the game data.",
    },
    "profile.enemies.card.locked": {
        text: "Not encountered",
        description: "Native tooltip on the padlock marking an enemy the account has never met.",
    },
} satisfies MessageMap;

// `dynamic`: the tier names are stored in the LEVEL_FILTERS table and resolved
// as `t(filter.labelKey)`, so the extractor has no literal call site for them.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
