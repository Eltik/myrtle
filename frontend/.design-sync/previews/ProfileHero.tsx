import { ProfileHero } from "frontend";

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
    resume: "Chernobog veteran. Currently farming 5-10 for Orirock Cubes and pretending IS#5 is going fine.",
    friend_num_limit: 200,
    // Sign-in days must stay under the elapsed game days since `register_ts`,
    // or the hero renders "1,876/1,583 days".
    cumulative_signin: 1421,
    operator_count: 231,
    item_count: 418,
    skin_count: 96,
    non_default_skin_count: 47,
    updated_at: new Date(NOW_S * 1000).toISOString(),
};

const MAXED = {
    ...base,
    id: "10000817",
    uid: "10000817",
    nickname: "Kyostinv",
    nick_number: null,
    level: 120,
    exp: 0,
    avatar_id: "char_4064_mlynar",
    server: "EN",
    grade: "SS+",
    resume: "Max level, 100% handbook, still missing one Bipolar Nanoflake.",
    cumulative_signin: 1902,
    register_ts: 1547596800,
};

const FRESH = {
    ...base,
    id: "70115552",
    uid: "70115552",
    nickname: "Amiya Enjoyer",
    nick_number: null,
    level: 31,
    exp: 4180,
    avatar_id: "char_002_amiya",
    server: "JP",
    grade: "C",
    resume: null,
    cumulative_signin: 42,
    register_ts: NOW_S - 46 * 24 * 60 * 60,
};

export const DoctorOverview = () => <ProfileHero profile={base} />;

export const MaxLevelVeteran = () => <ProfileHero profile={MAXED} />;

export const NewDoctor = () => <ProfileHero profile={FRESH} />;
