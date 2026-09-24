import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "stats.heading": {
        text: "Reading statistics",
        description: "Heading of the section on the Progress tab that turns word counts into reading time.",
    },
    "stats.blurb": {
        text: "Every figure here is a word count divided by your own reading speed. Nothing in the game's data says how fast you read, so change the two numbers below and the whole section follows.",
        description: "Explanation under the Reading statistics heading.",
    },
    "stats.pending": {
        text: "The library index carries no word counts yet, so there is nothing to time. Reading statistics appear once it does.",
        description: "Shown in place of the whole statistics section when the backend sends no word counts at all.",
    },
    "stats.wpm": {
        text: "Reading speed",
        description: "Label of the words-per-minute input.",
    },
    "stats.wpm.unit": {
        text: "words a minute",
        description: "Unit beside the reading speed input.",
    },
    "stats.minutesPerDay": {
        text: "Reading time",
        description: "Label of the minutes-per-day input.",
    },
    "stats.minutesPerDay.unit": {
        text: "minutes a day",
        description: "Unit beside the daily reading time input.",
    },
    "stats.wordsRead": {
        text: "Words read",
        description: "Label of the stat summing the words of every story that counts as read.",
    },
    "stats.wordsReadValue": {
        text: "{read} of {total}",
        description: "Value of the words-read stat: words read over the library's whole word count.",
    },
    "stats.timeSpent": {
        text: "Time spent",
        description: "Label of the stat turning the words read into a reading span.",
    },
    "stats.timeLeft": {
        text: "Time left",
        description: "Label of the stat turning the words not yet read into a reading span.",
    },
    "stats.coverToCover": {
        text: "Cover to cover",
        description: "Label of the stat turning the library's whole word count into a reading span.",
    },
    "stats.pace.days": {
        text: "About {value} days left to go at {perDay} minutes a day.",
        description: "Pace line under the statistics, when what is left reads in days.",
    },
    "stats.pace.months": {
        text: "About {value} months left to go at {perDay} minutes a day.",
        description: "Pace line under the statistics, when what is left reads in months.",
    },
    "stats.pace.done": {
        text: "Nothing left to read. The whole library is behind you.",
        description: "Pace line shown once every story counts as read.",
    },
    "stats.table.category": {
        text: "Category",
        description: "Column heading of the per-category statistics table.",
    },
    "stats.table.stories": {
        text: "Stories",
        description: "Column heading: stories read over stories that have a script.",
    },
    "stats.table.words": {
        text: "Words",
        description: "Column heading: words in the category.",
    },
    "stats.table.time": {
        text: "Time",
        description: "Column heading: how long the category takes at the reader's speed.",
    },
    "stats.table.fraction": {
        text: "{read} of {total}",
        description: "A cell of the statistics table: stories read over stories with a script.",
    },
    "stats.bucket.main": {
        text: "Main story",
        description: "Row of the statistics table covering the Main Theme chapters.",
    },
    "stats.bucket.events": {
        text: "Events",
        description: "Row of the statistics table covering the side story events.",
    },
    "stats.bucket.side": {
        text: "Side stories",
        description: "Row of the statistics table covering the vignettes and the intermezzi, the two short-form shelves.",
    },
    "stats.bucket.records": {
        text: "Operator records",
        description: "Row of the statistics table covering the operator record sets.",
    },
    "stats.longest": {
        text: "Longest chapters and events",
        description: "Heading of the table ranking the longest story groups by word count.",
    },
    "stats.longest.note": {
        text: "The {count} longest of the library, by word count, with how long each takes at your speed. Operator records are left out: a 400-word record beside a 90,000-word chapter is not the same kind of thing.",
        description: "Note under the longest-groups heading. {count} is how many rows the table shows.",
    },
    "stats.total": {
        text: "Whole library",
        description: "Label of the row summing the library's stories and words.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
