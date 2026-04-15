import { env } from "#/env";

export async function backendFetch(path: string, init: RequestInit & { bearerToken?: string } = {}) {
    const { bearerToken, headers, ...rest } = init;
    return fetch(`${env.BACKEND_URL}/api${path}`, {
        ...rest,
        headers: {
            "Content-Type": "application/json",
            ...(bearerToken ? { Authorization: `Bearer ${bearerToken}` } : {}),
            ...headers,
        },
    });
}
