import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.stagePicker.title": {
        text: "Pick a stage",
        description: "Title of the stage picker dialog in the Squad Builder.",
    },
    "squad.stagePicker.desc": {
        text: "Its deploy limit and starting DP feed the analysis.",
        description: "Description under the stage picker title. DP is deployment points.",
    },
    "squad.stagePicker.group": {
        text: "Mode",
        description: "Accessible name of the select that picks a game mode (Main Story, Events, ...).",
    },
    "squad.stagePicker.zone": {
        text: "Chapter or event",
        description: "Accessible name of the select that narrows the list to one chapter or event.",
    },
    "squad.stagePicker.allZones": {
        text: "All chapters and events",
        description: "Option in the chapter/event select that shows every stage in the mode.",
    },
    "squad.stagePicker.search": {
        text: "Search by code or name",
        description: "Placeholder and accessible name of the stage search box.",
    },
    "squad.stagePicker.empty": {
        text: "No stages match.",
        description: "Shown when the search and filters leave no stage.",
    },
    "squad.stagePicker.hard": {
        text: "Hard",
        description: "Tag on a stage card for a harder variant (Adverse Environment, six-star, Hard chapter) that shares its code with the standard stage.",
    },
    "squad.stagePicker.loading": {
        text: "Loading stages...",
        description: "Shown while the stage list loads.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
