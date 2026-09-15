import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.top.kicker": {
        text: "Most pulled · per rarity",
        description: "Small uppercase label above the player's most-pulled operators. Keep the middle dot.",
    },
    "history.top.title": {
        text: "Your top operators.",
        description: "Heading of the panel ranking the operators the player pulled most often.",
    },
    "history.top.empty": {
        text: "No {rarity}★ operators pulled yet.",
        description: "Empty state when the player has pulled nothing of the selected rarity. {rarity} is a star count, 6, 5 or 4.",
    },
    "history.top.col.operator": {
        text: "Operator",
        description: "Column heading for the operator's name. Rendered uppercase.",
    },
    "history.top.col.copies": {
        text: "Copies",
        description: "Column heading for how many times the player pulled that operator. Rendered uppercase in a narrow column.",
    },
    "history.top.col.frequency": {
        text: "Frequency",
        description: "Column heading over the bar comparing each operator's count with the most-pulled one. Rendered uppercase.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
