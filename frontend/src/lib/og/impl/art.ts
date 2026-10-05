/**
 * Entity art for the server-rendered images, made safe for satori.
 *
 * satori decodes PNG, JPEG, GIF and SVG and nothing else. The story sprite
 * thumbnail route serves lossless WebP by default (every
 * `/story-sprite-thumb/{id}` on production answers `image/webp`, e.g.
 * `avg_npc_253` at 36,244 bytes), and handed that, satori drew the cell
 * empty. So the images ask for PNG where a route can serve it, and inline
 * every picture themselves, dropping any whose type satori cannot decode
 * rather than embedding bytes it would draw as nothing.
 */

import { entityIconURL } from "#/lib/api/tier-entities";

/** Content types satori decodes from a data URI. */
const SATORI_IMAGE_TYPES: ReadonlySet<string> = new Set(["image/png", "image/jpeg", "image/gif", "image/svg+xml"]);

/**
 * Icon routes whose default response is WebP and that serve PNG on
 * `?format=png`. Of the 13 entity kinds' icons, only story sprites use one;
 * the rest are PNG files under `/assets` or the PNG `/avatar` and
 * `/enemy-icon` routes.
 */
const WEBP_ICON_ROUTES = ["/story-sprite-thumb/"] as const;

/** `contentType` (a `Content-Type` header value) as the media type satori can decode, or `undefined` when it cannot. */
export function satoriImageType(contentType: string | null | undefined): string | undefined {
    const type = contentType?.split(";")[0]?.trim().toLowerCase();
    if (type === "image/jpg") return "image/jpeg";
    return type && SATORI_IMAGE_TYPES.has(type) ? type : undefined;
}

/** {@link entityIconURL} for a server-rendered image: a WebP-default route is asked for PNG. */
export function ogEntityIconURL(icon: string, base: string, server?: string): string {
    const url = entityIconURL(icon, base, server);
    if (!WEBP_ICON_ROUTES.some((route) => icon.startsWith(route))) return url;
    return `${url}${url.includes("?") ? "&" : "?"}format=png`;
}

/** `url`'s bytes as a data URI, or `undefined` when the fetch fails or the response is not a type satori decodes. */
export async function fetchToDataURI(url: string): Promise<string | undefined> {
    if (!url) return undefined;
    try {
        const res = await fetch(url);
        if (!res.ok) return undefined;
        const type = satoriImageType(res.headers.get("content-type"));
        if (!type) return undefined;
        const buf = Buffer.from(await res.arrayBuffer());
        return `data:${type};base64,${buf.toString("base64")}`;
    } catch {
        return undefined;
    }
}

/** Fetches at once when inlining a board's art: a full tier list holds a few hundred tiles. */
const INLINE_CONCURRENCY = 16;

/** Every distinct non-empty URL in `urls` inlined as a data URI; a URL that failed or is not decodable maps to `undefined`. */
export async function inlineArt(urls: Iterable<string | null | undefined>): Promise<Map<string, string | undefined>> {
    const unique = [...new Set([...urls].filter((u): u is string => !!u))];
    const out = new Map<string, string | undefined>();
    let next = 0;
    const worker = async () => {
        while (next < unique.length) {
            const url = unique[next++] as string;
            out.set(url, await fetchToDataURI(url));
        }
    };
    await Promise.all(Array.from({ length: Math.min(INLINE_CONCURRENCY, unique.length) }, worker));
    return out;
}
