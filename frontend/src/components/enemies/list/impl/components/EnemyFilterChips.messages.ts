import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "chips.threat": {
        text: "Threat",
        description: "Label before the Normal / Elite / Boss filter chips. Short uppercase label on the chip rail.",
    },
    "chips.damage": {
        text: "Damage",
        description: "Label before the damage-type filter chips. Short uppercase label on the chip rail.",
    },
    "chips.range": {
        text: "Range",
        description: "Label before the Melee / Ranged / None filter chips. Short uppercase label on the chip rail.",
    },
    "chips.race": {
        text: "Race",
        description: "Label before the race filter chips; the race names themselves are game data.",
    },
    "chips.immune": {
        text: "Immune to",
        description: "Label before the status-effect filter chips (Stun, Sleep, ...). Selecting several shows only enemies immune to ALL of them. Short uppercase label on the chip rail.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
