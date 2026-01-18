/**
 * Type definitions for the leaderboard API endpoint
 */

/**
 * Sort options for the leaderboard
 */
export type SortBy = "total_score" | "operator_score" | "stage_score" | "roguelike_score" | "sandbox_score" | "medal_score" | "base_score" | "composite_score" | "grade";

/**
 * Field selection mode for leaderboard entries
 * - "minimal": Core fields only (rank, uid, server, nickname, level, avatarId, totalScore, grade, updatedAt)
 * - "full": All fields including individual scores and gradeBreakdown
 */
export type LeaderboardFieldsMode = "minimal" | "full";

/**
 * Activity metrics breakdown for grade calculation
 */
export interface ActivityMetrics {
    /** Days since last login */
    daysSinceLogin: number;
    /** Login recency score (0-100) - based on last_online_ts */
    loginRecencyScore: number;
    /** Login frequency score (0-100) - based on check-in cycle completion */
    loginFrequencyScore: number;
    /** Mission completion consistency score (0-100) */
    consistencyScore: number;
    /** Combined activity score (0-100) */
    totalActivityScore: number;
    /** Days checked in during current cycle (count of 1s in check_in_history) */
    checkInsThisCycle: number;
    /** Total days in current check-in cycle window (typically 15-16) */
    checkInCycleLength: number;
    /** Check-in completion rate for current cycle (0-100) */
    checkInCompletionRate: number;
}

/**
 * Engagement metrics breakdown for grade calculation
 */
export interface EngagementMetrics {
    /** Content variety score (0-100) - how many content types engaged */
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
    /** Field selection mode: "minimal" (default) or "full" */
    fields?: LeaderboardFieldsMode;
}

/**
 * Single leaderboard entry representing a user's ranking
 * Note: Score fields and gradeBreakdown are only present when fields=full
 */
export interface LeaderboardEntry {
    // Core fields (always present)
    rank: number;
    uid: string;
    server: string;
    nickname: string;
    level: number;
    avatarId: string | null;
    totalScore: number;
    /** Grade letter: "S", "A", "B", "C", "D", or "F" */
    grade: string;
    /** ISO 8601 timestamp */
    updatedAt: string;

    // Optional fields (only present when fields=full)
    operatorScore?: number;
    stageScore?: number;
    roguelikeScore?: number;
    sandboxScore?: number;
    medalScore?: number;
    baseScore?: number;
    compositeScore?: number;
    /** Detailed breakdown of how the grade was calculated */
    gradeBreakdown?: GradeBreakdown;
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
    fields: string;
}

/**
 * Full leaderboard response from the API
 */
export interface LeaderboardResponse {
    entries: LeaderboardEntry[];
    pagination: PaginationInfo;
    meta: LeaderboardMeta;
}
