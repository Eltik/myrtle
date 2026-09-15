import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "feature.data.kicker": {
        text: "Live data",
        description: "Eyebrow label on the first landing-page feature card. Rendered uppercase.",
    },
    "feature.data.title": {
        text: "400+ operators, instant search",
        description: "Headline of the first landing-page feature card. 'Operators' are the game's playable characters.",
    },
    "feature.data.desc": {
        text: "Skills, talents, modules, skins, voice lines. Filter by faction, archetype, or tag.",
        description: "Body of the first landing-page feature card. Every noun here is a game concept; 'archetype' is an operator's sub-class.",
    },
    "feature.sync.kicker": {
        text: "Roster sync",
        description: "Eyebrow label on the second landing-page feature card. A roster is a player's own operator collection. Rendered uppercase.",
    },
    "feature.sync.title": {
        text: "Your account, mirrored",
        description: "Headline of the second landing-page feature card.",
    },
    "feature.sync.desc": {
        text: "Link a Yostar account and see your live box, E2 progress, base layout in real time.",
        description: "Body of the second landing-page feature card. 'Yostar' is the publisher's account system, 'box' is community slang for the operators a player owns, and 'E2' is the game's second promotion tier - all three stay as-is.",
    },
    "feature.tools.kicker": {
        text: "Tools",
        description: "Eyebrow label on the third landing-page feature card. Rendered uppercase.",
    },
    "feature.tools.title": {
        text: "DPS, recruit, randomizer",
        description: "Headline of the third landing-page feature card, naming three calculators. 'DPS' is damage per second.",
    },
    "feature.tools.desc": {
        text: "Interactive calculators and community-maintained tier lists for every stage meta.",
        description: "Body of the third landing-page feature card. 'Stage' is a game level and 'meta' is community shorthand for the prevailing strategy.",
    },
    "feature.explore": {
        text: "Explore",
        description: "Link at the foot of every landing-page feature card, followed by an arrow icon. A verb.",
    },
    "feature.arrowIcon": {
        text: "Right arrow",
        description: "Accessible name of the decorative arrow icon after the 'Explore' link.",
    },
} satisfies MessageMap;

// `dynamic`: the three cards' copy is stored in the FEATURES table and resolved
// as `t(f.kKey)`, so the extractor has no literal call site to match it against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
