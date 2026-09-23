import { ChevronDown, Lock, Search, X } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { Input } from "#/components/ui/input";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { IStage, IZone, StageClearsMap } from "#/types/stages";
import type { IActivityLookup } from "../activity-lookup";
import type { IRandomizerSettings } from "../types";
import { buildStageGroups, type IStageGroup, isStageCleared, STAGE_SECTION_LABEL_KEYS, STAGE_SECTION_ORDER, type StageGroupSection } from "../utils";
import type { messages as utilMessages } from "../utils.messages";
import { FieldGroup, SwitchRow } from "./FilterControls";
import type { messages } from "./StageFiltersPanel.messages";

/** This panel renders its own chrome plus the group labels `utils.ts` derives. */
type StageFiltersT = TypedT<typeof messages & typeof utilMessages>;

interface IStageFiltersPanelProps {
    settings: IRandomizerSettings;
    onChange: (next: Partial<IRandomizerSettings>) => void;
    hasProfile: boolean;
    stages: IStage[];
    zones: IZone[];
    activityLookup: IActivityLookup;
    stageClears: StageClearsMap | null;
}

type CheckboxState = "checked" | "unchecked" | "indeterminate";

/** A copy of `set` with `key` flipped in or out. */
function toggled<T>(set: ReadonlySet<T>, key: T): Set<T> {
    const next = new Set(set);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    return next;
}

function countSelected(stages: IStage[], deselected: ReadonlySet<string>): number {
    let n = 0;
    for (const s of stages) if (!deselected.has(s.stageId)) n++;
    return n;
}

