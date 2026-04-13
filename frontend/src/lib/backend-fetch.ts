import { env } from "~/env";

export interface BackendFetchOptions extends Omit<RequestInit, "headers"> {
    headers?: HeadersInit;
    /** JWT Bearer token for authenticated requests */
    bearerToken?: string;
}

/**
 * Fetch wrapper for backend API calls that automatically includes
 * the internal service key header for rate limit bypass.
 *
 * @param path - The path to append to BACKEND_URL (e.g., "/leaderboard")
 * @param options - Standard fetch options, plus optional bearerToken for auth
 */
export async function backendFetch(path: string, options: BackendFetchOptions = {}): Promise<Response> {
    const { headers: customHeaders, bearerToken, ...restOptions } = options;

    const url = new URL(`/api${path}`, env.BACKEND_URL);

    const headers = new Headers(customHeaders);

    // Always set Content-Type if not already set
    if (!headers.has("Content-Type")) {
        headers.set("Content-Type", "application/json");
    }

    // Include service key for rate limit bypass
    headers.set("X-Internal-Service-Key", env.INTERNAL_SERVICE_KEY);

    // Include Bearer token for authenticated requests
    if (bearerToken) {
        headers.set("Authorization", `Bearer ${bearerToken}`);
    }

    return fetch(url.toString(), {
        ...restOptions,
        headers,
    });
}
