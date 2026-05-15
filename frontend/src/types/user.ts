export interface IUserProfile {
    id: string;
    uid: string;
    nickname: string | null;
    /** In-game discriminator suffix (`Eltik#1234`). String of digits, may be null. */
    nick_number: string | null;
    level: number | null;
    avatar_id: string | null;
    secretary: string | null;
    secretary_skin_id: string | null;
    resume_id: string | null;
    role: string;
    server: string;
    // Score
    total_score: number | null;
    grade: string | null;
    // Settings
    public_profile: boolean | null;
    store_gacha: boolean | null;
    share_stats: boolean | null;
    // Status
    exp: number | null;
    orundum: number | null;
    lmd: number | null;
    sanity: number | null;
    max_sanity: number | null;
    gacha_tickets: number | null;
    ten_pull_tickets: number | null;
    monthly_sub_end: number | null;
    register_ts: number | null;
    last_online_ts: number | null;
    resume: string | null;
    friend_num_limit: number | null;
    // Counts
    operator_count: number | null;
    item_count: number | null;
    skin_count: number | null;
    /** Owned skins excluding defaults — skin IDs containing `@`. */
    non_default_skin_count: number | null;
}
