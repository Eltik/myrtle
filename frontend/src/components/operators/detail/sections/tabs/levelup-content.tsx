"use client";

import { ChevronRight } from "lucide-react";
import Image from "next/image";
import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";

interface LevelUpContentProps {
    operator: Operator;
}

export function LevelUpContent({ operator }: LevelUpContentProps) {
    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Level-Up Costs</h2>
                <p className="text-muted-foreground text-sm">Materials required for promotions, skill upgrades, and modules</p>
            </div>

            <Tabs className="w-full" defaultValue="elite">
                <TabsList className="mb-6 grid w-full grid-cols-3">
                    <TabsTrigger value="elite">Elite Promotion</TabsTrigger>
                    <TabsTrigger value="skills">Skill Mastery</TabsTrigger>
                    <TabsTrigger value="modules">Modules</TabsTrigger>
                </TabsList>

                <TabsContent value="elite">
                    <ElitePromotionTab operator={operator} />
                </TabsContent>

                <TabsContent value="skills">
                    <SkillMasteryTab operator={operator} />
                </TabsContent>

                <TabsContent value="modules">
                    <ModulesTab operator={operator} />
                </TabsContent>
            </Tabs>
        </div>
    );
}

function ElitePromotionTab({ operator }: { operator: Operator }) {
    return (
        <div className="grid gap-4 md:grid-cols-2">
            {operator.phases.map((phase, idx) => {
                if (idx === 0 || !phase.EvolveCost || phase.EvolveCost.length === 0) return null;

                return (
                    // biome-ignore lint/suspicious/noArrayIndexKey: Static promotion phases
                    <div className="rounded-lg border border-border bg-card/30 p-4" key={idx}>
                        <div className="mb-3 flex items-center gap-3">
                            <div className="flex items-center gap-2">
                                <Image alt={`E${idx - 1}`} height={24} src={`/api/cdn/upk/arts/elite_hub/elite_${idx - 1}.png`} width={24} />
                                <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                <Image alt={`E${idx}`} height={24} src={`/api/cdn/upk/arts/elite_hub/elite_${idx}.png`} width={24} />
                            </div>
                            <span className="font-medium text-foreground">Elite {idx}</span>
                        </div>

                        <div className="flex flex-wrap gap-4">
                            {phase.EvolveCost.map((cost, costIdx) => (
                                // biome-ignore lint/suspicious/noArrayIndexKey: Static cost list
                                <MaterialItem count={cost.Count} id={cost.Id} image={cost.Image} key={costIdx} />
                            ))}
                        </div>
                    </div>
                );
            })}

            {operator.phases.length <= 1 && <div className="py-8 text-center text-muted-foreground md:col-span-2">This operator cannot be promoted.</div>}
        </div>
    );
}

