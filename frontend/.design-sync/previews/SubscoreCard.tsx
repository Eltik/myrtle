import { SubscoreCard } from "frontend";
import { Crown, Hammer, Medal, Mountain, Sparkles, Swords } from "lucide-react";

// `ISubscore` rows, copied from the Score tab's SUBSCORES table — the `weight`
// values mirror the backend's SECTION_WEIGHT_* constants, so the "% of grade"
// pill on each card is the real share.
const OPERATOR = { key: "operator_score", label: "Operator", description: "Roster depth & investment", icon: Crown, color: "oklch(0.74 0.17 75)", weight: 0.85 };
const BASE = { key: "base_score", label: "Base", description: "Drone & facility upgrades", icon: Hammer, color: "oklch(0.70 0.16 145)", weight: 0.35 };
const STAGE = { key: "stage_score", label: "Stages", description: "Story & event clears", icon: Swords, color: "oklch(0.62 0.20 255)", weight: 0.6 };
const ROGUELIKE = { key: "roguelike_score", label: "Roguelike", description: "IS endings & relics", icon: Sparkles, color: "oklch(0.65 0.22 340)", weight: 0.3 };
const SANDBOX = { key: "sandbox_score", label: "Sandbox", description: "RA progress & nodes", icon: Mountain, color: "oklch(0.70 0.14 200)", weight: 0.2 };
const MEDAL = { key: "medal_score", label: "Medals", description: "Achievement collection", icon: Medal, color: "oklch(0.62 0.22 295)", weight: 0.2 };

// The card is a Collapsible; the improvements panel only mounts once a Doctor
// expands it, so a static card is always the collapsed summary face.
export const HeaviestSection = () => (
    <div className="mx-auto grid max-w-md grid-cols-1">
        <SubscoreCard sub={OPERATOR} score={0.9128} improvements={null} isImprovementsLoading={false} />
    </div>
);

export const NearlyMaxed = () => (
    <div className="mx-auto grid max-w-md grid-cols-1">
        <SubscoreCard sub={BASE} score={0.9407} improvements={null} isImprovementsLoading={false} />
    </div>
);

export const BarelyStarted = () => (
    <div className="mx-auto grid max-w-md grid-cols-1">
        <SubscoreCard sub={SANDBOX} score={0.0417} improvements={null} isImprovementsLoading={false} />
    </div>
);

// How the six cards read as the grid the Score tab actually lays out.
export const SubscoreGrid = () => (
    <div className="grid grid-cols-1 items-start gap-3 sm:grid-cols-2">
        <SubscoreCard sub={STAGE} score={0.8814} improvements={null} isImprovementsLoading={false} />
        <SubscoreCard sub={ROGUELIKE} score={0.7431} improvements={null} isImprovementsLoading={false} />
        <SubscoreCard sub={MEDAL} score={0.8209} improvements={null} isImprovementsLoading={false} />
        <SubscoreCard sub={SANDBOX} score={0.5127} improvements={null} isImprovementsLoading={false} />
    </div>
);
