import type { QueryClient } from "@tanstack/react-query";

/*
 * The query keys each kind of admin write can move, in one place so every
 * call site invalidates the same set.
 */

/** A tier-list grant change: every grant listing, and the grantee's own "granted" list. */
export function invalidateTierListGrants(queryClient: QueryClient): void {
    void queryClient.invalidateQueries({ queryKey: ["admin", "tier-lists", "permissions"] });
    void queryClient.invalidateQueries({ queryKey: ["tier-lists", "granted"] });
}

/** A language grant change. */
export function invalidateLocaleGrants(queryClient: QueryClient): void {
    void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "permissions"] });
}

/** Every query a locale write can change. */
export function invalidateLocaleQueries(queryClient: QueryClient): void {
    void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "locales"] });
    void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "progress"] });
}

/** Every query a translation write can move: the list, the progress cards and the history. */
const TRANSLATION_QUERY_KEYS = [
    ["admin", "i18n", "messages"],
    ["admin", "i18n", "progress"],
    ["admin", "i18n", "audit"],
] as const;

export function invalidateTranslationQueries(queryClient: QueryClient): void {
    for (const queryKey of TRANSLATION_QUERY_KEYS) void queryClient.invalidateQueries({ queryKey: [...queryKey] });
}
