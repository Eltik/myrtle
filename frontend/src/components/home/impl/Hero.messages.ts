import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "hero.changelog": {
        text: "changelog →",
        description: "Link in the version pill at the top of the landing hero. The arrow is part of the label.",
    },
    "hero.title": {
        text: "The {game} companion.",
        description: "Hero headline. {game} is the styled game name 'Arknights', which is never translated; move it wherever the sentence needs it.",
    },
    "hero.blurb": {
        text: "400+ operators, complete stats, community tier lists, and live roster sync.",
        description: "Landing-page subheading. 'Operators' are the game's playable characters.",
    },
    "hero.hint": {
        text: "Hit {keys} to jump anywhere.",
        description: "Keyboard hint under the hero blurb. {keys} is the pair of key caps, '⌘ K' on a Mac and 'Ctrl K' elsewhere; move it wherever the sentence needs it.",
    },
    "hero.search": {
        text: "Search operators",
        description: "Primary hero button; opens the search command palette.",
    },
    "hero.searchIcon": {
        text: "Search",
        description: "Accessible name of the magnifier icon inside the hero's search button.",
    },
    "hero.arrowIcon": {
        text: "Right arrow",
        description: "Accessible name of the decorative arrow icon at the end of a hero button.",
    },
    "hero.viewProfile": {
        text: "View profile",
        description: "Secondary hero button shown to a signed-in visitor; opens their own profile.",
    },
    "hero.linkYostar": {
        text: "Link Yostar account",
        description: "Secondary hero button shown to a signed-out visitor. 'Yostar' is the game publisher's account system and stays as-is.",
    },
    "hero.stat.operators": {
        text: "operators",
        description: "Caption under the operator count in the hero's stat row. Rendered uppercase by CSS. Always plural.",
    },
    "hero.stat.tierLists": {
        text: "tier lists",
        description: "Caption under the active-tier-list count in the hero's stat row. Rendered uppercase by CSS. Always plural.",
    },
    "hero.stat.rosters": {
        text: "rosters synced",
        description: "Caption under the synced-roster count in the hero's stat row. A roster is a player's own operator collection. Rendered uppercase by CSS.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
