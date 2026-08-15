import { MedalPanel } from "frontend";
import type { ReactNode } from "react";

const MEDAL_ACCENT = "oklch(0.62 0.22 295)";

/** The expanded Medals subscore card the panel always lives inside. */
const PanelFrame = ({ pct, children }: { pct: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl">
        <div className="flex items-center justify-between p-4 sm:p-5">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Medals · Achievement collection</span>
            <span className="font-bold text-2xl tabular-nums" style={{ color: MEDAL_ACCENT }}>
                {pct}
            </span>
        </div>
        <div className="border-border/30 border-t">{children}</div>
    </div>
);

// The capture clock is pinned to 2024-05-15, so these windows read as 7d / 17d left.
const ENDS_SOON = 1716379200;
const ENDS_LATER = 1717243200;

const withGaps = {
    medals: {
        event_in_window_missing: [
            { medal_id: "medal_act22side_1", name: "Lone Trail · Full Marks", rarity: "T3", get_method: "3-star every stage in Lone Trail, including EX.", description: "", is_hidden: false, end_time: ENDS_SOON, owned_pct: 12 },
            { medal_id: "medal_act22side_2", name: "Lone Trail · Trailblazer", rarity: "T2", get_method: "Clear LT-EX-6 with the challenge condition active.", description: "", is_hidden: false, end_time: ENDS_LATER, owned_pct: 0.4 },
            { medal_id: "medal_act22side_3", name: "Lone Trail · Quiet Hours", rarity: "T2D5", get_method: "", description: "", is_hidden: true, end_time: ENDS_LATER, owned_pct: 31 },
        ],
        permanent_missing: [
            { medal_id: "medal_nt_1", name: "Sweeping Tempest", rarity: "T3", get_method: "3-star every Annihilation map currently in rotation.", description: "", is_hidden: false, end_time: null, owned_pct: 8 },
            { medal_id: "medal_nt_2", name: "Elite Trainer", rarity: "T2", get_method: "Raise ten operators to Mastery 3 on any skill.", description: "", is_hidden: false, end_time: null, owned_pct: 44 },
            { medal_id: "medal_nt_3", name: "Base Foreman", rarity: "T2", get_method: "Upgrade every facility in the base to its maximum level.", description: "", is_hidden: false, end_time: null, owned_pct: 27 },
            { medal_id: "medal_nt_4", name: "Peerless Marksman", rarity: "T1D5", get_method: "Clear H12-4 without deploying a Defender.", description: "", is_hidden: false, end_time: null, owned_pct: 63 },
        ],
        operator_locked: [{ medal_id: "medal_collab_1", name: "Hunter's Oath", rarity: "T3", get_method: "Deploy the collaboration operator in a ranked Annihilation run.", description: "", is_hidden: false, end_time: null, operator_lock: { operatorId: "char_1012_skadi2", operatorName: "Rathalos S Noir Corne", reason: "collab" }, owned_pct: 3 }],
        unobtainable_missing: [{ medal_id: "medal_act12side_1", name: "Come Catastrophes · Vigil", rarity: "T2", get_method: "Earned during the original event run.", description: "", is_hidden: false, end_time: 1704067200, owned_pct: 19 }],
    },
};

const allEarned = {
    medals: { permanent_missing: [], event_in_window_missing: [], operator_locked: [], unobtainable_missing: [] },
};

export const MissingMedals = () => (
    <PanelFrame pct="82.1%">
        <MedalPanel improvements={withGaps} accent={MEDAL_ACCENT} />
    </PanelFrame>
);

// Every reachable medal earned — all four buckets empty, so the panel collapses
// to a single congratulatory hint.
export const EverythingEarned = () => (
    <PanelFrame pct="100%">
        <MedalPanel improvements={allEarned} accent={MEDAL_ACCENT} />
    </PanelFrame>
);
