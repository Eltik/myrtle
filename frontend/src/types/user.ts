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
    /** Lifetime cumulative sign-in days (the in-game "total days of sign-ins" counter). */
    cumulative_signin: number | null;
    // Counts
    operator_count: number | null;
    item_count: number | null;
    skin_count: number | null;
    /** Owned skins excluding defaults - skin IDs containing `@`. */
    non_default_skin_count: number | null;
}

/** Daily sign-in state from the game's `checkIn` section (`/get-user-checkin`). */
export interface IUserCheckin {
    /** Current month's calendar: one entry per day, `1` = claimed, `0` = not. */
    history: number[];
    /** Lifetime cumulative sign-in days (the "total days of sign-ins" counter). */
    cumulative_signin: number;
    /** Active monthly sign-in series id (e.g. `signin<N>`). */
    checkin_group_id: string | null;
    /** Days already claimed in the current month's calendar. */
    reward_index: number;
    /** Whether a daily sign-in is claimable right now (as of the last sync). */
    can_check_in: boolean;
    /** Account creation time (Unix seconds) - for "days since joining". */
    register_ts: number | null;
    /** Player's last in-game online time (Unix seconds) - distinct from sync time. */
    last_online_ts: number | null;
    /** ISO timestamp of the last DB sync. `history` is a snapshot as of this
     *  moment, which may be days/weeks before "now". */
    updated_at: string;
}
