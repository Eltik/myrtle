import { useState } from "react";
import type { IImprovementsResponse, IStageGap, IStagePoolImprovements } from "#/lib/api/user";
import { EmptyHint, PANEL_PADDING, Pill, ProgressLine, SectionHeader } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const INITIAL_VISIBLE = 12;

export function StagePanel({ improvements, accent }: IProps) {
    const { permanent, event } = improvements.stages;
    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            <StageBucket title="Permanent" pool={permanent} accent={accent} />
            <StageBucket title="Event" pool={event} accent={accent} />
        </div>
    );
}

function StageBucket({ title, pool, accent }: { title: string; pool: IStagePoolImprovements; accent: string }) {
    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title={title} count={`${pool.three_starred} / ${pool.total} 3★`} accent={accent} />
            <div className="grid gap-2.5 sm:grid-cols-3">
                <ProgressLine label="Cleared" current={pool.cleared} max={pool.total} accent={accent} />
                <ProgressLine label="3-starred" current={pool.three_starred} max={pool.total} accent={accent} />
                <ProgressLine label="Not 3-starred" current={pool.cleared - pool.three_starred} max={pool.total} accent={accent} />
            </div>

            <StageList title={pool.missing.length === 0 ? "Nothing missing" : "Missing"} stages={pool.missing} accent={accent} emptyLabel="All stages cleared." stateColor="rose" />
            <StageList title={pool.not_three_starred.length === 0 ? "All 3-starred" : "Cleared, not 3★"} stages={pool.not_three_starred} accent={accent} emptyLabel="All clears are 3★." stateColor="amber" />
        </div>
    );
}

function StageList({ title, stages, accent, emptyLabel, stateColor }: { title: string; stages: IStageGap[]; accent: string; emptyLabel: string; stateColor: "rose" | "amber" }) {
    const [showAll, setShowAll] = useState(false);
    if (stages.length === 0) {
        return (
            <div className="flex flex-col gap-1.5">
                <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">{title}</span>
                <EmptyHint>{emptyLabel}</EmptyHint>
            </div>
        );
    }

    const visible = showAll ? stages : stages.slice(0, INITIAL_VISIBLE);
    const remaining = stages.length - visible.length;

    return (
        <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between">
                <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">{title}</span>
                <span className="font-mono text-[10px] tabular-nums text-muted-foreground">{stages.length} total</span>
            </div>
            <div className="flex flex-wrap gap-1.5">
                {visible.map((s) => (
                    <StageChip key={s.stage_id} stage={s} accent={accent} stateColor={stateColor} />
                ))}
            </div>
            {remaining > 0 && (
                <button type="button" onClick={() => setShowAll(true)} className="self-start font-mono text-[10.5px] uppercase tracking-wider text-muted-foreground transition-colors hover:text-foreground">
                    Show {remaining} more
                </button>
            )}
            {showAll && stages.length > INITIAL_VISIBLE && (
                <button type="button" onClick={() => setShowAll(false)} className="self-start font-mono text-[10.5px] uppercase tracking-wider text-muted-foreground transition-colors hover:text-foreground">
                    Show less
                </button>
            )}
        </div>
    );
}

function StageChip({ stage, accent, stateColor }: { stage: IStageGap; accent: string; stateColor: "rose" | "amber" }) {
    const stateDot = stateColor === "rose" ? "bg-rose-500/65" : "bg-amber-500/75";
    const tooltipLabel = stage.name ? ` — ${stage.name}` : "";
    return (
        <span className="group flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/15 px-2 py-1 text-[11px] transition-colors hover:border-border/70 hover:bg-muted/30" title={`${stage.code}${tooltipLabel}`}>
            <span className={`size-1.5 shrink-0 rounded-full ${stateDot}`} aria-hidden />
            <span className="font-mono font-semibold tabular-nums" style={{ color: `color-mix(in oklch, ${accent} 55%, var(--foreground))` }}>
                {stage.code}
            </span>
            {stage.name && <span className="hidden max-w-[16ch] truncate text-muted-foreground sm:inline">{stage.name}</span>}
            {stage.weight !== 1 && <Pill color={accent}>×{stage.weight.toFixed(2)}</Pill>}
        </span>
    );
}
