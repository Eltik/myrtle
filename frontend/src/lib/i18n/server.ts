import { createServerFn } from "@tanstack/react-start";
import { getCookie, getRequestHeader, getRequestUrl } from "@tanstack/react-start/server";

import { type IBootstrap, isEnabledLocale, loadBootstrap } from "./catalog";
import { DEFAULT_LOCALE, LOCALE_COOKIE, negotiateLocale, parseLocaleFromPath } from "./locale";
import { requestLocale } from "./request-locale.server";

/**
 * Resolve the locale for this request and load its catalog.
 *
 * Precedence is the path, then the remembered cookie, then `Accept-Language`,
 * then English. The path wins because a shared link must mean the same thing
 * for everyone who opens it - a cookie that overrode the URL would make
 * `/ja/operators` render in English for a visitor whose last visit was
 * English, which is the bug that makes people stop sharing links.
 */
export const getI18nBootstrapFn = createServerFn({ method: "GET" }).handler(async (): Promise<IBootstrap> => {
    const fromPath = requestLocale();

    if (fromPath) {
        // A locale in the path that the backend does not serve is NOT silently
        // downgraded to English. `loadBootstrap` returns the de-prefixed URL as
        // `redirectTo` and the root route sends the visitor there, so a `/ja`
        // URL either renders Japanese or stops being a `/ja` URL. Rendering
        // English under `/ja` with `lang="en"` is indistinguishable from a
        // translation silently failing to apply.
        const url = getRequestUrl();
        const bare = `${parseLocaleFromPath(url.pathname).rest}${url.search}`;
        return await loadBootstrap(fromPath, DEFAULT_LOCALE, bare);
    }

    // A remembered locale that has since been disabled just falls back; there
    // is no wrong URL to correct in that case.
    const remembered = getCookie(LOCALE_COOKIE);
    if (remembered) return await loadBootstrap(remembered, DEFAULT_LOCALE);

    return await loadBootstrap(DEFAULT_LOCALE, DEFAULT_LOCALE);
});

/**
 * What the bare root should redirect a returning visitor to. Kept separate
 * from the bootstrap so that only the root pays for the negotiation, and every
 * content page stays cacheable without a `Vary: Cookie`.
 */
export const getPreferredLocaleFn = createServerFn({ method: "GET" }).handler(async (): Promise<string> => {
    const remembered = getCookie(LOCALE_COOKIE);
    if (remembered && (await isEnabledLocale(remembered))) return remembered;

    const { fetchManifest } = await import("./catalog");
    const manifest = await fetchManifest();
    const available = manifest?.locales.map((l) => l.code) ?? [DEFAULT_LOCALE];
    return negotiateLocale(getRequestHeader("accept-language"), available);
});
