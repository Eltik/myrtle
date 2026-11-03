import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.notFound.title": {
        text: "Tier list not found",
        description: "Heading shown when the requested tier list does not exist.",
    },
    "detail.notFound.body": {
        text: "It may have been removed or the link could be wrong.",
        description: "Paragraph under the not-found heading.",
    },
    "detail.notFound.action": {
        text: "Browse all lists",
        description: "Link from the not-found page back to the tier-list browse page.",
    },
    "detail.snapshot.failedTitle": {
        text: "Couldn't load version {version}",
        description: "Error heading when a link asks for an older published version of a tier list and the version list failed to load. {version} is the version number.",
    },
    "detail.snapshot.failedBody": {
        text: "The board below is the latest version, not that snapshot.",
        description: "Under the failed-version heading: says the board shown is the current one instead of the requested snapshot.",
    },
    "detail.snapshot.retry": {
        text: "Try again",
        description: "Button that retries loading the tier list's published versions.",
    },
    "detail.snapshot.missingTitle": {
        text: "There is no version {version}",
        description: "Warning heading when a link asks for a published version number this tier list does not have. {version} is the requested number.",
    },
    "detail.snapshot.missingBody": {
        text: "The board below is the latest version.",
        description: "Under the missing-version heading: says the board shown is the current one.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
