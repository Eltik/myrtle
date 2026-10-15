import { createServerFn } from "@tanstack/react-start";
import { getCookie, getRequestHeader, getRequestUrl } from "@tanstack/react-start/server";

import { type IBootstrap, isEnabledLocale, loadBootstrap } from "./catalog";
import { DEFAULT_LOCALE, GAMEDATA_SERVER_COOKIE, LOCALE_COOKIE, negotiateLocale, parseLocaleFromPath } from "./locale";
import { requestLocale } from "./request-locale.server";

/**
 * Path, then the remembered cookie, then `Accept-Language`, then English. The
 * path wins so a shared `/ja/...` link renders Japanese for everyone, whatever
 * their cookie says.
 */
export const getI18nBootstrapFn = createServerFn({ method: "GET" }).handler(async (): Promise<IBootstrap> => {
    const fromPath = requestLocale();
    const pickedServer = getCookie(GAMEDATA_SERVER_COOKIE);

    if (fromPath) {
        // An unserved path locale redirects to the de-prefixed URL (`redirectTo`)
        // rather than rendering English under `/ja`, which looks like a broken
        // translation.
        const url = getRequestUrl();
        const bare = `${parseLocaleFromPath(url.pathname).rest}${url.search}`;
        return await loadBootstrap(fromPath, DEFAULT_LOCALE, bare, pickedServer);
    }

    // A remembered locale that has since been disabled just falls back; there
    // is no wrong URL to correct in that case.
    const remembered = getCookie(LOCALE_COOKIE);
    if (remembered) return await loadBootstrap(remembered, DEFAULT_LOCALE, undefined, pickedServer);

    return await loadBootstrap(DEFAULT_LOCALE, DEFAULT_LOCALE, undefined, pickedServer);
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
