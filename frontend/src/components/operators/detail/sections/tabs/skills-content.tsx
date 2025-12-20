"use client";

import { ChevronDown, Columns, Rows, Zap } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { memo, useCallback, useMemo, useState } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { Slider } from "~/components/ui/shadcn/slider";
import { Switch } from "~/components/ui/shadcn/switch";
import { ToggleGroup, ToggleGroupItem } from "~/components/ui/shadcn/toggle-group";
import { descriptionToHtml } from "~/lib/description-parser";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import type { SkillLevel as SkillLevelType } from "~/types/api/impl/skill";

interface SkillsContentProps {
    operator: Operator;
}

// Comparison row component - horizontal layout for minimal eye movement (Fitts's Law)
interface SkillComparisonRowProps {
    levelIndex: number;
    levelData: SkillLevelType;
    formatSkillLevel: (level: number) => string;
    isFirst?: boolean;
    isLast?: boolean;
}

const SkillComparisonRow = memo(function SkillComparisonRow({ levelIndex, levelData, formatSkillLevel, isFirst, isLast }: SkillComparisonRowProps) {
    const descriptionHtml = useMemo(() => descriptionToHtml(levelData.description ?? "", levelData.blackboard ?? []), [levelData.description, levelData.blackboard]);

    return (
        <motion.div
            animate={{ opacity: 1 }}
            className={cn(
                "flex flex-col gap-3 border-border bg-card/30 p-3 md:flex-row md:items-start md:gap-4 md:p-4",
                isFirst && "rounded-t-lg border-x border-t",
                !isFirst && !isLast && "border-x border-t",
                isLast && "rounded-b-lg border",
                !isFirst && !isLast && !isFirst && "border-t-border/50",
            )}
            exit={{ opacity: 0 }}
            initial={{ opacity: 0 }}
            transition={{ duration: 0.15, delay: isFirst ? 0 : 0.03 }}
        >
            {/* Level Badge - Fixed width column */}
            <div className="flex shrink-0 items-center gap-2 md:w-14 md:flex-col md:items-start md:gap-1">
                <Badge className={cn("font-mono text-sm", levelIndex >= 7 ? "bg-primary/20 text-primary" : "bg-secondary/50")} variant="secondary">
                    {formatSkillLevel(levelIndex)}
                </Badge>
            </div>

            {/* Description - Flexible width, takes remaining space */}
            <div className="min-w-0 flex-1">
                <p
                    className="text-foreground text-sm leading-relaxed"
                    // biome-ignore lint/security/noDangerouslySetInnerHtml: Intentional HTML rendering for skill descriptions
                    dangerouslySetInnerHTML={{ __html: descriptionHtml }}
                />
            </div>

            {/* Stats - Fixed width columns on the right */}
            <div className="flex shrink-0 items-center gap-2 md:gap-3">
                <div className="w-14 rounded border border-border/50 bg-secondary/20 px-2 py-1.5 text-center">
                    <div className="text-[9px] text-muted-foreground md:hidden">SP</div>
                    <div className="font-mono font-semibold text-foreground text-sm">{levelData.spData?.spCost ?? "-"}</div>
                </div>
                <div className="w-14 rounded border border-border/50 bg-secondary/20 px-2 py-1.5 text-center">
                    <div className="text-[9px] text-muted-foreground md:hidden">Init</div>
                    <div className="font-mono font-semibold text-foreground text-sm">{levelData.spData?.initSp ?? "-"}</div>
                </div>
                <div className="w-14 rounded border border-border/50 bg-secondary/20 px-2 py-1.5 text-center">
                    <div className="text-[9px] text-muted-foreground md:hidden">Dur</div>
                    <div className="font-mono font-semibold text-foreground text-sm">{levelData.duration && levelData.duration > 0 ? `${levelData.duration}s` : "-"}</div>
                </div>
            </div>
        </motion.div>
    );
});