function SkillMasteryTab({ operator }: { operator: Operator }) {
    const [selectedSkill, setSelectedSkill] = useState(0);

    if (!operator.skills || operator.skills.length === 0) {
        return <div className="py-8 text-center text-muted-foreground">No skills available for this operator.</div>;
    }

    const skill = operator.skills[selectedSkill];
    const levelUpCosts = skill?.levelUpCostCond ?? [];

    return (
        <div className="space-y-4">
            {/* Skill selector */}
            <div className="flex flex-wrap gap-2">
                {operator.skills.map((s, idx) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: Static skill list
                    <button className={cn("rounded-lg border px-4 py-2 font-medium text-sm transition-all", selectedSkill === idx ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/30 text-muted-foreground hover:text-foreground")} key={idx} onClick={() => setSelectedSkill(idx)} type="button">
                        {s.static?.Levels?.[0]?.name ?? `Skill ${idx + 1}`}
                    </button>
                ))}
            </div>

            {/* Mastery costs */}
            <div className="space-y-3">
                {levelUpCosts.map((costData, idx) => {
                    if (!costData.LevelUpCost || costData.LevelUpCost.length === 0) return null;
                    return (
                        // biome-ignore lint/suspicious/noArrayIndexKey: Static mastery level list
                        <div className="rounded-lg border border-border bg-card/30 p-4" key={idx}>
                            <div className="mb-3 flex items-center gap-2">
                                <span className="rounded bg-primary/20 px-2 py-0.5 font-semibold text-primary text-sm">M{idx + 1}</span>
                            </div>
                            <div className="flex flex-wrap gap-4">
                                {costData.LevelUpCost.map((cost, costIdx) => (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static cost list
                                    <MaterialItem count={cost.Count} id={cost.Id} image={cost.Image} key={costIdx} />
                                ))}
                            </div>
                        </div>
                    );
                })}
            </div>

            {levelUpCosts.length === 0 && <div className="py-8 text-center text-muted-foreground">No mastery data available for this skill.</div>}
        </div>
    );
}

// Helper function to format attribute keys to readable names
function formatStatKey(key: string): string {
    const statKeyMap: Record<string, string> = {
        atk: "ATK",
        def: "DEF",
        max_hp: "Max HP",
        attack_speed: "ASPD",
        magic_resistance: "RES",
        cost: "DP Cost",
        respawn_time: "Redeploy",
        block_cnt: "Block",
        hp_recovery_per_sec: "HP Regen",
        sp_recovery_per_sec: "SP Regen",
        base_attack_time: "Attack Interval",
    };
    return statKeyMap[key.toLowerCase()] ?? key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

// Helper function to format stat values (percentages vs flat values)
function formatStatValue(value: number): string {
    // Values less than 2 and greater than -2 are typically percentages (e.g., 0.15 = 15%)
    if (Math.abs(value) < 2 && value !== 0 && !Number.isInteger(value)) {
        const percentage = Math.round(value * 100);
        return `${percentage >= 0 ? "+" : ""}${percentage}%`;
    }
    return `${value >= 0 ? "+" : ""}${value}`;
}

function ModulesTab({ operator }: { operator: Operator }) {
    if (!operator.modules || operator.modules.length === 0) {
        return <div className="py-8 text-center text-muted-foreground">This operator has no modules.</div>;
    }

    return (
        <div className="space-y-4">
            {operator.modules.map((mod) => (
                <div className="rounded-lg border border-border bg-card/30 p-4" key={mod.uniEquipId}>
                    {/* Module Header */}
                    <div className="mb-4 flex items-center gap-3">
                        {mod.image && <Image alt={mod.uniEquipName ?? "Module"} className="rounded object-contain" height={48} src={`/api/cdn${mod.image}`} width={48} />}
                        <div className="flex-1">
                            <div className="flex items-center gap-2">
                                <h4 className="font-medium text-foreground">{mod.uniEquipName ?? "Module"}</h4>
                                <span className="rounded bg-secondary px-1.5 py-0.5 text-muted-foreground text-xs">{mod.typeName1}</span>
                            </div>
                            <p className="text-muted-foreground text-xs">
                                Unlock: E{mod.unlockEvolvePhase} Lv.{mod.unlockLevel}
                                {mod.unlockFavorPoint > 0 && ` · Trust ${Math.floor(mod.unlockFavorPoint / 100)}%`}
                            </p>
                        </div>
                    </div>

                    {/* Module Stages */}
                    {mod.data?.phases && mod.data.phases.length > 0 && (
                        <div className="space-y-3">
                            {mod.data.phases.map((phase, phaseIdx) => {
                                // Get costs for this specific stage from itemCost (keys are "1", "2", "3")
                                const stageKey = String(phaseIdx + 1);
                                const stageCosts = mod.itemCost?.[stageKey] ?? [];

                                return (
                                    <div className="rounded border border-border/50 bg-secondary/20 p-3" key={phase.equipLevel}>
                                        <div className="mb-2 flex items-center gap-2">
                                            <span className="rounded bg-primary/20 px-2 py-0.5 font-semibold text-primary text-sm">Stage {phaseIdx + 1}</span>
                                        </div>

                                        {/* Materials for this stage */}
                                        {stageCosts.length > 0 ? (
                                            <div className="flex flex-wrap gap-4">
                                                {stageCosts.map((cost, costIdx) => (
                                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static cost list
                                                    <MaterialItem count={cost.count} id={cost.id} image={cost.image} key={costIdx} size="sm" />
                                                ))}
                                            </div>
                                        ) : (
                                            <p className="text-muted-foreground text-xs italic">No material data available</p>
                                        )}

                                        {/* Attribute bonuses for this stage */}
                                        {phase.attributeBlackboard && phase.attributeBlackboard.length > 0 && (
                                            <div className="mt-3 flex flex-wrap gap-2">
                                                {phase.attributeBlackboard.map((attr, attrIdx) => (
                                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static attribute list
                                                    <span className="rounded bg-primary/10 px-2 py-1 text-primary text-xs" key={attrIdx}>
                                                        {formatStatKey(attr.key)}: {formatStatValue(attr.value)}
                                                    </span>
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            ))}
        </div>
    );
}

function MaterialItem({ id, count, image, size = "md" }: { id: string; count: number; image?: string | null; size?: "sm" | "md" }) {
    const sizeClass = size === "sm" ? "h-10 w-10" : "h-12 w-12";
    // Use backend-provided image path if available, otherwise fallback to default path
    const imageSrc = image ? `/api/cdn${image}` : `/api/cdn/upk/arts/items/icons/${id}.png`;

    return (
        <div className="flex flex-col items-center gap-1">
            <div className={cn("relative rounded-lg border border-border/50 bg-secondary/30 p-1", sizeClass)}>
                <Image alt={id} className="object-contain" fill src={imageSrc} />
            </div>
            <span className="font-mono text-foreground text-xs">x{count}</span>
        </div>
    );
}
