/**
 * Carrying an OAuth authorization request from `/oauth/authorize` to the
 * consent page without the router touching it.
 *
 * The router parses every search value with `JSON.parse` and writes the URL
 * back in its own spelling: a request for `?state=1e3` is redirected to
 * `?state=1000`, `state=007` becomes the number 7. `state` belongs to the
 * site that sent the player and must return to it byte for byte, or that
 * site's check that the answer matches its question fails.
 *
 * So `/oauth/authorize` is a server route that never reaches the router's
 * search handling. It packs the raw query string into ONE value, base64url
 * behind a `q.` prefix, and redirects to `/oauth/consent?request=...`. That
 * value is a string no JSON parser accepts, so the router leaves it alone.
 */

import type { AuthorizeParams } from "#/types/generated/AuthorizeParams";

const PREFIX = "q.";

const PARAMS = ["response_type", "client_id", "redirect_uri", "scope", "state", "code_challenge", "code_challenge_method"] as const satisfies readonly (keyof AuthorizeParams)[];

/** The raw query string (with or without its `?`) as one opaque search value. */
export function packAuthorizeRequest(search: string): string {
    const bytes = new TextEncoder().encode(search.replace(/^\?/, ""));
    let binary = "";
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return PREFIX + btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

/**
 * The authorization request inside a packed value. Anything that is not one
 * (absent, hand-edited, truncated) reads as an empty request, which the
 * backend then answers with its own "client_id is required".
 */
export function unpackAuthorizeRequest(packed: unknown): AuthorizeParams {
    const request: AuthorizeParams = {};
    if (typeof packed !== "string" || !packed.startsWith(PREFIX)) return request;
    let search: string;
    try {
        const binary = atob(packed.slice(PREFIX.length).replaceAll("-", "+").replaceAll("_", "/"));
        search = new TextDecoder().decode(Uint8Array.from(binary, (c) => c.charCodeAt(0)));
    } catch {
        return request;
    }
    const query = new URLSearchParams(search);
    for (const key of PARAMS) {
        const value = query.get(key);
        if (value !== null) request[key] = value;
    }
    return request;
}
