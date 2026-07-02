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

interface IOgResponseArgs {
    kind: string;
    // Identifier passed to the kind's `fetch` to load template data.
    fetchId: string;
    // Cheap cache version from the OG URL. This lets cache hits avoid loading
    // template data just to compute a content hash.
    version?: string;
    // Identifier used as the cache filename. Defaults to `fetchId`. Use this
    // when the public id (e.g. a title) is not a stable cache key.
    cacheId?: string;
    // When set, the response includes Content-Disposition: attachment with
    // this filename so the browser saves the image instead of inlining it.
    attachmentFilename?: string;
}

export async function ogResponse({ kind, fetchId, version, cacheId = fetchId, attachmentFilename }: IOgResponseArgs): Promise<Response> {
    const handler = getHandler(kind);
    const cacheVersion = handler.cacheVersion(fetchId, version);

    let png = await readCache(kind, cacheId, cacheVersion);
    if (!png) {
        const data = await handler.fetch(fetchId);
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
