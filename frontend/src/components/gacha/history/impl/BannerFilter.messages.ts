import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.filter.label": {
        text: "Banners",
        description: "Label of the field that limits the pull history to some banners. A banner is one headhunting event the player pulled on.",
    },
    "history.filter.placeholder": {
        text: "Every banner",
        description: "Placeholder of the banner filter while no banner is chosen, meaning the whole history is shown.",
    },
    "history.filter.noMatches": {
        text: "No banner matches",
        description: "Shown in the banner filter's menu when the typed text matches no banner the player pulled on.",
    },
    "history.filter.pulls": {
        text: "{count, plural, one {# pull} other {# pulls}}",
        description: "How many pulls the player made on one banner, shown beside its name in the banner filter's menu.",
    },
    "history.filter.clear": {
        text: "Show every banner",
        description: "Button that removes every banner from the filter, showing the whole history again.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
