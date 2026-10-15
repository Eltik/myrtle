import { describe, expect, it } from "vitest";
import { OG_CONFIG } from "#/lib/og/impl/config";
import { SITEMAP_PATHS, sitemapXml } from "./sitemap";

describe("sitemapXml", () => {
    const xml = sitemapXml();
    const locs = [...xml.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]);

    it("lists every public page once, as an absolute URL on the site", () => {
        const base = OG_CONFIG.siteURL.replace(/\/$/, "");
        expect(locs).toHaveLength(SITEMAP_PATHS.length);
        expect(new Set(locs).size).toBe(locs.length);
        for (const loc of locs) expect(loc.startsWith(`${base}/`)).toBe(true);
        expect(locs).toContain(`${base}/tools/dps`);
    });

    it("is a sitemaps.org urlset", () => {
        expect(xml.startsWith('<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">')).toBe(true);
        expect(xml.trimEnd().endsWith("</urlset>")).toBe(true);
    });

    it("leaves out signed-in, noindex and redirect-only pages", () => {
        for (const path of ["/settings", "/gacha/history", "/help-me", "/dyntest", "/discord", "/donate"]) {
            expect(locs.some((loc) => loc.endsWith(path))).toBe(false);
        }
    });

    it("escapes what XML cannot carry raw", () => {
        expect(sitemapXml(["/a?b=1&c=2"])).toContain("/a?b=1&amp;c=2</loc>");
    });
});
