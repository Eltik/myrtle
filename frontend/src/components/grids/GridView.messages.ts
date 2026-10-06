import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "view.back": {
        text: "All grids",
        description: "Link from a grid's page back to the grid browser.",
    },
    "view.by": {
        text: "by {name}",
        description: "Byline on a grid's page. {name} is the owner's display name.",
    },
    "view.forks": {
        text: "{count, plural, one {{formatted} use} other {{formatted} uses}}",
        description: "How many grids were made from this one with 'Use this template'. {formatted} is {count} formatted for the locale.",
    },
    "view.cellHint": {
        text: "Tap a cell to see it full screen.",
        description: "Shown under a grid on its page on phones only, where the cells are too small to read: tapping one opens the full-screen cell viewer.",
    },
    "view.unlisted": {
        text: "Unlisted",
        description: "Shown in a grid's byline when it is hidden from the browse list but reachable by link.",
    },
    "view.templateOf": {
        text: "Template:",
        description: "Precedes a link to the grid this one was made from with 'Use this template'.",
    },
    "view.fork": {
        text: "Use this template",
        description: "Button that copies the grid's title, size and labels, without its picks, into a new grid the viewer owns, then opens it in the editor.",
    },
    "view.fork.errTitle": {
        text: "Could not copy this grid",
        description: "Toast title when 'Use this template' failed; the body is the error.",
    },
    "view.edit": {
        text: "Edit",
        description: "Opens the grid in the editor. Shown to its owner.",
    },
    "view.download": {
        text: "Download PNG",
        description: "Downloads the grid as a PNG image.",
    },
    "view.downloading": {
        text: "Preparing…",
        description: "Replaces 'Download PNG' while the image is being made.",
    },
    "view.download.errTitle": {
        text: "Download failed",
        description: "Toast title when the PNG could not be made.",
    },
    "view.download.errBody": {
        text: "The image could not be generated. Try again in a moment.",
        description: "Toast body when the PNG could not be made.",
    },
    "view.copy": {
        text: "Copy link",
        description: "Copies the grid's page address to the clipboard.",
    },
    "view.copy.okTitle": {
        text: "Link copied",
        description: "Toast after the grid's link was copied.",
    },
    "view.copy.errTitle": {
        text: "Could not copy the link",
        description: "Toast when the clipboard refused the link.",
    },
    "view.missing.title": {
        text: "Grid not found",
        description: "Heading of a grid page whose grid does not exist.",
    },
    "view.missing.body": {
        text: "It may have been deleted, or the link is wrong.",
        description: "Body of a grid page whose grid does not exist.",
    },
    "view.missing.action": {
        text: "Browse grids",
        description: "Button from the not-found notice to the grid browser.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
