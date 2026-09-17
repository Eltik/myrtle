import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `header/**` and `lib/registry/**` share the `nav` namespace. */
export const namespace = "nav";

export const messages = {
    "header.home": {
        text: "Home",
        description: "Top-level nav link to the landing page.",
    },
    "header.collection": {
        text: "Collection",
        description: "Top-level nav menu grouping the reference browsers (operators, enemies, stages).",
    },
    "header.tools": {
        text: "Tools",
        description: "Top-level nav menu grouping the calculators and planners.",
    },
    "header.gacha": {
        text: "Gacha",
        description: "Top-level nav menu for pull tracking. 'Gacha' is the game's random-draw system and stays as-is.",
    },
    "header.tierLists": {
        text: "Tier Lists",
        description: "Top-level nav link to the tier-list section, where operators are ranked into tiers.",
    },
    "header.players": {
        text: "Players",
        description: "Top-level nav menu for other people's profiles, search and the leaderboard.",
    },
    "header.collection.operators.label": {
        text: "Operators",
        description: "Collection menu entry for the operator (playable character) browser.",
    },
    "header.collection.operators.desc": {
        text: "Every operator released in Arknights.",
        description: "Blurb under the Operators entry in the Collection menu. 'Arknights' is the game's name.",
    },
    "header.collection.enemies.label": {
        text: "Enemies",
        description: "Collection menu entry for the enemy browser.",
    },
    "header.collection.enemies.desc": {
        text: "Every enemy catalogued, with stats and traits.",
        description: "Blurb under the Enemies entry in the Collection menu.",
    },
    "header.collection.stages.label": {
        text: "Stages",
        description: "Collection menu entry for the stage (mission map) browser.",
    },
    "header.collection.stages.desc": {
        text: "Every stage, mapped with an enemy-pathing simulator.",
        description: "Blurb under the Stages entry in the Collection menu.",
    },
    "header.gacha.community.label": {
        text: "Community",
        description: "Gacha menu entry for community-wide pull statistics.",
    },
    "header.gacha.community.desc": {
        text: "Pull rates, top operators, and timing across opted-in doctors",
        description: "Blurb under the Community entry in the Gacha menu. A 'pull' is one random draw; 'player' is the site's term; the game itself says 'Doctor'.",
    },
    "header.gacha.history.label": {
        text: "History",
        description: "Gacha menu entry for the visitor's own recorded pulls.",
    },
    "header.gacha.history.desc": {
        text: "Your synced pulls, rarity splits, and pity counters",
        description: "Blurb under the History entry in the Gacha menu. 'Pity' is the counter that guarantees a rare result after enough pulls.",
    },
    "header.players.myProfile.label": {
        text: "My Profile",
        description: "Players menu entry linking to the signed-in visitor's own profile.",
    },
    "header.players.myProfile.desc": {
        text: "Open your profile",
        description: "Blurb under the My Profile entry in the Players menu.",
    },
    "header.players.search.label": {
        text: "Search",
        description: "Players menu entry for looking up another player.",
    },
    "header.players.search.desc": {
        text: "Find profiles by nickname or UID",
        description: "Blurb under the Search entry in the Players menu. UID is the in-game account number.",
    },
    "header.players.leaderboard.label": {
        text: "Leaderboard",
        description: "Players menu entry for the score leaderboard.",
    },
    "header.players.leaderboard.desc": {
        text: "Top players ranked by score",
        description: "Blurb under the Leaderboard entry in the Players menu. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "header.searchOperators": {
        text: "Search operators",
        description: "Accessible name of the wide search button in the header bar, which opens the command palette.",
    },
    "header.searchOperatorsPlaceholder": {
        text: "Search operators…",
        description: "Visible text inside the wide search button. Ends in a single ellipsis character.",
    },
    "header.search": {
        text: "Search",
        description: "Accessible name of the icon-only search button shown on narrow screens.",
    },
    "header.support": {
        text: "Support",
        description: "Label of the heart-icon link to the donation page. 'Support' here means financially supporting the site.",
    },
    "header.settings": {
        text: "Settings",
        description: "Label of the cog-icon link to the settings page.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
