/**
 * Type definitions for the search API endpoint
 */

/**
 * Sort options for search results
 * Note: Backend uses snake_case for sort fields
 */
export type SearchSortBy = "total_score" | "composite_score" | "operator_score" | "stage_score" | "roguelike_score" | "sandbox_score" | "medal_score" | "base_score" | "level" | "nickname" | "created_at" | "updated_at" | "register_ts";

/**
 * Valid server values
 */
export type SearchServer = "en" | "jp" | "cn" | "kr" | "tw" | "bili";

/**
 * Valid grade values
 */
export type SearchGrade = "S" | "A" | "B" | "C" | "D" | "F";

/**
 * Query parameters for the search endpoint
 */
export interface SearchQuery {
    /** Fuzzy search by nickname */
    nickname?: string;
    /** Exact search by UID */
    uid?: string;
    /** Search in user bio/resume */
    resume?: string;
    /** Filter by server */
    server?: SearchServer;
    /** Filter by grade */
    grade?: SearchGrade;
    /** Filter by secretary character ID */
    secretary?: string;
    /** Level range (format: "min,max" or "min" or ",max") */
    level?: string;
    /** Total score range */
    totalScore?: string;
    /** Composite score range */
    compositeScore?: string;
    /** Operator score range */
    operatorScore?: string;
    /** Stage score range */
    stageScore?: string;
    /** Roguelike score range */
    roguelikeScore?: string;
    /** Sandbox score range */
    sandboxScore?: string;
    /** Medal score range */
    medalScore?: string;
    /** Base score range */
    baseScore?: string;
    /** Query logic: "and" or "or" (default: "and") */
    logic?: "and" | "or";
    /** Field to sort by (default: totalScore) */
    sortBy?: SearchSortBy;
    /** Sort order (default: "desc") */
    order?: "asc" | "desc";
    /** Results per page (default: 25, max: 100) */
    limit?: number;
    /** Pagination offset (default: 0) */
    offset?: number;
    /** Comma-separated fields to include: "data", "score", "settings" */
    fields?: string;
}

/**
 * Single search result entry
 */
export interface SearchResultEntry {
    uid: string;
    server: string;
    nickname: string;
    level: number;
    avatarId: string | null;
    secretary: string | null;
    grade: string;
    totalScore: number;
    updatedAt: string;
    /** Optional full user data (only if requested via 'fields' param) - contains status.resume, status.registerTs, etc. */
    data?: Record<string, unknown>;
    /** Optional full score data (only if requested via 'fields' param) */
    score?: Record<string, unknown>;
    /** Optional full settings data (only if requested via 'fields' param) */
    settings?: Record<string, unknown>;
}

/**
 * Pagination information for search results
 */
export interface SearchPaginationInfo {
    limit: number;
    offset: number;
    total: number;
    hasMore: boolean;
}

/**
 * Metadata about the search query
 */
export interface SearchMeta {
    queryLogic: string;
    sortBy: string;
    order: string;
    filtersApplied: string[];
    cached: boolean;
}

/**
 * Full search response from the API
 */
export interface SearchResponse {
    results: SearchResultEntry[];
    pagination: SearchPaginationInfo;
    meta: SearchMeta;
}
