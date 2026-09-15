import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.ignorePromotion": {
        text: "Ignore promotion",
        description: "Switch label: plan as though every operator were promoted far enough to use its best base skills.",
    },
    "profile.base.ignorePromotion.tooltip": {
        text: "Plan with every operator’s highest base skills, even the ones they are not promoted far enough to use. Skills you have not unlocked yet show greyed out.",
        description: "Tooltip on the 'Ignore promotion' switch. 'Base skills' are the game's own term for an operator's RIIC abilities. The apostrophe is a typographic one.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
