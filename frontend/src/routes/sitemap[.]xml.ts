import { createFileRoute } from "@tanstack/react-router";
import { sitemapXml } from "#/lib/sitemap";

export const Route = createFileRoute("/sitemap.xml")({
    server: {
        handlers: {
            GET: () => new Response(sitemapXml(), { headers: { "Content-Type": "application/xml; charset=utf-8", "Cache-Control": "public, max-age=86400" } }),
        },
    },
});
