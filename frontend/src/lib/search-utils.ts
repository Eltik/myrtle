import type { SearchQuery, SearchResponse, UserProfile } from "~/types/api";

/**
 * Builds URLSearchParams from a SearchQuery object.
 * v3 uses `q`, `limit`, `offset` only.
 */
export function buildSearchParams(params: SearchQuery): URLSearchParams {
    const searchParams = new URLSearchParams();

    if (params.q) {
        searchParams.set("q", params.q);
    }
    if (params.limit !== undefined) {
        searchParams.set("limit", String(params.limit));
    }
    if (params.offset !== undefined) {
        searchParams.set("offset", String(params.offset));
    }

    return searchParams;
}

/**
 * Builds URLSearchParams from a Next.js query object (string | string[] | undefined)
 * Used in API routes and getServerSideProps
 */
export function buildSearchParamsFromQuery(query: Record<string, string | string[] | undefined>): URLSearchParams {
    const searchParams = new URLSearchParams();

    if (query.q && typeof query.q === "string") {
        searchParams.set("q", query.q);
    }
    if (query.limit && typeof query.limit === "string") {
        searchParams.set("limit", query.limit);
    }
    if (query.offset && typeof query.offset === "string") {
        searchParams.set("offset", query.offset);
    }

    return searchParams;
}

/**
 * Simple in-memory cache for search results
 * Uses a Map with cache key based on search params
 */
const searchCache = new Map<string, { data: SearchResponse; timestamp: number }>();
const CACHE_TTL_MS = 30_000; // 30 seconds cache TTL
const MAX_CACHE_SIZE = 50; // Maximum number of cached entries

/**
 * Generates a cache key from search params
 */
function getCacheKey(params: URLSearchParams): string {
    const sorted = new URLSearchParams([...params.entries()].sort());
    return sorted.toString();
}

/**
 * Cleans up expired cache entries and enforces size limit
 */
function cleanupCache(): void {
    const now = Date.now();

    for (const [key, entry] of searchCache.entries()) {
        if (now - entry.timestamp > CACHE_TTL_MS) {
            searchCache.delete(key);
        }
    }

    if (searchCache.size > MAX_CACHE_SIZE) {
        const entries = [...searchCache.entries()].sort((a, b) => a[1].timestamp - b[1].timestamp);
        const toRemove = entries.slice(0, searchCache.size - MAX_CACHE_SIZE);
        for (const [key] of toRemove) {
            searchCache.delete(key);
        }
    }
}

/**
 * Gets cached search results if available and not expired
 */
export function getCachedSearch(params: URLSearchParams): SearchResponse | null {
    const key = getCacheKey(params);
    const cached = searchCache.get(key);

    if (cached && Date.now() - cached.timestamp < CACHE_TTL_MS) {
        return cached.data;
    }

    if (cached) {
        searchCache.delete(key);
    }

    return null;
}

/**
 * Stores search results in cache
 */
export function setCachedSearch(params: URLSearchParams, data: SearchResponse): void {
    cleanupCache();
    const key = getCacheKey(params);
    searchCache.set(key, { data, timestamp: Date.now() });
}

/**
 * AbortController manager for search requests
 * Ensures only one request is active at a time
 */
let currentAbortController: AbortController | null = null;

/**
 * Gets a new AbortController for a search request,
 * canceling any previous pending request
 */
export function getSearchAbortController(): AbortController {
    if (currentAbortController) {
        currentAbortController.abort();
    }

    currentAbortController = new AbortController();
    return currentAbortController;
}

/**
 * Clears the current abort controller reference
 * Call this when a request completes successfully
 */
export function clearSearchAbortController(): void {
    currentAbortController = null;
}

/**
 * Fetches search results from the API with caching and request cancellation.
 * The v3 API route returns UserProfile[] which we wrap with pagination info.
 */
export async function fetchSearchResultsCached(params: SearchQuery, options?: { signal?: AbortSignal; skipCache?: boolean }): Promise<SearchResponse> {
    const searchParams = buildSearchParams(params);

    if (!options?.skipCache) {
        const cached = getCachedSearch(searchParams);
        if (cached) {
            return cached;
        }
    }

    const response = await fetch(`/api/search?${searchParams.toString()}`, {
        signal: options?.signal,
    });

    if (!response.ok) {
        let errorMessage = `Failed to fetch search results (${response.status})`;
        try {
            const errorData = (await response.json()) as { error?: string };
            if (errorData.error) {
                errorMessage = `${errorMessage}: ${errorData.error}`;
            }
        } catch {}
        throw new Error(errorMessage);
    }

    // v3 API returns UserProfile[] directly - wrap with pagination
    const results = (await response.json()) as UserProfile[];
    const limit = params.limit ?? 24;
    const offset = params.offset ?? 0;

    const data: SearchResponse = {
        results: Array.isArray(results) ? results : [],
        pagination: {
            limit,
            offset,
            total: Array.isArray(results) ? (results.length < limit ? offset + results.length : offset + limit + 1) : 0,
            hasMore: Array.isArray(results) && results.length >= limit,
        },
    };

    setCachedSearch(searchParams, data);

    return data;
}
