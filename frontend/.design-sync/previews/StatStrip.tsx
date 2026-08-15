import { StatStrip } from "frontend";

const NOW_S = Math.floor(Date.now() / 1000);

const base = {
    id: "18220561",
    uid: "18220561",
    nickname: "Eltik",
    nick_number: "1024",
    level: 114,
    avatar_id: "char_102_texas",
    secretary: "char_102_texas",
    secretary_skin_id: null,
    resume_id: null,
    role: "USER",
    server: "EN",
    total_score: 8422,
    grade: "A",
    public_profile: true,
    store_gacha: true,
    share_stats: true,
    exp: 51240,
    orundum: 128740,
    lmd: 4318220,
    sanity: 121,
    max_sanity: 135,
    gacha_tickets: 42,
    ten_pull_tickets: 6,
    monthly_sub_end: null,
    register_ts: 1579132800,
    last_online_ts: NOW_S - 3600,
    resume: null,
    friend_num_limit: 200,
    cumulative_signin: 1876,
    operator_count: 231,
    item_count: 418,
    skin_count: 96,
    non_default_skin_count: 47,
    updated_at: new Date(NOW_S * 1000).toISOString(),
};

const WHALE = { ...base, orundum: 1284300, operator_count: 312, item_count: 604, non_default_skin_count: 188 };

const FRESH = { ...base, orundum: 1200, operator_count: 24, item_count: 61, non_default_skin_count: 0 };

export const RosterSummary = () => <StatStrip profile={base} rosterCount={231} />;

export const CompletionistAccount = () => <StatStrip profile={WHALE} rosterCount={312} />;

export const NewAccount = () => <StatStrip profile={FRESH} rosterCount={24} />;
