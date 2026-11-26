import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "notes.title": {
        text: "Operator notes",
        description: "Heading of the collapsible panel holding this site's own written notes about an operator.",
    },
    "notes.pros": {
        text: "Pros",
        description: "Heading over the operator's strengths. Short, beside a thumbs-up icon.",
    },
    "notes.cons": {
        text: "Cons",
        description: "Heading over the operator's weaknesses. Short, beside a thumbs-down icon.",
    },
    "notes.notes": {
        text: "Notes",
        description: "Heading over free-form remarks about the operator.",
    },
    "notes.trivia": {
        text: "Trivia",
        description: "Heading over incidental facts about the operator.",
    },
    "notes.showMore": {
        text: "Show more",
        description: "Button that expands a note clipped to a few lines.",
    },
    "notes.showLess": {
        text: "Show less",
        description: "Button that re-collapses an expanded note.",
    },
    "notes.loading": {
        text: "Loading operator notes…",
        description: "Screen-reader text for the spinner shown while the community notes for this operator load.",
    },
    "notes.error": {
        text: "Couldn't load the operator notes.",
        description: "Shown in place of the notes section when its request failed, beside a Try again button. Not the same as an operator with no notes, which shows nothing.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
