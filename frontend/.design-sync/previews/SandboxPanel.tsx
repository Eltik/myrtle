import { SandboxPanel } from "frontend";
import type { ReactNode } from "react";

const SANDBOX_ACCENT = "oklch(0.70 0.14 200)";

/** The expanded Sandbox subscore card the panel always lives inside. */
const PanelFrame = ({ pct, children }: { pct: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl">
        <div className="flex items-center justify-between p-4 sm:p-5">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Sandbox · RA progress &amp; nodes</span>
            <span className="font-bold text-2xl tabular-nums" style={{ color: SANDBOX_ACCENT }}>
                {pct}
            </span>
        </div>
        <div className="border-border/30 border-t">{children}</div>
    </div>
);

const inProgress = {
    sandbox: {
        total: 0.5128,
        topics: [
            {
                topic_id: "sandbox_1",
                name: "Tales Within the Sand",
                weight: 0.5,
                total: 0.762,
                categories: [
                    { key: "achievements", label: "Achievements", weight: 0.3, score: 0.563, parts: [{ label: "Completed", current: 40, max: 71 }] },
                    { key: "exploration", label: "Exploration", weight: 0.2, score: 0.771, parts: [{ label: "Map nodes", current: 90, max: 127 }, { label: "Zones", current: 5, max: 6 }] },
                    { key: "tech", label: "Tech unlocks", weight: 0.15, score: 0.926, parts: [{ label: "Unlocked", current: 50, max: 54 }] },
                    { key: "quests", label: "Story acts", weight: 0.15, score: 1, parts: [{ label: "Completed", current: 7, max: 7 }] },
                    { key: "base", label: "Base building", weight: 0.1, score: 0.844, parts: [{ label: "Base level", current: 6, max: 8 }, { label: "Blueprints", current: 90, max: 96 }] },
                    { key: "content", label: "Content depth", weight: 0.1, score: 0.656, parts: [{ label: "Recipes", current: 30, max: 53 }, { label: "Music", current: 8, max: 10 }, { label: "Rifts cleared", current: 3, max: 5 }] },
                ],
            },
            {
                topic_id: "sandbox_2",
                name: "Relaunch Anchor",
                weight: 0.5,
                total: 0.2635,
                categories: [
                    { key: "achievements", label: "Achievements", weight: 0.3, score: 0.169, parts: [{ label: "Completed", current: 10, max: 59 }] },
                    { key: "exploration", label: "Exploration", weight: 0.2, score: 0.459, parts: [{ label: "Map nodes", current: 12, max: 29 }, { label: "Zones", current: 2, max: 5 }] },
                    { key: "tech", label: "Tech unlocks", weight: 0.15, score: 0.211, parts: [{ label: "Unlocked", current: 12, max: 57 }] },
                    { key: "quests", label: "Story acts", weight: 0.15, score: 0.286, parts: [{ label: "Completed", current: 2, max: 7 }] },
                    { key: "base", label: "Base building", weight: 0.1, score: 0.361, parts: [{ label: "Base level", current: 5, max: 12 }, { label: "Band levels", current: 5, max: 18 }] },
                    { key: "content", label: "Content depth", weight: 0.1, score: 0.317, parts: [{ label: "Recipes", current: 6, max: 21 }, { label: "Music", current: 3, max: 9 }] },
                ],
            },
        ],
    },
};

const untouched = {
    sandbox: {
        total: 0,
        topics: [
            {
                topic_id: "sandbox_1",
                name: "Tales Within the Sand",
                weight: 0.5,
                total: 0,
                categories: [
                    { key: "achievements", label: "Achievements", weight: 0.3, score: 0, parts: [{ label: "Completed", current: 0, max: 71 }] },
                    { key: "exploration", label: "Exploration", weight: 0.2, score: 0, parts: [{ label: "Map nodes", current: 0, max: 127 }, { label: "Zones", current: 0, max: 6 }] },
                    { key: "tech", label: "Tech unlocks", weight: 0.15, score: 0, parts: [{ label: "Unlocked", current: 0, max: 54 }] },
                    { key: "quests", label: "Story acts", weight: 0.15, score: 0, parts: [{ label: "Completed", current: 0, max: 7 }] },
                    { key: "base", label: "Base building", weight: 0.1, score: 0, parts: [{ label: "Base level", current: 0, max: 8 }, { label: "Blueprints", current: 0, max: 96 }] },
                    { key: "content", label: "Content depth", weight: 0.1, score: 0, parts: [{ label: "Recipes", current: 0, max: 53 }, { label: "Music", current: 0, max: 10 }, { label: "Rifts cleared", current: 0, max: 5 }] },
                ],
            },
            {
                topic_id: "sandbox_2",
                name: "Relaunch Anchor",
                weight: 0.5,
                total: 0,
                categories: [
                    { key: "achievements", label: "Achievements", weight: 0.3, score: 0, parts: [{ label: "Completed", current: 0, max: 59 }] },
                    { key: "exploration", label: "Exploration", weight: 0.2, score: 0, parts: [{ label: "Map nodes", current: 0, max: 29 }, { label: "Zones", current: 0, max: 5 }] },
                    { key: "tech", label: "Tech unlocks", weight: 0.15, score: 0, parts: [{ label: "Unlocked", current: 0, max: 57 }] },
                    { key: "quests", label: "Story acts", weight: 0.15, score: 0, parts: [{ label: "Completed", current: 0, max: 7 }] },
                    { key: "base", label: "Base building", weight: 0.1, score: 0, parts: [{ label: "Base level", current: 0, max: 12 }, { label: "Band levels", current: 0, max: 18 }] },
                    { key: "content", label: "Content depth", weight: 0.1, score: 0, parts: [{ label: "Recipes", current: 0, max: 21 }, { label: "Music", current: 0, max: 9 }] },
                ],
            },
        ],
    },
};

export const ReclamationProgress = () => (
    <PanelFrame pct="51.3%">
        <SandboxPanel improvements={inProgress} accent={SANDBOX_ACCENT} />
    </PanelFrame>
);

// Every part reads zero, so the panel leads with the "play RA" hint above the
// still-rendered topic grids.
export const NoProgressDetected = () => (
    <PanelFrame pct="0.0%">
        <SandboxPanel improvements={untouched} accent={SANDBOX_ACCENT} />
    </PanelFrame>
);
