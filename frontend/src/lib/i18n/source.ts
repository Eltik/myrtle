import sourceCatalog from "./source-catalog.json";

/**
 * The English source text for every key, extracted from the components at
 * build time and shipped in the bundle.
 *
 * This is the floor beneath the whole system: if the database is unreachable,
 * a key has no translation in the requested locale or its fallback, or a key
 * is brand new and has not been synced yet, the page still renders real
 * English rather than a raw key. It is never the authority on what is
 * *rendered* - a row in `ui_messages` for `en` outranks it, which is how a
 * copy fix ships without a rebuild - but it guarantees the site cannot break
 * by being un-translated.
 *
 * Regenerate with `bun run i18n:extract`.
 */
export const SOURCE_CATALOG: Readonly<Record<string, string>> = sourceCatalog as Record<string, string>;

export function sourceMessage(key: string): string | undefined {
    return SOURCE_CATALOG[key];
}
