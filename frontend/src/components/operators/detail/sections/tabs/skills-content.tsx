"use client";

import { ChevronDown, Zap } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { useState } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { Slider } from "~/components/ui/shadcn/slider";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";

interface SkillsContentProps {
    operator: Operator;
}

export function SkillsContent({ operator }: SkillsContentProps) {
    const [skillLevel, setSkillLevel] = useState((operator.skills?.[0]?.static?.Levels?.length ?? 1) - 1);
    const [selectedSkillIndex, setSelectedSkillIndex] = useState(operator.skills.length > 0 ? operator.skills.length - 1 : 0);
    const [showTalents, setShowTalents] = useState(true);

    if (!operator.skills || operator.skills.length === 0) {
        return (
            <div className="flex items-center justify-center p-12">
                <p className="text-muted-foreground">No skills available for this operator.</p>
            </div>
        );
    }

    const selectedSkill = operator.skills[selectedSkillIndex];
    const skillData = selectedSkill?.static?.Levels?.[skillLevel];

    const formatSkillLevel = (level: number) => {
        if (level < 7) return `Lv.${level + 1}`;
        return `M${level - 6}`;
    };

    const getSpTypeLabel = (spType: string) => {
        switch (spType) {
            case "INCREASE_WITH_TIME":
                return "Auto Recovery";
            case "INCREASE_WHEN_ATTACK":
                return "Offensive Recovery";
            case "INCREASE_WHEN_TAKEN_DAMAGE":
                return "Defensive Recovery";
            default:
                return spType;
        }
    };

    const getSkillTypeLabel = (skillType: number | string) => {
        if (skillType === 0 || skillType === "0") return "Passive";
        if (skillType === 1 || skillType === "1") return "Manual Trigger";
        if (skillType === 2 || skillType === "2") return "Auto Trigger";
        return "Unknown";
    };

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
                            <Zap className="h-4 w-4" />
                            <span className="truncate">{name}</span>
                        </button>
                    );
                })}
            </div>

            {/* Skill Level Control */}
            <div className="mb-6 rounded-lg border border-border bg-secondary/20 p-4">
                <div className="flex flex-col gap-4 md:flex-row md:items-center">
                    <div className="flex-1 space-y-2">
                        <div className="flex items-center justify-between">
                            <span className="text-muted-foreground text-sm">Skill Level</span>
                            <span className="font-mono text-foreground text-sm">{formatSkillLevel(skillLevel)}</span>
                        </div>
                        <Slider className="w-full" max={(selectedSkill?.static?.Levels?.length ?? 1) - 1} min={0} onValueChange={(val) => setSkillLevel(val[0] ?? 0)} step={1} value={[skillLevel]} />
                    </div>
                    <Select onValueChange={(val) => setSkillLevel(Number.parseInt(val, 10))} value={skillLevel.toString()}>
                        <SelectTrigger className="w-32">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            {(selectedSkill?.static?.Levels ?? []).map((_, i) => (
                                <SelectItem key={i} value={i.toString()}>
                                    {formatSkillLevel(i)}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            </div>

            {/* Skill Details */}
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
                                    dangerouslySetInnerHTML={{
                                        __html: formatSkillDescription(skillData.description ?? "", skillData.blackboard ?? []),
                                    }}
                                />
                            </div>
                        </div>
                    )}
                </motion.div>
            </AnimatePresence>

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
                                if (!candidate) return null;
                                return (
                                    <div className="rounded-lg border border-border bg-card/30 p-4" key={idx}>
                                        <h4 className="mb-2 font-medium text-foreground">{candidate.Name ?? `Talent ${idx + 1}`}</h4>
                                        <p
                                            className="text-muted-foreground text-sm"
                                            dangerouslySetInnerHTML={{
                                                __html: formatSkillDescription(candidate.Description ?? "", candidate.Blackboard ?? []),
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
}

function formatSkillDescription(text: string, blackboard: Array<{ key: string; value: number; valueStr?: string | null }>): string {
    if (!text) return "";

    let result = text.replace(/<.[a-z]{2,5}?\.[^<]+>|<\/[^<]*>|<color=[^>]+>/g, "").trim();

    const regex = /\{([^}]+)\}/g;
    result = result.replace(regex, (match, content) => {
        const tag = content.split(":")[0]?.toLowerCase() ?? "";
        const negative = tag.startsWith("-");
        const key = negative ? tag.slice(1) : tag;
        const variable = blackboard.find((v) => v.key === key);

        if (!variable) return match;

        const value = negative ? -variable.value : variable.value;
        const isPercent = content.includes("%");

        if (isPercent) {
            return `<span class="font-semibold text-primary">${Math.round(value * 100)}%</span>`;
        }
        return `<span class="font-semibold text-primary">${value}</span>`;
    });

    return result;
}
