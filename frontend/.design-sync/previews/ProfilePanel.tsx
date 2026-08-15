import { ProfilePanel } from "frontend";

const DOCTOR = {
    id: "9f2b0c74-3c8e-4a11-9d21-7c6b0f5a4e18",
    uid: "10345678",
    nickname: "Eltik",
    nick_number: "1734",
    level: 120,
    avatar_id: null,
    secretary: "char_4064_mlynar",
    secretary_skin_id: null,
    resume_id: null,
    role: "user",
    server: "en",
    total_score: 8734,
    grade: "A",
    public_profile: true,
    store_gacha: true,
    share_stats: true,
    exp: 12045,
    orundum: 18420,
    lmd: 4218993,
    sanity: 132,
    max_sanity: 135,
    gacha_tickets: 21,
    ten_pull_tickets: 3,
    monthly_sub_end: 1718409600,
    register_ts: 1584230400,
    last_online_ts: 1715766000,
    resume: "Rhodes Island, Reserve Op A4",
    friend_num_limit: 200,
    cumulative_signin: 1284,
    operator_count: 231,
    item_count: 1873,
    skin_count: 96,
    non_default_skin_count: 41,
    updated_at: "2024-05-15T09:12:00Z",
};

const noop = () => {};

export const Default = () => <ProfilePanel user={DOCTOR} onResync={noop} syncing={false} />;

export const Resyncing = () => <ProfilePanel user={DOCTOR} onResync={noop} syncing={true} />;

/** A doctor who has never set an assistant in-game: the avatar slot falls back to the person glyph. */
export const NoAssistantSet = () => <ProfilePanel user={{ ...DOCTOR, nickname: "Kaltsit", nick_number: null, secretary: null, secretary_skin_id: null, level: 87, server: "jp" }} onResync={noop} syncing={false} />;
