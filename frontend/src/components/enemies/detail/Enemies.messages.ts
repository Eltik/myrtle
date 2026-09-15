import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "detail.breadcrumb": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb <nav> landmark above the enemy's name.",
    },
    "detail.breadcrumb.collection": {
        text: "Collection",
        description: "First breadcrumb crumb, naming the section the enemy handbook lives in.",
    },
    "detail.breadcrumb.enemies": {
        text: "Enemies",
        description: "Breadcrumb link back to the enemy list. The last crumb is the enemy's own name, which is game data.",
    },
    "detail.notFound.title": {
        text: "Enemy not found",
        description: "Heading shown when the URL names an enemy id the backend has no enemy for.",
    },
    "detail.notFound.body": {
        text: "No enemy with id {id} exists in the handbook.",
        description: "Shown when the URL names an enemy id the backend has no enemy for. {id} is that id, rendered in a monospace box, and may move wherever the sentence needs it. 'Handbook' is the game's own enemy reference screen.",
    },
    "detail.notFound.back": {
        text: "Back to Enemy Database",
        description: "Link out of the enemy-not-found page, back to the enemy list.",
    },
    "detail.tab.overview": {
        text: "Overview",
        description: "Tab label: the enemy's description, traits and tags.",
    },
    "detail.tab.stats": {
        text: "Stats",
        description: "Tab label: the enemy's stat blocks and status immunities.",
    },
    "detail.tab.skills": {
        text: "Skills",
        description: "Tab label: the enemy's skills and their cooldowns.",
    },
    "detail.tab.appears": {
        text: "Appears In",
        description: "Tab label: which stages the enemy shows up in.",
    },
    "detail.tab.chibi": {
        text: "Chibi",
        description: "Tab label: the enemy's animated sprite. 'Chibi' is the game community's word for it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
