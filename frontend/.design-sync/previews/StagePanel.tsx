import { StagePanel } from "frontend";
import type { ReactNode } from "react";

const STAGE_ACCENT = "oklch(0.62 0.20 255)";

/** The chrome the panel always renders inside: the expanded Stages subscore card. */
const PanelFrame = ({ pct, children }: { pct: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl">
        <div className="flex items-center justify-between p-4 sm:p-5">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Stages · Story &amp; event clears</span>
            <span className="font-bold text-2xl tabular-nums" style={{ color: STAGE_ACCENT }}>
                {pct}
            </span>
        </div>
        <div className="border-border/30 border-t">{children}</div>
    </div>
);

const stage = (code: string, name: string, state: number, zone_id = "main_12") => ({
    stage_id: `${zone_id}_${code.replace(/[^\w]/g, "_")}`,
    code,
    name,
    zone_id,
    weight: 1,
    state,
});

const anni = (code: string, name: string, state: number, zone_id: string, weight: number, rotation: { status: "active" | "past" | "future"; startTs: number; endTs: number } | null) => ({
    stage_id: `${zone_id}_${code.toLowerCase()}`,
    code,
    name,
    zone_id,
    weight,
    state,
    rotation,
});

const withGaps = {
    stages: {
        permanent: {
            total: 812,
            cleared: 764,
            three_starred: 701,
            missing: [stage("12-16", "Sundered Chord", 0), stage("12-17", "Ashen Vigil", 0), stage("12-18", "The Long Way Down", 0), stage("H12-1", "Unwelcome Guest", 0), stage("H11-4", "Cinder Path", 0, "main_11"), anni("Sami", "Sami Ruins", 0, "camp_zone_r5", 0.85, { status: "active", startTs: 1715126400, endTs: 1715731200 })],
            not_three_starred: [stage("11-14", "Frostbitten", 2, "main_11"), stage("12-9", "Torchlight", 2), stage("H10-4", "Sword Alone", 2, "main_10"), stage("S6-4", "Quicksand", 2, "main_06"), anni("Lungmen", "Lungmen Downtown", 2, "camp_zone_3", 1, null)],
        },
        event: {
            total: 246,
            cleared: 218,
            three_starred: 193,
            missing: [stage("LT-EX-6", "Lone Trail", 0, "act22side"), stage("GA-EX-8", "Guide Ahead", 0, "act17side"), stage("IW-EX-7", "Ideal City", 0, "act16side")],
            not_three_starred: [stage("WD-EX-8", "Where Vernal Winds Will Never Blow", 2, "act20side"), stage("CW-EX-5", "Come Catastrophes or Wakes of Vultures", 2, "act12side")],
        },
    },
};

const allClear = {
    stages: {
        permanent: { total: 812, cleared: 812, three_starred: 812, missing: [], not_three_starred: [] },
        event: { total: 246, cleared: 246, three_starred: 246, missing: [], not_three_starred: [] },
    },
};

export const StageGaps = () => (
    <PanelFrame pct="88.1%">
        <StagePanel improvements={withGaps} accent={STAGE_ACCENT} />
    </PanelFrame>
);

// Nothing left to clear: both pools fall back to their empty hints and the
// Annihilation section drops out entirely.
export const EverythingThreeStarred = () => (
    <PanelFrame pct="100%">
        <StagePanel improvements={allClear} accent={STAGE_ACCENT} />
    </PanelFrame>
);
