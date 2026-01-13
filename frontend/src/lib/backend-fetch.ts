import { env } from "~/env";

export interface BackendFetchOptions extends Omit<RequestInit, "headers"> {
    headers?: HeadersInit;
}

/**
 * Fetch wrapper for backend API calls that automatically includes
 * the internal service key header for rate limit bypass.
 *
 * @param path - The path to append to BACKEND_URL (e.g., "/leaderboard")
 * @param options - Standard fetch options
 */
export async function backendFetch(path: string, options: BackendFetchOptions = {}): Promise<Response> {
    const { headers: customHeaders, ...restOptions } = options;

    const url = new URL(path, env.BACKEND_URL);

    const headers = new Headers(customHeaders);

    // Always set Content-Type if not already set
    if (!headers.has("Content-Type")) {
        headers.set("Content-Type", "application/json");
    }

    // Include service key for rate limit bypass
    headers.set("X-Internal-Service-Key", env.INTERNAL_SERVICE_KEY);

    return fetch(url.toString(), {
        ...restOptions,
        headers,
    });
}
