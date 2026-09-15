import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "activity.title": {
        text: "Commit activity",
        description: "Heading above the commits-per-day bar strip on the changelog page. Rendered uppercase.",
    },
    "activity.busiest": {
        text: "Busiest",
        description: "Label to the left of the busiest day's date and commit count, reading 'Busiest Sep 12 · 9'. An adjective.",
    },
    "activity.aria": {
        text: "Commits per day over the last {days} days. Busiest day {label} with {count} commits.",
        description: "Accessible description of the commits-per-day bar strip. {days} and {count} are rendered unformatted, and both labels are always plural in the source.",
    },
    "activity.aria.noDay": {
        text: "n/a",
        description: "Stand-in used in the bar strip's accessible description when the range holds no commits at all. Short for 'not applicable'.",
    },
    "activity.bar": {
        text: "{label}: {count, plural, one {# commit} other {# commits}}",
        description: "Hover title of one day's bar in the commit-activity strip. {label} is an already-formatted short date such as 'Sep 12'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
