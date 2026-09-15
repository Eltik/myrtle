import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "planner.delete.names": {
        text: "{names} and {count} more",
        description: "Shortens a long list of plan names, e.g. 'Amiya, Texas, Exusiai and 4 more'. {names} is already a comma-separated list of operator names from the game data.",
    },
    "planner.delete.fallbackName": {
        text: "This plan",
        description: "Stands in for the plan name in the confirmation dialog while the names are unavailable.",
    },
    "planner.delete.title": {
        text: "{count, plural, one {Delete plan?} other {Delete # plans?}}",
        description: "Title of the dialog confirming that plans will be deleted.",
    },
    "planner.delete.desc": {
        text: "{names} and {count, plural, one {its} other {their}} promotion, level, skill, and module goals will be permanently removed. This cannot be undone.",
        description: "Body of the delete confirmation. {names} is the bold list of plan names, or planner.delete.fallbackName when they are not to hand, and may move wherever the sentence needs it. 'Promotion' and 'module' are the game's own progression systems.",
    },
    "planner.delete.cancel": {
        text: "Cancel",
        description: "Button that dismisses the delete confirmation without deleting anything.",
    },
    "planner.delete.confirm": {
        text: "{count, plural, one {Delete plan} other {Delete # plans}}",
        description: "Button that carries out the deletion.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
