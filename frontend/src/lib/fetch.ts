import { env } from "#/env";
import { BackendUnreachableError } from "#/lib/api/_shared";

export async function backendFetch(path: string, init: RequestInit & { bearerToken?: string } = {}) {
    const { bearerToken, headers, ...rest } = init;
    // `BACKEND_URL` is a server-only var: t3-env THROWS if it's read in the
    // browser. Gate by runtime so client-side (re)fetches use the client-exposed
    // `VITE_BACKEND_URL` and resolve to an absolute URL instead of throwing.
    const base = (typeof window === "undefined" ? env.BACKEND_URL : env.VITE_BACKEND_URL) ?? "";
    try {
        return await fetch(`${base}/api${path}`, {
            ...rest,
            headers: {
                "Content-Type": "application/json",
                ...(bearerToken ? { Authorization: `Bearer ${bearerToken}` } : {}),
                ...headers,
            },
        });
    } catch (err) {
        // `fetch` only throws when no response came back at all. Undici's
        // message for that is "fetch failed", which reached login toasts
        // verbatim; name the situation and keep the socket code for the report.
        // The raw error stays here in the server log (it names the internal
        // backend address), only the code travels to the browser.
        console.error(`backendFetch ${rest.method ?? "GET"} /api${path}: no response`, err);
        throw new BackendUnreachableError(err);
    }
}
