import { ALL_NAMESPACES, type Catalog, DEFAULT_LOCALE, formatMessage, type MessageValues, sourceMessage } from "#/lib/i18n";
import { fetchCatalog, fetchManifest } from "#/lib/i18n/catalog";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import type { messages } from "./meta.messages";

/**
 * `meta` messages without a hook, for route `head()` functions. The router runs
 * every `beforeLoad` before any `head()`, so `match.context.i18n` already holds
 * the root's bootstrap (locale + catalog) and no fetch is needed.
 *
 *     head: ({ match }) => {
 *         const t = metaT(match.context.i18n);
 *         return seo({ title: t("operators.title") });
 *     }
 *
 * `metaSourceForLocale` is for the OG image handlers, which have no router
 * match, only the request's locale, so they fetch the catalog.
 */

/** The `meta` namespace, matching `meta.messages.ts` and the README's table. */
const META_NAMESPACE = "meta";

/** `IBootstrap` satisfies this structurally, so `head()` passes `match.context.i18n` straight in. */
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
 * The `head()` counterpart to `useT("meta")`, typed against `meta.messages.ts`.
 * A missing source falls back to the bundled English, not raw keys.
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
