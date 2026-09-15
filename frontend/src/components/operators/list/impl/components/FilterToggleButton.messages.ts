import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "filters.toggle.hide": {
        text: "Hide filters",
        description: "Accessible name and tooltip of the filter-panel toggle while the panel is open.",
    },
    "filters.toggle.show": {
        text: "Show filters",
        description: "Accessible name and tooltip of the filter-panel toggle while the panel is closed.",
    },
    "filters.toggle.label": {
        text: "Filter",
        description: "Small uppercase caption inside the toggle button on desktop. Very tight space - one short word.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
