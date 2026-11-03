import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "library.title": {
        text: "Stories",
        description: "Page title of the story library.",
    },
    "library.description": {
        text: "Read every Arknights story: main chapters, events and operator records.",
        description: "Page subtitle of the story library.",
    },
    "library.breadcrumb": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb trail above the page title.",
    },
    "library.breadcrumb.archive": {
        text: "Archive",
        description: "First breadcrumb above the page title: the section the story library belongs to.",
    },
    "library.breadcrumb.stories": {
        text: "Stories",
        description: "Last breadcrumb above the page title: the current page.",
    },
    "library.counts": {
        text: "{groups} chapters · {records} operators · {stories} stories",
        description: "Muted meta line under the page title: how much the library holds. {stories} arrives already formatted (1,887).",
    },
    "library.tab.browse": {
        text: "Browse",
        description: "First mode tab: the chapter sections.",
    },
    "library.tab.reading": {
        text: "Reading order",
        description: "Top-level tab holding the reading orders the data can derive.",
    },
    "library.tab.illustrations": {
        text: "Illustrations",
        description: "Top-level tab holding the backgrounds, CG artwork and character sprites each chapter uses.",
    },
    "library.tab.progress": {
        text: "Progress",
        description: "Top-level tab holding the reading summary and the backup controls.",
    },
    "route.loadFailed": {
        text: "Couldn't load this story. Check your connection and try again.",
        description: "Shown on the story reader page when the story or the library index failed to load, beside a Retry button.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
