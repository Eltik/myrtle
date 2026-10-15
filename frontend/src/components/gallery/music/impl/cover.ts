/**
 * SIZED COVERS. Monster Siren covers are 500 x 500 JPEGs on `web.hycdn.cn`
 * (about 144 KB each), and the page drew them at 28 to 218 CSS px: a row of
 * All Songs decoded a 500 px JPEG into a 28 px box. The CDN is Aliyun OSS and
 * honours `x-oss-process`, so a cover is asked for at the width it is drawn
 * at, as WebP: 64 px WebP is 896 B against 144,119 B for the original
 * (measured 2026-10-11 on `ab46323f...jpg`; `imageMogr2`, `imageView2` and
 * `?w=` are ignored by this CDN and return the original).
 *
 * Only URLs on a host the Siren catalogue itself serves covers from, and
 * without a query, are rewritten: `buildCatalog` hands every album
 * `coverUrl` host to `rememberSirenCoverHosts`, so no CDN host is typed in
 * here and a host the API moves to is picked up. Anything else (a URL that
 * already carries parameters, any other host) passes through.
 * Kill switch: `?fullcovers=1` on the page URL serves every original again.
 */
const sizedHosts = new Set<string>();

/** Record the hosts of the Siren catalogue's cover URLs, the only ones `sizedCover` rewrites. */
export function rememberSirenCoverHosts(urls: Iterable<string>): void {
    for (const url of urls) {
        try {
            sizedHosts.add(new URL(url).host);
        } catch {
            // Not a URL: nothing to size.
        }
    }
}

/** Device pixels per CSS pixel the thumbnails are cut for. */
const DPR = 2;

function disabled(): boolean {
    if (typeof window === "undefined") return false;
    try {
        return new URLSearchParams(window.location.search).get("fullcovers") === "1";
    } catch {
        return false;
    }
}

const off = disabled();

/** `url` cut to `cssPx` wide (at 2x) as WebP, when the CDN can; otherwise `url` unchanged. */
export function sizedCover(url: string, cssPx: number): string;
export function sizedCover(url: string | null, cssPx: number): string | null;
export function sizedCover(url: string | null, cssPx: number): string | null {
    if (!url || off || url.includes("?")) return url;
    let host: string;
    try {
        host = new URL(url).host;
    } catch {
        return url;
    }
    if (!sizedHosts.has(host)) return url;
    const w = Math.max(16, Math.min(500, Math.round(cssPx * DPR)));
    return `${url}?x-oss-process=image/resize,w_${w}/format,webp`;
}
