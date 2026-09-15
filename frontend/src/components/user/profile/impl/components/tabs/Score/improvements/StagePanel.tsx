import { useState } from "react";
import type { IImprovementsResponse, IStageGap, IStagePoolImprovements } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./StagePanel.messages";
import { EmptyHint, PANEL_PADDING, Pill, ProgressLine, SectionHeader, ShowMoreButton, TEXT_BADGE, TEXT_KICKER, TEXT_META } from "./shared";
import type { messages as sharedMessages } from "./shared.messages";

/** This panel also renders the show-more chrome declared in `shared.messages.ts`. */
type PanelT = TypedT<typeof messages & typeof sharedMessages>;

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const INITIAL_VISIBLE = 12;

function isAnnihilation(stage: IStageGap): boolean {
    return stage.zone_id.startsWith("camp_zone");
}

export function StagePanel({ improvements, accent }: IProps) {
    const t: TypedT<typeof messages> = useT("user");
    const { permanent, event } = improvements.stages;
    // Annihilation lives in both pools after the rotation-scoring split (the
    // three permanent maps stay permanent; rotating maps moved to the event
    // pool so recency decay applies). Gather them into one coherent section
    // rather than scattering them across the Permanent and Event buckets.
    const annihilation = [...permanent.missing, ...permanent.not_three_starred, ...event.missing, ...event.not_three_starred].filter(isAnnihilation);
    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            <StageBucket title={t("score.improvements.stage.permanent")} pool={permanent} accent={accent} />
            <StageBucket title={t("score.improvements.stage.event")} pool={event} accent={accent} />
            <AnnihilationSection stages={annihilation} accent={accent} />
        </div>
    );
}

function StageBucket({ title, pool, accent }: { title: string; pool: IStagePoolImprovements; accent: string }) {
    const t: TypedT<typeof messages> = useT("user");
    // Annihilation maps are surfaced in their own section (they span both pools
    // and have rotation rules), so keep them out of the regular stage lists.
    const regularMissing = pool.missing.filter((s) => !isAnnihilation(s));
    const regularNotThree = pool.not_three_starred.filter((s) => !isAnnihilation(s));

    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title={title} count={t("score.improvements.stage.threeStarCount", { done: pool.three_starred, total: pool.total })} accent={accent} />
            <div className="grid gap-2.5 sm:grid-cols-3">
                <ProgressLine label={t("score.improvements.stage.cleared")} current={pool.cleared} max={pool.total} accent={accent} />
                <ProgressLine label={t("score.improvements.stage.threeStarred")} current={pool.three_starred} max={pool.total} accent={accent} />
                <ProgressLine label={t("score.improvements.stage.notThreeStarred")} current={pool.cleared - pool.three_starred} max={pool.total} accent={accent} />
            </div>

            <StageList title={t("score.improvements.stage.missing")} stages={regularMissing} accent={accent} emptyLabel={t("score.improvements.stage.missing.empty")} />
            <StageList title={t("score.improvements.stage.notThree.title")} subtitle={t("score.improvements.stage.notThree.subtitle")} stages={regularNotThree} accent={accent} emptyLabel={t("score.improvements.stage.notThree.empty")} />
        </div>
    );
}

function AnnihilationSection({ stages, accent }: { stages: IStageGap[]; accent: string }) {
    const t: TypedT<typeof messages> = useT("user");
    if (stages.length === 0) return null;
    // Permanent maps carry no rotation window; the active rotation is "active".
    // Anything past/future has rotated out and can't be maxed right now.
    const available = stages.filter((s) => !s.rotation || s.rotation.status === "active");
    const locked = stages.filter((s) => s.rotation && s.rotation.status !== "active");

    return (
        <div className="flex flex-col gap-3">
            {/* "Annihilation" is the game mode's own name and stays as game vocabulary. */}
            <SectionHeader title="Annihilation" count={t("score.improvements.stage.annihilation.count", { n: available.length })} accent={accent} />
            {available.length > 0 ? <StageList title={t("score.improvements.stage.annihilation.available")} subtitle={t("score.improvements.stage.annihilation.availableSubtitle")} stages={available} accent={accent} emptyLabel="" /> : <EmptyHint>{t("score.improvements.stage.annihilation.maxed")}</EmptyHint>}
            {locked.length > 0 && <StageList title={t("score.improvements.stage.annihilation.locked")} subtitle={t("score.improvements.stage.annihilation.lockedSubtitle")} stages={locked} accent={accent} emptyLabel="" locked />}
        </div>
    );
}

function StageList({ title, subtitle, stages, accent, emptyLabel, locked = false }: { title: string; subtitle?: string; stages: IStageGap[]; accent: string; emptyLabel: string; locked?: boolean }) {
    const t: PanelT = useT("user");
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
                <span className={cn(TEXT_BADGE, "text-muted-foreground")}>{t("score.improvements.stage.total", { n: stages.length })}</span>
            </div>
            {subtitle && <p className={cn(TEXT_META, "-mt-1 text-muted-foreground/80")}>{subtitle}</p>}
            <div className="flex flex-wrap gap-1.5">
                {visible.map((s) => (
                    <StageChip key={s.stage_id} stage={s} accent={accent} locked={locked} />
                ))}
            </div>
            {remaining > 0 && <ShowMoreButton onClick={() => setShowAll(true)} label={t("score.improvements.showMore", { n: remaining })} />}
            {showAll && stages.length > INITIAL_VISIBLE && <ShowMoreButton onClick={() => setShowAll(false)} label={t("score.improvements.showLess")} />}
        </div>
    );
}

function StageChip({ stage, accent, locked = false }: { stage: IStageGap; accent: string; locked?: boolean }) {
    const t: TypedT<typeof messages> = useT("user");
    // Dot reflects clear state: rose = uncleared (state < 2), amber = cleared
    // but not 3★ (state 2). Locked (rotated-out) maps are muted regardless.
    const stateDot = locked ? "bg-muted-foreground/40" : stage.state >= 2 ? "bg-amber-500/75" : "bg-rose-500/65";
    const tooltipLabel = stage.name ? t("score.improvements.stage.chip.named", { code: stage.code, name: stage.name }) : stage.code;
    // Annihilation maps share a duplicated nation Code (e.g. several maps read
    // "Yan"), so the distinct map Name is the identifier worth leading with.
    // Regular stages keep the code (e.g. "1-7") as the primary label.
    const leadWithName = isAnnihilation(stage) && !!stage.name;
    const primary = leadWithName ? stage.name : stage.code;
    const secondary = leadWithName ? stage.code : stage.name;
    return (
        <span className={cn("group flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/15 px-2 py-1 transition-colors hover:border-border/70 hover:bg-muted/30", locked && "opacity-55")} title={locked ? t("score.improvements.stage.chip.lockedTitle", { label: tooltipLabel }) : tooltipLabel}>
            <span className={`size-1.5 shrink-0 rounded-full ${stateDot}`} aria-hidden />
            <span className={cn(TEXT_BADGE, "max-w-[22ch] truncate font-semibold")} style={{ color: `color-mix(in oklch, ${accent} 55%, var(--foreground))` }}>
                {primary}
            </span>
            {secondary && <span className="hidden max-w-[16ch] truncate text-[11px] text-muted-foreground sm:inline">{secondary}</span>}
            {stage.weight !== 1 && <Pill color={accent}>×{stage.weight.toFixed(2)}</Pill>}
        </span>
    );
}
