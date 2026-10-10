import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "live.eyebrow": {
        text: "Happening now · {server}",
        description: "Small uppercase label above the home page's list of running events and banners. {server} is the game server the reader picked, by region name, such as 'Global' or 'China'.",
    },
    "live.title": {
        text: "Ending soonest",
        description: "Heading of the home page section listing the running events and banners that close first.",
    },
    "live.seeAll": {
        text: "All current banners →",
        description: "Link from the home page's live section to the community gacha page, which lists every banner open right now on the reader's game server. The arrow is part of the label.",
    },
    "live.empty": {
        text: "Nothing running on {server} right now.",
        description: "Shown in the home page's live section when no event or banner is open. {server} is the game server's region name, such as 'Global' or 'China'.",
    },
    "live.upcoming": {
        text: "See what's coming next →",
        description: "Link under the empty message in the home page's live section, to the release planner, which forecasts upcoming events and banners. Shown only on the server the planner forecasts. The arrow is part of the label.",
    },
    "live.error": {
        text: "Couldn't load what's running on {server}.",
        description: "Shown in the home page's live section when the event or banner data failed to load. {server} is the game server's region name, such as 'Global' or 'China'.",
    },
    "live.eventDetail": {
        text: "Event · ends {date}",
        description: "Second line of a running event's card on the home page. {date} is a short month and day, such as 'Oct 7'.",
    },
    "live.bannerKind": {
        text: "{kind} banner",
        description: "Tag on a running banner's card on the home page. {kind} is the banner type, such as 'Limited' or 'Kernel'. A banner is a gacha pool.",
    },
    "live.bannerUnlisted": {
        text: "Rate-up operators not listed",
        description: "Second line of a running banner's card on the home page when the game data names no boosted operators for it and has no art for it. 'Rate-up' is the game's term for a boosted pull chance.",
    },
    "live.bannerMore": {
        text: "{names} +{count} more",
        description: "A banner's featured operators when there are more than three: {names} is the first two, comma-separated, and {count} is how many are left out.",
    },
    "live.countdown": {
        text: "{days}d {clock}",
        description: "Time left before an event or banner closes, ticking every second. {days} is whole days; {clock} is hours, minutes and seconds as HH:MM:SS. Keep it short.",
    },
    "live.ended": {
        text: "ended",
        description: "Shown in place of the countdown once an event or banner has closed.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
