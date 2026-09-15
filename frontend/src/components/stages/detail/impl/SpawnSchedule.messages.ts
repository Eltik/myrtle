import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "spawns.title": {
        text: "Spawn Schedule",
        description: "Kicker over the table of enemy spawns, ordered by the second they appear.",
    },
    "spawns.groups": {
        text: "{count, plural, one {# group} other {# groups}}",
        description: "How many spawn entries the schedule holds, beside its heading. A 'group' is one spawn action, which may release several enemies.",
    },
    "spawns.hiddenRoutes": {
        text: "Hidden Routes",
        description: "Label before the pills naming spawn groups the game hides until a condition is met.",
    },
    "spawns.col.enemy": {
        text: "Enemy",
        description: "Spawn table column header: which enemy spawns.",
    },
    "spawns.col.wave": {
        text: "Wave",
        description: "Spawn table column header: which wave the spawn belongs to. Also the label on the phone-width card.",
    },
    "spawns.col.count": {
        text: "Count",
        description: "Spawn table column header: how many enemies this entry releases. Also the label on the phone-width card.",
    },
    "spawns.col.interval": {
        text: "Interval",
        description: "Spawn table column header: seconds between the enemies of one entry. Also the label on the phone-width card.",
    },
    "spawns.col.preDelay": {
        text: "Pre-Delay",
        description: "Spawn table column header: seconds of delay before the entry fires. Also the label on the phone-width card.",
    },
    "spawns.col.start": {
        text: "Start ~",
        description: "Spawn table column header: approximate second the entry starts. The '~' marks it as approximate; keep it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
