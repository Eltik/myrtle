import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "oauth";

export const messages = {
    "authorize.signIn.title": {
        text: "Sign in to continue",
        description: "Card title on the OAuth consent page when the visitor is signed out. Another website sent them here to confirm who they are.",
    },
    "authorize.signIn.body": {
        text: "Another site is asking to confirm who you are on myrtle.moe. Sign in first, then you decide whether to allow it.",
        description: "Card body on the consent page when the visitor is signed out.",
    },
    "authorize.signIn.action": {
        text: "Sign in",
        description: "Button on the signed-out consent page that opens the sign-in dialog.",
    },
    "authorize.loading": {
        text: "Checking the request…",
        description: "Status line on the consent page while the request from the other site is being checked.",
    },
    "authorize.redirecting": {
        text: "Sending you back…",
        description: "Status line on the consent page while the browser is being sent back to the other site.",
    },
    "authorize.invalid.title": {
        text: "This sign-in link doesn't work",
        description: "Card title on the consent page when the other site's request cannot be accepted (unknown application, or a return address it never registered).",
    },
    "authorize.invalid.body": {
        text: "The site that sent you here made a request myrtle.moe can't accept, so nothing was shared. Go back and try again. If it keeps happening, its developer needs the reason below.",
        description: "Card body under that title. The technical reason follows in a monospace box.",
    },
    "authorize.failed.title": {
        text: "Couldn't check the request",
        description: "Card title on the consent page when checking the request failed for a reason on our side (offline, server error). The reason follows.",
    },
    "authorize.failed.retry": {
        text: "Try again",
        description: "Button under that error that checks the request again.",
    },
    "authorize.title": {
        text: "{app} wants to know who you are",
        description: "Heading of the consent card. {app} is the name the other site registered, shown as written.",
    },
    "authorize.by": {
        text: "Registered by {owner}. Not made by or affiliated with myrtle.moe.",
        description: "Line under the consent heading. {owner} is the in-game name of the player who registered the application.",
    },
    "authorize.account": {
        text: "Signed in as {name}",
        description: "Row on the consent card naming the account that would be shared. {name} is the visitor's in-game name with its # tag.",
    },
    "authorize.switch": {
        text: "Not you? Sign out",
        description: "Link button next to the signed-in account on the consent card. Signs out so another account can sign in.",
    },
    "authorize.willSee": {
        text: "It will see",
        description: "Label over the list of what the application learns. 'It' is the application.",
    },
    "authorize.scope.identify.name": {
        text: "Your nickname and its # number",
        description: "One thing the application learns: the visitor's in-game name and the digits after the #.",
    },
    "authorize.scope.identify.account": {
        text: "Your in-game UID and server",
        description: "One thing the application learns. UID is the in-game account number; server is the game region (EN, JP, ...).",
    },
    "authorize.scope.identify.avatar": {
        text: "Your avatar",
        description: "One thing the application learns: the visitor's in-game profile picture.",
    },
    "authorize.wontSee": {
        text: "It won't be able to",
        description: "Label over the list of what the application cannot do. 'It' is the application.",
    },
    "authorize.cannot.private": {
        text: "See anything your profile keeps private",
        description: "One thing the application cannot do.",
    },
    "authorize.cannot.change": {
        text: "Change your settings or your data",
        description: "One thing the application cannot do.",
    },
    "authorize.cannot.game": {
        text: "Reach your game account or its login",
        description: "One thing the application cannot do: it gets no access to the Arknights account itself.",
    },
    "authorize.again": {
        text: "You've authorized this before.",
        description: "Note on the consent card when the visitor already authorized this application earlier.",
    },
    "authorize.redirectNote": {
        text: "You'll be sent to {host} either way. Remove access any time in Settings, under Applications.",
        description: "Footnote on the consent card. {host} is the other site's address in monospace; move it wherever the sentence needs it.",
    },
    "authorize.approve": {
        text: "Authorize",
        description: "Primary button on the consent card: allow the application to learn who the visitor is.",
    },
    "authorize.deny": {
        text: "Cancel",
        description: "Secondary button on the consent card: refuse and go back to the other site.",
    },
    "authorize.decideFailed": {
        text: "Couldn't finish",
        description: "Error toast title when sending the visitor's answer failed. The body explains why.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
