import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "page.kicker": {
        text: "Changelog",
        description: "Eyebrow label above the changelog page heading. Rendered uppercase.",
    },
    "page.title": {
        text: "What's shipped",
        description: "Heading of the changelog page. 'Shipped' means released to users. No full stop in the source.",
    },
    "page.blurb": {
        text: "Every commit pushed to myrtle.moe, pulled live from GitHub and grouped by day.",
        description: "Caption under the changelog heading. 'myrtle.moe' is this site's name and 'GitHub' a product name; both stay as-is.",
    },
    "page.tab.notes": {
        text: "Release notes",
        description: "Tab showing the hand-written release notes rather than the raw commit feed.",
    },
    "page.tab.commits": {
        text: "Commits",
        description: "Tab showing the raw commit feed rather than the hand-written notes. A noun.",
    },
    "page.stat.commits": {
        text: "commits · {range}",
        description: "Caption under the commit count, naming the active time window, e.g. 'commits · this week'. {range} is a lower-case range name. Always plural.",
    },
    "page.stat.contributors": {
        text: "{count, plural, one {contributor} other {contributors}}",
        description: "Caption under the count of people who authored commits in the active window. The number itself is rendered above this label, not inside it.",
    },
    "page.stat.branch": {
        text: "branch",
        description: "Caption under the name of the git branch the commits were read from.",
    },
    "page.stat.lastSynced": {
        text: "last synced",
        description: "Caption under the relative time of the last fetch from GitHub, shown when no branch is pinned.",
    },
    "page.group.today": {
        text: "Today",
        description: "Heading of the commit group for the current day.",
    },
    "page.group.yesterday": {
        text: "Yesterday",
        description: "Heading of the commit group for the previous day.",
    },
    "page.group.commits": {
        text: "{count, plural, one {# commit} other {# commits}}",
        description: "How many commits landed on one day, shown next to that day's heading.",
    },
    "page.truncated": {
        text: "Older history is capped - see the full log on GitHub.",
        description: "Note under the commit feed when the fetch window hit its cap. 'GitHub' is a product name and stays as-is.",
    },
    "page.statsLink": {
        text: "Looking for site stats instead? →",
        description: "Link at the foot of the changelog page, pointing at the site-statistics page. The arrow is part of the label.",
    },
    "page.empty.title": {
        text: "No commits in {range}",
        description: "Empty state heading when the active window holds no commits, e.g. 'No commits in this week'. {range} is a lower-case range name. No full stop in the source.",
    },
    "page.empty.body": {
        text: "Nothing landed in this window yet. Try a wider range above to see recent work.",
        description: "Empty state body under 'No commits in …'.",
    },
    "range.day.label": {
        text: "Today",
        description: "Time-window tab covering the last 24 hours.",
    },
    "range.day.short": {
        text: "24h",
        description: "Narrow-screen form of the 'Today' time-window tab. Keep it to three or four characters.",
    },
    "range.day.lower": {
        text: "today",
        description: "The 'Today' window named mid-sentence, as in 'No commits in today' and 'commits · today'. Lower-case in English.",
    },
    "range.week.label": {
        text: "This Week",
        description: "Time-window tab covering the last seven days. Title case in the source.",
    },
    "range.week.short": {
        text: "7d",
        description: "Narrow-screen form of the 'This Week' time-window tab. 'd' is short for days.",
    },
    "range.week.lower": {
        text: "this week",
        description: "The 'This Week' window named mid-sentence, as in 'No commits in this week'. Lower-case in English.",
    },
    "range.month.label": {
        text: "This Month",
        description: "Time-window tab covering the last thirty days. Title case in the source.",
    },
    "range.month.short": {
        text: "30d",
        description: "Narrow-screen form of the 'This Month' time-window tab. 'd' is short for days.",
    },
    "range.month.lower": {
        text: "this month",
        description: "The 'This Month' window named mid-sentence, as in 'No commits in this month'. Lower-case in English.",
    },
    "range.quarter.label": {
        text: "3 Months",
        description: "Time-window tab covering the last ninety days. Title case in the source.",
    },
    "range.quarter.short": {
        text: "90d",
        description: "Narrow-screen form of the '3 Months' time-window tab. 'd' is short for days.",
    },
    "range.quarter.lower": {
        text: "3 months",
        description: "The '3 Months' window named mid-sentence, as in 'No commits in 3 months'. Lower-case in English.",
    },
} satisfies MessageMap;

// `dynamic`: the range names are resolved through RANGE_KEYS, keyed by the
// range id that `lib/api/changelog` defines, so those twelve keys have no
// literal call site for the extractor to match.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
