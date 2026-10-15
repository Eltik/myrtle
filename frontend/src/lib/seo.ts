import { basepathForLocale, DEFAULT_LOCALE } from "#/lib/i18n/locale";
import { OG_CONFIG } from "#/lib/og/impl/config";

export interface ISeoInput {
    title: string;
    description?: string;
    image?: string;
    path?: string;
    type?: "website" | "article" | "profile";
    noindex?: boolean;
    /** The locale this page is rendered in. Defaults to the source locale. */
    locale?: string;
    /** Unused: `seo()` emits no `hreflang`, `RootDocument` does (see below). */
    alternateLocales?: string[];
}

export interface ISeoOutput {
    meta: Array<{ title?: string; name?: string; property?: string; content?: string }>;
    links: Array<{ rel: string; href: string; as?: string; hrefLang?: string }>;
}

export function seo(input: ISeoInput): ISeoOutput {
    const fullTitle = input.title.includes(OG_CONFIG.siteName) ? input.title : `${input.title} • ${OG_CONFIG.siteName}`;
    const image = absolute(input.image ?? OG_CONFIG.defaultImage);
    const locale = input.locale ?? DEFAULT_LOCALE;
    const description = input.description;

    const meta: ISeoOutput["meta"] = [
        { title: fullTitle },
        { property: "og:title", content: fullTitle },
        { property: "og:image", content: image },
        { property: "og:type", content: input.type ?? "website" },
        { property: "og:site_name", content: OG_CONFIG.siteName },
        { property: "og:locale", content: locale.replace("-", "_") },
        { name: "twitter:card", content: "summary_large_image" },
        { name: "twitter:title", content: fullTitle },
        { name: "twitter:image", content: image },
    ];

    if (description) {
        meta.push({ name: "description", content: description }, { property: "og:description", content: description }, { name: "twitter:description", content: description });
    }

    if (input.noindex) meta.push({ name: "robots", content: "noindex, nofollow" });

    // `canonical` and `og:url` are NOT emitted here. Both need the active
    // locale's path prefix, and a route's `head()` can't see the locale (no
    // React context, no request URL). Threading a locale through every `seo()`
    // call site means any one missed silently declares the English page as a
    // translation's canonical, which de-indexes the translation.
    //
    // `RootDocument` has both the locale and the pathname, so it emits exactly
    // one canonical, one `og:url` and the full `hreflang` set for every route.
    // See `routes/__root.tsx`.
    //
    // The og:image is deliberately not preloaded: the page never renders it, so a
    // preload only spends bandwidth the real LCP image competes for.
    const links: ISeoOutput["links"] = [];

    return { meta, links };
}

/** `/operators` + `ja` -> `/ja/operators`; the source locale keeps the bare path. */
export function localizedPath(path: string, locale: string): string {
    const base = basepathForLocale(locale);
    if (base === "/") return path;
    const tail = path.startsWith("/") ? path : `/${path}`;
    return tail === "/" ? base : `${base}${tail}`;
}

export function absolute(pathOrURL: string): string {
    if (/^https?:\/\//i.test(pathOrURL)) return pathOrURL;
    const base = OG_CONFIG.siteURL.replace(/\/$/, "");
    const tail = pathOrURL.startsWith("/") ? pathOrURL : `/${pathOrURL}`;
    return `${base}${tail}`;
}
