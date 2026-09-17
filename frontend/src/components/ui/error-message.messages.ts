import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

/**
 * The body of an error toast, one message per failure the client can tell
 * apart. See `classifyFailure` in `lib/api/_shared.ts` for how a thrown error
 * lands in one of these. `{status}` is the HTTP status number; `{code}` is a
 * socket-level code such as ECONNREFUSED; `{message}` is the server's own text.
 */
export const messages = {
    "errorMessage.offline": {
        text: "You're offline. Check your connection and try again.",
        description: "Error toast body when the browser reports no network connection.",
    },
    "errorMessage.unreachable": {
        text: "The server couldn't be reached. It may be down or restarting; try again in a minute.",
        description: "Error toast body when the request got no answer from the server at all.",
    },
    "errorMessage.unreachableWithCode": {
        text: "The server couldn't be reached ({code}). It may be down or restarting; try again in a minute.",
        description: "Same as errorMessage.unreachable, with the low-level network code in parentheses. {code} is a technical identifier such as ECONNREFUSED and stays as-is.",
    },
    "errorMessage.timeout": {
        text: "The server took too long to respond. Try again in a moment.",
        description: "Error toast body when the request timed out waiting for the server.",
    },
    "errorMessage.rateLimited": {
        text: "Too many attempts. Wait a minute and try again.",
        description: "Error toast body when the server refused the request for being sent too often (HTTP 429).",
    },
    "errorMessage.unavailable": {
        text: "The server is temporarily unavailable ({status}). Try again in a minute.",
        description: "Error toast body when the server is up but cannot serve requests right now (HTTP 503). {status} is the number.",
    },
    "errorMessage.serverError": {
        text: "The server ran into a problem ({status}). Try again in a minute.",
        description: "Error toast body for an unexpected error on the server (HTTP 5xx). {status} is the number.",
    },
    "errorMessage.unauthorized": {
        text: "You're not allowed to do that ({status}). If you were signed in, your session may have expired.",
        description: "Error toast body when the server rejected the request as unauthorized or forbidden (HTTP 401 or 403). {status} is the number.",
    },
    "errorMessage.syncFailed": {
        text: "Signed in, but the first roster sync failed ({status}). Reload the page: your account is connected, and you can sync again from your profile.",
        description: "Error toast body after a successful sign-in whose immediate roster download failed. {status} is the HTTP status number.",
    },
    "errorMessage.rejected": {
        text: "The request was rejected ({status}).",
        description: "Error toast body when the server refused the request and gave no readable reason. {status} is the HTTP status number.",
    },
    "errorMessage.unknown": {
        text: "Something went wrong.",
        description: "Error toast body when the failure carried no message at all.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
