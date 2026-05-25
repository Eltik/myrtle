import { useState } from "react";
import type { IImprovementsResponse, IStageGap, IStagePoolImprovements } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { EmptyHint, PANEL_PADDING, Pill, ProgressLine, SectionHeader, ShowMoreButton, TEXT_BADGE, TEXT_KICKER, TEXT_META } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const INITIAL_VISIBLE = 12;

function isAnnihilation(stage: IStageGap): boolean {
    return stage.zone_id.startsWith("camp_zone");
}

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
    const annihilationGaps = pool.not_three_starred.filter(isAnnihilation);
    const regularGaps = pool.not_three_starred.filter((s) => !isAnnihilation(s));

    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title={title} count={`${pool.three_starred} / ${pool.total} 3★`} accent={accent} />
            <div className="grid gap-2.5 sm:grid-cols-3">
                <ProgressLine label="Cleared" current={pool.cleared} max={pool.total} accent={accent} />
                <ProgressLine label="3-starred" current={pool.three_starred} max={pool.total} accent={accent} />
                <ProgressLine label="Not 3-starred" current={pool.cleared - pool.three_starred} max={pool.total} accent={accent} />
            </div>

            <StageList title="Missing" stages={pool.missing} accent={accent} emptyLabel="All stages cleared." stateColor="rose" />
            <StageList
                title="Cleared, not 3★"
                subtitle="Run again with a sharper squad to hit 3-star."
                stages={regularGaps}
                accent={accent}
                emptyLabel={annihilationGaps.length > 0 ? `Every regular clear is 3★. ${annihilationGaps.length} Annihilation map${annihilationGaps.length === 1 ? "" : "s"} below still need${annihilationGaps.length === 1 ? "s" : ""} maxing.` : "Every clear is 3★."}
                stateColor="amber"
            />
            {annihilationGaps.length > 0 && <StageList title="Annihilation - not maxed" subtitle="State 2 means cleared but enemy-kill count below the cap. Higher kill counts unlock the Orundum reward tier." stages={annihilationGaps} accent={accent} emptyLabel="All Annihilation maps are maxed." stateColor="amber" />}
        </div>
    );
}

function StageList({ title, subtitle, stages, accent, emptyLabel, stateColor }: { title: string; subtitle?: string; stages: IStageGap[]; accent: string; emptyLabel: string; stateColor: "rose" | "amber" }) {
    const [showAll, setShowAll] = useState(false);
    if (stages.length === 0) {
        return (
            <div className="flex flex-col gap-1.5">
                <span className={cn(TEXT_KICKER, "text-muted-foreground")}>{title}</span>
                <EmptyHint>{emptyLabel}</EmptyHint>
            </div>
        );
    }

    const visible = showAll ? stages : stages.slice(0, INITIAL_VISIBLE);
    const remaining = stages.length - visible.length;

    return (
        <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between">
                <span className={cn(TEXT_KICKER, "text-muted-foreground")}>{title}</span>
                <span className={cn(TEXT_BADGE, "text-muted-foreground")}>{stages.length} total</span>
            </div>
            {subtitle && <p className={cn(TEXT_META, "-mt-1 text-muted-foreground/80")}>{subtitle}</p>}
            <div className="flex flex-wrap gap-1.5">
                {visible.map((s) => (
                    <StageChip key={s.stage_id} stage={s} accent={accent} stateColor={stateColor} />
                ))}
            </div>
            {remaining > 0 && <ShowMoreButton onClick={() => setShowAll(true)} label={`Show ${remaining} more`} />}
            {showAll && stages.length > INITIAL_VISIBLE && <ShowMoreButton onClick={() => setShowAll(false)} label="Show less" />}
        </div>
    );
}

function StageChip({ stage, accent, stateColor }: { stage: IStageGap; accent: string; stateColor: "rose" | "amber" }) {
    const stateDot = stateColor === "rose" ? "bg-rose-500/65" : "bg-amber-500/75";
    const tooltipLabel = stage.name ? ` - ${stage.name}` : "";
    return (
        <span className="group flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/15 px-2 py-1 transition-colors hover:border-border/70 hover:bg-muted/30" title={`${stage.code}${tooltipLabel}`}>
            <span className={`size-1.5 shrink-0 rounded-full ${stateDot}`} aria-hidden />
            <span className={cn(TEXT_BADGE, "font-semibold")} style={{ color: `color-mix(in oklch, ${accent} 55%, var(--foreground))` }}>
                {stage.code}
            </span>
            {stage.name && <span className="hidden max-w-[16ch] truncate text-[11px] text-muted-foreground sm:inline">{stage.name}</span>}
            {stage.weight !== 1 && <Pill color={accent}>×{stage.weight.toFixed(2)}</Pill>}
        </span>
    );
}
