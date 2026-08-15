import { ImprovementsPanel } from "frontend";
import { Medal, Mountain, Sparkles, Swords } from "lucide-react";
import type { ReactNode } from "react";

const ROGUELIKE = { key: "roguelike_score", label: "Roguelike", description: "IS endings & relics", icon: Sparkles, color: "oklch(0.65 0.22 340)", weight: 0.3 };
const SANDBOX = { key: "sandbox_score", label: "Sandbox", description: "RA progress & nodes", icon: Mountain, color: "oklch(0.70 0.14 200)", weight: 0.2 };
const STAGE = { key: "stage_score", label: "Stages", description: "Story & event clears", icon: Swords, color: "oklch(0.62 0.20 255)", weight: 0.6 };
const MEDAL = { key: "medal_score", label: "Medals", description: "Achievement collection", icon: Medal, color: "oklch(0.62 0.22 295)", weight: 0.2 };

/** The expanded subscore card the dispatcher always renders inside. */
const PanelFrame = ({ sub, pct, children }: { sub: { label: string; description: string; color: string }; pct: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl">
        <div className="flex items-center justify-between p-4 sm:p-5">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                {sub.label} · {sub.description}
            </span>
            <span className="font-bold text-2xl tabular-nums" style={{ color: sub.color }}>
                {pct}
            </span>
        </div>
        <div className="border-border/30 border-t">{children}</div>
    </div>
);

const improvements = {
    uid: "1000048871",
    roguelike: [
        {
            theme_id: "rogue_3",
            theme_name: "Mizuki & Caerula Arbor",
            endings: { current: 3, max: 5 },
            difficulty: { highest_cleared: 12, max: 15 },
            collectibles: { relics: { current: 214, max: 286 }, capsules: { current: 18, max: 24 }, bands: { current: 9, max: 12 } },
            bp: { current: 180, max: 200 },
            challenges: { current: 22, max: 30 },
        },
    ],
    sandbox: {
        total: 0.5127,
        categories: [
            { key: "base", label: "Base development", weight: 0.3, score: 0.72, parts: [{ label: "Buildings", current: 26, max: 38 }, { label: "Blueprints", current: 41, max: 57 }] },
            { key: "exploration", label: "Map exploration", weight: 0.25, score: 0.58, parts: [{ label: "Nodes revealed", current: 118, max: 204 }] },
            { key: "collection", label: "Collection", weight: 0.25, score: 0.41, parts: [{ label: "Items", current: 92, max: 224 }] },
            { key: "achievement", label: "Achievements", weight: 0.2, score: 0.26, parts: [{ label: "Trophies", current: 17, max: 65 }] },
        ],
    },
};

// A Doctor expanded the Roguelike card: the dispatcher picks RoguelikePanel.
export const RoguelikeBreakdown = () => (
    <PanelFrame sub={ROGUELIKE} pct="74.3%">
        <ImprovementsPanel sub={ROGUELIKE} improvements={improvements} isLoading={false} />
    </PanelFrame>
);

// Same dispatcher, different card — Sandbox routes to SandboxPanel.
export const SandboxBreakdown = () => (
    <PanelFrame sub={SANDBOX} pct="51.3%">
        <ImprovementsPanel sub={SANDBOX} improvements={improvements} isLoading={false} />
    </PanelFrame>
);

// The improvements query is shared across all six cards, so the first expand
// pays the wait and every card shows this skeleton.
export const Loading = () => (
    <PanelFrame sub={STAGE} pct="88.1%">
        <ImprovementsPanel sub={STAGE} improvements={null} isLoading={true} />
    </PanelFrame>
);

// Private or never-synced profile: no improvements payload to dispatch on.
export const Unavailable = () => (
    <PanelFrame sub={MEDAL} pct="82.1%">
        <ImprovementsPanel sub={MEDAL} improvements={null} isLoading={false} />
    </PanelFrame>
);
