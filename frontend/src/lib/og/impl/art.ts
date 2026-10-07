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

/** Template data fields that hold a picture URL: `charArtURL`, `factionLogoURL`, `previewImageURL`, `avatarURL`, ... */
const IMAGE_URL_KEY = /(art|icon|logo|avatar|image)URL$/i;

function collectRemoteImageURLs(value: unknown, out: Set<string>): void {
    if (Array.isArray(value)) {
        for (const item of value) collectRemoteImageURLs(item, out);
        return;
    }
    if (value === null || typeof value !== "object") return;
    for (const [key, v] of Object.entries(value)) {
        if (typeof v === "string") {
            if (IMAGE_URL_KEY.test(key) && /^https?:\/\//.test(v)) out.add(v);
        } else {
            collectRemoteImageURLs(v, out);
        }
    }
}

function replaceRemoteImageURLs<T>(value: T, art: Map<string, string | undefined>): T {
    if (Array.isArray(value)) return value.map((item) => replaceRemoteImageURLs(item, art)) as unknown as T;
    if (value === null || typeof value !== "object") return value;
    const out: Record<string, unknown> = {};
    for (const [key, v] of Object.entries(value)) {
        out[key] = typeof v === "string" && art.has(v) && IMAGE_URL_KEY.test(key) ? art.get(v) : replaceRemoteImageURLs(v, art);
    }
    return out as T;
}

/**
 * `data` with every remote picture URL inlined as a data URI, and every one
 * that failed to load or is not a type satori decodes replaced by `undefined`.
 *
 * Handed a URL, satori fetches it itself, and on a body it cannot decode it
 * draws nothing and logs "Can't load image <url>: Unsupported image type:
 * unknown" through console.error. The faction logo route answers 404 (a JSON
 * body) for factions with no logo sprite (`logo_laterano.png`,
 * `logo_leithanien.png`), so every operator card of those factions put that
 * line in the error log while still rendering. Every template already treats
 * a missing URL as "draw nothing there", so dropping the URL here draws the
 * same card without the log line, and without satori fetching it a second time.
 *
 * Runs on the render path only, after the content hash, so cache keys are
 * unchanged. URLs already inlined (`data:`) are left alone.
 */
export async function inlineRemoteImages<T>(data: T): Promise<T> {
    const urls = new Set<string>();
    collectRemoteImageURLs(data, urls);
    if (urls.size === 0) return data;
    return replaceRemoteImageURLs(data, await inlineArt(urls));
}
