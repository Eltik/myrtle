import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** Command-palette page entries. Plain `.ts`, so entries carry keys - see `tools.messages.ts`. */
export const namespace = "nav";

export const messages = {
    "page.operators.label": {
        text: "Operators",
        description: "Palette entry for the operator (playable character) browser.",
    },
    "page.operators.desc": {
        text: "Browse every operator · stats, skills, modules",
        description: "Blurb under the Operators palette entry. Separator is a middle dot; stats/skills/modules are in-game systems.",
    },
    "page.stages.label": {
        text: "Stages",
        description: "Palette entry for the stage (mission map) browser.",
    },
    "page.stages.desc": {
        text: "Every stage, mapped with an enemy-pathing simulator",
        description: "Blurb under the Stages palette entry.",
    },
    "page.tierLists.label": {
        text: "Tier Lists",
        description: "Palette entry for the tier-list section. A tier list ranks operators into tiers.",
    },
    "page.tierLists.desc": {
        text: "Official and community tier lists for every operator",
        description: "Blurb under the Tier Lists palette entry.",
    },
    "page.playersSearch.label": {
        text: "Search Players",
        description: "Palette entry for the player search. The site says 'player' throughout; Arknights itself calls them Doctors, so a region may prefer the game's term.",
    },
    "page.playersSearch.desc": {
        text: "Find player profiles by nickname or UID",
        description: "Blurb under the Search Doctors palette entry. UID is the in-game account number.",
    },
    "page.playersLeaderboard.label": {
        text: "Leaderboard",
        description: "Palette entry for the player leaderboard.",
    },
    "page.playersLeaderboard.desc": {
        text: "Top players ranked by score",
        description: "Blurb under the Leaderboard palette entry. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "page.gachaCommunity.label": {
        text: "Gacha Community",
        description: "Palette entry for community-wide pull statistics. 'Gacha' is the random-draw system and stays as-is.",
    },
    "page.gachaCommunity.desc": {
        text: "Pull rates, top operators, and timing across opted-in doctors",
        description: "Blurb under the Gacha Community palette entry. A 'pull' is one random draw.",
    },
    "page.gachaHistory.label": {
        text: "Gacha History",
        description: "Palette entry for the visitor's own recorded pulls.",
    },
    "page.gachaHistory.desc": {
        text: "Your synced pulls, rarity splits, and pity counters",
        description: "Blurb under the Gacha History palette entry. 'Pity' is the counter that guarantees a rare result after enough pulls.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a registry/constants entry and resolved
// by the consuming component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
