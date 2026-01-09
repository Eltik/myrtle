/**
 * Type definitions for the leaderboard API endpoint
 */

/**
 * Sort options for the leaderboard
 */
export type SortBy = "total_score" | "operator_score" | "stage_score" | "roguelike_score" | "sandbox_score" | "medal_score" | "base_score" | "composite_score" | "grade";

/**
 * Query parameters for the leaderboard endpoint
 */
export interface LeaderboardQuery {
    /** Field to sort by (default: total_score) */
    sort_by?: SortBy;
    /** Sort order: "asc" or "desc" (default: desc) */
    order?: "asc" | "desc";
    /** Optional server filter (en, jp, cn, kr, tw, bili) */
    server?: string;
    /** Results per page (default: 25, max: 100) */
    limit?: number;
    /** Pagination offset (default: 0) */
    offset?: number;
}

/**
 * Single leaderboard entry representing a user's ranking
 */
export interface LeaderboardEntry {
    rank: number;
    uid: string;
    server: string;
    nickname: string;
    level: number;
    avatarId: string | null;
    totalScore: number;
    operatorScore: number;
    stageScore: number;
    roguelikeScore: number;
    sandboxScore: number;
    medalScore: number;
    baseScore: number;
    compositeScore: number;
    /** Grade letter: "S", "A", "B", "C", "D", or "F" */
    grade: string;
    /** ISO 8601 timestamp */
    updatedAt: string;
}

/**
 * Pagination information for leaderboard results
 */
export interface PaginationInfo {
    limit: number;
    offset: number;
    total: number;
    hasMore: boolean;
}

/**
 * Metadata about the leaderboard query
 */
export interface LeaderboardMeta {
    sortBy: string;
    order: string;
    serverFilter: string | null;
}

/**
 * Full leaderboard response from the API
 */
export interface LeaderboardResponse {
    entries: LeaderboardEntry[];
    pagination: PaginationInfo;
    meta: LeaderboardMeta;
}
