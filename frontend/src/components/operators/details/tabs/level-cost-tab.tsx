"use client";

import { useEffect, useState } from "react";
import { motion } from "motion/react";
import Image from "next/image";
import type { Item, Operator } from "~/types/api";
import { Separator } from "~/components/ui/shadcn/separator";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";

interface LevelCostTabProps {
    operator: Operator;
}

interface MaterialCost {
    id: string;
    count: number;
}

export function LevelCostTab({ operator }: LevelCostTabProps) {
    const [materials, setMaterials] = useState<Record<string, Item>>({});
    const [activeTab, setActiveTab] = useState("elite");

    // Fetch materials
    useEffect(() => {
        const fetchMaterials = async () => {
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ type: "materials" }),
                });
                const data = (await response.json()) as { data: Item[] };
                // Convert array to object keyed by itemId for easy lookup
                const materialsMap: Record<string, Item> = {};
                for (const item of data.data ?? []) {
                    if (item.itemId) {
                        materialsMap[item.itemId] = item;
                    }
                }
                setMaterials(materialsMap);
            } catch (error) {
                console.error("Failed to fetch materials:", error);
            }
        };
        void fetchMaterials();
    }, []);

    // Elite promotion costs
    const eliteCosts = operator.phases
        .filter((phase, index) => index > 0 && phase.EvolveCost)
        .map((phase, index) => ({
            elite: `E${index + 1}`,
            costs: phase.EvolveCost ?? [],
        }));

    // Skill level costs (shared across all skills)
    const skillLevelCosts = operator.allSkillLevelUp ?? [];

    // Skill mastery costs (per skill)
    const masterySkills = operator.skills?.filter((s) => s.levelUpCostCond && s.levelUpCostCond.length > 0);

    return (
        <div className="w-full space-y-6">
            <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold md:text-3xl">Level-Up Cost</h2>
            </div>
            <Separator />

            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList className="grid w-full grid-cols-3">
                    <TabsTrigger value="elite">Elite Promotion</TabsTrigger>
                    <TabsTrigger value="skill">Skill Level Up</TabsTrigger>
                    <TabsTrigger value="module">Module</TabsTrigger>
                </TabsList>

                {/* Elite Promotion Tab */}
                <TabsContent value="elite" className="mt-4">
                    <AnimatedGroup preset="blur-slide" className="space-y-4">
                        {eliteCosts.length > 0 ? (
                            eliteCosts.map((elite, index) => (
                                <motion.div key={elite.elite} initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: index * 0.1 }} className="rounded-lg border border-border bg-card/50 p-4">
                                    <h4 className="mb-3 font-semibold text-primary">{elite.elite} Promotion</h4>
                                    <div className="flex flex-wrap gap-2">
                                        {elite.costs.map((cost: MaterialCost, i: number) => (
                                            <MaterialItem key={i} material={materials[cost.id]} count={cost.count} />
                                        ))}
                                    </div>
                                </motion.div>
                            ))
                        ) : (
                            <p className="text-muted-foreground">This operator cannot be promoted.</p>
                        )}
                    </AnimatedGroup>
                </TabsContent>

                {/* Skill Level Up Tab */}
                <TabsContent value="skill" className="mt-4">
                    <AnimatedGroup preset="blur-slide" className="space-y-4">
                        {/* Skill Levels 2-7 */}
                        {skillLevelCosts.length > 0 && (
                            <div className="rounded-lg border border-border bg-card/50 p-4">
                                <h4 className="mb-3 font-semibold text-primary">Skill Levels 2-7</h4>
                                <div className="space-y-3">
                                    {skillLevelCosts.map((levelCost, index) => (
                                        <div key={index} className="flex items-center gap-3">
                                            <span className="w-12 text-sm text-muted-foreground">Lv{index + 2}</span>
                                            <div className="flex flex-wrap gap-2">
                                                {levelCost.LvlUpCost?.map((cost: MaterialCost, i: number) => (
                                                    <MaterialItem key={i} material={materials[cost.id]} count={cost.count} />
                                                ))}
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* Mastery Costs */}
                        {masterySkills && masterySkills.length > 0 && (
                            <div className="rounded-lg border border-border bg-card/50 p-4">
                                <h4 className="mb-3 font-semibold text-primary">Skill Mastery</h4>
                                <div className="space-y-4">
                                    {masterySkills.map((skill, skillIndex) => (
                                        <div key={skill.skillId} className="space-y-2">
                                            <h5 className="text-sm font-medium">{skill.static?.Levels?.[0]?.name ?? `Skill ${skillIndex + 1}`}</h5>
                                            {skill.levelUpCostCond?.map((mastery, mIndex) => (
                                                <div key={mIndex} className="flex items-center gap-3">
                                                    <span className="w-12 text-sm text-muted-foreground">M{mIndex + 1}</span>
                                                    <div className="flex flex-wrap gap-2">
                                                        {mastery.LevelUpCost?.map((cost: MaterialCost, i: number) => (
                                                            <MaterialItem key={i} material={materials[cost.id]} count={cost.count} />
                                                        ))}
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {skillLevelCosts.length === 0 && (!masterySkills || masterySkills.length === 0) && <p className="text-muted-foreground">No skill upgrade costs available.</p>}
                    </AnimatedGroup>
                </TabsContent>

                {/* Module Tab */}
                <TabsContent value="module" className="mt-4">
                    <AnimatedGroup preset="blur-slide" className="space-y-4">
                        {operator.modules && operator.modules.length > 0 ? (
                            operator.modules
                                .filter((m) => m.type !== "INITIAL")
                                .map((module, index) => (
                                    <motion.div key={module.uniEquipId} initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: index * 0.1 }} className="rounded-lg border border-border bg-card/50 p-4">
                                        <h4 className="mb-3 font-semibold text-primary">{module.uniEquipName}</h4>
                                        {module.itemCost &&
                                            Object.entries(module.itemCost).map(([stage, costs]) => (
                                                <div key={stage} className="mb-2 flex items-center gap-3">
                                                    <span className="w-16 text-sm text-muted-foreground">Stage {stage}</span>
                                                    <div className="flex flex-wrap gap-2">
                                                        {Array.isArray(costs) ? costs.map((cost: MaterialCost, i: number) => <MaterialItem key={i} material={materials[cost.id]} count={cost.count} />) : <MaterialItem material={materials[(costs as MaterialCost).id]} count={(costs as MaterialCost).count} />}
                                                    </div>
                                                </div>
                                            ))}
                                    </motion.div>
                                ))
                        ) : (
                            <p className="text-muted-foreground">This operator has no modules.</p>
                        )}
                    </AnimatedGroup>
                </TabsContent>
            </Tabs>
        </div>
    );
}

function MaterialItem({ material, count }: { material?: Item; count: number }) {
    const iconUrl = material?.iconId ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/items/${material.iconId}.png` : "/abstract-material.png";

    return (
        <div className="flex items-center gap-1.5 rounded-md bg-muted/50 px-2 py-1">
            <div className="relative h-8 w-8">
                <Image src={iconUrl || "/placeholder.svg"} alt={material?.name ?? "Material"} fill className="object-contain" />
            </div>
            <span className="text-sm font-medium">x{count}</span>
        </div>
    );
}
