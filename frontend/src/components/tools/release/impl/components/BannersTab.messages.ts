import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.banners.rosterNote": {
        text: "Standard banners carry no roster in CN data; rows without operators are real banners with unknown rate-ups.",
        description: "Note above the banner list. 'CN' is the Chinese game server; a 'rate-up' is a boosted pull chance.",
    },
    "release.banners.independence.title": {
        text: "Measured per rule type: how often CN and EN run a different number of pools of that type under the same event. At half or more, the type is EN-scheduled and gets no estimate.",
        description: "Tooltip on the note about EN-scheduled banner types. 'CN' and 'EN' are the Chinese and English game servers; 'pool' is the game's word for one banner.",
    },
    "release.banners.independence": {
        text: "EN schedules these itself: {list}. Their rows say so unless the rate-ups match a known EN pool.",
        description: "Note listing the banner types the English server schedules on its own. {list} is a comma-separated list built from the item below.",
    },
    "release.banners.independence.item": {
        text: "{type} ({rate}% of {count} events)",
        description: "One entry in the EN-scheduled list, e.g. 'limited (62% of 34 events)'. {type} is a banner rule type from the game data, lowercased.",
    },
    "release.banners.showPast": {
        text: "Show past",
        description: "Switch label: also list banners whose window has already closed.",
    },
    "release.banners.count": {
        text: "{shown} of {total}",
        description: "How many banners the filters keep out of all of them, e.g. '18 of 240'.",
    },
    "release.banners.empty.title": {
        text: "No banners",
        description: "Empty-state heading when the banner list is empty.",
    },
    "release.banners.empty.none": {
        text: "The backend returned no CN pools.",
        description: "Empty-state body when the server sent nothing at all. 'Pool' is the game's own word for one banner.",
    },
    "release.banners.empty.filtered": {
        text: "Every banner is filtered out. Turn on Show past.",
        description: "Empty-state body when the filter hid everything. 'Show past' names the switch above, so keep it matching.",
    },
    "release.banners.faceStrip.title": {
        text: "{banner}: rate-up operators",
        description: "Tooltip on the row of operator portraits standing in for missing banner art. A 'rate-up' is a boosted pull chance.",
    },
    "release.banners.eventArt": {
        text: "Event art: {event}",
        description: "Tooltip on banner artwork borrowed from the event it runs under.",
    },
    "release.banners.align.content": {
        text: "matched by rate-ups",
        description: "How a CN banner was matched to an EN one: by the operators it boosts.",
    },
    "release.banners.align.anchor": {
        text: "matched by event window (heuristic)",
        description: "How a CN banner was matched to an EN one: by the event it runs under, which is a guess rather than a certainty.",
    },
    "release.banners.roster.rateUps": {
        text: "Rate-ups",
        description: "Label before the operators a CN banner boosts. A 'rate-up' is a boosted pull chance.",
    },
    "release.banners.roster.enRateUps": {
        text: "EN rate-ups",
        description: "Label before the operators the matching EN banner boosts. 'EN' is the English game server.",
    },
    "release.banners.roster.entered": {
        text: "Rate-ups (entered)",
        description: "Label before rate-up operators a person typed into this site's override table.",
    },
    "release.banners.roster.entered.title": {
        text: "Rate-ups entered on this banner's override row; the CN client carries no roster for this banner kind",
        description: "Tooltip on the '(entered)' rate-up label.",
    },
    "release.banners.roster.debut": {
        text: "Debut, from the ledger",
        description: "Label before the operators making their first appearance on this banner. 'Ledger' is this site's record of debuts.",
    },
    "release.banners.cn": {
        text: "CN",
        description: "Small heading before a banner's Chinese-server date. The game server's usual abbreviation.",
    },
    "release.banners.since": {
        text: "since {date}",
        description: "Shown in place of a date for a pool that opened once and never closed.",
    },
    "release.banners.with": {
        text: "with {event}",
        description: "Names the event a banner runs under, e.g. 'with Ideal City'. {event} is that event's name and may move wherever the phrase needs it. Used only when the match was made from the banner's rate-ups, which is a certainty.",
    },
    "release.banners.probablyWith": {
        text: "probably with {event}",
        description: "Same as the phrase above, but for a banner matched to an event only by the dates lining up, which can name the wrong event. Must read as a guess. {event} is the event's name and may move wherever the phrase needs it.",
    },
} satisfies MessageMap;

// `dynamic`: the match-method keys are stored in a lookup table and resolved as
// `t(ALIGN_LABEL_KEYS[method])`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