export const SkillsContent = memo(function SkillsContent({ operator }: SkillsContentProps) {
    const [skillLevel, setSkillLevel] = useState((operator.skills?.[0]?.static?.Levels?.length ?? 1) - 1);
    const [selectedSkillIndex, setSelectedSkillIndex] = useState(operator.skills.length > 0 ? operator.skills.length - 1 : 0);
    const [showTalents, setShowTalents] = useState(true);
    const [comparisonMode, setComparisonMode] = useState(false);
    const [selectedComparisonLevels, setSelectedComparisonLevels] = useState<string[]>([]);

    const selectedSkill = operator.skills[selectedSkillIndex];
    const skillData = selectedSkill?.static?.Levels?.[skillLevel];

    // Get all available levels for this skill
    const allSkillLevels = useMemo(() => {
        const levels = selectedSkill?.static?.Levels ?? [];
        return levels.map((level, idx) => ({ index: idx, data: level }));
    }, [selectedSkill?.static?.Levels]);

    // Get default comparison levels (SL7-M3 if available, otherwise last 4 levels)
    const defaultComparisonLevels = useMemo(() => {
        const totalLevels = selectedSkill?.static?.Levels?.length ?? 0;
        if (totalLevels >= 10) {
            // Has mastery: default to SL7, M1, M2, M3
            return ["6", "7", "8", "9"];
        }
        if (totalLevels >= 4) {
            // No mastery but has enough levels: show last 4
            return Array.from({ length: 4 }, (_, i) => String(totalLevels - 4 + i));
        }
        // Few levels: show all
        return Array.from({ length: totalLevels }, (_, i) => String(i));
    }, [selectedSkill?.static?.Levels?.length]);

    // Initialize selected levels when skill changes or comparison mode is enabled
    const effectiveComparisonLevels = useMemo(() => {
        if (selectedComparisonLevels.length === 0) {
            return defaultComparisonLevels;
        }
        // Filter out any levels that don't exist for this skill
        const maxLevel = (selectedSkill?.static?.Levels?.length ?? 1) - 1;
        const validLevels = selectedComparisonLevels.filter((l) => Number.parseInt(l, 10) <= maxLevel);
        return validLevels.length > 0 ? validLevels : defaultComparisonLevels;
    }, [selectedComparisonLevels, defaultComparisonLevels, selectedSkill?.static?.Levels?.length]);

    // Get the levels to display in comparison view based on selection
    const comparisonLevels = useMemo(() => {
        return allSkillLevels
            .filter((level) => effectiveComparisonLevels.includes(String(level.index)))
            .sort((a, b) => a.index - b.index);
    }, [allSkillLevels, effectiveComparisonLevels]);

    // Check if skill has mastery levels (more than 7 levels)
    const hasMasteryLevels = (selectedSkill?.static?.Levels?.length ?? 0) > 7;

    // Handle comparison level selection
    const handleComparisonLevelChange = useCallback((values: string[]) => {
        // Ensure at least one level is always selected
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

    const formatSkillLevel = useCallback((level: number) => {
        if (level < 7) return `Lv.${level + 1}`;
        return `M${level - 6}`;
    }, []);

    const getSpTypeLabel = useCallback((spType: string) => {
        switch (spType) {
            case "INCREASE_WITH_TIME":
                return "Auto Recovery";
            case "INCREASE_WHEN_ATTACK":
                return "Offensive Recovery";
            case "INCREASE_WHEN_TAKEN_DAMAGE":
                return "Defensive Recovery";
            case "UNKNOWN_8":
                return "N/A";
            default:
                return spType;
        }
    }, []);

    const getSkillTypeLabel = useCallback((skillType: number | string) => {
        if (skillType === 0 || skillType === "PASSIVE") return "Passive";
        if (skillType === 1 || skillType === "MANUAL") return "Manual Trigger";
        if (skillType === 2 || skillType === "AUTO") return "Auto Trigger";
        return "Unknown";
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
            <div className="mb-6 rounded-lg border border-border bg-secondary/20 p-4">
                {/* Comparison Mode Toggle */}
                {hasMasteryLevels && (
                    <div className="mb-4 flex items-center justify-between border-border border-b pb-4">
                        <div className="flex items-center gap-2">
                            {comparisonMode ? <Columns className="h-4 w-4 text-primary" /> : <Rows className="h-4 w-4 text-muted-foreground" />}
                            <span className="text-sm">Compare Skill Levels</span>
                        </div>
                        <Switch checked={comparisonMode} onCheckedChange={setComparisonMode} />
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

                {/* Comparison Mode Level Selector */}
                {comparisonMode && (
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <span className="text-muted-foreground text-sm">Select levels to compare</span>
                            <span className="text-muted-foreground text-xs">{comparisonLevels.length} selected</span>
                        </div>
                        <ToggleGroup
                            className="flex flex-wrap gap-1.5"
                            onValueChange={handleComparisonLevelChange}
                            type="multiple"
                            value={effectiveComparisonLevels}
                            variant="outline"
                        >
                            {allSkillLevels.map((level) => (
                                <ToggleGroupItem
                                    className={cn(
                                        "h-8 min-w-12 px-2 text-xs",
                                        level.index >= 7 && "data-[state=on]:bg-primary/20 data-[state=on]:text-primary",
                                    )}
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
                            <div className="p-4 md:p-6">
                                {/* Skill Header */}
                                <div className="mb-4 flex items-start gap-4">
                                    {selectedSkill?.static?.Image && (
                                        <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-lg border border-border bg-secondary/50">
                                            <Image alt={skillData.name ?? "Skill"} className="object-contain" height={48} src={`/api/cdn${selectedSkill.static.Image}`} width={48} />
                                        </div>
                                    )}
                                    <div className="min-w-0 flex-1">
                                        <h3 className="font-semibold text-foreground text-lg">{skillData.name}</h3>
                                        <div className="mt-1 flex flex-wrap gap-2">
                                            <Badge className="bg-secondary/50" variant="secondary">
                                                {getSpTypeLabel(skillData.spData?.spType ?? "")}
                                            </Badge>
                                            <Badge className="bg-secondary/50" variant="secondary">
                                                {getSkillTypeLabel(skillData.skillType ?? 0)}
                                            </Badge>
                                        </div>
                                    </div>
                                </div>

                                {/* SP Info */}
                                <div className="mb-4 grid grid-cols-3 gap-3">
                                    <div className="rounded-lg border border-border/50 bg-secondary/20 p-3 text-center">
                                        <div className="text-muted-foreground text-xs">SP Cost</div>
                                        <div className="font-mono font-semibold text-foreground text-lg">{skillData.spData?.spCost ?? "-"}</div>
                                    </div>
                                    <div className="rounded-lg border border-border/50 bg-secondary/20 p-3 text-center">
                                        <div className="text-muted-foreground text-xs">Initial SP</div>
                                        <div className="font-mono font-semibold text-foreground text-lg">{skillData.spData?.initSp ?? "-"}</div>
                                    </div>
                                    <div className="rounded-lg border border-border/50 bg-secondary/20 p-3 text-center">
                                        <div className="text-muted-foreground text-xs">Duration</div>
                                        <div className="font-mono font-semibold text-foreground text-lg">{skillData.duration && skillData.duration > 0 ? `${skillData.duration}s` : "-"}</div>
                                    </div>
                                </div>

                                {/* Description */}
                                <div className="rounded-lg border border-border/50 bg-secondary/10 p-4">
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

            {/* Skill Details - Comparison View */}
            {comparisonMode && (
                <div className="space-y-3">
                    {/* Skill Header (shown once above the comparison rows) */}
                    <div className="flex items-start gap-4 rounded-lg border border-border bg-card/30 p-4">
                        {selectedSkill?.static?.Image && (
                            <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg border border-border bg-secondary/50">
                                <Image alt={comparisonLevels[0]?.data.name ?? "Skill"} className="object-contain" height={36} src={`/api/cdn${selectedSkill.static.Image}`} width={36} />
                            </div>
                        )}
                        <div className="min-w-0 flex-1">
                            <h3 className="font-semibold text-foreground text-lg">{comparisonLevels[0]?.data.name}</h3>
                            <div className="mt-1 flex flex-wrap gap-2">
                                <Badge className="bg-secondary/50" variant="secondary">
                                    {getSpTypeLabel(comparisonLevels[0]?.data.spData?.spType ?? "")}
                                </Badge>
                                <Badge className="bg-secondary/50" variant="secondary">
                                    {getSkillTypeLabel(comparisonLevels[0]?.data.skillType ?? 0)}
                                </Badge>
                            </div>
                        </div>
                    </div>

                    {/* Column Headers - Desktop only */}
                    <div className="hidden items-center gap-4 px-4 text-muted-foreground text-xs md:flex">
                        <div className="w-14 shrink-0">Level</div>
                        <div className="min-w-0 flex-1">Description</div>
                        <div className="flex shrink-0 gap-3">
                            <div className="w-14 text-center">SP</div>
                            <div className="w-14 text-center">Init</div>
                            <div className="w-14 text-center">Dur</div>
                        </div>
                    </div>

                    {/* Comparison Rows */}
                    <AnimatePresence mode="wait">
                        <motion.div
                            animate={{ opacity: 1 }}
                            className="flex flex-col"
                            exit={{ opacity: 0 }}
                            initial={{ opacity: 0 }}
                            key={`comparison-${selectedSkillIndex}-${effectiveComparisonLevels.join("-")}`}
                            transition={{ duration: 0.2 }}
                        >
                            {comparisonLevels.map((level, idx) => (
                                <SkillComparisonRow
                                    formatSkillLevel={formatSkillLevel}
                                    isFirst={idx === 0}
                                    isLast={idx === comparisonLevels.length - 1}
                                    key={level.index}
                                    levelData={level.data}
                                    levelIndex={level.index}
                                />
                            ))}
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
                                    <div className="rounded-lg border border-border bg-card/30 p-4" key={idx}>
                                        <h4 className="mb-2 font-medium text-foreground">{candidate.Name ?? `Talent ${idx + 1}`}</h4>
                                        <p
                                            className="text-muted-foreground text-sm"
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
