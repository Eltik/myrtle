import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.breadcrumb.tools": {
        text: "Tools",
        description: "First crumb of the breadcrumb trail, naming the section this tool lives in.",
    },
    "release.title": {
        text: "Release Planner",
        description: "Page heading and last breadcrumb.",
    },
    "release.tab.planner": {
        text: "Planner",
        description: "Tab where the reader budgets Originite Prime against upcoming outfits.",
    },
    "release.tab.pulls": {
        text: "Pulls planner",
        description: "Tab for budgeting gacha draws. A 'pull' is one draw.",
    },
    "release.tab.events": {
        text: "Events",
        description: "Tab listing the story events and when they are expected on the English server.",
    },
    "release.tab.banners": {
        text: "Banners",
        description: "Tab listing the gacha banners and when they are expected on the English server.",
    },
    "release.tab.calendar": {
        text: "Calendar",
        description: "Tab showing everything laid out as a month calendar.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
