import { useQuery } from "@tanstack/react-query";
import { Columns, GitCompareArrows, Rows } from "lucide-react";
import { memo, useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { LoadFailed } from "#/components/ui/load-failed";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { operatorGamedataServer } from "#/lib/api/gamedata";
import { type IRange, rangesQueryOptions } from "#/lib/api/ranges";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { blackboardDiff } from "#/lib/operators/cn-changes";
import { cn } from "#/lib/utils";
import type { IOperatorListItem, ISkillLevel } from "#/types/operators";
import { asset } from "../../assets";
import { templateWithCnValues } from "../../cnText";
import { useCnUpdatesContext } from "../../cnUpdates";
import { descriptionToHtml } from "../../description";
import { computeSkillDiff, formatBlackboardValue, formatSkillLevel, getSkillTypeLabel, getSpTypeLabel } from "../../helpers";
import type { messages as helperMessages } from "../../helpers.messages";
import { useCommunityDefaults } from "../../useCommunityDefaults";
import { CnDescription, CnTag, CnValue } from "../CnMarkers";
import { CommunitySharePill } from "../CommunitySharePill";
import { LevelBreakdown } from "../LevelBreakdown";
import { OperatorRange } from "../OperatorRange";
import type { messages } from "./SkillsContent.messages";

interface ISkillsContentProps {
    operator: IOperatorListItem;
}

/** Own chrome plus the labels `helpers.ts` derives. */
type SkillsT = TypedT<typeof messages & typeof helperMessages>;

/** A pressed neutral toggle: a 2px foreground border and the muted fill, never a shadow, with the
 *  padding giving back the extra pixel so the button keeps its size. */
const PRESSED_NEUTRAL = "border-2 border-foreground/70 bg-muted px-[11px] text-foreground";

export const SkillsContent = memo(function SkillsContent({ operator }: ISkillsContentProps) {
    const t: SkillsT = useT("operators");
    const compareLabelId = useId();
    const levelLabelId = useId();
    const cnUpdates = useCnUpdatesContext();
    // Skills merged in from CN come after this server's, so the tab still opens on this server's last.
    const ownSkillCount = operator.skills.filter((s) => !cnUpdates?.skillIds.has(s.skillId)).length;
    const [selectedSkillIndex, setSelectedSkillIndex] = useState(ownSkillCount > 0 ? ownSkillCount - 1 : 0);
    const [skillLevel, setSkillLevel] = useState(Math.max(0, (operator.skills[selectedSkillIndex]?.static?.levels ?? []).length - 1));
    const [comparisonMode, setComparisonMode] = useState(false);
    const [showDifferencesOnly, setShowDifferencesOnly] = useState(false);
    const [comparisonSelection, setComparisonSelection] = useState<number[] | null>(null);
    const [showRangeDiff, setShowRangeDiff] = useState(false);

    const skillData = (operator.skills[selectedSkillIndex]?.static?.levels ?? [])[skillLevel];
    const skillDescription = useMemo(() => descriptionToHtml(skillData?.description ?? "", skillData?.blackboard ?? []), [skillData?.description, skillData?.blackboard]);
    const selectedSkillId = operator.skills[selectedSkillIndex]?.skillId;
    const selectedIsCn = selectedSkillId ? (cnUpdates?.skillIds.has(selectedSkillId) ?? false) : false;
    // The same level of a skill both servers have, as CN's record carries it, where CN changed its values.
    const cnSkillData = selectedSkillId && cnUpdates?.changes.some((c) => c.kind === "skill_values_changed" && c.skillId === selectedSkillId) ? cnUpdates.cn.skills.find((s) => s.skillId === selectedSkillId)?.static?.levels?.[skillLevel] : undefined;
    // This server's text with CN's numbers, or CN's own text where this server's cannot take them.
    const cnSkillDescription = useMemo(() => {
        if (!cnSkillData || !skillData?.description || blackboardDiff(skillData.blackboard, cnSkillData.blackboard).length === 0) return null;
        const html = templateWithCnValues(skillData.description, skillData.blackboard ?? [], cnSkillData.blackboard ?? [], cnSkillData.description);
        if (html === null) return cnSkillData.description ? { html: descriptionToHtml(cnSkillData.description, cnSkillData.blackboard ?? []), lang: "zh-CN" } : null;
        return html === descriptionToHtml(skillData.description, skillData.blackboard ?? []) ? null : { html };
    }, [cnSkillData, skillData?.description, skillData?.blackboard]);
    /** CN's value of one number, formatted as its cell formats its own, where it differs. */
    const onCn = (here: number | null | undefined, there: number | null | undefined, format: (n: number) => string = String) => (typeof here === "number" && typeof there === "number" && here !== there ? format(there) : null);
    const cnSpCost = onCn(skillData?.spData?.spCost, cnSkillData?.spData?.spCost);
    const cnInitSp = onCn(skillData?.spData?.initSp, cnSkillData?.spData?.initSp);
    const cnDuration = onCn(skillData?.duration, cnSkillData?.duration, (n) => (n > 0 ? t("skills.durationSeconds", { value: n }) : "-"));

    const levelsCount = (operator.skills[selectedSkillIndex]?.static?.levels ?? []).length;

    const localeServer = useGamedataServer();
    const { data: ranges, isError: rangesFailed, refetch: refetchRanges } = useQuery(rangesQueryOptions(operatorGamedataServer(operator.server, localeServer)));
    const baseRangeId = operator.phases[operator.phases.length - 1]?.rangeId ?? null;
    const baseRange: IRange | undefined = baseRangeId ? ranges?.[baseRangeId] : undefined;
    const skillRangeId = skillData?.rangeId ?? null;
    const rangeChanges = skillRangeId !== null && skillRangeId !== baseRangeId;
    const skillRange: IRange | undefined = rangeChanges && skillRangeId ? ranges?.[skillRangeId] : undefined;

    const defaultComparison = useMemo(() => {
        if (levelsCount >= 10) return [6, 7, 8, 9];
        if (levelsCount >= 4) return Array.from({ length: 4 }, (_, i) => levelsCount - 4 + i);
        return Array.from({ length: levelsCount }, (_, i) => i);
    }, [levelsCount]);

    const comparisonLevels = useMemo(() => {
        const sel = comparisonSelection ?? defaultComparison;
        const valid = sel.filter((i) => i >= 0 && i < levelsCount);
        return (valid.length > 0 ? valid : defaultComparison).slice().sort((a, b) => a - b);
    }, [comparisonSelection, defaultComparison, levelsCount]);

    const allLevelIndices = useMemo(() => Array.from({ length: levelsCount }, (_, i) => i), [levelsCount]);

    const toggleComparisonLevel = useCallback(
        (idx: number) => {
            setComparisonSelection((prev) => {
                const base = prev ?? defaultComparison;
                if (base.includes(idx)) {
                    const next = base.filter((i) => i !== idx);
                    return next.length === 0 ? base : next;
                }
                return [...base, idx];
            });
        },
        [defaultComparison],
    );

    const selectionSettled = useRef(false);

    const handleSkillChange = useCallback(
        (idx: number) => {
            selectionSettled.current = true;
            setSelectedSkillIndex(idx);
            const next = operator.skills[idx]?.static?.levels?.length ?? 1;
            setSkillLevel(next - 1);
            setComparisonSelection(null);
        },
        [operator.skills],
    );

    // Open on the viewer's own default skill, then the community's, then leave
    // the last skill this component already chose. `null` means "no answer",
    // which is different from index 0, so it falls through rather than
    // selecting S1.
    const { ownSkillIndex, communitySkillIndex, skillShares, skillTotal, masteries, ownMasteries } = useCommunityDefaults(operator);
    useEffect(() => {
        if (selectionSettled.current) return;
        const idx = ownSkillIndex ?? communitySkillIndex;
        if (idx == null || idx >= operator.skills.length) return;
        selectionSettled.current = true;
        setSelectedSkillIndex(idx);
        setSkillLevel(Math.max(0, (operator.skills[idx]?.static?.levels ?? []).length - 1));
    }, [ownSkillIndex, communitySkillIndex, operator.skills]);

    if (!operator.skills || operator.skills.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center gap-4 p-16 text-center">
                <div className="flex h-16 w-16 items-center justify-center rounded-full border border-border bg-secondary/30">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-7 w-7 text-subtle-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5} role="img" aria-label={t("skills.empty.icon")}>
                        <title>{t("skills.empty.icon")}</title>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                </div>
                <div className="space-y-1">
                    <p className="font-medium text-foreground text-sm">{t("skills.empty.title")}</p>
                    <p className="max-w-xs text-muted-foreground text-xs leading-relaxed">{t("skills.empty.body")}</p>
                </div>
            </div>
        );
    }

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">{t("skills.title")}</h2>
                <p className="text-muted-foreground text-sm">{t("skills.subtitle")}</p>
            </div>

            <ToggleGroup aria-label={t("skills.title")} className="mb-6 w-full flex-wrap gap-2" onValueChange={(next) => next[0] !== undefined && handleSkillChange(Number(next[0]))} value={[String(selectedSkillIndex)]} variant="outline">
                {operator.skills.map((skill, idx) => {
                    const name = skill.static?.levels?.[0]?.name ?? t("skills.fallbackName", { index: idx + 1 });
                    const image = skill.static?.image;
                    const isCn = cnUpdates?.skillIds.has(skill.skillId) ?? false;
                    return (
                        <ToggleGroupItem className="h-auto w-full justify-start px-4 py-2 text-muted-foreground hover:text-foreground data-pressed:text-foreground sm:h-auto sm:w-auto" key={skill.skillId ?? idx} value={String(idx)}>
                            {image && <img alt="" className="h-5 w-5 shrink-0 object-contain" decoding="async" loading="lazy" src={asset(image, isCn ? "cn" : operator.server)} />}
                            {isCn ? <span lang="zh-CN">{name}</span> : name}
                            {isCn && <CnTag />}
                            {/* Keyed by skillId, never by position: the game's
                                own default index is positional, but matching on
                                it here would repeat the module-order bug. */}
                            <CommunitySharePill className="ms-auto sm:ms-0" share={skill.skillId ? skillShares.get(skill.skillId) : undefined} total={skillTotal} cohort={t("skills.cohort")} />
                        </ToggleGroupItem>
                    );
                })}
            </ToggleGroup>

            {/* Where E2 owners stop on THIS skill. Sits under the selector so it
                tracks whichever skill is open, rather than repeating four
                strips at once. */}
            {(() => {
                const skillId = operator.skills[selectedSkillIndex]?.skillId;
                const mastery = skillId ? masteries.get(skillId) : undefined;
                if (!mastery) return null;
                return (
                    <LevelBreakdown
                        className="mb-6"
                        buckets={mastery.buckets}
                        total={mastery.total}
                        title={t("skills.mastery.title")}
                        labels={[t("skills.mastery.none"), t("skills.mastery.m1"), t("skills.mastery.m2"), t("skills.mastery.m3")]}
                        summary={t("skills.mastery.summary")}
                        cohort={t("skills.mastery.cohort")}
                        ownLevel={skillId ? (ownMasteries.get(skillId) ?? null) : null}
                    />
                );
            })()}

            <div className="mb-6 rounded-md border border-border bg-secondary/20 p-5">
                {levelsCount > 7 && (
                    <div className="mb-5 flex items-center justify-between gap-4 border-border border-b pb-5">
                        <span className="flex items-center gap-2.5">
                            {comparisonMode ? <Columns className="h-4 w-4 text-primary" /> : <Rows className="h-4 w-4 text-muted-foreground" />}
                            <span className="font-medium text-foreground text-sm" id={compareLabelId}>
                                {t("skills.compare.toggle")}
                            </span>
                        </span>
                        <Switch aria-labelledby={compareLabelId} checked={comparisonMode} onCheckedChange={setComparisonMode} />
                    </div>
                )}
                {!comparisonMode ? (
                    <div className="flex flex-col gap-4 md:flex-row md:items-center">
                        <div className="flex-1 space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground text-sm" id={levelLabelId}>
                                    {t("skills.level")}
                                </span>
                                <span className="font-mono text-foreground text-sm">{formatSkillLevel(skillLevel, t)}</span>
                            </div>
                            <Slider aria-labelledby={levelLabelId} min={0} max={Math.max(0, levelsCount - 1)} step={1} value={[skillLevel]} onValueChange={(v) => setSkillLevel(Array.isArray(v) ? (v[0] ?? 0) : v)} />
                        </div>
                        <Select value={String(skillLevel)} onValueChange={(v) => setSkillLevel(Number.parseInt(String(v), 10))}>
                            <SelectTrigger className="w-36">
                                <SelectValue>{(value) => formatSkillLevel(Number(value), t)}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {operator.skills[selectedSkillIndex].static?.levels.map((_, i) => (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: skill level index is the canonical identifier
                                    <SelectItem key={`level-${i}`} value={String(i)}>
                                        {formatSkillLevel(i, t)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                ) : (
                    <div className="space-y-4">
                        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                            <div className="flex items-center gap-2">
                                <span className="font-medium text-foreground text-sm">{t("skills.compare.selectLevels")}</span>
                                <span className="text-muted-foreground text-xs">{t("skills.compare.selectedCount", { count: comparisonLevels.length })}</span>
                            </div>
                            <button
                                type="button"
                                aria-pressed={showDifferencesOnly}
                                onClick={() => setShowDifferencesOnly((s) => !s)}
                                className={cn("inline-flex h-9 items-center gap-2 rounded-sm font-medium text-xs transition-colors", showDifferencesOnly ? PRESSED_NEUTRAL : "border border-border bg-secondary/50 px-3 text-foreground hover:bg-muted")}
                            >
                                <GitCompareArrows className="h-4 w-4" />
                                {showDifferencesOnly ? t("skills.compare.showFull") : t("skills.compare.showDiff")}
                            </button>
                        </div>
                        <div className="flex flex-wrap gap-2">
                            {allLevelIndices.map((idx) => {
                                const isOn = comparisonLevels.includes(idx);
                                const isMastery = idx >= 7;
                                return (
                                    <button
                                        type="button"
                                        key={`tl-${idx}`}
                                        aria-pressed={isOn}
                                        onClick={() => toggleComparisonLevel(idx)}
                                        className={cn("h-9 min-w-14 rounded-sm font-medium text-xs transition-colors", isOn ? (isMastery ? PRESSED_NEUTRAL : "border-2 border-primary bg-primary/10 px-[11px] text-foreground") : "border border-border bg-secondary/30 px-3 text-muted-foreground hover:text-foreground")}
                                    >
                                        {formatSkillLevel(idx, t)}
                                    </button>
                                );
                            })}
                        </div>
                    </div>
                )}
            </div>

            {!comparisonMode && skillData && (
                <div className="rounded-lg border border-border bg-card/30 p-5 md:p-6">
                    <div className="mb-5 flex items-start gap-4">
                        {operator.skills[selectedSkillIndex].static?.image && (
                            <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-md border border-border bg-secondary/50">
                                <img alt={skillData.name ?? t("skills.imageAlt")} className="h-12 w-12 object-contain" decoding="async" loading="lazy" src={asset(operator.skills[selectedSkillIndex].static.image, selectedIsCn ? "cn" : operator.server)} />
                            </div>
                        )}
                        <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                                <h3 lang={selectedIsCn ? "zh-CN" : undefined} className="font-semibold text-foreground text-lg">
                                    {skillData.name}
                                </h3>
                                {selectedIsCn && <CnTag />}
                            </div>
                            <div className="mt-2 flex flex-wrap gap-2">
                                <Badge className="rounded-sm bg-secondary/50" variant="secondary">
                                    {getSpTypeLabel(skillData.spData?.spType ?? "", t)}
                                </Badge>
                                <Badge className="rounded-sm bg-secondary/50" variant="secondary">
                                    {getSkillTypeLabel(skillData.skillType ?? "", t)}
                                </Badge>
                            </div>
                        </div>
                    </div>

                    <dl className="mb-5 grid grid-cols-3 gap-3">
                        <div className="min-w-0 rounded-sm border border-border/50 bg-secondary/20 p-3 text-center">
                            <dt className="text-muted-foreground text-xs">{t("skills.spCost")}</dt>
                            <dd className="mt-1 font-mono font-semibold text-foreground text-lg">
                                {skillData.spData?.spCost ?? "-"}
                                {cnSpCost && <CnValue value={cnSpCost} />}
                            </dd>
                        </div>
                        <div className="min-w-0 rounded-sm border border-border/50 bg-secondary/20 p-3 text-center">
                            <dt className="text-muted-foreground text-xs">{t("skills.initialSp")}</dt>
                            <dd className="mt-1 font-mono font-semibold text-foreground text-lg">
                                {skillData.spData?.initSp ?? "-"}
                                {cnInitSp && <CnValue value={cnInitSp} />}
                            </dd>
                        </div>
                        <div className="min-w-0 rounded-sm border border-border/50 bg-secondary/20 p-3 text-center">
                            <dt className="text-muted-foreground text-xs">{t("skills.duration")}</dt>
                            <dd className="mt-1 font-mono font-semibold text-foreground text-lg">
                                {skillData.duration && skillData.duration > 0 ? t("skills.durationSeconds", { value: skillData.duration }) : "-"}
                                {cnDuration && <CnValue value={cnDuration} />}
                            </dd>
                        </div>
                    </dl>

                    <div className="rounded-sm border border-border/50 bg-secondary/10 p-4">
                        <p
                            lang={selectedIsCn ? "zh-CN" : undefined}
                            className="text-foreground text-sm leading-relaxed"
                            // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized
                            dangerouslySetInnerHTML={{ __html: skillDescription }}
                        />
                        {cnSkillDescription && <CnDescription {...cnSkillDescription} />}
                    </div>

                    {skillRange && (
                        <div className="mt-5 rounded-sm border border-border/50 bg-secondary/10 p-4">
                            <div className="mb-3 flex items-center justify-between">
                                <span className="font-medium text-muted-foreground text-xs uppercase tracking-wide">{t("skills.range")}</span>
                                {baseRange && (
                                    <button
                                        type="button"
                                        aria-pressed={showRangeDiff}
                                        onClick={() => setShowRangeDiff((s) => !s)}
                                        className={cn("inline-flex h-8 items-center gap-2 rounded-sm font-medium text-xs transition-colors", showRangeDiff ? PRESSED_NEUTRAL : "border border-border bg-secondary/50 px-3 text-foreground hover:bg-muted")}
                                    >
                                        <GitCompareArrows className="h-4 w-4" />
                                        {showRangeDiff ? t("skills.range.hideDiff") : t("skills.compare.showDiff")}
                                    </button>
                                )}
                            </div>
                            {showRangeDiff && baseRange ? (
                                <div className="flex flex-wrap items-start gap-6">
                                    <div className="space-y-1.5">
                                        <span className="block font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("skills.range.original")}</span>
                                        <OperatorRange range={baseRange} />
                                    </div>
                                    <div className="space-y-1.5">
                                        <span className="block font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("skills.range.onSkill")}</span>
                                        <OperatorRange range={skillRange} />
                                    </div>
                                </div>
                            ) : (
                                <OperatorRange range={skillRange} />
                            )}
                        </div>
                    )}
                    {/* A skill that changes the range showed no range at all when the request failed. */}
                    {rangeChanges && rangesFailed && !ranges && <LoadFailed className="mt-5" message={t("skills.range.error")} onRetry={() => void refetchRanges()} />}
                </div>
            )}
            {comparisonMode && (
                <div className="space-y-4">
                    <div className="flex items-start gap-4 rounded-md border border-border bg-card/50 p-5 shadow-sm">
                        {operator.skills[selectedSkillIndex].static?.image && (
                            <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-sm border border-border bg-secondary/50 shadow-sm">
                                <img alt={operator.skills[selectedSkillIndex].static.levels[comparisonLevels[0] ?? 0]?.name ?? t("skills.imageAlt")} className="h-10 w-10 object-contain" decoding="async" loading="lazy" src={asset(operator.skills[selectedSkillIndex].static.image, operator.server)} />
                            </div>
                        )}
                        <div className="min-w-0 flex-1">
                            <h3 className="font-semibold text-foreground text-lg">{operator.skills[selectedSkillIndex].static?.levels[comparisonLevels[0] ?? 0]?.name}</h3>
                            <div className="mt-2 flex flex-wrap gap-2">
                                <Badge className="rounded-sm bg-secondary shadow-sm" variant="secondary">
                                    {getSpTypeLabel(operator.skills[selectedSkillIndex].static?.levels[comparisonLevels[0] ?? 0]?.spData?.spType ?? "", t)}
                                </Badge>
                                <Badge className="rounded-sm bg-secondary shadow-sm" variant="secondary">
                                    {getSkillTypeLabel(operator.skills[selectedSkillIndex].static?.levels[comparisonLevels[0] ?? 0]?.skillType ?? "", t)}
                                </Badge>
                            </div>
                        </div>
                    </div>

                    <div className="hidden items-center gap-6 px-4 font-medium text-muted-foreground text-xs uppercase tracking-wider md:flex">
                        <div className="w-20 shrink-0">{t("skills.column.level")}</div>
                        <div className="min-w-0 flex-1">{t("skills.column.description")}</div>
                        <div className="flex shrink-0 gap-3">
                            <div className="w-14 text-center">{t("skills.column.sp")}</div>
                            <div className="w-14 text-center">{t("skills.column.init")}</div>
                            <div className="w-14 text-center">{t("skills.column.dur")}</div>
                        </div>
                    </div>

                    <div className="flex flex-col">
                        {comparisonLevels.map((idx, i) => {
                            const data = operator.skills[selectedSkillIndex].static?.levels[idx];
                            if (!data) return null;
                            const prev = i > 0 ? (operator.skills[selectedSkillIndex].static?.levels[comparisonLevels[i - 1]] ?? null) : null;
                            const rowRangeId = data.rangeId ?? null;
                            const rowRangeChanges = rowRangeId !== null && rowRangeId !== baseRangeId;
                            const rowRange = rowRangeChanges && rowRangeId ? ranges?.[rowRangeId] : undefined;
                            return (
                                <SkillComparisonRow
                                    key={`cmp-${idx}`}
                                    levelIndex={idx}
                                    levelData={data}
                                    prevLevelData={prev}
                                    isFirst={i === 0}
                                    isLast={i === comparisonLevels.length - 1}
                                    showDifferencesOnly={showDifferencesOnly}
                                    range={rowRange}
                                    baseRange={baseRange}
                                    showRangeDiff={showRangeDiff}
                                    onToggleRangeDiff={() => setShowRangeDiff((s) => !s)}
                                />
                            );
                        })}
                    </div>
                </div>
            )}
        </div>
    );
});

interface ISkillComparisonRowProps {
    levelIndex: number;
    levelData: ISkillLevel;
    prevLevelData: ISkillLevel | null;
    isFirst?: boolean;
    isLast?: boolean;
    showDifferencesOnly?: boolean;
    range?: IRange;
    baseRange?: IRange;
    showRangeDiff?: boolean;
    onToggleRangeDiff?: () => void;
}

export const SkillComparisonRow = memo(function SkillComparisonRow({ levelIndex, levelData, prevLevelData, isFirst, isLast, showDifferencesOnly, range, baseRange, showRangeDiff, onToggleRangeDiff }: ISkillComparisonRowProps) {
    const t: SkillsT = useT("operators");
    const descriptionHtml = useMemo(() => descriptionToHtml(levelData.description ?? "", levelData.blackboard ?? []), [levelData.description, levelData.blackboard]);
    const diff = useMemo(() => computeSkillDiff(prevLevelData, levelData), [prevLevelData, levelData]);
    const hasChanges = diff.spCostChanged || diff.initSpChanged || diff.durationChanged || diff.blackboardChanges.size > 0;

    const diffSummary = useMemo(() => {
        if (!showDifferencesOnly || isFirst || !prevLevelData) return null;
        const out: { label: string; value: string }[] = [];
        if (diff.spCostChanged) out.push({ label: t("skills.spCost"), value: `${prevLevelData.spData?.spCost} → ${levelData.spData?.spCost}` });
        if (diff.initSpChanged) out.push({ label: t("skills.initialSp"), value: `${prevLevelData.spData?.initSp} → ${levelData.spData?.initSp}` });
        if (diff.durationChanged) {
            const prevDur = prevLevelData.duration && prevLevelData.duration > 0 ? t("skills.durationSeconds", { value: prevLevelData.duration }) : "-";
            const currDur = levelData.duration && levelData.duration > 0 ? t("skills.durationSeconds", { value: levelData.duration }) : "-";
            out.push({ label: t("skills.duration"), value: `${prevDur} → ${currDur}` });
        }
        for (const [key, { prev, curr }] of diff.blackboardChanges) {
            const display = key.charAt(0).toUpperCase() + key.slice(1).replace(/_/g, " ");
            out.push({ label: display, value: `${formatBlackboardValue(key, prev)} → ${formatBlackboardValue(key, curr)}` });
        }
        return out;
    }, [showDifferencesOnly, isFirst, diff, prevLevelData, levelData, t]);

    if (showDifferencesOnly && !isFirst && !hasChanges) return null;

    return (
        <div className={cn("flex flex-col gap-4 border-border bg-card/30 p-5 md:flex-row md:items-start md:gap-6 md:p-6", isFirst && "rounded-t-lg border", !isFirst && !isLast && "border-x border-b", isLast && "rounded-b-lg border-x border-b", isFirst && isLast && "rounded-lg")}>
            <div className="flex shrink-0 items-center gap-2 md:w-20 md:flex-col md:items-start md:gap-1.5">
                <Badge className={cn("rounded-sm font-mono text-sm shadow-sm", levelIndex >= 7 ? "border-border bg-muted text-foreground" : "border-border bg-muted/50 text-muted-foreground")} variant="secondary">
                    {formatSkillLevel(levelIndex, t)}
                </Badge>
            </div>

            <div className="min-w-0 flex-1">
                {showDifferencesOnly && !isFirst && diffSummary && diffSummary.length > 0 ? (
                    <div className="space-y-2.5">
                        {diffSummary.map((c, idx) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: diff entries are positionally generated and stable per render
                            <div className="flex items-baseline gap-3 rounded-sm border border-border bg-muted/30 px-4 py-2.5" key={`${c.label}-${idx}`}>
                                <span className="shrink-0 font-medium text-muted-foreground text-xs uppercase tracking-wide">{c.label}</span>
                                <span className="font-medium font-mono text-foreground text-sm">{c.value}</span>
                            </div>
                        ))}
                    </div>
                ) : (
                    <p
                        className="text-foreground text-sm leading-relaxed"
                        // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized
                        dangerouslySetInnerHTML={{ __html: descriptionHtml }}
                    />
                )}
                {range && (
                    <div className="mt-4 rounded-sm border border-border/50 bg-secondary/10 p-3">
                        <div className="mb-2 flex items-center justify-between">
                            <span className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("skills.range")}</span>
                            {baseRange && onToggleRangeDiff && (
                                <button
                                    type="button"
                                    aria-pressed={showRangeDiff}
                                    onClick={onToggleRangeDiff}
                                    className={cn("inline-flex h-7 items-center gap-1.5 rounded-sm font-medium text-[11px] transition-colors", showRangeDiff ? "border-2 border-foreground/70 bg-muted px-[7px] text-foreground" : "border border-border bg-secondary/50 px-2 text-foreground hover:bg-muted")}
                                >
                                    <GitCompareArrows className="h-3 w-3" />
                                    {showRangeDiff ? t("skills.range.hideDiff") : t("skills.compare.showDiff")}
                                </button>
                            )}
                        </div>
                        {showRangeDiff && baseRange ? (
                            <div className="flex flex-wrap items-start gap-4">
                                <div className="space-y-1.5">
                                    <span className="block font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("skills.range.original")}</span>
                                    <OperatorRange range={baseRange} />
                                </div>
                                <div className="space-y-1.5">
                                    <span className="block font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("skills.range.onSkill")}</span>
                                    <OperatorRange range={range} />
                                </div>
                            </div>
                        ) : (
                            <OperatorRange range={range} />
                        )}
                    </div>
                )}
            </div>

            <div className="flex shrink-0 items-center gap-2.5 md:gap-3">
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-3.5 py-1.5 transition-all duration-200", diff.spCostChanged ? "border-foreground/40 bg-muted" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("skills.column.sp")}</div>
                    <div className="mt-0.5 font-mono font-semibold text-base text-foreground">{levelData.spData?.spCost ?? "-"}</div>
                </div>
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-3.5 py-1.5 transition-all duration-200", diff.initSpChanged ? "border-foreground/40 bg-muted" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("skills.column.init")}</div>
                    <div className="mt-0.5 font-mono font-semibold text-base text-foreground">{levelData.spData?.initSp ?? "-"}</div>
                </div>
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-3.5 py-1.5 transition-all duration-200", diff.durationChanged ? "border-foreground/40 bg-muted" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">{t("skills.column.dur")}</div>
                    <div className="mt-0.5 font-mono font-semibold text-base text-foreground">{levelData.duration && levelData.duration > 0 ? t("skills.durationSeconds", { value: levelData.duration }) : "-"}</div>
                </div>
            </div>
        </div>
    );
});
