import { readCache, writeCache } from "./cache";
import { getHandler } from "./registry";
import { renderOgPng } from "./render";

const PNG_HEADERS: Record<string, string> = {
    "Content-Type": "image/png",
    "Cache-Control": "public, max-age=31536000, s-maxage=31536000, immutable",
    "CDN-Cache-Control": "public, max-age=31536000, immutable",
    "Vercel-CDN-Cache-Control": "public, max-age=31536000, immutable, stale-while-revalidate=86400",
};

interface IOgResponseArgs {
    kind: string;
    // Identifier passed to the kind's `fetch` to load template data.
    fetchId: string;
    // Identifier used as the cache filename. Defaults to `fetchId`. Use this
    // when the public id (e.g. a title) is not a stable cache key.
    cacheId?: string;
    // When set, the response includes Content-Disposition: attachment with
    // this filename so the browser saves the image instead of inlining it.
    attachmentFilename?: string;
}

export async function ogResponse({ kind, fetchId, cacheId = fetchId, attachmentFilename }: IOgResponseArgs): Promise<Response> {
    const handler = getHandler(kind);

    const data = await handler.fetch(fetchId);
    if (!data) return new Response("Not found", { status: 404 });

    const hash = handler.hash(data);

    let png = await readCache(kind, cacheId, hash);
    if (!png) {
        try {
            const dimensions = handler.dimensions?.(data);
            png = await renderOgPng(handler.template(data), dimensions);
            await writeCache(kind, cacheId, hash, png);
        } catch (err) {
            console.error(`[og] render failed for ${kind}/${cacheId}:`, err);
            return new Response("Render failed", { status: 500 });
        }
    }

    const headers: Record<string, string> = { ...PNG_HEADERS, ETag: `"${hash}"` };
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
