import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.breadcrumb.label": {
        text: "breadcrumb",
        description: "Accessible name of the breadcrumb trail above the community gacha heading.",
    },
    "community.breadcrumb.gacha": {
        text: "Gacha",
        description: "First breadcrumb crumb, naming the section this page lives in. 'Gacha' is the game's random-draw system and stays as-is.",
    },
    "community.breadcrumb.current": {
        text: "Community",
        description: "Last breadcrumb crumb: this page.",
    },
    "community.header.title": {
        text: "What everyone’s {emphasis}.",
        description: "Heading of the community gacha page. {emphasis} is the accent-coloured word labelled by community.header.titleEmphasis and may sit anywhere the sentence needs it. Keep the full stop.",
    },
    "community.header.titleEmphasis": {
        text: "pulling",
        description: "The accent-coloured word substituted into community.header.title as {emphasis}. 'Pulling' is the community term for spending on headhunting.",
    },
    "community.header.blurbUsers": {
        text: "{count} doctors",
        description: "The bold count in the sentence under the community gacha heading. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "community.header.blurb": {
        text: "Aggregated across {doctors} who opted into anonymous sharing. Rates are observational, not official.",
        description: "Sentence under the community gacha heading. {doctors} is the bold count of contributing players, labelled by community.header.blurbUsers, and may move wherever the sentence needs it.",
    },
    "community.header.updated": {
        text: "updated {when}",
        description: "Chip saying how long ago the figures were recomputed, e.g. 'updated 5m ago'. Lowercase, in a very narrow chip.",
    },
    "community.header.cached": {
        text: "cached",
        description: "Chip shown when the figures came from the server's cache rather than a fresh computation. Rendered in uppercase by the style.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
