import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "routeError.title": {
        text: "Couldn't load this page",
        description: "Heading shown in place of a page whose data or code failed to load. No full stop: it is a heading.",
    },
    "routeError.description": {
        text: "Try loading it again. If it fails a second time, the rest of the site still works, so start again from the home page.",
        description: "Line under the heading that names the next step: retry first, then go home. The two buttons below it do exactly those two things.",
    },
    "routeError.retry": {
        text: "Try again",
        description: "Button that reloads the failed page's data in place, without a full browser reload.",
    },
    "routeError.home": {
        text: "Go to the home page",
        description: "Second button: navigates to the site root.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
