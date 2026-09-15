import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "ownership.e2.title": {
        text: "{e2Owners} of {owners} owners at E2 ({exact}%)",
        description: "Hover text on the community badge in E2 mode. {e2Owners} and {owners} are formatted counts, {exact} is the rate to two decimals. 'E2' is the game's name for the second promotion.",
    },
    "ownership.owned.title": {
        text: "{owners} owners ({exact}% of imported players)",
        description: "Hover text on the community badge in ownership mode. {owners} is a formatted count, {exact} the share to two decimals.",
    },
    "ownership.e2.aria": {
        text: "E2''d by {pct} of owners",
        description: "Accessible name of the community badge in E2 mode. {pct} is an already-formatted percentage. The doubled apostrophe is ICU escaping and renders as one.",
    },
    "ownership.owned.aria": {
        text: "Owned by {pct} of players",
        description: "Accessible name of the community badge in ownership mode. {pct} is an already-formatted percentage.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
