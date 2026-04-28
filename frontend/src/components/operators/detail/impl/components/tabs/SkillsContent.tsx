import { Columns, GitCompareArrows, Rows } from "lucide-react";
import { memo, useCallback, useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { cn } from "#/lib/utils";
import type { IOperatorListItem, ISkillLevel } from "#/types/operators";
import { asset } from "../../assets";
import { descriptionToHtml } from "../../description";
import { computeSkillDiff, formatBlackboardValue, formatSkillLevel, getSkillTypeLabel, getSpTypeLabel } from "../../helpers";

interface ISkillsContentProps {
    operator: IOperatorListItem;
}

export const SkillsContent = memo(function SkillsContent({ operator }: ISkillsContentProps) {
    const [selectedSkillIndex, setSelectedSkillIndex] = useState(operator.skills.length > 0 ? operator.skills.length - 1 : 0);
    const [skillLevel, setSkillLevel] = useState(Math.max(0, (operator.skills[selectedSkillIndex].static?.levels ?? []).length - 1));
    const [comparisonMode, setComparisonMode] = useState(false);
    const [showDifferencesOnly, setShowDifferencesOnly] = useState(false);
    const [comparisonSelection, setComparisonSelection] = useState<number[] | null>(null);

    const skillData = (operator.skills[selectedSkillIndex].static?.levels ?? [])[skillLevel];
    const skillDescription = useMemo(() => descriptionToHtml(skillData?.description ?? "", skillData?.blackboard ?? []), [skillData?.description, skillData?.blackboard]);

    const levelsCount = (operator.skills[selectedSkillIndex].static?.levels ?? []).length;

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

    const handleSkillChange = useCallback(
        (idx: number) => {
            setSelectedSkillIndex(idx);
            const next = operator.skills[idx]?.static?.levels?.length ?? 1;
            setSkillLevel(next - 1);
            setComparisonSelection(null);
        },
        [operator.skills],
    );

    if (!operator.skills || operator.skills.length === 0) {
        return (
            <div className="flex items-center justify-center p-12">
                <p className="text-muted-foreground">No skills available for this operator.</p>
            </div>
        );
    }

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Skills</h2>
                <p className="text-muted-foreground text-sm">Skill details, mastery information, and skill comparisons.</p>
            </div>

            <div className="mb-6 flex flex-wrap gap-2">
                {operator.skills.map((skill, idx) => {
                    const name = skill.static?.levels?.[0]?.name ?? `Skill ${idx + 1}`;
                    const image = skill.static?.image;
                    const isSelected = selectedSkillIndex === idx;
                    return (
                        <button
                            key={skill.skillId ?? idx}
                            type="button"
                            onClick={() => handleSkillChange(idx)}
                            className={cn("flex items-center gap-2 rounded-lg border px-4 py-2 font-medium text-sm transition-colors", isSelected ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/30 text-muted-foreground hover:border-primary/50 hover:text-foreground")}
                        >
                            {image && <img alt="" className="h-5 w-5 shrink-0 object-contain" decoding="async" loading="lazy" src={asset(image)} />}
                            {name}
                        </button>
                    );
                })}
            </div>

            <div className="mb-6 rounded-md border border-border bg-secondary/20 p-5">
                {levelsCount > 7 && (
                    <div className="mb-5 flex items-center justify-between gap-4 border-b border-border pb-5">
                        <span className="flex items-center gap-2.5">
                            {comparisonMode ? <Columns className="h-4 w-4 text-primary" /> : <Rows className="h-4 w-4 text-muted-foreground" />}
                            <span className="font-medium text-foreground text-sm">Compare Skill Levels</span>
                        </span>
                        <Switch checked={comparisonMode} onCheckedChange={setComparisonMode} />
                    </div>
                )}
                {!comparisonMode ? (
                    <div className="flex flex-col gap-4 md:flex-row md:items-center">
                        <div className="flex-1 space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground text-sm">Skill Level</span>
                                <span className="font-mono text-foreground text-sm">{formatSkillLevel(skillLevel)}</span>
                            </div>
                            <Slider min={0} max={Math.max(0, levelsCount - 1)} step={1} value={[skillLevel]} onValueChange={(v) => setSkillLevel(Array.isArray(v) ? (v[0] ?? 0) : v)} />
                        </div>
                        <Select value={String(skillLevel)} onValueChange={(v) => setSkillLevel(Number.parseInt(String(v), 10))}>
                            <SelectTrigger className="w-36">
                                <SelectValue>{(value) => formatSkillLevel(Number(value))}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {operator.skills[selectedSkillIndex].static?.levels.map((_, i) => (
                                    <SelectItem key={`level-${i}`} value={String(i)}>
                                        {formatSkillLevel(i)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                ) : (
                    <div className="space-y-4">
                        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                            <div className="flex items-center gap-2">
                                <span className="font-medium text-foreground text-sm">Select Levels</span>
                                <span className="text-muted-foreground text-xs">({comparisonLevels.length} selected)</span>
                            </div>
                            <button
                                type="button"
                                onClick={() => setShowDifferencesOnly((s) => !s)}
                                className={cn("inline-flex h-9 items-center gap-2 rounded-sm border px-3 font-medium text-xs transition-colors", showDifferencesOnly ? "border-border bg-muted text-foreground shadow-sm ring-1 ring-border" : "border-border bg-secondary/50 text-foreground hover:bg-muted")}
                            >
                                <GitCompareArrows className="h-4 w-4" />
                                {showDifferencesOnly ? "Show Full" : "Show Diff"}
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
                                        onClick={() => toggleComparisonLevel(idx)}
                                        className={cn(
                                            "h-9 min-w-14 rounded-sm border px-3 font-medium text-xs transition-colors",
                                            isOn ? (isMastery ? "border-border bg-muted text-foreground shadow-sm ring-1 ring-border" : "border-primary bg-primary/10 text-primary") : "border-border bg-secondary/30 text-muted-foreground hover:text-foreground",
                                        )}
                                    >
                                        {formatSkillLevel(idx)}
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
                                <img alt={skillData.name ?? "Skill"} className="h-12 w-12 object-contain" decoding="async" loading="lazy" src={asset(operator.skills[selectedSkillIndex].static.image)} />
                            </div>
                        )}
                        <div className="min-w-0 flex-1">
                            <h3 className="font-semibold text-foreground text-lg">{skillData.name}</h3>
                            <div className="mt-2 flex flex-wrap gap-2">
                                <Badge className="rounded-sm bg-secondary/50" variant="secondary">
                                    {getSpTypeLabel(skillData.spData?.spType ?? "")}
                                </Badge>
                                <Badge className="rounded-sm bg-secondary/50" variant="secondary">
                                    {getSkillTypeLabel(skillData.skillType ?? "")}
                                </Badge>
                            </div>
                        </div>
                    </div>

                    <div className="mb-5 grid grid-cols-3 gap-3">
                        <div className="rounded-sm border border-border/50 bg-secondary/20 p-3 text-center">
                            <div className="text-muted-foreground text-xs">SP Cost</div>
                            <div className="mt-1 font-mono font-semibold text-foreground text-lg">{skillData.spData?.spCost ?? "-"}</div>
                        </div>
                        <div className="rounded-sm border border-border/50 bg-secondary/20 p-3 text-center">
                            <div className="text-muted-foreground text-xs">Initial SP</div>
                            <div className="mt-1 font-mono font-semibold text-foreground text-lg">{skillData.spData?.initSp ?? "-"}</div>
                        </div>
                        <div className="rounded-sm border border-border/50 bg-secondary/20 p-3 text-center">
                            <div className="text-muted-foreground text-xs">Duration</div>
                            <div className="mt-1 font-mono font-semibold text-foreground text-lg">{skillData.duration && skillData.duration > 0 ? `${skillData.duration}s` : "-"}</div>
                        </div>
                    </div>

                    <div className="rounded-sm border border-border/50 bg-secondary/10 p-4">
                        <p
                            className="text-foreground text-sm leading-relaxed"
                            // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized
                            dangerouslySetInnerHTML={{ __html: skillDescription }}
                        />
                    </div>
                </div>
            )}
            {comparisonMode && (
                <div className="space-y-4">
                    <div className="flex items-start gap-4 rounded-md border border-border bg-card/50 p-5 shadow-sm">
                        {operator.skills[selectedSkillIndex].static?.image && (
                            <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-sm border border-border bg-secondary/50 shadow-sm">
                                <img alt={operator.skills[selectedSkillIndex].static.levels[comparisonLevels[0] ?? 0]?.name ?? "Skill"} className="h-10 w-10 object-contain" decoding="async" loading="lazy" src={asset(operator.skills[selectedSkillIndex].static.image)} />
                            </div>
                        )}
                        <div className="min-w-0 flex-1">
                            <h3 className="font-semibold text-foreground text-lg">{operator.skills[selectedSkillIndex].static?.levels[comparisonLevels[0] ?? 0]?.name}</h3>
                            <div className="mt-2 flex flex-wrap gap-2">
                                <Badge className="rounded-sm bg-secondary shadow-sm" variant="secondary">
                                    {getSpTypeLabel(operator.skills[selectedSkillIndex].static?.levels[comparisonLevels[0] ?? 0]?.spData?.spType ?? "")}
                                </Badge>
                                <Badge className="rounded-sm bg-secondary shadow-sm" variant="secondary">
                                    {getSkillTypeLabel(operator.skills[selectedSkillIndex].static?.levels[comparisonLevels[0] ?? 0]?.skillType ?? "")}
                                </Badge>
                            </div>
                        </div>
                    </div>

                    <div className="hidden items-center gap-6 px-4 font-medium text-muted-foreground text-xs uppercase tracking-wider md:flex">
                        <div className="w-20 shrink-0">Level</div>
                        <div className="min-w-0 flex-1">Description / Changes</div>
                        <div className="flex shrink-0 gap-3">
                            <div className="w-14 text-center">SP</div>
                            <div className="w-14 text-center">Init</div>
                            <div className="w-14 text-center">Dur</div>
                        </div>
                    </div>

                    <div className="flex flex-col">
                        {comparisonLevels.map((idx, i) => {
                            const data = operator.skills[selectedSkillIndex].static?.levels[idx];
                            if (!data) return null;
                            const prev = i > 0 ? (operator.skills[selectedSkillIndex].static?.levels[comparisonLevels[i - 1]] ?? null) : null;
                            return <SkillComparisonRow key={`cmp-${idx}`} levelIndex={idx} levelData={data} prevLevelData={prev} isFirst={i === 0} isLast={i === comparisonLevels.length - 1} showDifferencesOnly={showDifferencesOnly} />;
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
}

export const SkillComparisonRow = memo(function SkillComparisonRow({ levelIndex, levelData, prevLevelData, isFirst, isLast, showDifferencesOnly }: ISkillComparisonRowProps) {
    const descriptionHtml = useMemo(() => descriptionToHtml(levelData.description ?? "", levelData.blackboard ?? []), [levelData.description, levelData.blackboard]);
    const diff = useMemo(() => computeSkillDiff(prevLevelData, levelData), [prevLevelData, levelData]);
    const hasChanges = diff.spCostChanged || diff.initSpChanged || diff.durationChanged || diff.blackboardChanges.size > 0;

    const diffSummary = useMemo(() => {
        if (!showDifferencesOnly || isFirst || !prevLevelData) return null;
        const out: { label: string; value: string }[] = [];
        if (diff.spCostChanged) out.push({ label: "SP Cost", value: `${prevLevelData.spData?.spCost} → ${levelData.spData?.spCost}` });
        if (diff.initSpChanged) out.push({ label: "Initial SP", value: `${prevLevelData.spData?.initSp} → ${levelData.spData?.initSp}` });
        if (diff.durationChanged) {
            const prevDur = prevLevelData.duration && prevLevelData.duration > 0 ? `${prevLevelData.duration}s` : "-";
            const currDur = levelData.duration && levelData.duration > 0 ? `${levelData.duration}s` : "-";
            out.push({ label: "Duration", value: `${prevDur} → ${currDur}` });
        }
        for (const [key, { prev, curr }] of diff.blackboardChanges) {
            const display = key.charAt(0).toUpperCase() + key.slice(1).replace(/_/g, " ");
            out.push({ label: display, value: `${formatBlackboardValue(key, prev)} → ${formatBlackboardValue(key, curr)}` });
        }
        return out;
    }, [showDifferencesOnly, isFirst, diff, prevLevelData, levelData]);

    if (showDifferencesOnly && !isFirst && !hasChanges) return null;

    return (
        <div className={cn("flex flex-col gap-4 border-border bg-card/30 p-5 md:flex-row md:items-start md:gap-6 md:p-6", isFirst && "rounded-t-lg border", !isFirst && !isLast && "border-x border-b", isLast && "rounded-b-lg border-x border-b", isFirst && isLast && "rounded-lg")}>
            <div className="flex shrink-0 items-center gap-2 md:w-20 md:flex-col md:items-start md:gap-1.5">
                <Badge className={cn("rounded-sm font-mono text-sm shadow-sm", levelIndex >= 7 ? "border-border bg-muted text-foreground" : "border-border bg-muted/50 text-muted-foreground")} variant="secondary">
                    {formatSkillLevel(levelIndex)}
                </Badge>
            </div>

            <div className="min-w-0 flex-1">
                {showDifferencesOnly && !isFirst && diffSummary && diffSummary.length > 0 ? (
                    <div className="space-y-2.5">
                        {diffSummary.map((c, idx) => (
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
            </div>

            <div className="flex shrink-0 items-center gap-2.5 md:gap-3">
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-3.5 py-1.5 transition-all duration-200", diff.spCostChanged ? "border-border bg-muted shadow-sm ring-1 ring-border" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">SP</div>
                    <div className="mt-0.5 font-mono font-semibold text-base text-foreground">{levelData.spData?.spCost ?? "-"}</div>
                </div>
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-3.5 py-1.5 transition-all duration-200", diff.initSpChanged ? "border-border bg-muted shadow-sm ring-1 ring-border" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">Init</div>
                    <div className="mt-0.5 font-mono font-semibold text-base text-foreground">{levelData.spData?.initSp ?? "-"}</div>
                </div>
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-3.5 py-1.5 transition-all duration-200", diff.durationChanged ? "border-border bg-muted shadow-sm ring-1 ring-border" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">Dur</div>
                    <div className="mt-0.5 font-mono font-semibold text-base text-foreground">{levelData.duration && levelData.duration > 0 ? `${levelData.duration}s` : "-"}</div>
                </div>
            </div>
        </div>
    );
});
