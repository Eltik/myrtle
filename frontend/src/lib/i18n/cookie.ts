const ONE_YEAR_SECONDS = 31_536_000;

/**
 * Write a reader preference cookie (the language, the game-data server), or
 * forget it with `null`.
 *
 * Site-wide, a year, `SameSite=Lax`: the cookie is read on a top-level
 * navigation, and a preference does not go stale. `CookieStore` where the
 * browser has it, `document.cookie` otherwise; `CookieStore` is Chromium-only
 * today, so the fallback is the path Safari and Firefox actually take, not
 * dead code.
 *
 * Resolves once the cookie is written and never rejects. A caller that reloads
 * right after must await it, because a reload that races an async
 * `CookieStore` write renders with the old value. A blocked cookie costs only
 * the remembered preference, so failures are swallowed.
 */
export async function writePreferenceCookie(name: string, value: string | null): Promise<void> {
    try {
        if (typeof cookieStore !== "undefined") {
            if (value === null) await cookieStore.delete({ name, path: "/" });
            else await cookieStore.set({ name, value, path: "/", expires: Date.now() + ONE_YEAR_SECONDS * 1000, sameSite: "lax" });
            return;
        }
        const encoded = value === null ? "" : encodeURIComponent(value);
        const maxAge = value === null ? 0 : ONE_YEAR_SECONDS;
        // biome-ignore lint/suspicious/noDocumentCookie: CookieStore is unavailable in this branch by construction - that is what the guard above tests.
        document.cookie = `${name}=${encoded}; path=/; max-age=${maxAge}; samesite=lax`;
    } catch {
        // See above: a blocked cookie only loses the remembered preference.
    }
}
