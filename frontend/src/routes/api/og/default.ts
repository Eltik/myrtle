import { createFileRoute } from "@tanstack/react-router";
import { readCache, writeCache } from "#/lib/og/impl/cache";
import { getHandler } from "#/lib/og/impl/registry";
import { renderOgPng } from "#/lib/og/impl/render";

// Title shown on the site-wide fallback OG (any page that doesn't supply its
// own `image` to seo() falls back to this through OG_CONFIG.defaultImage).
const DEFAULT_TITLE = "Operator data, rosters, and tier lists.";
const CACHE_KIND = "default";
const CACHE_ID = "_root";

export const Route = createFileRoute("/api/og/default")({
    server: {
        handlers: {
            GET: async () => {
                const handler = getHandler(CACHE_KIND);
                const data = await handler.fetch(DEFAULT_TITLE);
                if (!data) return new Response("Not found", { status: 404 });

                const hash = handler.hash(data);

                let png = await readCache(CACHE_KIND, CACHE_ID, hash);
                if (!png) {
                    try {
                        png = await renderOgPng(handler.template(data));
                        await writeCache(CACHE_KIND, CACHE_ID, hash, png);
                    } catch (err) {
                        console.error("[og] render failed for default:", err);
                        return new Response("Render failed", { status: 500 });
                    }
                }

                return new Response(new Uint8Array(png), {
                    status: 200,
                    headers: {
                        "Content-Type": "image/png",
                        "Cache-Control": "public, max-age=31536000, immutable",
                        ETag: `"${hash}"`,
                    },
                });
            },
        },
    },
});
