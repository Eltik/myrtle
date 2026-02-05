import type { SearchQuery, SearchResponse } from "~/types/api";

/**
 * All search parameter keys that can be passed to the backend
 */
const SEARCH_PARAM_KEYS = [
    // Text search params
    "nickname",
    "uid",
    "resume",
    // Exact match filters
    "server",
    "grade",
    "secretary",
    // Range query params
    "level",
    "totalScore",
    "compositeScore",
    "operatorScore",
    "stageScore",
    "roguelikeScore",
    "sandboxScore",
    "medalScore",
    "baseScore",
    // Query options
    "logic",
    "sortBy",
    "order",
    "limit",
    "offset",
    "fields",
] as const;

/**
 * Builds URLSearchParams from a SearchQuery object
 * Works for both client-side and server-side usage
 */
export function buildSearchParams(params: SearchQuery): URLSearchParams {
    const searchParams = new URLSearchParams();

    for (const key of SEARCH_PARAM_KEYS) {
        const value = params[key as keyof SearchQuery];
        if (value !== undefined && value !== null && value !== "") {
            searchParams.set(key, String(value));
        }
    }

    return searchParams;
}

/**
 * Builds URLSearchParams from a Next.js query object (string | string[] | undefined)
 * Used in API routes and getServerSideProps
 */
export function buildSearchParamsFromQuery(query: Record<string, string | string[] | undefined>): URLSearchParams {
    const searchParams = new URLSearchParams();

    for (const key of SEARCH_PARAM_KEYS) {
        const value = query[key];
        if (value && typeof value === "string") {
            searchParams.set(key, value);
        }
    }

    const levelMin = query.levelMin;
    const levelMax = query.levelMax;
    if ((levelMin && typeof levelMin === "string") || (levelMax && typeof levelMax === "string")) {
        const min = levelMin && typeof levelMin === "string" ? levelMin : "";
        const max = levelMax && typeof levelMax === "string" ? levelMax : "";
        searchParams.set("level", `${min},${max}`);
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
 * Fetches search results from the API with caching and request cancellation
 * @param params - Search query parameters
 * @param options - Fetch options including abort signal
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

    const data = (await response.json()) as SearchResponse;

    setCachedSearch(searchParams, data);

    return data;
}
