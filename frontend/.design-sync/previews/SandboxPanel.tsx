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
        total: 0.5127,
        categories: [
            { key: "base", label: "Base development", weight: 0.3, score: 0.72, parts: [{ label: "Buildings", current: 26, max: 38 }, { label: "Blueprints", current: 41, max: 57 }] },
            { key: "exploration", label: "Map exploration", weight: 0.25, score: 0.58, parts: [{ label: "Nodes revealed", current: 118, max: 204 }, { label: "Rift gates", current: 4, max: 9 }] },
            { key: "collection", label: "Collection", weight: 0.25, score: 0.41, parts: [{ label: "Items", current: 92, max: 224 }, { label: "Recipes", current: 63, max: 148 }] },
            { key: "achievement", label: "Achievements", weight: 0.2, score: 0.26, parts: [{ label: "Trophies", current: 17, max: 65 }] },
        ],
    },
};

const untouched = {
    sandbox: {
        total: 0,
        categories: [
            { key: "base", label: "Base development", weight: 0.3, score: 0, parts: [{ label: "Buildings", current: 0, max: 38 }] },
            { key: "exploration", label: "Map exploration", weight: 0.25, score: 0, parts: [{ label: "Nodes revealed", current: 0, max: 204 }] },
            { key: "collection", label: "Collection", weight: 0.25, score: 0, parts: [{ label: "Items", current: 0, max: 224 }] },
            { key: "achievement", label: "Achievements", weight: 0.2, score: 0, parts: [{ label: "Trophies", current: 0, max: 65 }] },
        ],
    },
};

export const ReclamationProgress = () => (
    <PanelFrame pct="51.3%">
        <SandboxPanel improvements={inProgress} accent={SANDBOX_ACCENT} />
    </PanelFrame>
);

// Every part reads zero, so the panel leads with the "start RA" hint above the
// still-rendered category grid.
export const NoProgressDetected = () => (
    <PanelFrame pct="0.0%">
        <SandboxPanel improvements={untouched} accent={SANDBOX_ACCENT} />
    </PanelFrame>
);