export function StageFiltersPanel({ settings, onChange, hasProfile, stages, zones, activityLookup, stageClears }: IStageFiltersPanelProps): React.ReactElement {
    const t: StageFiltersT = useT("tools");
    const [query, setQuery] = React.useState("");
    const [openGroups, setOpenGroups] = React.useState<Set<string>>(() => new Set());
    const [collapsedSections, setCollapsedSections] = React.useState<Set<StageGroupSection>>(() => new Set());

    const groups = React.useMemo(() => buildStageGroups(stages, zones, activityLookup, t), [stages, zones, activityLookup, t]);

    const trimmedQuery = compactForSearch(query);

    const visibleGroups = React.useMemo(() => {
        let pool = groups;
        if (settings.onlyAvailableStages) pool = pool.filter((g) => g.isOpen);
        // A view filter, like `onlyAvailableStages`: it hides uncleared stages
        // and leaves `deselectedStageIds` alone, so turning it off restores the
        // pool. (It once wrote every uncleared stage into the deselection, which
        // nothing undid.)
        if (settings.onlyCompletedStages) {
            pool = pool.map((group) => ({ ...group, stages: group.stages.filter((s) => isStageCleared(s.stageId, stageClears)) }) satisfies IStageGroup).filter((group) => group.stages.length > 0);
        }
        if (!trimmedQuery) return pool;
        const matches = (text: string | null | undefined) => (text ? compactForSearch(text).includes(trimmedQuery) : false);
        return pool
            .map((group) => {
                if (matches(group.label) || matches(group.sublabel)) return group;
                const stagesHit = group.stages.filter((s) => matches(s.code) || matches(s.name));
                if (stagesHit.length === 0) return null;
                return { ...group, stages: stagesHit } satisfies IStageGroup;
            })
            .filter((g): g is IStageGroup => g !== null);
    }, [groups, trimmedQuery, settings.onlyAvailableStages, settings.onlyCompletedStages, stageClears]);

    const groupsBySection = React.useMemo(() => {
        const map = new Map<StageGroupSection, IStageGroup[]>();
        for (const section of STAGE_SECTION_ORDER) map.set(section, []);
        for (const g of visibleGroups) map.get(g.section)?.push(g);
        return map;
    }, [visibleGroups]);

    const deselected = React.useMemo(() => new Set(settings.deselectedStageIds), [settings.deselectedStageIds]);

    const totalStages = stages.length;
    const selectedCount = totalStages - deselected.size;

    const setDeselected = React.useCallback((next: Set<string>) => onChange({ deselectedStageIds: Array.from(next) }), [onChange]);

    const onToggleStage = React.useCallback((stageId: string) => setDeselected(toggled(deselected, stageId)), [deselected, setDeselected]);

    /** A group with anything selected deselects whole; a fully deselected one selects whole. */
    const onToggleGroup = React.useCallback(
        (group: IStageGroup) => {
            const next = new Set(deselected);
            const anySelected = group.stages.some((s) => !next.has(s.stageId));
            for (const s of group.stages) {
                if (anySelected) next.add(s.stageId);
                else next.delete(s.stageId);
            }
            setDeselected(next);
        },
        [deselected, setDeselected],
    );

    const onSelectAll = React.useCallback(() => setDeselected(new Set()), [setDeselected]);
    const onDeselectAll = React.useCallback(() => setDeselected(new Set(stages.map((s) => s.stageId))), [setDeselected, stages]);

    const onToggleExpand = React.useCallback((groupId: string) => setOpenGroups((prev) => toggled(prev, groupId)), []);
    const onToggleSection = React.useCallback((section: StageGroupSection) => setCollapsedSections((prev) => toggled(prev, section)), []);

    // An active search opens every section and group so the hits are visible.
    const searching = trimmedQuery !== "";

    return (
        <div className="flex flex-col gap-4">
            <FieldGroup label={t("randomizer.stages.eligibility")}>
                <SwitchRow label={t("randomizer.stages.onlyAvailable")} description={t("randomizer.stages.onlyAvailable.desc")} checked={settings.onlyAvailableStages} onChange={(v) => onChange({ onlyAvailableStages: v })} />
                <SwitchRow label={t("randomizer.stages.onlyCleared")} description={t("randomizer.stages.onlyCleared.desc")} checked={settings.onlyCompletedStages} onChange={(v) => onChange({ onlyCompletedStages: v })} locked={!hasProfile || !stageClears} />
            </FieldGroup>

            <FieldGroup label={t("randomizer.stages.pool")}>
                <div className="relative">
                    <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
                    <Input value={query} onChange={(e) => setQuery(e.currentTarget.value)} placeholder={t("randomizer.stages.search")} size="sm" className="rounded-md border-input pl-7" />
                    {query && (
                        <button type="button" onClick={() => setQuery("")} className="absolute top-1/2 right-1.5 inline-flex size-5 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground" aria-label={t("randomizer.stages.clearSearch")}>
                            <X className="size-3" />
                        </button>
                    )}
                </div>

                <div className="flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] text-muted-foreground uppercase tracking-[0.16em]">
                    <span>
                        <span className="text-foreground">{selectedCount}</span> / {totalStages} {t("randomizer.stages.count")}
                    </span>
                    <div className="flex flex-wrap gap-1.5">
                        <Button onClick={onSelectAll} size="xs" variant="ghost" disabled={deselected.size === 0}>
                            {t("randomizer.stages.all")}
                        </Button>
                        <Button onClick={onDeselectAll} size="xs" variant="ghost" disabled={selectedCount === 0}>
                            {t("randomizer.stages.none")}
                        </Button>
                    </div>
                </div>

                <div className="flex flex-col gap-3">
                    {visibleGroups.length === 0 && <p className="rounded-md border border-border/50 bg-card/40 px-3 py-6 text-center text-[12px] text-muted-foreground">{t("randomizer.stages.noMatch")}</p>}
                    {STAGE_SECTION_ORDER.map((section) => {
                        const sectionGroups = groupsBySection.get(section) ?? [];
                        if (sectionGroups.length === 0) return null;
                        const isExpanded = !collapsedSections.has(section) || searching;
                        const totalSectionStages = sectionGroups.reduce((acc, g) => acc + g.stages.length, 0);
                        const selectedSectionStages = sectionGroups.reduce((acc, g) => acc + countSelected(g.stages, deselected), 0);
                        return (
                            <section key={section} className="flex flex-col gap-1.5">
                                <button type="button" onClick={() => onToggleSection(section)} className="flex items-center gap-1.5 rounded-sm px-0.5 outline-none focus-visible:ring-2 focus-visible:ring-ring" aria-expanded={isExpanded}>
                                    <ChevronDown aria-hidden="true" className={cn("size-3 text-muted-foreground/70 transition-transform duration-150", !isExpanded && "-rotate-90")} />
                                    <h3 className="font-mono text-[10px] text-muted-foreground/70 uppercase tracking-[0.2em]">{t(STAGE_SECTION_LABEL_KEYS[section])}</h3>
                                    <span className="font-mono text-[10px] text-muted-foreground/50 uppercase tracking-[0.16em]">{t("randomizer.stages.sectionCount", { selected: selectedSectionStages, total: totalSectionStages })}</span>
                                </button>
                                {isExpanded && sectionGroups.map((group) => <EventRow key={group.id} group={group} isOpen={openGroups.has(group.id) || searching} onToggleExpand={() => onToggleExpand(group.id)} onToggleGroup={() => onToggleGroup(group)} deselected={deselected} onToggleStage={onToggleStage} t={t} />)}
                            </section>
                        );
                    })}
                </div>
            </FieldGroup>
        </div>
    );
}

interface IEventRowProps {
    group: IStageGroup;
    isOpen: boolean;
    onToggleExpand: () => void;
    onToggleGroup: () => void;
    deselected: Set<string>;
    onToggleStage: (stageId: string) => void;
    t: StageFiltersT;
}

