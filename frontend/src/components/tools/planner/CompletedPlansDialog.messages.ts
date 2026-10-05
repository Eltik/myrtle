import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "planner.completed.title": {
        text: "{count, plural, one {Completed plan} other {Completed plans}}",
        description: "Title of the dialog listing plans whose targets are all reached.",
    },
    "planner.completed.desc": {
        text: "Your last sync shows these operators at their targets. Delete the plans you no longer need.",
        description: "Body of the completed-plans dialog, above the checklist of plans.",
    },
    "planner.completed.keep": {
        text: "Keep them",
        description: "Button that closes the completed-plans dialog without deleting anything and hides the notice until another plan completes.",
    },
    "planner.completed.delete": {
        text: "{count, plural, =0 {Delete} one {Delete # plan} other {Delete # plans}}",
        description: "Button that deletes the checked completed plans; the count is how many are checked.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
