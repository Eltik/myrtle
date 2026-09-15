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
    "release.intro": {
        text: "When CN content lands on EN: confirmed from EN game data, announced from an override, estimated from the CN-to-EN lag.",
        description: "Blurb under the page heading. 'CN' and 'EN' are the Chinese and English game servers; 'lag' is how far behind EN runs.",
    },
    "release.autoTranslate": {
        text: "Auto-translate CN names",
        description: "Switch label: fill in English names for Chinese-only rows where this site can work one out.",
    },
    "release.autoTranslate.desc": {
        text: "Names come from EN game data where an id exists on both servers; anything still in Chinese is marked for your browser's translator.",
        description: "Explanation under the auto-translate switch. 'EN' is the English game server.",
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
    "release.tab.skins": {
        text: "Skins",
        description: "Tab listing the outfits and when they are expected on the English server.",
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
