import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "community.tab": {
        text: "Community",
        description: "Top-level tab of the story library holding what everyone else has read.",
    },
    "community.header": {
        text: "{players} synced players · computed {when}",
        description: "Line at the top of the Community tab: how many accounts the numbers are counted over and how long ago they were counted.",
    },
    "community.empty.title": {
        text: "Community reading is not available yet",
        description: "Heading shown in place of the Community tab when the backend serves no reading aggregate.",
    },
    "community.empty.body": {
        text: "This server has not computed what the community has read. It appears here once it does.",
        description: "Explanation under the Community tab's empty-state heading.",
    },
    "community.section.top": {
        text: "Most read chapters and events",
        description: "Heading of the ranked list of the chapters and events the most accounts have read something in.",
    },
    "community.section.top.blurb": {
        text: "Chapters and events by the number of synced players who have read at least one of their stories. Operator records are not ranked here.",
        description: "Explanation under the heading of the most-read chapters list.",
    },
    "community.section.stories": {
        text: "Most read stories",
        description: "Heading of the ranked list of individual stories the most accounts have read.",
    },
    "community.section.stories.blurb": {
        text: "Single stories by the number of synced players who have read them.",
        description: "Explanation under the heading of the most-read stories list.",
    },
    "community.section.bottom": {
        text: "Least read",
        description: "Heading of the ranked list of the chapters and events the fewest accounts have read.",
    },
    "community.section.bottom.blurb": {
        text: "The chapters and events with the fewest readers, counting only those with at least one.",
        description: "Explanation under the heading of the least-read chapters list.",
    },
    "community.section.depth": {
        text: "How far people read",
        description: "Heading of the bar strip showing how many accounts have read each story of one chapter.",
    },
    "community.section.depth.blurb": {
        text: "Readers of each story of one chapter, in the order the library lists them. It is not a funnel: a special stage or an interlude can be entered by its own door, so a later story can carry more readers than an earlier one.",
        description: "Explanation under the heading of the reading-depth bar strip, warning that the bars are not a funnel.",
    },
    "community.readers": {
        text: "READERS",
        description: "Column label over the number of accounts that have read something.",
    },
    "community.finished": {
        text: "FINISHED",
        description: "Column label over the number of accounts that have read every story of a chapter.",
    },
    "community.ofPlayers": {
        text: "{share} of players",
        description: "Line under a reader count naming what the percentage is a share of.",
    },
    "community.notMeasurable": {
        text: "Not measurable",
        description: "Shown in place of a chapter's finisher count when no story in it is gated behind a stage, so finishing it cannot be observed.",
    },
    "community.notMeasurable.why": {
        text: "Most synced accounts are known only through their stage records, and no story in this chapter is gated behind a stage, so nobody can be counted as having finished it.",
        description: "Tooltip on the 'Not measurable' finisher count explaining why the number is missing.",
    },
    "community.skipped": {
        text: "{count, plural, one {# chapter has no reader and is not listed.} other {# chapters have no reader and are not listed.}}",
        description: "Note under the least-read list saying how many chapters were left out for having no readers at all.",
    },
    "community.open": {
        text: "Open {name}",
        description: "Accessible name of a ranked row that opens the chapter sheet.",
    },
    "community.depth.picker": {
        text: "Chapter",
        description: "Label of the picker choosing which chapter the reading-depth bars are drawn for.",
    },
    "community.depth.drop": {
        text: "{first} start, {last} finish",
        description: "One line under the reading-depth bars: readers of the chapter's first story and of its last.",
    },
    "community.depth.bar": {
        text: "{name}: {readers} readers, {share} of players",
        description: "Accessible name and tooltip of one bar in the reading-depth strip.",
    },
    "community.depth.none": {
        text: "No chapter carries a reading-depth curve.",
        description: "Shown in place of the reading-depth bars when the aggregate carries no curve at all.",
    },
    "community.empty.rows": {
        text: "No chapter has a reader yet.",
        description: "Shown in place of the ranked lists when every chapter counts zero readers.",
    },
} satisfies MessageMap;

// `dynamic`: the kind badge is keyed from a `StoryKind` value
// (`browse.badge.${kind}`), which the Browse catalog declares.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
