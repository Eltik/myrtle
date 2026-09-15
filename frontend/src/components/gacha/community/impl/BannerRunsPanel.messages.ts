import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.runs.kicker": {
        text: "Banner runs",
        description: "Small uppercase label above the panel listing recent, running and announced banners.",
    },
    "community.runs.titleActive": {
        text: "{count} active right now.",
        description: "Banner-runs heading when at least one banner is running. {count} is the accent-coloured number of running banners and may sit anywhere the sentence needs it.",
    },
    "community.runs.titleNone": {
        text: "Recent and upcoming banners.",
        description: "Banner-runs heading used when no banner is running at the moment.",
    },
    "community.runs.empty": {
        text: "No banners match this filter.",
        description: "Empty state when the chosen filter leaves no banners in the list.",
    },
    "community.runs.dateRange": {
        text: "{open} → {close}",
        description: "A banner's run window, e.g. 'Mar 3, 2025 → Mar 17, 2025'. Keep the arrow.",
    },
    "community.runs.noPulls": {
        text: "No community pulls yet",
        description: "Shown on a banner card when nobody who shares their records has pulled on it. Rendered uppercase.",
    },
    "community.runs.communityPulls": {
        text: "community pulls",
        description: "Unit after the pull count on a banner card. Rendered uppercase.",
    },
    "community.runs.contributors": {
        text: "{count} contributing doctors",
        description: "Tooltip on a banner card's player count. 'Doctor' is what Arknights calls the player.",
    },
    "community.runs.people": {
        text: "{count} ppl",
        description: "Compact player count on a banner card, e.g. '1.2k ppl'. 'ppl' abbreviates people; the space is a non-breaking one and the card is narrow.",
    },
    "community.runs.moreOperators": {
        text: "{count} more",
        description: "Tooltip on the '+3' chip that stands in for featured operators the card has no room to show.",
    },
    "community.runs.guarantee": {
        text: "{name} within {count} pulls",
        description: "A banner's guarantee, e.g. 'Guaranteed 5★ within 10 pulls'. {name} is the guarantee's own name from the game data where it has one.",
    },
    "community.runs.guaranteeDefault": {
        text: "Guaranteed 5★",
        description: "Stand-in name for a banner's guarantee when the game data leaves it blank.",
    },
    "community.runs.featuredSixStar": {
        text: "Featured 6★",
        description: "Label over the row of 6-star operators a banner boosts. Rendered uppercase.",
    },
    "community.runs.sort": {
        text: "Sort",
        description: "Label of the segmented control choosing how the banner cards are ordered. Rendered uppercase.",
    },
    "community.runs.filter": {
        text: "Filter",
        description: "Label of the segmented control choosing which banners are listed. Rendered uppercase.",
    },
    "community.runs.filter.all": {
        text: "All",
        description: "Banner filter: every banner in the window. Fits a narrow segmented button.",
    },
    "community.runs.filter.active": {
        text: "Active",
        description: "Banner filter: only banners running right now. Fits a narrow segmented button.",
    },
    "community.runs.sort.timeline": {
        text: "Timeline",
        description: "Banner sort: running first, then announced, then most recently ended. Fits a narrow segmented button.",
    },
    "community.runs.sort.pulls": {
        text: "Pulls",
        description: "Banner sort: most community pulls first. Fits a narrow segmented button.",
    },
    "community.runs.sort.popularity": {
        text: "Popularity",
        description: "Banner sort: most contributing players first. Fits a narrow segmented button.",
    },
    "community.runs.metric.pulls": {
        text: "community pulls",
        description: "Names the metric a card was ranked by, inside the rank tooltip: 'Ranked #3 by community pulls'.",
    },
    "community.runs.metric.popularity": {
        text: "contributing doctors",
        description: "Names the metric a card was ranked by, inside the rank tooltip: 'Ranked #3 by contributing doctors'. 'Doctor' is what Arknights calls the player.",
    },
    "community.runs.rankTitle": {
        text: "Ranked #{rank} by {metric}",
        description: "Tooltip on a banner card's rank badge while a metric sort is active.",
    },
    "community.runs.showFewer": {
        text: "Show fewer",
        description: "Button that collapses the banner list back to the first page. Rendered uppercase.",
    },
    "community.runs.showAll": {
        text: "Show all {count}",
        description: "Button that expands the banner list to every match. Rendered uppercase.",
    },
} satisfies MessageMap;

// `dynamic`: the filter, sort and metric keys are stored in the lookup tables
// at the top of the component and resolved as `t(option.labelKey)`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
