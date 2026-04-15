import type { UserProfile } from "./user";

/** v3 search query - simplified to single query string */
export interface SearchQuery {
    q?: string;
    limit?: number;
    offset?: number;
}

/** v3 raw search response - returns UserProfile array directly */
export type SearchApiResponse = UserProfile[];

/**
 * Client-side search response with pagination wrapper.
 * The v3 backend returns UserProfile[] directly, so the SSR/API route
 * wraps it with pagination info for the component to consume.
 */
export interface SearchResponse {
    results: UserProfile[];
    pagination: {
        limit: number;
        offset: number;
        total: number;
        hasMore: boolean;
    };
}
