/**
 * WHAT A REFUSED QUESTION MEANS, for the ask box.
 *
 * The backend's refusals (contract v1 section 4, `services/trevor/ask.rs`):
 * - 401: not signed in. Asking needs an account; the panels do not.
 * - 403: Turnstile said no (or the job is someone else's).
 * - 409: the account already has its open questions waiting.
 * - 413: the body is over the limit.
 * - 429: the hourly allowance (30 an hour) is spent, or the site-wide limiter tripped.
 * - 503: the queue is full, or Turnstile could not be reached ("Trevor is busy").
 *
 * Each of those gets its own state and sentence; anything else falls back to
 * the site's describer (`useErrorMessage`), which already names offline,
 * unreachable and timed-out. Classification is `classifyFailure`'s, so an
 * error that crossed the server-function boundary (a plain `Error` carrying
 * `kind` and `status`) reads the same as one thrown in the browser.
 */
import { classifyFailure } from "#/lib/api/_shared";

export type TrevorFailureKind = "signedOut" | "verify" | "tooMany" | "tooLong" | "rateLimited" | "busy" | "other";

export function trevorFailureKind(err: unknown): TrevorFailureKind {
    const f = classifyFailure(err);
    if (f.kind !== "http") return "other";
    switch (f.status) {
        case 401:
            return "signedOut";
        case 403:
            return "verify";
        case 409:
            return "tooMany";
        case 413:
            return "tooLong";
        case 429:
            return "rateLimited";
        case 503:
            return "busy";
        default:
            return f.code === "RATE_LIMITED" ? "rateLimited" : "other";
    }
}
