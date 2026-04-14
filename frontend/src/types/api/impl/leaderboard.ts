/** Sort options for the v3 leaderboard */
export type SortBy = "total_score" | "operator_score" | "stage_score" | "roguelike_score" | "sandbox_score" | "medal_score" | "base_score" | "skin_score";

/** v3 leaderboard query parameters */
export interface LeaderboardQuery {
    sort?: SortBy;
    server?: string;
    limit?: number;
    offset?: number;
}

/** v3 leaderboard entry from v_leaderboard database view */
export interface LeaderboardEntry {
    id: string;
    uid: string;
    nickname: string | null;
    level: number | null;
    avatar_id: string | null;
    secretary: string | null;
    secretary_skin_id: string | null;
    server: string;
    total_score: number | null;
    grade: string | null;
    operator_score: number | null;
    stage_score: number | null;
    roguelike_score: number | null;
    sandbox_score: number | null;
    medal_score: number | null;
    base_score: number | null;
    skin_score: number | null;
    rank_global: number | null;
    rank_server: number | null;
}

/** v3 leaderboard response */
export interface LeaderboardResponse {
    entries: LeaderboardEntry[];
    total: number;
}
