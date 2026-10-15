import { PreviewFrame, ProfileHero } from "frontend";

// PreviewFrame renders its children into a same-origin iframe `viewportWidth`
// px wide through a portal, mirroring the page's stylesheets into it, so the
// header's media queries see the previewed viewport (a 390px phone) rather
// than the editor's. It is decoration: no pointer events, hidden from
// assistive tech. PreviewStage scales it to fit; here it is shown unscaled at
// widths the card can hold. Profile fixture as in the ProfileHero preview.

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

/** A phone viewport: the header in its stacked layout with the art behind it. */
export const Phone = () => (
    <PreviewFrame title="Profile header at 390px" viewportWidth={390} height={430} onDocument={noop}>
        <div data-preview-root="" className="p-3">
            <ProfileHero profile={base} background={{ kind: "story_cg", id: "47_i01" }} />
        </div>
    </PreviewFrame>
);

/** A small-tablet viewport (720px): the side-by-side layout with the art on the right. */
export const Tablet = () => (
    <PreviewFrame title="Profile header at 720px" viewportWidth={720} height={330} onDocument={noop}>
        <div data-preview-root="" className="p-3">
            <ProfileHero profile={base} background={{ kind: "operator", id: "char_291_aglina" }} />
        </div>
    </PreviewFrame>
);
