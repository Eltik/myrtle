import { getRequestUrl } from "@tanstack/react-start/server";

import { parseLocaleFromPath } from "./locale";

/**
 * The locale claimed by the incoming request's path, or `null` for the
 * unprefixed default.
 *
 * Server-only, and named `.server.ts` so the bundler keeps
 * `@tanstack/react-start/server` out of the client graph - the same convention
 * `lib/api/_shared.server.ts` already uses for its cookie reads.
 */
export function requestLocale(): string | null {
    try {
        return parseLocaleFromPath(getRequestUrl().pathname).locale;
    } catch {
        // Outside a request scope (a build-time render, a warmup) there is no
        // URL to read and the default locale is the right answer.
        return null;
    }
}
