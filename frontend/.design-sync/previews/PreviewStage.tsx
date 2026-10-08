import { AdjustBar, PreviewStage } from "frontend";
import { useState } from "react";

// PreviewStage is the background editor's live preview: the real profile
// header rendered at a chosen viewport width (Desktop = the page's width,
// Phone = 390px) inside an isolated frame, scaled to fit the stage and capped
// at `maxHeight`. The author drags it to move the crop and scrolls or pinches
// to zoom; a drag along an axis with no slack reports it so the AdjustBar can
// say why nothing moved. The editor stacks it over the AdjustBar on a muted
// band, as composed here. Profile fixture as in the ProfileHero preview.

type Bg = { kind: "operator" | "skin" | "archive_pic" | "story_cg" | "story_scene"; id: string; focus_x?: number; focus_y?: number; scale?: number; elite?: number };

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

function Editor({ initial, viewportWidth, maxHeight = 260 }: { initial: Bg | null; viewportWidth: number; maxHeight?: number }) {
    const [draft, setDraft] = useState<Bg | null>(initial);
    const [deadAxis, setDeadAxis] = useState<"x" | "y" | null>(null);
    return (
        <section style={{ width: 860 }} className="flex flex-col gap-3 bg-muted/40 px-6 pt-5 pb-3">
            <PreviewStage profile={base} background={draft as never} viewportWidth={viewportWidth} maxHeight={maxHeight} onChange={(next) => setDraft(next as Bg)} onDeadAxis={setDeadAxis} />
            <AdjustBar draft={draft as never} onChange={(next) => setDraft(next as Bg)} onRemove={() => setDraft(null)} deadAxis={deadAxis} disabled={false} />
        </section>
    );
}

/** Desktop: the header at a 1280px page width, Angelina's art behind it. */
export const DesktopOperator = () => <Editor initial={{ kind: "operator", id: "char_291_aglina" }} viewportWidth={1280} />;

/** Desktop with a zoomed story CG crop. */
export const DesktopStoryCgZoomed = () => <Editor initial={{ kind: "story_cg", id: "47_i01", scale: 150, focus_x: 60, focus_y: 40 }} viewportWidth={1280} />;

/** Phone: the same header laid out at 390px, art filling from the top. */
export const PhoneOutfit = () => <Editor initial={{ kind: "skin", id: "char_1012_skadi2@iteration#2" }} viewportWidth={390} maxHeight={420} />;

/** No background: the header's plain look. */
export const NoBackground = () => <Editor initial={null} viewportWidth={1280} />;
