import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "pager.first": {
        text: "Go to first page",
        description: "Accessible name of the icon-only jump-to-start control.",
    },
    "pager.previous": {
        text: "Go to previous page",
        description: "Accessible name of the back control.",
    },
    "pager.next": {
        text: "Go to next page",
        description: "Accessible name of the forward control.",
    },
    "pager.last": {
        text: "Go to last page",
        description: "Accessible name of the icon-only jump-to-end control.",
    },
    "pager.page": {
        text: "Go to page {page}",
        description: "Accessible name of one numbered page button. {page} is the page number.",
    },
    "pager.goToPage": {
        text: "Go to page",
        description: "Accessible name of the submit button in the jump-to-page popover, which has no visible text.",
    },
    "pager.pageOf": {
        text: "Page {current} of {total}",
        description: "Page readout in the pager, e.g. 'Page 3 of 12'. Both numbers are bold and may move wherever the phrase needs them.",
    },
    "pager.jump.title": {
        text: "Jump to page",
        description: "Heading of the popover behind the ellipsis in the pager.",
    },
    "pager.jump.aria": {
        text: "Jump to a page between {from} and {to}",
        description: "Accessible name of the ellipsis that stands in for skipped pages. {from} and {to} are the first and last hidden page numbers.",
    },
    "pager.jump.range": {
        text: "Hidden range: {from}-{to} · Total {total}",
        description: "Caption under the jump-to-page heading. Keep the middle dot separator.",
    },
    "pager.jump.input": {
        text: "Page number",
        description: "Accessible name of the page-number input in the jump-to-page popover.",
    },
    "pager.jump.hint": {
        text: "Enter a number from 1 to {total}.",
        description: "Help text under the jump-to-page input. {total} is the last page number.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
