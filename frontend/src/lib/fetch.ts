import { env } from "#/env";
import { BackendUnreachableError } from "#/lib/api/_shared";

export async function backendFetch(path: string, init: RequestInit & { bearerToken?: string } = {}) {
    const { bearerToken, headers, ...rest } = init;
    // `BACKEND_URL` is a server-only var: t3-env THROWS if it's read in the
    // browser. Gate by runtime so client-side (re)fetches use the client-exposed
    // `VITE_BACKEND_URL` and resolve to an absolute URL instead of throwing.
    const isServer = typeof window === "undefined";
    const base = (isServer ? env.BACKEND_URL : env.VITE_BACKEND_URL) ?? "";
    // Server side, BACKEND_URL is a local-network hop (localhost:3060 or the compose network):
    // a compressed response is decompressed at once, so it costs CPU on both ends for no
    // bandwidth worth saving. A caller's own Accept-Encoding still wins.
    const identity = isServer && !(headers && new Headers(headers).has("accept-encoding"));
    try {
        return await fetch(`${base}/api${path}`, {
            ...rest,
            headers: {
                "Content-Type": "application/json",
                ...(identity ? { "Accept-Encoding": "identity" } : {}),
                ...(bearerToken ? { Authorization: `Bearer ${bearerToken}` } : {}),
                ...headers,
            },
        });
    } catch (err) {
        // `fetch` only throws when no response came back at all. Undici's
        // message for that is "fetch failed", which would reach login toasts
        // verbatim; name the situation and keep the socket code for the report.
        // The raw error stays here in the server log (it names the internal
        // backend address), only the code travels to the browser.
        console.error(`backendFetch ${rest.method ?? "GET"} /api${path}: no response`, err);
        throw new BackendUnreachableError(err);
    }
}
