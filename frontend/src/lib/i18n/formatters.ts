import { useMemo } from "react";
import { useHydrated } from "#/hooks/use-hydrated";
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
    /** A timestamp. Prints in UTC until hydration ends, then in the reader's time zone. */
    date: (value: string | number | Date, options?: Intl.DateTimeFormatOptions) => string;
    /** A timestamp's clock time. Same time zone rule as `date`. */
    time: (value: string | number | Date, options?: Intl.DateTimeFormatOptions) => string;
    /**
     * A calendar day built from local date parts (`new Date(y, m, d)`), not a timestamp. Always
     * prints in the zone it was built in: in UTC, Tokyo's local midnight is the previous day.
     */
    calendarDay: (value: Date, options?: Intl.DateTimeFormatOptions) => string;
    /** Locale-correct string comparison, for sorting names. */
    collator: Intl.Collator;
}

/**
 * The locale-bound counterparts of the formatters in `lib/utils.ts`.
 *
 * Those stay pure functions taking an optional locale (~160 call sites). Converted components
 * use this hook instead: it closes over the active locale, so nobody has to remember to pass it.
 *
 * Exists because 138 call sites reach for a bare `.toLocaleString()`, which silently takes the
 * *browser's* locale rather than the page's. On a Japanese page in an English browser those
 * disagree, and nothing in the type system notices.
 */
export function useFormatters(): IFormatters {
    const locale = useLocale();
    const hydrated = useHydrated();

    return useMemo<IFormatters>(
        () => ({
            number: (n) => formatNumber(n, locale),
            compact: (n) => formatNumberCompact(n, locale),
            percent: (fraction) => formatSharePct(fraction, locale),
            relative: (iso) => formatRelative(iso, locale),
            relativeLong: (iso) => formatRelative(iso, locale, "long"),
            relativeShort: (input) => formatRelativeShort(input, locale),
            date: (value, options) => new Intl.DateTimeFormat(locale, zoned(options ?? { dateStyle: "medium" }, hydrated)).format(value instanceof Date ? value : new Date(value)),
            time: (value, options) => new Intl.DateTimeFormat(locale, zoned(options ?? { timeStyle: "short" }, hydrated)).format(value instanceof Date ? value : new Date(value)),
            calendarDay: (value, options) => new Intl.DateTimeFormat(locale, options ?? { dateStyle: "medium" }).format(value),
            collator: new Intl.Collator(locale, { sensitivity: "base", numeric: true }),
        }),
        [locale, hydrated],
    );
}

/**
 * Until hydration ends, `date` and `time` print in UTC. The server cannot know the reader's time
 * zone, and the hydrating render has to print what the server printed: in the server's own zone,
 * "Feb 6" in the HTML against "Feb 5" in a browser on UTC was React #418 on the tier-list and
 * profile pages. The reader's zone takes over on the render after hydration. A caller's own
 * `timeZone` always wins.
 */
function zoned(options: Intl.DateTimeFormatOptions, hydrated: boolean): Intl.DateTimeFormatOptions {
    return hydrated || options.timeZone ? options : { ...options, timeZone: "UTC" };
}
