import { DEFAULT_LOCALE, LOCALE_SEGMENT } from "#/lib/i18n";
import { readCache, writeCache } from "./cache";
import { getHandler } from "./registry";
import { OgRenderUnavailableError, renderOgPng } from "./render";

const PNG_HEADERS: Record<string, string> = {
    "Content-Type": "image/png",
    "Cache-Control": "public, max-age=31536000, s-maxage=31536000, immutable",
    "CDN-Cache-Control": "public, max-age=31536000, immutable",
    "Vercel-CDN-Cache-Control": "public, max-age=31536000, immutable, stale-while-revalidate=86400",
};

/** Reads the `v` cache-version param from an OG request URL. */
export function ogVersion(request: Request): string | undefined {
    return new URL(request.url).searchParams.get("v") ?? undefined;
}

/**
 * Reads the `locale` param from an OG request URL.
 *
 * An OG image is fetched by a crawler, not a browser: there is no cookie, no
 * `Accept-Language` worth trusting and no locale in the path, so the page that
 * embeds the card has to say which locale it was rendered in. `defaultOgURL`
 * puts it in the URL it hands to `seo()`; this reads it back.
 *
 * Shape-checked against `LOCALE_SEGMENT` rather than the manifest, which would
 * need a round trip: an unknown-but-well-formed code resolves to the bundled
 * English anyway, and anything else is dropped before it can reach a cache
 * path.
 */
export function ogLocale(request: Request): string | undefined {
    const raw = new URL(request.url).searchParams.get("locale");
    return raw && LOCALE_SEGMENT.test(raw) ? raw : undefined;
}

interface IOgResponseArgs {
    kind: string;
    // Identifier passed to the kind's `fetch` to load template data.
    fetchId: string;
    // Cheap cache version from the OG URL. This lets cache hits avoid loading
    // template data just to compute a content hash.
    version?: string;
    // Locale the embedding page was rendered in, from the OG URL. Passed to
    // the handler's `fetch` so it can resolve its text, and folded into the
    // cache key so one locale's card is never served for another.
    locale?: string;
    // Identifier used as the cache filename. Defaults to `fetchId`. Use this
    // when the public id (e.g. a title) is not a stable cache key.
    cacheId?: string;
    // When set, the response includes Content-Disposition: attachment with
    // this filename so the browser saves the image instead of inlining it.
    attachmentFilename?: string;
}

export async function ogResponse({ kind, fetchId, version, locale, cacheId = fetchId, attachmentFilename }: IOgResponseArgs): Promise<Response> {
    const handler = getHandler(kind);
    // The locale is part of the cache key, not only of the render. The `v`
    // content hash already covers the card's translated words, but nothing
    // else about the request does, and a card whose text happened to match
    // another locale's would otherwise share that locale's PNG forever - the
    // responses are `immutable` for a year, so a collision here is permanent.
    //
    // The source locale adds no suffix, which is what keeps every English
    // entry already on disk and in the CDN valid.
    const baseVersion = handler.cacheVersion(fetchId, version);
    const cacheVersion = locale && locale !== DEFAULT_LOCALE ? `${baseVersion}-${locale}` : baseVersion;

    let png = await readCache(kind, cacheId, cacheVersion);
    if (!png) {
        const data = await handler.fetch(fetchId, locale);
        if (!data) return new Response("Not found", { status: 404 });

        try {
            const dimensions = handler.dimensions?.(data);
            png = await renderOgPng(handler.template(data), dimensions);
            await writeCache(kind, cacheId, cacheVersion, png);
        } catch (err) {
            if (err instanceof OgRenderUnavailableError) {
                // Overloaded/wedged render pipeline - degrade to a retryable 503
                // rather than a 500 so callers (and CDNs) treat it as transient.
                console.warn(`[og] render unavailable for ${kind}/${cacheId}:`, err);
                return new Response("Render unavailable", { status: 503, headers: { "Retry-After": "5" } });
            }
            console.error(`[og] render failed for ${kind}/${cacheId}:`, err);
            return new Response("Render failed", { status: 500 });
        }
    }

    const headers: Record<string, string> = { ...PNG_HEADERS, ETag: `"${cacheVersion}"` };
    if (attachmentFilename) {
        const safe = attachmentFilename.replace(/[\\/"\r\n]/g, "_");
        headers["Content-Disposition"] = `attachment; filename="${safe}"`;
        headers["Cache-Control"] = "private, max-age=0, must-revalidate";
        delete headers["CDN-Cache-Control"];
        delete headers["Vercel-CDN-Cache-Control"];
    }

    return new Response(new Uint8Array(png), {
        status: 200,
        headers,
    });
}
