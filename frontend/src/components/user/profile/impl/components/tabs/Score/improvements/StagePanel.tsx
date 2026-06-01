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
    // Annihilation lives in both pools after the rotation-scoring split (the
    // three permanent maps stay permanent; rotating maps moved to the event
    // pool so recency decay applies). Gather them into one coherent section
    // rather than scattering them across the Permanent and Event buckets.
    const annihilation = [...permanent.missing, ...permanent.not_three_starred, ...event.missing, ...event.not_three_starred].filter(isAnnihilation);
    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            <StageBucket title="Permanent" pool={permanent} accent={accent} />
            <StageBucket title="Event" pool={event} accent={accent} />
            <AnnihilationSection stages={annihilation} accent={accent} />
        </div>
    );
}

function StageBucket({ title, pool, accent }: { title: string; pool: IStagePoolImprovements; accent: string }) {
    // Annihilation maps are surfaced in their own section (they span both pools
    // and have rotation rules), so keep them out of the regular stage lists.
    const regularMissing = pool.missing.filter((s) => !isAnnihilation(s));
    const regularNotThree = pool.not_three_starred.filter((s) => !isAnnihilation(s));

    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title={title} count={`${pool.three_starred} / ${pool.total} 3★`} accent={accent} />
            <div className="grid gap-2.5 sm:grid-cols-3">
                <ProgressLine label="Cleared" current={pool.cleared} max={pool.total} accent={accent} />
                <ProgressLine label="3-starred" current={pool.three_starred} max={pool.total} accent={accent} />
                <ProgressLine label="Not 3-starred" current={pool.cleared - pool.three_starred} max={pool.total} accent={accent} />
            </div>

            <StageList title="Missing" stages={regularMissing} accent={accent} emptyLabel="All stages cleared." />
            <StageList title="Cleared, not 3★" subtitle="Run again with a sharper squad to hit 3-star." stages={regularNotThree} accent={accent} emptyLabel="Every clear is 3★." />
        </div>
    );
}

function AnnihilationSection({ stages, accent }: { stages: IStageGap[]; accent: string }) {
    if (stages.length === 0) return null;
    // Permanent maps carry no rotation window; the active rotation is "active".
    // Anything past/future has rotated out and can't be maxed right now.
    const available = stages.filter((s) => !s.rotation || s.rotation.status === "active");
    const locked = stages.filter((s) => s.rotation && s.rotation.status !== "active");

    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title="Annihilation" count={`${available.length} to do now`} accent={accent} />
            {available.length > 0 ? (
                <StageList title="Available now" subtitle="The three permanent maps and the current weekly rotation. State 2 means cleared but enemy-kill count is below the cap; reaching the cap unlocks the full Orundum reward." stages={available} accent={accent} emptyLabel="" />
            ) : (
                <EmptyHint>Every currently-playable Annihilation map is maxed.</EmptyHint>
            )}
            {locked.length > 0 && (
                <StageList title="Not in rotation" subtitle="Rotating maps that aren't playable right now, so they can't be maxed until their rotation returns. These are scored on a recency curve rather than as permanent gaps - shown for reference." stages={locked} accent={accent} emptyLabel="" locked />
            )}
        </div>
    );
}

function StageList({ title, subtitle, stages, accent, emptyLabel, locked = false }: { title: string; subtitle?: string; stages: IStageGap[]; accent: string; emptyLabel: string; locked?: boolean }) {
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
                    <StageChip key={s.stage_id} stage={s} accent={accent} locked={locked} />
                ))}
            </div>
            {remaining > 0 && <ShowMoreButton onClick={() => setShowAll(true)} label={`Show ${remaining} more`} />}
            {showAll && stages.length > INITIAL_VISIBLE && <ShowMoreButton onClick={() => setShowAll(false)} label="Show less" />}
        </div>
    );
}

function StageChip({ stage, accent, locked = false }: { stage: IStageGap; accent: string; locked?: boolean }) {
    // Dot reflects clear state: rose = uncleared (state < 2), amber = cleared
    // but not 3★ (state 2). Locked (rotated-out) maps are muted regardless.
    const stateDot = locked ? "bg-muted-foreground/40" : stage.state >= 2 ? "bg-amber-500/75" : "bg-rose-500/65";
    const tooltipLabel = stage.name ? ` - ${stage.name}` : "";
    // Annihilation maps share a duplicated nation Code (e.g. several maps read
    // "Yan"), so the distinct map Name is the identifier worth leading with.
    // Regular stages keep the code (e.g. "1-7") as the primary label.
    const leadWithName = isAnnihilation(stage) && !!stage.name;
    const primary = leadWithName ? stage.name : stage.code;
    const secondary = leadWithName ? stage.code : stage.name;
    return (
        <span className={cn("group flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/15 px-2 py-1 transition-colors hover:border-border/70 hover:bg-muted/30", locked && "opacity-55")} title={`${stage.code}${tooltipLabel}${locked ? " - rotated out, not currently playable" : ""}`}>
            <span className={`size-1.5 shrink-0 rounded-full ${stateDot}`} aria-hidden />
            <span className={cn(TEXT_BADGE, "max-w-[22ch] truncate font-semibold")} style={{ color: `color-mix(in oklch, ${accent} 55%, var(--foreground))` }}>
                {primary}
            </span>
            {secondary && <span className="hidden max-w-[16ch] truncate text-[11px] text-muted-foreground sm:inline">{secondary}</span>}
            {stage.weight !== 1 && <Pill color={accent}>×{stage.weight.toFixed(2)}</Pill>}
        </span>
    );
}
