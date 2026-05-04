import { createFileRoute } from "@tanstack/react-router";
import { readCache, writeCache } from "#/lib/og/impl/cache";
import { getHandler } from "#/lib/og/impl/registry";
import { renderOgPng } from "#/lib/og/impl/render";

export const Route = createFileRoute("/api/og/$kind/$id")({
    server: {
        handlers: {
            GET: async ({ params }) => {
                const { kind, id } = params;
                const handler = getHandler(kind);

                const data = await handler.fetch(id);
                if (!data) return new Response("Not found", { status: 404 });

                const hash = handler.hash(data);

                let png = await readCache(kind, id, hash);
                if (!png) {
                    try {
                        png = await renderOgPng(handler.template(data));
                        await writeCache(kind, id, hash, png);
                    } catch (err) {
                        console.error(`[og] render failed for ${kind}/${id}:`, err);
                        return new Response("Render failed", { status: 500 });
                    }
                }

                return new Response(new Uint8Array(png), {
                    status: 200,
                    headers: {
                        "Content-Type": "image/png",
                        // Hash-versioned URL → safe to cache forever at the edge.
                        "Cache-Control": "public, max-age=31536000, immutable",
                        ETag: `"${hash}"`,
                    },
                });
            },
        },
    },
});
