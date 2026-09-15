import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.timing.kicker": {
        text: "Pull timing",
        description: "Small uppercase label above the charts showing when the community pulls.",
    },
    "community.timing.title": {
        text: "When the community pulls.",
        description: "Heading of the panel charting pull activity over time.",
    },
    "community.timing.legend.totalPulls": {
        text: "Total pulls",
        description: "Chart legend entry for the line showing all pulls, whatever the banner. Rendered uppercase.",
    },
    "community.timing.byDate": {
        text: "By date · last {count} days",
        description: "Heading over the day-by-day activity chart, saying how long the window is. Rendered uppercase; keep the middle dot.",
    },
    "community.timing.byHour": {
        text: "By hour of day · UTC",
        description: "Heading over the hour-of-day chart. The hours are in UTC rather than the viewer's zone. Rendered uppercase; keep the middle dot.",
    },
    "community.timing.byDayOfWeek": {
        text: "By day of week",
        description: "Heading over the day-of-week chart. Rendered uppercase.",
    },
    "community.timing.hover.pulls": {
        text: "pulls",
        description: "Unit after the pull count in the chart's hover card, e.g. '1,204 pulls'. Rendered uppercase.",
    },
    "community.timing.hover.more": {
        text: "+{count} more",
        description: "Last line of the chart's hover card when more banners were running that day than the card can list.",
    },
    "community.timing.band": {
        text: "{name} - {type}",
        description: "Tooltip on a banner's run bar under the chart: the banner's name, then its bucket. The name comes from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
