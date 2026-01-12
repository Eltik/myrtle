/**
 * Type definitions for the leaderboard API endpoint
 */

/**
 * Sort options for the leaderboard
 */
export type SortBy = "total_score" | "operator_score" | "stage_score" | "roguelike_score" | "sandbox_score" | "medal_score" | "base_score" | "composite_score" | "grade";

/**
 * Activity metrics breakdown for grade calculation
 */
export interface ActivityMetrics {
    /** Days since last login */
    daysSinceLogin: number;
    /** Login recency score (0-100) */
    loginRecencyScore: number;
    /** Login frequency score (0-100) */
    loginFrequencyScore: number;
    /** Mission completion consistency score (0-100) */
    consistencyScore: number;
    /** Combined activity score (0-100) */
    totalActivityScore: number;
}

/**
 * Engagement metrics breakdown for grade calculation
 */
export interface EngagementMetrics {
    /** Content variety score (0-100) */
    contentVarietyScore: number;
    /** Roguelike engagement depth (0-100) */
    roguelikeDepthScore: number;
    /** Stage completion diversity (0-100) */
    stageDiversityScore: number;
    /** Account progression depth (0-100) */
    progressionDepthScore: number;
    /** Combined engagement score (0-100) */
    totalEngagementScore: number;
    /** Number of content types engaged (out of 6) */
    contentTypesEngaged: number;
}

/**
 * Detailed grade breakdown showing how the grade was calculated
 */
export interface GradeBreakdown {
    /** Account age in days */
    accountAgeDays: number;
    /** Normalized score component (0-100) - 50% weight */
    normalizedScore: number;
    /** Activity metrics (30% weight) */
    activityMetrics: ActivityMetrics;
    /** Engagement metrics (20% weight) */
    engagementMetrics: EngagementMetrics;
    /** Estimated percentile ranking */
    percentileEstimate: number;
}

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
    /** Detailed breakdown of how the grade was calculated */
    gradeBreakdown: GradeBreakdown;
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
