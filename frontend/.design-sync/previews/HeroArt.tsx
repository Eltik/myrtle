import { ProfileHero } from "frontend";

// HeroArt is the owner-chosen art behind the profile header (profile_layout.background):
// an operator's elite art, an outfit, or a gallery picture (Archives, story CG or
// scene), under a scrim mixed from the card colour so the header text keeps its
// contrast; from sm up the art sits right and the scrim fills from the left. It
// has no frame of its own (it is absolutely positioned inside the header), so the
// honest composition is the header that mounts it: ProfileHero with a
// `background`. Profile fixture as in the ProfileHero preview.

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

/** Operator art: Angelina at elite 2 (the default for a 6★). */
export const OperatorElite2 = () => <ProfileHero profile={base} background={{ kind: "operator", id: "char_291_aglina" }} />;

/** An outfit: Skadi the Corrupting Heart, "Red Countess". */
export const Outfit = () => <ProfileHero profile={{ ...base, avatar_id: "char_1012_skadi2" }} background={{ kind: "skin", id: "char_1012_skadi2@iteration#2" }} />;

/** A story CG from Here A People Sows, zoomed to 140% with its crop moved up. */
export const StoryCgZoomed = () => <ProfileHero profile={base} background={{ kind: "story_cg", id: "47_i01", scale: 140, focus_x: 55, focus_y: 35 }} />;

/** An Archives picture from Near Light, with the owner's change-background control. */
export const ArchivePictureOwner = () => <ProfileHero profile={base} background={{ kind: "archive_pic", id: "act13side_pic_0" }} onChangeBackground={() => undefined} />;
