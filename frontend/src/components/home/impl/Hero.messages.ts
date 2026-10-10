import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "hero.eyebrow": {
        text: "myrtle.moe",
        description: "Small uppercase label above the home page headline. It is the site's own name; keep it as-is unless the site is known by another name in this language.",
    },
    "hero.title": {
        text: "Everything Arknights.",
        description: "Home page headline. Use the name the game is published under in this language (明日方舟 in Chinese, アークナイツ in Japanese, 명일방주 in Korean).",
    },
    "hero.lead": {
        text: "Share your roster, plan pulls and skins, see what’s coming to EN, and check tier lists and leaderboards — all in one place.",
        description: "Paragraph under the home page headline. A 'roster' is a player's own operator collection; 'pulls' are gacha rolls; 'EN' is the English game server.",
    },
    "hero.hint": {
        text: "Hit {keys} to jump anywhere.",
        description: "Keyboard hint under the hero paragraph; clicking it opens the search palette. {keys} is the pair of key caps, '⌘ K' on a Mac and 'Ctrl K' elsewhere; move it wherever the sentence needs it.",
    },
    "hero.myrtleAlt": {
        text: "Myrtle, asleep under a blanket",
        description: "Alt text for the chibi of Myrtle (an operator, the site's mascot) beside the home page headline.",
    },
    "hero.cue.title": {
        text: "Want to know some of Myrtle's functions?",
        description: "Button at the bottom of the home page hero that scrolls down to the live events. Myrtle is the site's mascot operator speaking playfully.",
    },
    "hero.cue.subtitle": {
        text: "See what's happening now",
        description: "Second line of the scroll-down button in the home page hero; the section below lists the events and banners running now.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
