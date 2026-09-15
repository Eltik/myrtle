import { useMemo } from "react";

import { formatNumber, formatNumberCompact, formatRelative, formatRelativeShort, formatSharePct } from "#/lib/utils";
import { useLocale } from "./context";

export interface IFormatters {
    /** "1,234,567" */
    number: (n: number | null | undefined) => string;
    /** "1.2k" in English, "123.5万" in Japanese. */
    compact: (n: number | null | undefined) => string;
    /** A fraction in [0, 1] as a percentage. */
    percent: (fraction: number) => string;
    /** "5m ago", "yesterday" - the narrow style this site uses in dense UI. */
    relative: (iso: string | null | undefined) => string;
    /**
     * "3 weeks ago", "yesterday" - the long style, for places that had room
     * for the full words before i18n and must keep them.
     */
    relativeLong: (iso: string | null | undefined) => string;
    /** The verbose admin-dashboard variant. */
    relativeShort: (input: string | number | null | undefined) => string;
    date: (value: string | number | Date, options?: Intl.DateTimeFormatOptions) => string;
    time: (value: string | number | Date, options?: Intl.DateTimeFormatOptions) => string;
    /** Locale-correct string comparison, for sorting names. */
    collator: Intl.Collator;
}

/**
 * The locale-bound counterparts of the formatters in `lib/utils.ts`.
 *
 * Those stay pure functions taking an optional locale, so the ~160 existing
 * call sites keep compiling and keep rendering exactly what they render today.
 * This hook is what converted components should use instead: it closes over
 * the active locale, so a component never has to remember to pass it.
 *
 * The bug this exists to kill: 138 call sites in this codebase reach for a
 * bare `.toLocaleString()`, which silently takes the *browser's* locale rather
 * than the page's. On a Japanese page opened in an English browser those
 * disagree, and nothing in the type system notices.
 */
export function useFormatters(): IFormatters {
    const locale = useLocale();

    return useMemo<IFormatters>(
        () => ({
            number: (n) => formatNumber(n, locale),
            compact: (n) => formatNumberCompact(n, locale),
            percent: (fraction) => formatSharePct(fraction, locale),
            relative: (iso) => formatRelative(iso, locale),
            relativeLong: (iso) => formatRelative(iso, locale, "long"),
            relativeShort: (input) => formatRelativeShort(input, locale),
            date: (value, options) => new Intl.DateTimeFormat(locale, options ?? { dateStyle: "medium" }).format(value instanceof Date ? value : new Date(value)),
            time: (value, options) => new Intl.DateTimeFormat(locale, options ?? { timeStyle: "short" }).format(value instanceof Date ? value : new Date(value)),
            collator: new Intl.Collator(locale, { sensitivity: "base", numeric: true }),
        }),
        [locale],
    );
}
