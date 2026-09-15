import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "breadcrumb.label": {
        text: "breadcrumb",
        description: "Accessible name of the breadcrumb <nav> landmark. Lowercase on purpose: screen readers announce it as 'breadcrumb navigation'.",
    },
    "breadcrumb.more": {
        text: "More",
        description: "Screen-reader-only text behind the ellipsis that stands in for the breadcrumb steps collapsed out of the trail.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
