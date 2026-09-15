import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.versions.sectionLabel": {
        text: "Tier list versions",
        description: "Accessible name of the bar for switching between published snapshots of the list.",
    },
    "detail.versions.heading": {
        text: "Version",
        description: "Small uppercase label at the start of the version bar.",
    },
    "detail.versions.latest": {
        text: "Latest (live)",
        description: "The option showing the list as it stands now, rather than a published snapshot.",
    },
    "detail.versions.latestHint": {
        text: "live state",
        description: "Shown beside the version button when the live list is on screen. Lowercase; it follows a middle dot.",
    },
    "detail.versions.latestDescription": {
        text: "Always reflects the most recent edits",
        description: "Explains the 'Latest (live)' option, under its name in the menu.",
    },
    "detail.versions.trigger": {
        text: "Select version, currently {version}",
        description: "Accessible name of the version menu's button, naming the version on screen.",
    },
    "detail.versions.publishedCount": {
        text: "published",
        description: "Follows the count of published versions in the bar; the number itself is styled separately and rendered just before it.",
    },
    "detail.versions.backToLatest": {
        text: "Back to latest",
        description: "Link that leaves a published snapshot and shows the live list again.",
    },
    "detail.versions.snapshot": {
        text: "Snapshot",
        description: "Uppercase label on the banner shown while a published snapshot is on screen instead of the live list.",
    },
    "detail.versions.snapshotLine": {
        text: "v{version} · published",
        description: "On the snapshot banner: which version is on screen; the publication date follows it. Keep the 'v' prefix and the middle dot.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
