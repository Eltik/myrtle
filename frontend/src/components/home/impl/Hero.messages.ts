import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "hero.title": {
        text: "myrtle.moe",
        description: "Home page headline. It is the site's own name; keep it as-is unless the site is known by another name in this language.",
    },
    "hero.lead": {
        text: "Show off your roster, plan your pulls and skins, see what's coming to EN, and check the tier lists and leaderboards.",
        description: "Paragraph under the home page headline. A 'roster' is a player's own operator collection; 'pulls' are gacha rolls; 'EN' is the English game server.",
    },
    "hero.hint": {
        text: "Press {keys} to search.",
        description: "Keyboard hint under the hero paragraph; clicking it opens the search palette. {keys} is the pair of key caps, '⌘ K' on a Mac and 'Ctrl K' elsewhere; move it wherever the sentence needs it.",
    },
    "hero.keyCtrl": {
        text: "Ctrl",
        description: "Label on the Control key cap in the home page's search hint, shown everywhere but on a Mac. Use the label printed on that key on keyboards in this language, such as 'Strg' in German.",
    },
    "hero.myrtleAlt": {
        text: "Myrtle, asleep under a blanket",
        description: "Alt text for the chibi of Myrtle (an operator, the site's mascot) beside the home page headline.",
    },
    "hero.cue.title": {
        text: "See what's ending soon",
        description: "Link at the bottom of the home page hero that jumps down to the section headed 'Ending soon', which lists the running events and banners closing first. Keep it close to that heading's wording.",
    },
    "hero.cue.subtitle": {
        text: "Events and banners closing first",
        description: "Second line of the jump link at the bottom of the home page hero, saying what the 'Ending soon' section below lists: the running events and banners, the ones closing first at the top.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
