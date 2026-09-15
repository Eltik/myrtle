import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.breadcrumb.tools": {
        text: "Tools",
        description: "First crumb of the breadcrumb trail, naming the section this tool lives in.",
    },
    "birthdays.title": {
        text: "Birthday Calendar",
        description: "Page heading and last breadcrumb.",
    },
    "birthdays.intro": {
        text: "Every operator's birthday on one screen. Filter by rarity, class, or nation; jump to any month; or browse what's coming up. Tap any day to see who's celebrating.",
        description: "Blurb under the page heading. 'Class' and 'nation' are the game's own operator attributes.",
    },
    "birthdays.tab.calendar": {
        text: "Calendar",
        description: "Tab showing the birthdays laid out as a calendar.",
    },
    "birthdays.tab.list": {
        text: "List",
        description: "Tab showing the birthdays as a chronological list. A count follows it.",
    },
    "birthdays.tab.upcoming": {
        text: "Upcoming",
        description: "Tab showing only the birthdays coming up next.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
