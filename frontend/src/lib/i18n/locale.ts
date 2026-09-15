/** The source locale. Its text is the bundled fallback beneath every other. */
export const DEFAULT_LOCALE = "en";

/** Remembers the visitor's choice so the bare root can send them back to it. */
export const LOCALE_COOKIE = "locale";

/**
 * A leading path segment that *could* be a locale: `ja`, `zh-Hans`, `pt-BR`.
 *
 * Deliberately a shape test rather than a list. The set of enabled locales
 * lives in the database, and this has to answer synchronously while the router
 * is being constructed - before anything async can run. A segment matching
 * this shape is treated as a locale claim and then validated against the
 * manifest, which redirects an unknown one to the default locale.
 *
 * No route in this app is a bare two-letter segment, so there is nothing to
 * collide with today. A future `/fr` route would collide, and would fail
 * loudly the first time it is visited rather than silently.
 */
export const LOCALE_SEGMENT = /^[a-z]{2}(?:-[A-Za-z]{2,4})?$/;

export interface IParsedPath {
    /** The locale claimed by the path, or `null` for the unprefixed default. */
    locale: string | null;
    /** The path with any locale prefix removed, always starting with `/`. */
    rest: string;
}

export function parseLocaleFromPath(pathname: string): IParsedPath {
    const segments = pathname.split("/").filter(Boolean);
    const [first] = segments;

    if (!first || first === DEFAULT_LOCALE || !LOCALE_SEGMENT.test(first)) {
        return { locale: null, rest: pathname || "/" };
    }

    const rest = `/${segments.slice(1).join("/")}`;
    return { locale: first, rest: rest === "/" ? "/" : rest };
}

/**
 * The router basepath for a locale. English stays at the bare path so that
 * every URL already indexed keeps working and nothing needs redirecting.
 */
export function basepathForLocale(locale: string | null | undefined): string {
    return !locale || locale === DEFAULT_LOCALE ? "/" : `/${locale}`;
}

/** Best match for an `Accept-Language` header among the locales we serve. */
export function negotiateLocale(header: string | undefined, available: string[]): string {
    if (!header) return DEFAULT_LOCALE;

    const ranked = header
        .split(",")
        .map((part) => {
            const [tag, ...params] = part.trim().split(";");
            const q = params.find((p) => p.trim().startsWith("q="));
            return { tag: tag.trim(), q: q ? Number.parseFloat(q.split("=")[1]) || 0 : 1 };
        })
        .filter((entry) => entry.tag)
        .sort((a, b) => b.q - a.q);

    for (const { tag } of ranked) {
        const exact = available.find((code) => code.toLowerCase() === tag.toLowerCase());
        if (exact) return exact;
        // `ja-JP` should match a plain `ja`, and `zh` should reach `zh-Hans`.
        const base = tag.split("-")[0].toLowerCase();
        const loose = available.find((code) => code.toLowerCase().split("-")[0] === base);
        if (loose) return loose;
    }

    return DEFAULT_LOCALE;
}

/**
 * `dir` for the document. Broken out so adding an RTL locale is a one-line
 * change here rather than a hunt through the layout.
 */
const RTL = new Set(["ar", "he", "fa", "ur"]);

export function directionForLocale(locale: string): "ltr" | "rtl" {
    return RTL.has(locale.split("-")[0]) ? "rtl" : "ltr";
}
