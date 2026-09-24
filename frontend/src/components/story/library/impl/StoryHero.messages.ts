import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "hero.continue": {
        text: "Continue",
        description: "Button on the hero card that resumes the last story at the line the reader stopped on.",
    },
    "hero.startReading": {
        text: "Start reading",
        description: "Button on the hero card offered to a visitor who has read nothing yet; opens the Prologue.",
    },
    "hero.fromStart": {
        text: "From the start",
        description: "Button on the hero card that reopens the story at its first line.",
    },
    "hero.viewChapter": {
        text: "View chapter",
        description: "Button on the hero card that scrolls to the story's chapter and opens its story list.",
    },
    "hero.resumeAt": {
        text: "Stopped at line {line} of {total}",
        description: "Position line on the hero card; 1-based line counts.",
    },
    "hero.neverOpened": {
        text: "Not opened yet",
        description: "Position line on the hero card when the story has no saved position.",
    },
    "hero.groupRead": {
        text: "{read} of {total} read",
        description: "How much of the hero story's chapter has been finished.",
    },
    "hero.kicker": {
        text: "Continue reading",
        description: "Label above the hero card's story name for a returning reader.",
    },
    "hero.kickerFresh": {
        text: "Start here",
        description: "Label above the hero card's story name for a first-time visitor.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
