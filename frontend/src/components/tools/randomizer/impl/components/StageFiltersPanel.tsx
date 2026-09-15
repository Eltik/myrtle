import { ChevronDown, Lock, Search, X } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { Input } from "#/components/ui/input";
import { Switch } from "#/components/ui/switch";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { IStage, IZone, StageClearsMap } from "#/types/stages";
import type { IActivityLookup } from "../activity-lookup";
import type { IRandomizerSettings } from "../types";
import { buildStageGroups, type IStageGroup, isStageCleared, STAGE_SECTION_LABEL_KEYS, type StageGroupSection } from "../utils";
import type { messages as utilMessages } from "../utils.messages";
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

const SECTION_ORDER: StageGroupSection[] = ["MAIN", "EVENT", "OTHER"];

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
        if (!trimmedQuery) return pool;
        return pool
            .map((group) => {
                const groupHit = compactForSearch(group.label).includes(trimmedQuery) || (group.sublabel ? compactForSearch(group.sublabel).includes(trimmedQuery) : false);
                if (groupHit) return group;
                const stagesHit = group.stages.filter((s) => compactForSearch(s.code).includes(trimmedQuery) || (s.name ? compactForSearch(s.name).includes(trimmedQuery) : false));
                if (stagesHit.length === 0) return null;
                return { ...group, stages: stagesHit } satisfies IStageGroup;
            })
            .filter((g): g is IStageGroup => g !== null);
    }, [groups, trimmedQuery, settings.onlyAvailableStages]);

    const groupsBySection = React.useMemo(() => {
        const map = new Map<StageGroupSection, IStageGroup[]>();
        for (const section of SECTION_ORDER) map.set(section, []);
        for (const g of visibleGroups) map.get(g.section)?.push(g);
        return map;
    }, [visibleGroups]);

    const deselected = React.useMemo(() => new Set(settings.deselectedStageIds), [settings.deselectedStageIds]);

    const totalStages = stages.length;
    const selectedCount = totalStages - deselected.size;

    const setDeselected = React.useCallback(
        (next: Set<string>) => {
            onChange({ deselectedStageIds: Array.from(next) });
        },
        [onChange],
    );

    const onToggleStage = React.useCallback(
        (stageId: string) => {
            const next = new Set(deselected);
            if (next.has(stageId)) next.delete(stageId);
            else next.add(stageId);
            setDeselected(next);
        },
        [deselected, setDeselected],
    );

    const onToggleGroup = React.useCallback(
        (group: IStageGroup) => {
            const next = new Set(deselected);
            const anySelected = group.stages.some((s) => !next.has(s.stageId));
            if (anySelected) {
                for (const s of group.stages) next.add(s.stageId);
            } else {
                for (const s of group.stages) next.delete(s.stageId);
            }
            setDeselected(next);
        },
        [deselected, setDeselected],
    );

    const onSelectAll = React.useCallback(() => setDeselected(new Set()), [setDeselected]);
    const onDeselectAll = React.useCallback(() => setDeselected(new Set(stages.map((s) => s.stageId))), [setDeselected, stages]);

    const onToggleExpand = React.useCallback((groupId: string) => {
        setOpenGroups((prev) => {
            const next = new Set(prev);
            if (next.has(groupId)) next.delete(groupId);
            else next.add(groupId);
            return next;
        });
    }, []);

    const onToggleSection = React.useCallback((section: StageGroupSection) => {
        setCollapsedSections((prev) => {
            const next = new Set(prev);
            if (next.has(section)) next.delete(section);
            else next.add(section);
            return next;
        });
    }, []);

    const onChangeCompletedToggle = React.useCallback(
        (v: boolean) => {
            if (!v) {
                onChange({ onlyCompletedStages: false });
                return;
            }
            const next = new Set(deselected);
            for (const stage of stages) {
                if (!isStageCleared(stage.stageId, stageClears)) next.add(stage.stageId);
            }
            onChange({ onlyCompletedStages: true, deselectedStageIds: Array.from(next) });
        },
        [onChange, deselected, stages, stageClears],
    );

    return (
        <div className="flex flex-col gap-4">
            <FieldGroup label={t("randomizer.stages.eligibility")}>
                <SwitchRow label={t("randomizer.stages.onlyAvailable")} description={t("randomizer.stages.onlyAvailable.desc")} checked={settings.onlyAvailableStages} onChange={(v) => onChange({ onlyAvailableStages: v })} />
                <SwitchRow label={t("randomizer.stages.onlyCleared")} description={t("randomizer.stages.onlyCleared.desc")} checked={settings.onlyCompletedStages} onChange={onChangeCompletedToggle} locked={!hasProfile} />
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
                    {SECTION_ORDER.map((section) => {
                        const sectionGroups = groupsBySection.get(section) ?? [];
                        if (sectionGroups.length === 0) return null;
                        const isExpanded = !collapsedSections.has(section) || trimmedQuery !== "";
                        const totalSectionStages = sectionGroups.reduce((acc, g) => acc + g.stages.length, 0);
                        const selectedSectionStages = sectionGroups.reduce((acc, g) => acc + g.stages.reduce((a2, s) => a2 + (deselected.has(s.stageId) ? 0 : 1), 0), 0);
                        return (
                            <section key={section} className="flex flex-col gap-1.5">
                                <button type="button" onClick={() => onToggleSection(section)} className="flex items-center gap-1.5 rounded-sm px-0.5 outline-none focus-visible:ring-2 focus-visible:ring-ring" aria-expanded={isExpanded}>
                                    <ChevronDown aria-hidden="true" className={cn("size-3 text-muted-foreground/70 transition-transform duration-150", !isExpanded && "-rotate-90")} />
                                    <h3 className="font-mono text-[10px] text-muted-foreground/70 uppercase tracking-[0.2em]">{t(STAGE_SECTION_LABEL_KEYS[section])}</h3>
                                    <span className="font-mono text-[10px] text-muted-foreground/50 uppercase tracking-[0.16em]">{t("randomizer.stages.sectionCount", { selected: selectedSectionStages, total: totalSectionStages })}</span>
                                </button>
                                {isExpanded &&
                                    sectionGroups.map((group) => {
                                        const groupDeselectedCount = group.stages.reduce((acc, s) => acc + (deselected.has(s.stageId) ? 1 : 0), 0);
                                        const allDeselected = groupDeselectedCount === group.stages.length;
                                        const allSelected = groupDeselectedCount === 0;
                                        const isOpen = openGroups.has(group.id) || trimmedQuery !== "";
                                        return (
                                            <EventRow
                                                key={group.id}
                                                group={group}
                                                isOpen={isOpen}
                                                onToggleExpand={() => onToggleExpand(group.id)}
                                                checkboxState={allSelected ? "checked" : allDeselected ? "unchecked" : "indeterminate"}
                                                onToggleGroup={() => onToggleGroup(group)}
                                                deselected={deselected}
                                                onToggleStage={onToggleStage}
                                                t={t}
                                            />
                                        );
                                    })}
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
    checkboxState: "checked" | "unchecked" | "indeterminate";
    onToggleGroup: () => void;
    deselected: Set<string>;
    onToggleStage: (stageId: string) => void;
    t: StageFiltersT;
}

function EventRow({ group, isOpen, onToggleExpand, checkboxState, onToggleGroup, deselected, onToggleStage, t }: IEventRowProps): React.ReactElement {
    const selectedCount = group.stages.length - group.stages.reduce((acc, s) => acc + (deselected.has(s.stageId) ? 1 : 0), 0);
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

function FieldGroup({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-2.5">
            <p className="font-mono text-[10.5px] text-muted-foreground/90 uppercase tracking-[0.18em]">{label}</p>
            {children}
        </div>
    );
}

function SwitchRow({ label, description, checked, onChange, locked = false }: { label: string; description: string; checked: boolean; onChange: (v: boolean) => void; locked?: boolean }) {
    return (
        // biome-ignore lint/a11y/noLabelWithoutControl: Switch is a Base UI primitive; wrapping label provides click target and is correctly associated at runtime
        <label className={cn("flex items-start justify-between gap-3 rounded-md border border-border/50 bg-card/60 px-3 py-2.5 transition-colors hover:bg-accent/30", locked && "cursor-not-allowed opacity-60 hover:bg-card/60")}>
            <div className="min-w-0 flex-1">
                <p className="flex items-center gap-1.5 font-medium text-[12.5px] text-foreground">
                    {label}
                    {locked && <Lock aria-hidden="true" className="h-3 w-3 text-muted-foreground/70" />}
                </p>
                <p className="mt-0.5 text-[11.5px] text-muted-foreground leading-snug">{description}</p>
            </div>
            <Switch checked={locked ? false : checked} disabled={locked} onCheckedChange={onChange} />
        </label>
    );
}
