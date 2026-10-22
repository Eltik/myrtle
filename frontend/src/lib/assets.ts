/**
 * URLs for raw-tree assets: files under a server's `textures`, `spine`, `audio`,
 * `video` and `portraits`, the roots the backend's generic `/api/assets` route
 * serves and the object store mirrors. Every such URL is built here.
 *
 * With `VITE_ASSET_BASE_URL` unset, a URL is exactly the backend string the
 * helpers built before the object store existed: `{backend}/api/assets/{path}`
 * for the default server, `{backend}/api/{server}/assets/{path}` otherwise, the
 * path passed through unencoded. That is the kill switch.
 *
 * With it set, a URL for a store server is `{base}/{server}/{path}`, the server
 * always present (the default one too), each segment percent-encoded with the
 * `encodeURIComponent` set, which is the store's key layout.
 *
 * PATHS are what the backend's wire carries: raw characters, except that `#` is
 * written `%23` (a raw `#` would end the URL path) and some icon paths write
 * `[` and `]` as `%5B` and `%5D`. No file under the synced roots has a `%` in its
 * name (EN and CN trees, 2026-10-22), so `%` followed by two hex digits is
 * always an escape: the store encoder keeps escapes and encodes everything else,
 * which makes it idempotent and never decodes anything.
 *
 * Indexed routes (`/avatar`, `/portrait`, `/skill-icon`, `/item-icon`, story
 * thumbnails, `?format=png` conversions) resolve ids on the backend and are not
 * built here; the backend redirects those to the store itself.
 */

import { env } from "#/env";
import { DEFAULT_GAMEDATA_SERVER } from "#/lib/api/gamedata";

/**
 * Servers whose URLs point at the store directly. The backend answers a request
 * for `server` from that server's tree, then the default's, then CN's. JP and KR
 * hold only part of `textures` and `audio`, so a direct URL there would 404 where
 * the backend falls back; their URLs stay on the backend, which redirects to the
 * store for whichever tree has the file.
 */
const STORE_SERVERS: ReadonlySet<string> = new Set(["en", "cn"]);

/** The store's default `S3_ROOTS`. A path under any other top-level directory stays on the backend. */
const STORE_ROOTS: ReadonlySet<string> = new Set(["textures", "spine", "audio", "video", "portraits"]);

const ESCAPE_SPLIT = /(%[0-9A-Fa-f]{2})/;

/** `VITE_ASSET_BASE_URL` without a trailing slash, or `undefined` when the store is off. */
function storeBase(): string | undefined {
    const base = env.VITE_ASSET_BASE_URL;
    return base ? base.replace(/\/+$/, "") : undefined;
}

/** One path segment with every character outside the `encodeURIComponent` unreserved set encoded, existing `%XX` escapes kept. */
export function encodeAssetSegment(segment: string): string {
    // `split` with a capture group puts the escapes at the odd indices.
    return segment
        .split(ESCAPE_SPLIT)
        .map((part, i) => (i % 2 === 1 ? part : encodeURIComponent(part)))
        .join("");
}

/** `path` with each `/`-separated segment encoded by {@link encodeAssetSegment}. Leading, trailing and inner slashes are kept. */
export function encodeAssetPath(path: string): string {
    return path.split("/").map(encodeAssetSegment).join("/");
}

/**
 * The store URL for `path` on `server`, or `null` when the store is off, the
 * server is not served from it directly, or the path is outside the store's
 * roots. A `?query` on the path is kept verbatim.
 */
export function assetStoreURL(path: string, server: string = DEFAULT_GAMEDATA_SERVER): string | null {
    const base = storeBase();
    if (!base || !STORE_SERVERS.has(server)) return null;
    const rel = path.replace(/^\/+/, "");
    const q = rel.indexOf("?");
    const file = q < 0 ? rel : rel.slice(0, q);
    const query = q < 0 ? "" : rel.slice(q);
    const slash = file.indexOf("/");
    if (slash <= 0 || !STORE_ROOTS.has(file.slice(0, slash))) return null;
    return `${base}/${server}/${encodeAssetPath(file)}${query}`;
}

/**
 * The URL of a raw-tree asset. `path` is relative to the server's tree, with or
 * without a leading slash; `server` defaults to the site's default server.
 */
export function assetURL(path: string, server?: string): string {
    const resolved = server || DEFAULT_GAMEDATA_SERVER;
    const store = assetStoreURL(path, resolved);
    if (store) return store;
    const prefix = resolved === DEFAULT_GAMEDATA_SERVER ? "" : `${resolved}/`;
    return `${env.VITE_BACKEND_URL ?? ""}/api/${prefix}assets/${path.replace(/^\/+/, "")}`;
}

/** Whether `url` points into the object store. Always false with the store off. */
export function isAssetStoreURL(url: string): boolean {
    const base = storeBase();
    return !!base && (url === base || url.startsWith(`${base}/`));
}
