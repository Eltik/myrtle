import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.sustain.holds": {
        text: "Holds up",
        description: "Badge: the planned rotation keeps everybody working for the whole simulated stretch.",
    },
    "profile.base.sustain.depletes": {
        text: "Depletes",
        description: "Badge: somebody runs out of morale before the simulated stretch is over.",
    },
    "profile.base.sustain.holdsDetail": {
        text: "Nobody runs dry over {days, plural, one {# simulated day} other {# simulated days}}.",
        description: "Sentence beside the 'Holds up' badge. 'Runs dry' means an operator's morale reaches zero; the horizon is a handful of days.",
    },
    "profile.base.sustain.depletesDetail": {
        text: "{count, plural, one {# operator} other {# operators}} hit zero morale within {days, plural, one {# day} other {# days}}.",
        description: "Sentence beside the 'Depletes' badge. Both counts are small.",
    },
    "profile.base.sustain.dormOverflow": {
        text: " Dorms are {beds, plural, one {# bed} other {# beds}} short at peak.",
        description: "Appended to either sentence when the dormitories cannot hold everyone resting at once. Keep the leading space; 'dorms' are the in-game dormitories.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
