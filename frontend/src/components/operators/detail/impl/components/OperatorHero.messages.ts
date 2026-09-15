import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "hero.breadcrumb.operators": {
        text: "Operators",
        description: "Breadcrumb link back to the operator list.",
    },
    "hero.position.ranged": {
        text: "Ranged",
        description: "Badge naming where the operator can be placed: on ranged tiles. The game's own term for the placement type.",
    },
    "hero.position.melee": {
        text: "Melee",
        description: "Badge naming where the operator can be placed: on melee tiles. The game's own term for the placement type.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
