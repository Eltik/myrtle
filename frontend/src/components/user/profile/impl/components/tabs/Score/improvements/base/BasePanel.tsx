import type { CSSProperties } from "react";
import type { IUserScore } from "#/lib/api/user";
import { Bar } from "../../../Stats/primitives";
import { EmptyHint, PANEL_PADDING, SectionHeader } from "../shared";

// Mirrors the backend's UTILIZATION_WEIGHT / INFRASTRUCTURE_WEIGHT constants
// in `backend/src/core/grade/base/score.rs` - change both together.
const COMPONENTS = [
    {
        key: "base_utilization",
        label: "Stationing",
        weightPct: 75,
        description: "Sustained daily yield of your base as stationed, against the optimizer's best staffing of your own roster on the same rooms.",
    },
    {
        key: "base_infrastructure",
        label: "Upgrades",
        weightPct: 25,
        description: "What your rooms can achieve as built, against the same rooms at max level.",
    },
] as const;

interface IProps {
    score: IUserScore | null | undefined;
    accent: string;
}

/**
 * The Base subscore's inline panel: the grade's two stored components -
 * stationing utilization (75%) and infrastructure completeness (25%). The
 * full plan, layout and rotation tooling live in the Optimizer tab, so this
 * panel stays a pure "which term drags my score" readout.
 */
export function BasePanel({ score, accent }: IProps) {
    const util = score?.base_utilization ?? null;
    const infra = score?.base_infrastructure ?? null;
    if (util === null || infra === null) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>Component breakdown appears after the next score refresh.</EmptyHint>
            </div>
        );
    }

    const values: Record<(typeof COMPONENTS)[number]["key"], number> = {
        base_utilization: util,
        base_infrastructure: infra,
    };

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-4`} style={{ "--imp-accent": accent } as CSSProperties}>
            <SectionHeader title="Score breakdown" accent={accent} />
            <div className="flex flex-col gap-3.5">
                {COMPONENTS.map((c) => {
                    const pct = Math.min(Math.max(values[c.key] * 100, 0), 100);
                    return (
                        <div key={c.key} className="flex flex-col gap-1.5">
                            <div className="flex items-baseline justify-between gap-2">
                                <span className="font-medium text-[12px] text-foreground/90">
                                    {c.label}
                                    <span className="ml-1.5 font-mono text-[9.5px] text-muted-foreground/70 uppercase tracking-wider">{c.weightPct}% of this score</span>
                                </span>
                                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{pct.toFixed(1)}%</span>
                            </div>
                            <Bar color={accent} pct={pct} />
                            <p className="text-[10.5px] text-muted-foreground/80 leading-snug">{c.description}</p>
                        </div>
                    );
                })}
            </div>
            <p className="rounded-md border border-border/40 border-dashed bg-muted/15 px-3 py-2 text-[10.5px] text-muted-foreground">Plan restaffing and see the full room-by-room comparison in the Optimizer tab.</p>
        </div>
    );
}
