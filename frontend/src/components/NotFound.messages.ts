import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "common";

export const messages = {
    "notFound.title": {
        text: "We're counting on you again today, little apple.",
        description: "Heading line of the 404 page, a line Myrtle says in-game. Keep it warm and in character rather than literal.",
    },
    "notFound.description": {
        text: "There's nothing here right now.",
        description: "Second line of the 404 page, under the heading: the page the visitor asked for does not exist.",
    },
    "notFound.returnHome": {
        text: "Take me home",
        description: "The only 404 action: navigates to the site root.",
    },
    "notFound.imageAlt": {
        text: "Myrtle, face-down after dropping her snacks",
        description: "Alt text for the chibi of Myrtle that stands in for the 0 of the giant 404.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