function EventRow({ group, isOpen, onToggleExpand, onToggleGroup, deselected, onToggleStage, t }: IEventRowProps): React.ReactElement {
    const selectedCount = countSelected(group.stages, deselected);
    const checkboxState: CheckboxState = selectedCount === group.stages.length ? "checked" : selectedCount === 0 ? "unchecked" : "indeterminate";
    return (
        <div className="overflow-hidden rounded-md border border-border/50 bg-card/60">
            <div className="flex items-center gap-2 px-2.5 py-2">
                <Checkbox checked={checkboxState === "checked"} indeterminate={checkboxState === "indeterminate"} onCheckedChange={onToggleGroup} aria-label={t("randomizer.stages.toggleGroup", { group: group.label })} />
                <button type="button" onClick={onToggleExpand} className="flex min-w-0 flex-1 items-center gap-2 rounded-sm text-left outline-none focus-visible:ring-2 focus-visible:ring-ring" aria-expanded={isOpen}>
                    <ChevronDown aria-hidden="true" className={cn("size-3.5 shrink-0 text-muted-foreground transition-transform duration-150", isOpen && "rotate-180")} />
                    <div className="min-w-0 flex-1">
                        <p className="flex items-center gap-1.5 truncate font-medium text-[12.5px] text-foreground leading-tight">
                            {group.label}
                            {group.section !== "MAIN" && !group.isOpen && <Lock aria-hidden="true" className="size-3 shrink-0 text-muted-foreground/60" />}
                        </p>
                        <p className="mt-0.5 font-mono text-[10.5px] text-muted-foreground/80 uppercase tracking-[0.14em]">
                            {group.section === "MAIN" ? t("randomizer.stages.tag.mainline") : (group.sublabel ?? (group.section === "OTHER" ? t("randomizer.stages.tag.mini") : t("randomizer.stages.tag.event")))}
                            <span className="mx-1.5 opacity-50">·</span>
                            {t("randomizer.stages.groupCount", { selected: selectedCount, total: group.stages.length })}
                        </p>
                    </div>
                </button>
            </div>
            {isOpen && (
                <ul className="flex flex-col gap-px border-border/40 border-t bg-background/40 px-2 py-1.5">
                    {group.stages.map((stage) => {
                        const isDeselected = deselected.has(stage.stageId);
                        return (
                            <li key={stage.stageId}>
                                {/* biome-ignore lint/a11y/noLabelWithoutControl: Checkbox is a Base UI primitive with its own aria-label; wrapping label provides the click target */}
                                <label className={cn("flex cursor-pointer items-center gap-2.5 rounded-sm px-1.5 py-1.5 transition-colors hover:bg-accent/40", isDeselected && "opacity-60")}>
                                    <Checkbox checked={!isDeselected} onCheckedChange={() => onToggleStage(stage.stageId)} aria-label={t("randomizer.stages.toggleStage", { stage: stage.name ? t("randomizer.stages.stageLabel", { code: stage.code, name: stage.name }) : stage.code })} />
                                    <StageModeBadge stage={stage} t={t} />
                                    <span className="inline-flex min-w-13 shrink-0 justify-start font-medium font-mono text-[11.5px] text-foreground">{stage.code}</span>
                                    <span className="min-w-0 flex-1 truncate text-[11.5px] text-muted-foreground">{stage.name ?? ""}</span>
                                    {isChallengeModeStage(stage) && <span className="shrink-0 rounded-sm border border-amber-500/40 bg-amber-500/10 px-1 font-mono text-[9px] text-amber-500/90 uppercase tracking-[0.14em]">{t("randomizer.stages.badge.cm")}</span>}
                                </label>
                            </li>
                        );
                    })}
                </ul>
            )}
        </div>
    );
}

/**
 * NORMAL = standard `main_*` stage, ADVERSE = `tough_*` variant (chapters 10-14),
 * STORY = `easy_*` (filtered out of the playable pool, but defensively handled).
 */
function getStageMode(stage: IStage): "NORMAL" | "ADVERSE" | "STORY" | null {
    if (stage.stageId.startsWith("tough_")) return "ADVERSE";
    if (stage.stageId.startsWith("easy_")) return "STORY";
    if (stage.stageId.startsWith("main_")) return "NORMAL";
    return null;
}

/** FOUR_STAR is the legacy CM (ch 0-14); SIX_STAR is the ch 15+ challenge variant. */
function isChallengeModeStage(stage: IStage): boolean {
    return stage.difficulty === "FOUR_STAR" || stage.difficulty === "SIX_STAR";
}

function StageModeBadge({ stage, t }: { stage: IStage; t: StageFiltersT }): React.ReactElement | null {
    const mode = getStageMode(stage);
    if (mode === "ADVERSE") return <span className="shrink-0 rounded-sm border border-rose-500/40 bg-rose-500/10 px-1 font-mono text-[9px] text-rose-500/90 uppercase tracking-[0.14em]">{t("randomizer.stages.badge.adverse")}</span>;
    if (mode === "STORY") return <span className="shrink-0 rounded-sm border border-border/50 px-1 font-mono text-[9px] text-muted-foreground uppercase tracking-[0.14em]">{t("randomizer.stages.badge.story")}</span>;
    return null;
}
