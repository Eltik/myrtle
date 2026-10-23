import bundledCatalog from "./source-catalog.json";
import sourceCatalogUrl from "./source-catalog.json?url";

/**
 * The English source text for every key, extracted from the components at
 * build time.
 *
 * This is the floor beneath the whole system: if the database is unreachable,
 * a key has no translation in the requested locale or its fallback, or a key
 * is brand new and has not been synced yet, the page still renders real
 * English rather than a raw key. It is never the authority on what is
 * *rendered* - a row in `ui_messages` for `en` outranks it, which is how a
 * copy fix ships without a rebuild - but it guarantees the site cannot break
 * by being un-translated.
 *
 * The server, dev and tests read it from the bundle. The production client
 * does not: bundled, it was 30% of the entry chunk (495 KB of 1.66 MB), paid
 * on every route before anything else could run. There it ships as its own
 * content-hashed JSON asset, preloaded from the root `head()` alongside the
 * entry and filled in by {@link loadSourceCatalog} from the router's
 * `hydrate` hook, so it is complete before the first client render and the
 * hydrated text matches the server's. The file name is a content hash, so a
 * deploy that changes no strings costs returning visitors nothing.
 *
 * Regenerate with `bun run i18n:extract`.
 */
const BUNDLED = import.meta.env.SSR || !import.meta.env.PROD;

export const SOURCE_CATALOG: Readonly<Record<string, string>> = BUNDLED ? (bundledCatalog as Record<string, string>) : {};

/**
 * The catalog's asset URL in a production build, server included, because the
 * root `head()` that preloads it renders there; `null` in dev and tests.
 */
export const SOURCE_CATALOG_URL: string | null = import.meta.env.PROD ? sourceCatalogUrl : null;

let loading: Promise<void> | null = null;

/**
 * Fill {@link SOURCE_CATALOG} on the production client; a no-op wherever it
 * is bundled. One attempt per page load: a failed fetch leaves raw keys where
 * untranslated text would be, a degraded page rather than a broken one, and
 * must not block hydration.
 */
export function loadSourceCatalog(): Promise<void> {
    if (BUNDLED || !SOURCE_CATALOG_URL) return Promise.resolve();
    loading ??= fetch(SOURCE_CATALOG_URL)
        .then((res) => {
            if (!res.ok) throw new Error(`source catalog: HTTP ${res.status}`);
            return res.json() as Promise<Record<string, string>>;
        })
        .then((catalog) => {
            Object.assign(SOURCE_CATALOG, catalog);
        })
        .catch((error: unknown) => {
            console.error(error);
        });
    return loading;
}

export function sourceMessage(key: string): string | undefined {
    return SOURCE_CATALOG[key];
}
