import { OG_CONFIG } from "#/lib/og/impl/config";

export interface SeoInput {
    title: string;
    description?: string;
    image?: string;
    path?: string;
    type?: "website" | "article" | "profile";
    noindex?: boolean;
    // Emit a `<link rel="preload" as="image">` for the og:image. Use on pages
    // that supply their own image; skip when falling back to the site default.
    preloadImage?: boolean;
}

export interface SeoOutput {
    meta: Array<{ title?: string; name?: string; property?: string; content?: string }>;
    links: Array<{ rel: string; href: string; as?: string }>;
}

export function seo(input: SeoInput): SeoOutput {
    const fullTitle = input.title.includes(OG_CONFIG.siteName) ? input.title : `${input.title} • ${OG_CONFIG.siteName}`;
    const image = absolute(input.image ?? OG_CONFIG.defaultImage);
    const url = absolute(input.path ?? "/");
    const description = input.description;

    const meta: SeoOutput["meta"] = [
        { title: fullTitle },
        { property: "og:title", content: fullTitle },
        { property: "og:url", content: url },
        { property: "og:image", content: image },
        { property: "og:type", content: input.type ?? "website" },
        { property: "og:site_name", content: OG_CONFIG.siteName },
        { name: "twitter:card", content: "summary_large_image" },
        { name: "twitter:title", content: fullTitle },
        { name: "twitter:image", content: image },
    ];

    if (description) {
        meta.push({ name: "description", content: description }, { property: "og:description", content: description }, { name: "twitter:description", content: description });
    }

    if (input.noindex) meta.push({ name: "robots", content: "noindex, nofollow" });

    const links: SeoOutput["links"] = [{ rel: "canonical", href: url }];
    if (input.preloadImage && input.image) links.push({ rel: "preload", as: "image", href: image });

    return { meta, links };
}

function absolute(pathOrUrl: string): string {
    if (/^https?:\/\//i.test(pathOrUrl)) return pathOrUrl;
    const base = OG_CONFIG.siteURL.replace(/\/$/, "");
    const tail = pathOrUrl.startsWith("/") ? pathOrUrl : `/${pathOrUrl}`;
    return `${base}${tail}`;
}
