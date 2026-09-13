import { ImprovementsBasePanel } from "frontend";
import type { ReactNode } from "react";

// The Base subscore's inline panel on the Score tab. Since the 2026-09 rewrite it is a
// pure readout of the grade's two stored components (`base_utilization` weighted 75%,
// `base_infrastructure` 25%) read off `IUserScore`; the room-by-room plan moved to the
// Optimizer tab (`OptimizerBasePanel`).

const BASE_ACCENT = "oklch(0.70 0.16 145)";

/** The expanded Base subscore card the panel always lives inside. */
const PanelFrame = ({ pct, children }: { pct: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl">
        <div className="flex items-center justify-between p-4 sm:p-5">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Base · Drone &amp; facility upgrades</span>
            <span className="font-bold text-2xl tabular-nums" style={{ color: BASE_ACCENT }}>
                {pct}
            </span>
        </div>
        <div className="border-border/30 border-t">{children}</div>
    </div>
);

const score = (base_utilization: number | null, base_infrastructure: number | null, base_score: number) => ({
    user_id: "3b7c1f2e-6a1d-4c58-9d0e-2f8a7b4c1d90",
    total_score: 1842.5,
    operator_score: 912.3,
    stage_score: 401.8,
    roguelike_score: 188.0,
    sandbox_score: 64.2,
    medal_score: 121.4,
    base_score,
    base_utilization,
    base_infrastructure,
    skin_score: 154.8,
    grade: "A",
    calculated_at: "2024-05-14T21:10:00Z",
});

/** Both components stored: a well-staffed base with a couple of rooms still unbuilt. */
export const ScoreBreakdown = () => (
    <PanelFrame pct="94.1%">
        <ImprovementsBasePanel score={score(0.962, 0.875, 94.1)} accent={BASE_ACCENT} />
    </PanelFrame>
);

/** Utilization is the drag: most rooms built, but half the seats sit empty. */
export const UnderStaffed = () => (
    <PanelFrame pct="61.3%">
        <ImprovementsBasePanel score={score(0.52, 0.94, 61.3)} accent={BASE_ACCENT} />
    </PanelFrame>
);

/** A score computed before the components were stored: the panel degrades to one hint. */
export const BeforeRefresh = () => (
    <PanelFrame pct="0.0%">
        <ImprovementsBasePanel score={score(null, null, 0)} accent={BASE_ACCENT} />
    </PanelFrame>
);
