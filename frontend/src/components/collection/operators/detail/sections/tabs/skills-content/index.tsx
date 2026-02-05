"use client";

import { ChevronDown, Columns, GitCompareArrows, Rows, Zap } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { memo, useCallback, useMemo, useState } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { Slider } from "~/components/ui/shadcn/slider";
import { Switch } from "~/components/ui/shadcn/switch";
import { ToggleGroup, ToggleGroupItem } from "~/components/ui/shadcn/toggle-group";
import { descriptionToHtml } from "~/lib/description-parser";
import { formatSkillLevel, getSkillTypeLabel, getSpTypeLabel } from "~/lib/skill-helpers";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import { SkillComparisonRow } from "./impl/skill-comparison-row";

interface SkillsContentProps {
    operator: Operator;
}

export const SkillsContent = memo(function SkillsContent({ operator }: SkillsContentProps) {
    const [skillLevel, setSkillLevel] = useState((operator.skills?.[0]?.static?.Levels?.length ?? 1) - 1);
    const [selectedSkillIndex, setSelectedSkillIndex] = useState(operator.skills.length > 0 ? operator.skills.length - 1 : 0);
    const [showTalents, setShowTalents] = useState(true);
    const [comparisonMode, setComparisonMode] = useState(false);
    const [selectedComparisonLevels, setSelectedComparisonLevels] = useState<string[]>([]);
    const [showDifferencesOnly, setShowDifferencesOnly] = useState(false);

    const selectedSkill = operator.skills[selectedSkillIndex];
    const skillData = selectedSkill?.static?.Levels?.[skillLevel];

    const allSkillLevels = useMemo(() => {
        const levels = selectedSkill?.static?.Levels ?? [];
        return levels.map((level, idx) => ({ index: idx, data: level }));
    }, [selectedSkill?.static?.Levels]);

    const defaultComparisonLevels = useMemo(() => {
        const totalLevels = selectedSkill?.static?.Levels?.length ?? 0;
        if (totalLevels >= 10) {
            return ["6", "7", "8", "9"];
        }
        if (totalLevels >= 4) {
            return Array.from({ length: 4 }, (_, i) => String(totalLevels - 4 + i));
        }
        return Array.from({ length: totalLevels }, (_, i) => String(i));
    }, [selectedSkill?.static?.Levels?.length]);

    const effectiveComparisonLevels = useMemo(() => {
        if (selectedComparisonLevels.length === 0) {
            return defaultComparisonLevels;
        }
        const maxLevel = (selectedSkill?.static?.Levels?.length ?? 1) - 1;
        const validLevels = selectedComparisonLevels.filter((l) => Number.parseInt(l, 10) <= maxLevel);
        return validLevels.length > 0 ? validLevels : defaultComparisonLevels;
    }, [selectedComparisonLevels, defaultComparisonLevels, selectedSkill?.static?.Levels?.length]);

    const comparisonLevels = useMemo(() => {
        return allSkillLevels.filter((level) => effectiveComparisonLevels.includes(String(level.index))).sort((a, b) => a.index - b.index);
    }, [allSkillLevels, effectiveComparisonLevels]);

    const hasMasteryLevels = (selectedSkill?.static?.Levels?.length ?? 0) > 7;

    const handleComparisonLevelChange = useCallback((values: string[]) => {
        if (values.length > 0) {
            setSelectedComparisonLevels(values);
        }
    }, []);

    const skillDescriptionHtml = useMemo(() => descriptionToHtml(skillData?.description ?? "", skillData?.blackboard ?? []), [skillData?.description, skillData?.blackboard]);

    const handleSkillLevelChange = useCallback((val: number[]) => {
        setSkillLevel(val[0] ?? 0);
    }, []);

    const handleSkillLevelSelect = useCallback((val: string) => {
        setSkillLevel(Number.parseInt(val, 10));
    }, []);

    if (!operator.skills || operator.skills.length === 0) {
        return (
            <div className="flex items-center justify-center p-12">
                <p className="text-muted-foreground">No skills available for this operator.</p>
            </div>
        );
    }

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Skills & Talents</h2>
                <p className="text-muted-foreground text-sm">View skill details and talent information</p>
            </div>

            {/* Skill Tabs */}
            <div className="mb-6 flex flex-wrap gap-2">
                {operator.skills.map((skill, idx) => {
                    const name = skill.static?.Levels?.[0]?.name ?? `Skill ${idx + 1}`;
                    const isSelected = selectedSkillIndex === idx;
                    return (
                        <button
                            className={cn("flex items-center gap-2 rounded-lg border px-4 py-2 font-medium text-sm transition-all", isSelected ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/30 text-muted-foreground hover:border-primary/50 hover:text-foreground")}
                            key={skill.skillId ?? idx}
                            onClick={() => setSelectedSkillIndex(idx)}
                            type="button"
                        >
                            <span className="truncate">{name}</span>
                        </button>
                    );
                })}
            </div>

            {/* Skill Level Control */}
            <div className="mb-6 rounded-md border border-border bg-secondary/20 p-5">
                {/* Comparison Mode Toggle */}
                {hasMasteryLevels && (
                    <div className="mb-5 flex items-center justify-between gap-4 border-border border-b pb-5">
                        <div className="flex items-center gap-2.5">
                            {comparisonMode ? <Columns className="h-4 w-4 text-primary" /> : <Rows className="h-4 w-4 text-muted-foreground" />}
                            <span className="font-medium text-foreground text-sm">Compare Skill Levels</span>
                        </div>
                        <Switch checked={comparisonMode} className="scale-110" onCheckedChange={setComparisonMode} />
                    </div>
                )}

                {/* Single Level View Controls */}
                {!comparisonMode && (
                    <div className="flex flex-col gap-4 md:flex-row md:items-center">
                        <div className="flex-1 space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground text-sm">Skill Level</span>
                                <span className="font-mono text-foreground text-sm">{formatSkillLevel(skillLevel)}</span>
                            </div>
                            <Slider className="w-full" max={(selectedSkill?.static?.Levels?.length ?? 1) - 1} min={0} onValueChange={handleSkillLevelChange} step={1} value={[skillLevel]} />
                        </div>
                        <Select onValueChange={handleSkillLevelSelect} value={skillLevel.toString()}>
                            <SelectTrigger className="w-32">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                {(selectedSkill?.static?.Levels ?? []).map((_, i) => (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static skill level list
                                    <SelectItem key={i} value={i.toString()}>
                                        {formatSkillLevel(i)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                )}

                {comparisonMode && (
                    <div className="space-y-4">
                        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                            <div className="flex items-center gap-2">
                                <span className="font-medium text-foreground text-sm">Select Levels</span>
                                <span className="text-muted-foreground text-xs">({effectiveComparisonLevels.length} selected)</span>
                            </div>
                            <Button
                                className={cn("h-9 gap-2 rounded-sm font-medium text-xs transition-all", showDifferencesOnly ? "border-border bg-muted text-foreground shadow-sm ring-1 ring-border" : "border-border bg-secondary/50 text-foreground hover:bg-muted")}
                                onClick={() => setShowDifferencesOnly(!showDifferencesOnly)}
                                size="sm"
                                variant="outline"
                            >
                                <GitCompareArrows className="h-4 w-4" />
                                {showDifferencesOnly ? "Show Full" : "Show Diff"}
                            </Button>
                        </div>
                        <ToggleGroup className="flex flex-wrap gap-2" onValueChange={handleComparisonLevelChange} spacing={1} type="multiple" value={effectiveComparisonLevels} variant="outline">
                            {allSkillLevels.map((level) => (
                                <ToggleGroupItem
                                    className={cn("h-9 min-w-14 rounded-sm px-3 font-medium text-xs transition-all", level.index >= 7 && "data-[state=on]:border-border data-[state=on]:bg-muted data-[state=on]:text-foreground data-[state=on]:shadow-sm data-[state=on]:ring-1 data-[state=on]:ring-border")}
                                    key={level.index}
                                    value={String(level.index)}
                                >
                                    {formatSkillLevel(level.index)}
                                </ToggleGroupItem>
                            ))}
                        </ToggleGroup>
                    </div>
                )}
            </div>

            {/* Skill Details - Single View */}
            {!comparisonMode && (
                <AnimatePresence mode="wait">
                    <motion.div animate={{ opacity: 1, y: 0 }} className="rounded-lg border border-border bg-card/30" exit={{ opacity: 0, y: -10 }} initial={{ opacity: 0, y: 10 }} key={`${selectedSkillIndex}-${skillLevel}`} transition={{ duration: 0.2 }}>
                        {skillData && (
                            <div className="p-5 md:p-6">
                                {/* Skill Header */}
                                <div className="mb-5 flex items-start gap-4">
                                    {selectedSkill?.static?.Image && (
                                        <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-md border border-border bg-secondary/50">
                                            <Image alt={skillData.name ?? "Skill"} className="object-contain" height={48} src={`/api/cdn${selectedSkill.static.Image}`} width={48} />
                                        </div>
                                    )}
                                    <div className="min-w-0 flex-1">
                                        <h3 className="font-semibold text-foreground text-lg">{skillData.name}</h3>
                                        <div className="mt-2 flex flex-wrap gap-2">
                                            <Badge className="rounded-sm bg-secondary/50" variant="secondary">
                                                {getSpTypeLabel(skillData.spData?.spType ?? "")}
                                            </Badge>
                                            <Badge className="rounded-sm bg-secondary/50" variant="secondary">
                                                {getSkillTypeLabel(skillData.skillType ?? 0)}
                                            </Badge>
                                        </div>
                                    </div>
                                </div>

                                {/* SP Info */}
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

                                {/* Description */}
                                <div className="rounded-sm border border-border/50 bg-secondary/10 p-4">
                                    <p
                                        className="text-foreground text-sm leading-relaxed"
                                        // biome-ignore lint/security/noDangerouslySetInnerHtml: Intentional HTML rendering for skill descriptions
                                        dangerouslySetInnerHTML={{ __html: skillDescriptionHtml }}
                                    />
                                </div>
                            </div>
                        )}
                    </motion.div>
                </AnimatePresence>
            )}

            {comparisonMode && (
                <div className="space-y-4">
                    {/* Skill Header (shown once above the comparison rows) */}
                    <div className="flex items-start gap-4 rounded-md border border-border bg-card/50 p-5 shadow-sm">
                        {selectedSkill?.static?.Image && (
                            <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-sm border border-border bg-secondary/50 shadow-sm">
                                <Image alt={comparisonLevels[0]?.data.name ?? "Skill"} className="object-contain" height={40} src={`/api/cdn${selectedSkill.static.Image}`} width={40} />
                            </div>
                        )}
                        <div className="min-w-0 flex-1">
                            <h3 className="font-semibold text-foreground text-lg">{comparisonLevels[0]?.data.name}</h3>
                            <div className="mt-2 flex flex-wrap gap-2">
                                <Badge className="rounded-sm bg-secondary shadow-sm" variant="secondary">
                                    {getSpTypeLabel(comparisonLevels[0]?.data.spData?.spType ?? "")}
                                </Badge>
                                <Badge className="rounded-sm bg-secondary shadow-sm" variant="secondary">
                                    {getSkillTypeLabel(comparisonLevels[0]?.data.skillType ?? 0)}
                                </Badge>
                            </div>
                        </div>
                    </div>

                    <div className="hidden items-center gap-6 px-4 font-medium text-muted-foreground text-xs uppercase tracking-wider md:flex">
                        <div className="w-20 shrink-0">Level</div>
                        <div className="min-w-0 flex-1">Description / Changes</div>
                        <div className="flex shrink-0 gap-3">
                            <div className="w-16 text-center">SP Cost</div>
                            <div className="w-16 text-center">Initial</div>
                            <div className="w-16 text-center">Duration</div>
                        </div>
                    </div>

                    {/* Comparison Rows */}
                    <AnimatePresence mode="wait">
                        <motion.div animate={{ opacity: 1 }} className="flex flex-col" exit={{ opacity: 0 }} initial={{ opacity: 0 }} key={`comparison-${selectedSkillIndex}-${effectiveComparisonLevels.join("-")}-${showDifferencesOnly}`} transition={{ duration: 0.2 }}>
                            {comparisonLevels.map((level, idx) => {
                                const prevLevel = idx > 0 ? (comparisonLevels[idx - 1]?.data ?? null) : null;
                                return <SkillComparisonRow isFirst={idx === 0} isLast={idx === comparisonLevels.length - 1} key={level.index} levelData={level.data} levelIndex={level.index} prevLevelData={prevLevel} showDifferencesOnly={showDifferencesOnly} />;
                            })}
                        </motion.div>
                    </AnimatePresence>
                </div>
            )}

            <Separator className="my-6" />

            {/* Talents */}
            {operator.talents && operator.talents.length > 0 && (
                <Collapsible onOpenChange={setShowTalents} open={showTalents}>
                    <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                        <div className="flex items-center gap-2">
                            <Zap className="h-4 w-4 text-primary" />
                            <span className="font-medium text-sm">Talents</span>
                        </div>
                        <ChevronDown className={cn("h-4 w-4 transition-transform", showTalents && "rotate-180")} />
                    </CollapsibleTrigger>
                    <CollapsibleContent>
                        <div className="mt-3 space-y-3">
                            {operator.talents.map((talent, idx) => {
                                const candidate = talent.Candidates?.[talent.Candidates.length - 1];
                                if (!candidate || !candidate.Name) return null;
                                return (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static talent list
                                    <div className="rounded-md border border-border bg-card/30 p-5" key={idx}>
                                        <h4 className="mb-2 font-medium text-foreground">{candidate.Name ?? `Talent ${idx + 1}`}</h4>
                                        <p
                                            className="text-muted-foreground text-sm leading-relaxed"
                                            // biome-ignore lint/security/noDangerouslySetInnerHtml: Intentional HTML rendering for talent descriptions
                                            dangerouslySetInnerHTML={{
                                                __html: descriptionToHtml(candidate.Description ?? "", candidate.Blackboard ?? []),
                                            }}
                                        />
                                    </div>
                                );
                            })}
                        </div>
                    </CollapsibleContent>
                </Collapsible>
            )}
        </div>
    );
});
