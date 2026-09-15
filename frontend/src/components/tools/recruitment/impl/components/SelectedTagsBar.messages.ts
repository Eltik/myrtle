import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "recruit.selected": {
        text: "Selected",
        description: "Small heading at the start of the bar listing the recruitment tags currently picked.",
    },
    "recruit.selected.remove": {
        text: "Remove {tag}",
        description: "Accessible name of the button that unpicks one tag. {tag} is a recruitment tag name from the game data.",
    },
    "recruit.selected.comboCount": {
        text: "{count, plural, one {# combo} other {# combos}}",
        description: "How many tag combinations the current selection yields. 'Combo' is short for 'combination'; the space is very narrow.",
    },
    "recruit.selected.reset": {
        text: "Reset",
        description: "Button that unpicks every selected tag.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
