import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "pagination.label": {
        text: "pagination",
        description: "Accessible name of the pager <nav> landmark. Lowercase on purpose: screen readers announce it as 'pagination navigation'.",
    },
    "pagination.previousPage": {
        text: "Go to previous page",
        description: "Accessible name of the previous-page control, which shows only an arrow on small screens.",
    },
    "pagination.nextPage": {
        text: "Go to next page",
        description: "Accessible name of the next-page control, which shows only an arrow on small screens.",
    },
    "pagination.previous": {
        text: "Previous",
        description: "Visible label beside the back arrow. Hidden below the sm breakpoint, so keep it short.",
    },
    "pagination.next": {
        text: "Next",
        description: "Visible label beside the forward arrow. Hidden below the sm breakpoint, so keep it short.",
    },
    "pagination.morePages": {
        text: "More pages",
        description: "Screen-reader-only text behind the ellipsis that stands in for skipped page numbers.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
