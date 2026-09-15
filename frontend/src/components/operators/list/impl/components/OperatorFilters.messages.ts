import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "filters.aria": {
        text: "Operator filters",
        description: "Accessible name of the filter sidebar on the operator list.",
    },
    "filters.availability": {
        text: "Availability",
        description: "Field label over the Global / Upcoming choice.",
    },
    "filters.availability.global": {
        text: "Global",
        description: "Availability option: operators already released on the Global server.",
    },
    "filters.availability.upcoming": {
        text: "Upcoming (CN)",
        description: "Availability option: operators out on the Chinese server but not yet on Global. 'CN' is the server code and normally stays as-is.",
    },
    "filters.notes": {
        text: "Notes",
        description: "Field label over the has-notes choice, meaning this site's own written notes about an operator.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
