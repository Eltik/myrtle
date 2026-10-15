import { absolute } from "#/lib/seo";
import type { FileRoutesByFullPath } from "#/routeTree.gen";

/**
 * The public pages a crawler should find, by their route path. Typed against the route
 * tree, so renaming or removing a page fails `tsc` here instead of shipping a dead URL.
 *
 * Left out on purpose: per-user and signed-in pages (`/settings`, `/gacha/history`,
 * `/_authed/*`), the `noindex` help page, the dev-only `/dyntest`, and the two off-site
 * redirects (`/discord`, `/donate`). Detail pages (`/operators/$id` and the like) are
 * reached by crawling the list pages; listing them here would mean a backend read per
 * sitemap request.
 */
export const SITEMAP_PATHS = [
    "/",
    "/operators",
    "/enemies",
    "/stages",
    "/stories",
    "/stories/sprites",
    "/archives",
    "/announcements",
    "/tier-lists",
    "/grids",
    "/gacha/community",
    "/gallery/music",
    "/stats",
    "/changelog",
    "/tools/birthdays",
    "/tools/dps",
    "/tools/hps",
    "/tools/planner",
    "/tools/randomizer",
    "/tools/recruitment",
    "/tools/release",
    "/tools/squad-builder",
    "/user/leaderboard",
    "/user/search",
    "/privacy",
    "/terms",
] as const satisfies readonly (keyof FileRoutesByFullPath)[];

const escapeXml = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/** A sitemaps.org `urlset` of absolute URLs, one per path. */
export function sitemapXml(paths: readonly string[] = SITEMAP_PATHS): string {
    const urls = paths.map((p) => `  <url><loc>${escapeXml(absolute(p))}</loc></url>`).join("\n");
    return `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls}\n</urlset>\n`;
}
