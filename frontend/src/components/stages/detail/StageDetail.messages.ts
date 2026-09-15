import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "detail.breadcrumb": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb <nav> landmark above the stage title.",
    },
    "detail.breadcrumb.stages": {
        text: "Stages",
        description: "Breadcrumb link back to the stage list. The last crumb is the stage's own code, which is game data.",
    },
    "detail.notFound.title": {
        text: "Stage not found",
        description: "Heading shown when the URL names a stage id the backend has no stage for.",
    },
    "detail.notFound.body": {
        text: "No stage with id {id} exists.",
        description: "Shown when the URL names a stage id the backend has no stage for. {id} is that id, rendered in a monospace box, and may move wherever the sentence needs it.",
    },
    "detail.notFound.back": {
        text: "Back to Stages",
        description: "Link out of the stage-not-found page, back to the stage list.",
    },
    "detail.noMap": {
        text: "No map data available for this stage.",
        description: "Empty state in place of the map board when the backend has no level file for the stage.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
