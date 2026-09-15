import { basepathForLocale, DEFAULT_LOCALE } from "#/lib/i18n/locale";
import { OG_CONFIG } from "#/lib/og/impl/config";

export interface ISeoInput {
    title: string;
    description?: string;
    image?: string;
    path?: string;
    type?: "website" | "article" | "profile";
    noindex?: boolean;
    // Emit a `<link rel="preload" as="image">` for the og:image. Use on pages
    // that supply their own image; skip when falling back to the site default.
    preloadImage?: boolean;
    /** The locale this page is rendered in. Defaults to the source locale. */
    locale?: string;
    /**
     * Every locale this page exists in, for `hreflang`. Pass the codes from
     * `useI18n().available`; omit to emit no alternates, which is correct for
     * a single-locale deployment.
     */
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
    // The canonical must carry the locale prefix, or every translation would
    // declare the English page as its canonical and none of them would be
    // indexed - the exact failure that makes translating a reference site
    // pointless.
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

    // `canonical` and `og:url` are NOT emitted here.
    //
    // Both have to carry the active locale's path prefix, and a route's
    // `head()` cannot see the locale - it is not a React context and the
    // request URL is not readable from there. Emitting them per route would
    // have meant threading a locale argument through all 31 `seo()` call
    // sites, and any one that was missed would silently declare the English
    // page as a translation's canonical, which de-indexes the translation.
    //
    // `RootDocument` has both the locale and the pathname, so it emits exactly
    // one canonical, one `og:url` and the full `hreflang` set for every route.
    // See `routes/__root.tsx`.
    const links: ISeoOutput["links"] = [];

    if (input.preloadImage && input.image) links.push({ rel: "preload", as: "image", href: image });

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
