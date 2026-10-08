import { ProfileLayoutEditor } from "frontend";

// ProfileLayoutEditor is the owner's "arrange your profile" editor: every tab
// in display order with a visibility switch and move controls, seeded from
// `profile_layout.tabs` (a profile without a layout gets the default order,
// all visible). Saving writes only the `tabs` key. Profile fixture as in the
// ProfileHero preview; tab labels are the profile page's.

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

const LABELS = { showcase: "Showcase", stats: "Stats", score: "Score", roster: "Roster", plans: "Plans", inventory: "Inventory", enemies: "Enemies", optimizer: "Optimizer" } as const;

const noop = () => undefined;

const ARRANGED = {
    ...base,
    profile_layout: {
        tabs: [
            { id: "showcase", visible: true },
            { id: "roster", visible: true },
            { id: "stats", visible: true },
            { id: "score", visible: true },
            { id: "plans", visible: false },
            { id: "inventory", visible: true },
            { id: "enemies", visible: false },
            { id: "optimizer", visible: true },
        ],
    },
};

/** A profile with no saved layout: the default order, every tab shown. */
export const DefaultLayout = () => (
    <div style={{ minHeight: 640 }} className="relative w-full">
        <ProfileLayoutEditor profile={base as never} labels={LABELS} onClose={noop} />
    </div>
);

/** An arranged profile: Showcase first, Plans and Enemies hidden from visitors. */
export const Arranged = () => (
    <div style={{ minHeight: 640 }} className="relative w-full">
        <ProfileLayoutEditor profile={ARRANGED as never} labels={LABELS} onClose={noop} />
    </div>
);
