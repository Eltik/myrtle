import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.primes.currency": {
        text: "Originite Prime",
        description: "Alt text and hover title of the currency icon beside every Originite Prime amount. Originite Prime is the game's premium currency; use your server's official name for it.",
    },
    "release.primes.income": {
        text: "Income",
        description: "Row label in the balance box: Originite Prime coming in.",
    },
    "release.primes.expenses": {
        text: "Expenses",
        description: "Row label in the balance box: Originite Prime going out.",
    },
    "release.primes.balance": {
        text: "Balance",
        description: "Row label in the balance box: what is left after income and expenses.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
