import { BackgroundEditor } from "frontend";

// BackgroundEditor is the owner's full-screen "Header background" editor: a
// top bar (title, Desktop/Phone preview toggle, Cancel, Save), the live
// preview of the profile header over the AdjustBar on a muted band, and the
// ArtBrowser under it; scrolled past the preview, a compact strip keeps the
// preview and zoom in view. Opened from the owner's header control. Its art
// catalogues load through server functions the design bundle stubs to fail,
// so the browser half shows its load-failure state. Profile fixture as in
// the ProfileHero preview.

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

const noop = () => undefined;

/** Editing a saved operator background. */
export const OperatorSaved = () => (
    <div style={{ minHeight: 700 }} className="relative w-full">
        <BackgroundEditor profile={base} saved={{ kind: "operator", id: "char_291_aglina" }} onClose={noop} />
    </div>
);

/** A story CG saved with a zoomed crop. */
export const StoryCgSaved = () => (
    <div style={{ minHeight: 700 }} className="relative w-full">
        <BackgroundEditor profile={base} saved={{ kind: "story_cg", id: "47_i01", scale: 140, focus_x: 58, focus_y: 38 }} onClose={noop} />
    </div>
);

/** No background yet: the preview shows the plain header. */
export const NoneSaved = () => (
    <div style={{ minHeight: 700 }} className="relative w-full">
        <BackgroundEditor profile={base} saved={null} onClose={noop} />
    </div>
);
