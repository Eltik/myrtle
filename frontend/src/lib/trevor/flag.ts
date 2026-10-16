/**
 * THE TREVOR FLAG, and the one place that reads it.
 *
 * Trevor (the story question answerer) is provisional: its panel and its
 * toolbar button only exist when the build sets `VITE_TREVOR=1`. Anything else,
 * the variable missing included, is OFF, and off renders the reader and the
 * chapter sheet byte for byte as they were before Trevor
 * (`components/story/trevor/flag.test.tsx`).
 *
 * The value is read from `import.meta.env` at call time, never cached in a
 * module constant, so a test can flip it with `vi.stubEnv`. Vite inlines a
 * `VITE_*` variable at build time on both the client and the SSR bundle, so
 * the server render and the hydrating client always agree; a runtime
 * `process.env` read would let the two disagree and break hydration.
 */

/** True only when the build set `VITE_TREVOR=1`. */
export function trevorEnabled(): boolean {
    return import.meta.env.VITE_TREVOR === "1";
}

/**
 * The Cloudflare Turnstile site key, or null when none is configured. Without
 * one the ask box renders no widget and sends no token, which is what the
 * backend expects when its `TURNSTILE_SECRET` is unset (the dev default).
 */
export function turnstileSiteKey(): string | null {
    const key: unknown = import.meta.env.VITE_TURNSTILE_SITE_KEY;
    return typeof key === "string" && key.trim() !== "" ? key.trim() : null;
}
