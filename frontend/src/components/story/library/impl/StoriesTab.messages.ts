import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "stories.tab.main": {
        text: "Main Theme",
        description: "Sub-tab of the story library holding the mainline chapters.",
    },
    "stories.tab.side": {
        text: "Side Stories",
        description: "Sub-tab of the story library holding the large event stories.",
    },
    "stories.tab.intermezzi": {
        text: "Intermezzi",
        description: "Sub-tab of the story library holding the events the game shelves as Intermezzi (branchline events).",
    },
    "stories.tab.vignette": {
        text: "Vignettes",
        description: "Sub-tab of the story library holding the small seasonal event stories.",
    },
    "stories.tab.is": {
        text: "Integrated Strategies",
        description: "Sub-tab of the story library holding the roguelike mode's stories. Game mode name, keep the official translation.",
    },
    "stories.tab.reclamation": {
        text: "Reclamation Algorithm",
        description: "Sub-tab of the story library holding the survival mode's stories. Game mode name, keep the official translation.",
    },
    "stories.tab.sideContent": {
        text: "Other",
        description: "Sub-tab of the story library for groups the derivation could not place.",
    },
    "stories.search.placeholder": {
        text: "Search chapters and stories",
        description: "Placeholder of the search box above the chapter grid.",
    },
    "stories.search.aria": {
        text: "Search chapters and stories",
        description: "Accessible name of the search box above the chapter grid.",
    },
    "stories.hideFinished": {
        text: "Hide finished",
        description: "Checkbox that removes fully-read chapters from the grid.",
    },
    "stories.finishedCount": {
        text: "{done} of {total} chapters finished",
        description: "Counter above the chapter grid.",
    },
    "stories.sort.release": {
        text: "Release date",
        description: "Sort option: newest chapter first.",
    },
    "stories.sort.alpha": {
        text: "A to Z",
        description: "Sort option: chapters by name.",
    },
    "stories.sort.aria": {
        text: "Sort chapters",
        description: "Accessible name of the sort toggle above the chapter grid.",
    },
    "stories.empty": {
        text: "No chapter matches.",
        description: "Empty state under the chapter grid when the search and filters leave nothing.",
    },
    "stories.card.read": {
        text: "{read}/{total}",
        description: "Read fraction on a chapter card: stories read over stories that have a script.",
    },
    "stories.card.noneExtracted": {
        text: "{count} not extracted",
        description: "Stands in for the read fraction on a chapter whose stories all lack a script, so none of them can be read.",
    },
    "stories.card.words": {
        text: "{count} words",
        description: "Word count on a chapter card.",
    },
    "stories.card.expand": {
        text: "Show the stories in {name}",
        description: "Accessible name of a chapter card that opens its story list.",
    },
    "stories.card.collapse": {
        text: "Hide the stories in {name}",
        description: "Accessible name of an open chapter card that closes its story list.",
    },
    "stories.panel.progressAt": {
        text: "In progress at line {line}",
        description: "Note on a story row whose reading position was saved partway through.",
    },
    "stories.panel.notExtracted": {
        text: "Not extracted",
        description: "Badge on a story whose script file is missing from the served data.",
    },
    "stories.panel.close": {
        text: "Close",
        description: "Button that closes an open chapter's story list.",
    },
} satisfies MessageMap;

// `dynamic`: the sub-tab labels are keyed from a `StoriesTabKey` value (`stories.tab.${key}`).
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
