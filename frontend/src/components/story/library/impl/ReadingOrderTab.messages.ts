import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "reading.mode.aria": {
        text: "Reading order",
        description: "Accessible name of the row of buttons that picks a reading order.",
    },
    "reading.mode.release": {
        text: "Release order",
        description: "Reading order that follows the date each story group came out.",
    },
    "reading.mode.storyline": {
        text: "Storyline order",
        description: "Reading order that follows the Main Theme chapter by chapter.",
    },
    "reading.mode.timeline": {
        text: "Timeline order",
        description: "Reading order that would follow when each story happens in the world, which this site does not offer.",
    },
    "reading.release.pros": {
        text: "Reads the way the game shipped, so nothing refers back to a story you have not met yet. Events land where they interrupted the mainline, which is how most players first read them.",
        description: "The case for reading in release order, on the Reading Order tab.",
    },
    "reading.release.cons": {
        text: "A plot thread can go cold for months of real time between two chapters. Only five main story chapters carry a date, so the other twelve stand apart instead of woven in.",
        description: "The case against reading in release order, on the Reading Order tab.",
    },
    "reading.storyline.pros": {
        text: "The Main Theme runs end to end, with no six-month gap in the middle of a plot. The four arcs are the game's own, so each stretch ends where the writers meant it to.",
        description: "The case for reading in storyline order, on the Reading Order tab.",
    },
    "reading.storyline.cons": {
        text: "Nothing in the data says which chapter a side story belongs beside, so the side material trails the mainline instead of sitting in it. An event that answers a chapter can therefore be read long after it stopped being a surprise.",
        description: "The case against reading in storyline order, on the Reading Order tab.",
    },
    "reading.timeline.title": {
        text: "Not offered here",
        description: "Heading of the Timeline order panel, which carries an explanation instead of a list.",
    },
    "reading.timeline.body": {
        text: "A timeline order says when each story happens in Terra, and no field in the game's data says that. The reference site's timeline is written by hand, story by story, so this site would be copying a judgement rather than deriving one, and a wrong guess reads as a spoiler. Release order and storyline order below are both derived from the data and are shown instead.",
        description: "Explanation in the Timeline order panel of why the site offers no in-world timeline.",
    },
    "reading.section.undated": {
        text: "Main Theme, undated chapters",
        description: "Heading of the block of mainline chapters in release order whose zone carries no open date.",
    },
    "reading.section.undated.note": {
        text: "{undated} main story chapters sit in zones the game dates -1, so they stand here in chapter order rather than being placed among the events. The other {dated} carry a real zone open time and are interleaved below.",
        description: "Note under the undated Main Theme heading in release order. {undated} and {dated} are counted from the data, currently 12 and 5.",
    },
    "reading.section.dated": {
        text: "Dated groups, oldest first",
        description: "Heading of the block of events and vignettes in release order.",
    },
    "reading.section.unlinked": {
        text: "Side material, release order",
        description: "Heading of the block of events and vignettes that storyline order cannot place against a chapter.",
    },
    "reading.section.unlinked.note": {
        text: "No field links an event to the chapter it follows, so this block is in release order and says so rather than guessing a place for it.",
        description: "Note under the trailing block in storyline order.",
    },
    "reading.section.totals": {
        text: "{words} words · {time} · {done} of {groups} read",
        description: 'Totals line under a reading-order block heading. {time} is a reading span such as "1d 6h", derived from the reader\'s own words-per-minute setting, and {done} counts the chapters in the block that are finished.',
    },
    "reading.section.totalsUncounted": {
        text: "{done} of {groups} read",
        description: "Totals line under a reading-order block heading, on a block whose groups the backend sends no word counts for.",
    },
    "reading.next": {
        text: "Next",
        description: "Label of the pointer above a reading order, naming the first story in it that is not read.",
    },
    "reading.next.done": {
        text: "Everything in this order is read.",
        description: "Shown in place of the Next pointer once no story in the order is unread.",
    },
    "reading.total": {
        text: "{count} chapters",
        description: "Count above the reading order list.",
    },
    "reading.row.noDate": {
        text: "No date",
        description: "Shown in place of a release date on a group the data gives none.",
    },
    "reading.row.expand": {
        text: "Show the stories in {name}",
        description: "Accessible name of a reading order row that opens its story list.",
    },
    "reading.row.collapse": {
        text: "Hide the stories in {name}",
        description: "Accessible name of an open reading order row.",
    },
    "reading.row.read": {
        text: "{read}/{total}",
        description: "Read fraction on a reading order row: stories read over stories that have a script.",
    },
    "reading.row.noneExtracted": {
        text: "{count} not extracted",
        description: "Shown on a reading order row whose stories all lack a script.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
