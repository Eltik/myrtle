import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.events.stageOnly": {
        text: "Stage events only",
        description: "Switch label: show only events that come with combat stages, not pure-story ones.",
    },
    "release.events.showPast": {
        text: "Show past",
        description: "Switch label: also list events whose window has already closed.",
    },
    "release.events.count": {
        text: "{shown} of {total}",
        description: "How many events the filters keep out of all of them, e.g. '18 of 240'.",
    },
    "release.events.empty.title": {
        text: "No events",
        description: "Empty-state heading when the event list is empty.",
    },
    "release.events.empty.none": {
        text: "The backend returned no CN activities.",
        description: "Empty-state body when the server sent nothing at all. 'CN' is the Chinese game server; 'activity' is its own word for an event.",
    },
    "release.events.empty.filtered": {
        text: "Every event is filtered out. Turn on Show past or turn off Stage events only.",
        description: "Empty-state body when the filters hid everything. 'Show past' and 'Stage events only' name the two switches above, so keep them matching.",
    },
    "release.events.cn": {
        text: "CN",
        description: "Small heading before an event's Chinese-server dates. The game server's usual abbreviation.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
