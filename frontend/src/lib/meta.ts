import { ALL_NAMESPACES, type Catalog, DEFAULT_LOCALE, formatMessage, type MessageValues, sourceMessage } from "#/lib/i18n";
import { fetchCatalog, fetchManifest } from "#/lib/i18n/catalog";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import type { messages } from "./meta.messages";

/**
 * Resolving `meta` messages without a React hook.
 *
 * A route's `head()` renders user-facing text outside React: a plain function the router calls
 * while building a match's assets, so `useT()` and every other hook is unavailable. `head()`
 * does get the match, and `match.context` carries the `i18n` bootstrap (locale + whole catalog)
 * the root route's `beforeLoad` already loaded; the router runs every `beforeLoad` and loader
 * before any `head()`. So no fetch, no second catalog, no duplicated locale negotiation: this
 * is the lookup half of `useT()` as a plain function.
 *
 *     head: ({ match }) => {
 *         const t = metaT(match.context.i18n);
 *         return seo({ title: t("operators.title") });
 *     }
 *
 * `metaSourceForLocale` is the exception: the OG image handlers render on the server with no
 * router match, only a locale read off the request, so they fetch the catalog themselves.
 */

/** The `meta` namespace, matching `meta.messages.ts` and the README's table. */
const META_NAMESPACE = "meta";

/**
 * Everything resolving a message needs: which locale, and that locale's
 * catalog. `IBootstrap` from the root route context satisfies it structurally,
 * which is the whole point - `head()` passes `match.context.i18n` straight in.
 */
export interface IMetaSource {
    locale: string;
    messages: Catalog;
}

/**
 * Resolve one `meta` key. Same fallback order as `useT`: the locale's own
 * translation, then the English text bundled from the source catalog, then the
 * key itself so a brand-new key is visible rather than blank.
 */
export function metaMessage(key: string, source?: IMetaSource | null, values?: MessageValues): string {
    const full = fullMessageKey(META_NAMESPACE, key);
    const message = source?.messages[full] ?? sourceMessage(full) ?? full;
    return formatMessage(message, source?.locale ?? DEFAULT_LOCALE, values);
}

/**
 * `const t = metaT(match.context.i18n)` then `t("operators.title")` - the
 * `head()` counterpart to `const t = useT("meta")`, typed against
 * `meta.messages.ts` so a typo is a compile error.
 *
 * A missing source resolves to the bundled English, so a route whose context
 * has not been populated renders the source locale rather than raw keys.
 */
export function metaT(source?: IMetaSource | null): TypedT<typeof messages> {
    return (key, values) => metaMessage(key, source, values);
}

/**
 * The catalog for one locale, fetched rather than taken from a route context.
 *
 * For the source locale this returns `null`: the bundled source catalog is already the
 * answer, so the round trip is skipped. An unknown or un-served locale resolves the same way.
 */
export async function metaSourceForLocale(locale?: string | null): Promise<IMetaSource | null> {
    if (!locale || locale === DEFAULT_LOCALE) return null;

    const manifest = await fetchManifest();
    const entry = manifest?.locales.find((l) => l.code === locale);
    if (!entry) return null;

    const hash = entry.namespaces[ALL_NAMESPACES] ?? "latest";
    return { locale: entry.code, messages: await fetchCatalog(entry.code, hash) };
}
