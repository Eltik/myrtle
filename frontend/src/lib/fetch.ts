import { env } from "#/env";

export async function backendFetch(path: string, init: RequestInit & { bearerToken?: string } = {}) {
    const { bearerToken, headers, ...rest } = init;
    // `BACKEND_URL` is a server-only var: t3-env THROWS if it's read in the
    // browser. Gate by runtime so client-side (re)fetches use the client-exposed
    // `VITE_BACKEND_URL` and resolve to an absolute URL instead of throwing.
    const base = (typeof window === "undefined" ? env.BACKEND_URL : env.VITE_BACKEND_URL) ?? "";
    return fetch(`${base}/api${path}`, {
        ...rest,
        headers: {
            "Content-Type": "application/json",
            ...(bearerToken ? { Authorization: `Bearer ${bearerToken}` } : {}),
            ...headers,
        },
    });
}
